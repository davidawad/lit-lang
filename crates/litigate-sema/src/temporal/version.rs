//! Version sets of one logical declaration and bitemporal `as_of`
//! selection (SPEC §5.4, DESIGN §4.4).
//!
//! Versions are the same-named declarations of one package. `in_force` is
//! when the law applied; `known_at` is when the corpus recorded it. A query
//! at `(as_of, known_at)` sees only versions recorded by `known_at` and
//! picks the one whose window holds `as_of`.
use super::interval::{gaps, Window};

/// One version: when it applied, when it was recorded, and its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version<T> {
    /// When the version applied.
    pub in_force: Window,
    /// When the corpus recorded it.
    pub known_at: String,
    /// The declaration this version is.
    pub decl: T,
}

/// Why a query selected no version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Miss {
    /// No version recorded by the query's `known_at` is in force on the date
    /// (L3007). `hidden` lists versions in force that date but recorded later.
    NotInForce {
        /// Versions in force on the date whose `known_at` is after the query's.
        hidden: Vec<String>,
    },
    /// More than one version holds the date (an unchecked L3001 overlap).
    Ambiguous(Vec<Window>),
}

/// A pair of versions whose windows share days (L3001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlap {
    /// Index of the earlier-starting version.
    pub first: usize,
    /// Index of the later-starting version.
    pub second: usize,
    /// The shared days.
    pub common: Window,
}

/// Every version of one declaration, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionSet<T> {
    /// Versions in source order.
    pub versions: Vec<Version<T>>,
}

impl<T> Default for VersionSet<T> {
    fn default() -> Self {
        Self {
            versions: Vec::new(),
        }
    }
}

impl<T> VersionSet<T> {
    /// A set of `versions`.
    #[must_use]
    pub fn new(versions: Vec<Version<T>>) -> Self {
        Self { versions }
    }

    /// The single version in force on `as_of` among those recorded by
    /// `known_at` (`None`: every version, the latest corpus).
    ///
    /// # Errors
    /// [`Miss::NotInForce`] when none applies, [`Miss::Ambiguous`] when the
    /// set overlaps on that date.
    pub fn as_of(&self, as_of: &str, known_at: Option<&str>) -> Result<&Version<T>, Miss> {
        let (visible, hidden): (Vec<&Version<T>>, Vec<&Version<T>>) = self
            .versions
            .iter()
            .filter(|v| v.in_force.contains(as_of))
            .partition(|v| known_at.is_none_or(|k| v.known_at.as_str() <= k));
        match visible.as_slice() {
            [one] => Ok(one),
            [] => Err(Miss::NotInForce {
                hidden: hidden.iter().map(|v| v.in_force.display()).collect(),
            }),
            many => Err(Miss::Ambiguous(
                many.iter().map(|v| v.in_force.clone()).collect(),
            )),
        }
    }

    /// The version whose window starts on `from` (the `@date` selector).
    #[must_use]
    pub fn starting(&self, from: &str) -> Option<&Version<T>> {
        self.versions.iter().find(|v| v.in_force.from == from)
    }

    /// Index of the version that ends exactly where the one at `index`
    /// begins: the version an amendment replaces.
    #[must_use]
    pub fn predecessor(&self, index: usize) -> Option<usize> {
        let from = &self.versions.get(index)?.in_force.from;
        self.versions
            .iter()
            .position(|v| v.in_force.to.as_ref() == Some(from))
    }

    /// Every pair of overlapping versions (`effective-disjoint`), ordered by
    /// the later version's position.
    #[must_use]
    pub fn overlaps(&self) -> Vec<Overlap> {
        let mut out = Vec::new();
        for (j, b) in self.versions.iter().enumerate() {
            for (i, a) in self.versions.iter().enumerate().take(j) {
                if let Some(common) = a.in_force.intersect(&b.in_force) {
                    let (first, second) = if a.in_force.from <= b.in_force.from {
                        (i, j)
                    } else {
                        (j, i)
                    };
                    out.push(Overlap {
                        first,
                        second,
                        common,
                    });
                }
            }
        }
        out
    }

    /// The declaration's lifetime inside `package`: from its earliest
    /// version to the package's end (`None` when it has no version that far).
    #[must_use]
    pub fn lifetime(&self, package: &Window) -> Option<Window> {
        let first = self
            .versions
            .iter()
            .map(|v| v.in_force.from.clone())
            .min()?;
        Window {
            from: first,
            to: package.to.clone(),
        }
        .intersect(package)
    }

    /// Days of the lifetime no version covers (`effective-covered`).
    #[must_use]
    pub fn gaps(&self, package: &Window) -> Vec<Window> {
        self.lifetime(package).map_or_else(Vec::new, |life| {
            let cover: Vec<Window> = self.versions.iter().map(|v| v.in_force.clone()).collect();
            gaps(&life, &cover)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn v(from: &str, to: Option<&str>, known: &str, decl: u32) -> Version<u32> {
        Version {
            in_force: Window::new(from, to),
            known_at: known.into(),
            decl,
        }
    }

    fn answer_period() -> VersionSet<u32> {
        VersionSet::new(vec![
            v("1993-12-01", Some("2009-12-01"), "1993-04-22", 20),
            v("2009-12-01", None, "2009-03-26", 21),
        ])
    }

    #[test]
    fn as_of_selects_the_version_in_force() {
        let set = answer_period();
        assert_eq!(set.as_of("2009-11-30", None).map(|v| v.decl), Ok(20));
        assert_eq!(set.as_of("2009-12-01", None).map(|v| v.decl), Ok(21));
        assert_eq!(
            set.as_of("1990-01-01", None),
            Err(Miss::NotInForce { hidden: vec![] })
        );
    }

    #[test]
    fn known_at_hides_versions_the_corpus_had_not_recorded() {
        let set = answer_period();
        assert_eq!(
            set.as_of("2010-06-01", Some("2009-01-01")),
            Err(Miss::NotInForce {
                hidden: vec!["[2009-12-01, ..)".into()]
            })
        );
        assert_eq!(
            set.as_of("2010-06-01", Some("2009-03-26")).map(|v| v.decl),
            Ok(21)
        );
        assert_eq!(
            set.as_of("2000-06-01", Some("2009-01-01")).map(|v| v.decl),
            Ok(20)
        );
    }

    #[test]
    fn overlaps_gaps_and_predecessors() {
        let set = VersionSet::new(vec![
            v("2000-01-01", None, "2000-01-01", 1),
            v("2005-01-01", Some("2006-01-01"), "2000-01-01", 2),
            v("2008-01-01", None, "2000-01-01", 3),
        ]);
        let pairs: Vec<(usize, usize)> =
            set.overlaps().iter().map(|o| (o.first, o.second)).collect();
        assert_eq!(pairs, vec![(0, 1), (0, 2)]);
        assert!(matches!(
            set.as_of("2005-06-01", None),
            Err(Miss::Ambiguous(_))
        ));
        let gappy = VersionSet::new(vec![
            v("2000-01-01", Some("2001-01-01"), "2000-01-01", 1),
            v("2002-01-01", Some("2003-01-01"), "2000-01-01", 2),
        ]);
        let pkg = Window::new("1999-01-01", None::<String>);
        assert_eq!(
            gappy.gaps(&pkg),
            vec![
                Window::new("2001-01-01", Some("2002-01-01")),
                Window::new("2003-01-01", None::<String>)
            ]
        );
        assert_eq!(answer_period().predecessor(1), Some(0));
        assert_eq!(answer_period().predecessor(0), None);
        assert!(answer_period().gaps(&pkg).is_empty());
    }

    fn date() -> impl Strategy<Value = String> {
        (1990_u32..2030, 1_u32..=12, 1_u32..=28)
            .prop_map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"))
    }

    /// A well-formed set: consecutive versions split at sorted cut dates,
    /// optionally with a final open-ended version.
    fn well_formed() -> impl Strategy<Value = (VersionSet<usize>, Vec<String>)> {
        (proptest::collection::btree_set(date(), 2..7), any::<bool>()).prop_map(|(cuts, open)| {
            let cuts: Vec<String> = cuts.into_iter().collect();
            let mut versions: Vec<Version<usize>> = cuts
                .windows(2)
                .enumerate()
                .map(|(i, w)| Version {
                    in_force: Window::new(w[0].clone(), Some(w[1].clone())),
                    known_at: w[0].clone(),
                    decl: i,
                })
                .collect();
            if open {
                if let Some(last) = cuts.last() {
                    versions.push(Version {
                        in_force: Window::new(last.clone(), None::<String>),
                        known_at: last.clone(),
                        decl: versions.len(),
                    });
                }
            }
            (VersionSet::new(versions), cuts)
        })
    }

    proptest! {
        #[test]
        fn prop_resolution_is_total_and_unique_on_well_formed_sets((set, cuts) in well_formed(), d in date()) {
            prop_assert!(set.overlaps().is_empty());
            let holding: Vec<usize> = set.versions.iter().filter(|v| v.in_force.contains(&d)).map(|v| v.decl).collect();
            prop_assert!(holding.len() <= 1);
            match set.as_of(&d, None) {
                Ok(v) => prop_assert_eq!(vec![v.decl], holding),
                Err(miss) => {
                    prop_assert_eq!(miss, Miss::NotInForce { hidden: vec![] });
                    prop_assert!(holding.is_empty());
                    let first = cuts.first().cloned().unwrap_or_default();
                    let covered_end = set.versions.last().and_then(|v| v.in_force.to.clone());
                    prop_assert!(d < first || covered_end.is_some_and(|e| d >= e));
                }
            }
        }

        #[test]
        fn prop_a_later_non_overlapping_version_never_changes_earlier_answers(
            (set, _) in well_formed(), extra_from in date(), d in date(), known in proptest::option::of(date())
        ) {
            let Some(end) = set.versions.last().and_then(|v| v.in_force.to.clone()) else {
                return Ok(());
            };
            let start = extra_from.max(end);
            let mut grown = set.clone();
            grown.versions.push(Version { in_force: Window::new(start.clone(), None::<String>), known_at: start.clone(), decl: 99 });
            prop_assert!(grown.overlaps().is_empty());
            if d < start {
                let before = set.as_of(&d, known.as_deref()).map(|v| v.decl);
                let after = grown.as_of(&d, known.as_deref()).map(|v| v.decl);
                prop_assert_eq!(before, after);
            }
        }
    }
}
