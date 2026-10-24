//! Version sets of one logical declaration and bitemporal `as_of`
//! selection (SPEC §5.4, DESIGN §4.4; design review §2.6).
//!
//! Versions are the same-named declarations of one package. `in_force` is
//! when the law applied (valid time); `[known_at, retracted)` is when the
//! corpus held it (transaction time): a correction is a new version of the
//! same window recorded later, and the version it corrects is `retracted`
//! then. A query at `(as_of, known_at)` sees only versions the corpus held
//! on `known_at` (the latest corpus: those never retracted) and picks the
//! one whose window holds `as_of`, among those that apply to the
//! proceeding: a version may apply only to proceedings `commenced` in a
//! window (by default the query concerns a proceeding commenced on its
//! `as_of`). Before a declaration's `unknown_before` date its law is not
//! encoded: a query there with no version is a hole, not an absence of
//! law.
use super::interval::{gaps, Window};

/// One version: when it applied, when it was recorded, and its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version<T> {
    /// When the version applied.
    pub in_force: Window,
    /// When the corpus recorded it.
    pub known_at: String,
    /// When the corpus retracted it (a later version corrects it), if ever.
    pub retracted: Option<String>,
    /// The date before which the declaration's law is not encoded, if
    /// the version declares one.
    pub unknown_before: Option<String>,
    /// The commencement dates of the proceedings it applies to, if it
    /// applies only to some.
    pub commenced: Option<Window>,
    /// The declaration this version is.
    pub decl: T,
}

impl<T> Version<T> {
    /// Whether the corpus held this version on `known_at` (`None`: the
    /// latest corpus, which holds every version never retracted).
    #[must_use]
    pub fn held(&self, known_at: Option<&str>) -> bool {
        match known_at {
            Some(k) => {
                self.known_at.as_str() <= k && self.retracted.as_deref().is_none_or(|r| k < r)
            }
            None => self.retracted.is_none(),
        }
    }

    /// Whether the version applies to a proceeding commenced on
    /// `commenced`.
    #[must_use]
    pub fn applies(&self, commenced: &str) -> bool {
        self.commenced
            .as_ref()
            .is_none_or(|w| w.contains(commenced))
    }

    /// Whether no proceeding is one both this version and `other` apply to.
    #[must_use]
    pub fn apply_apart<U>(&self, other: &Version<U>) -> bool {
        match (&self.commenced, &other.commenced) {
            (Some(a), Some(b)) => !a.overlaps(b),
            _ => false,
        }
    }

    /// Whether the corpus never held this version and `other` at once.
    #[must_use]
    pub fn recorded_apart<U>(&self, other: &Version<U>) -> bool {
        let ends_before = |v: Option<&String>, start: &String| v.is_some_and(|end| end <= start);
        ends_before(self.retracted.as_ref(), &other.known_at)
            || ends_before(other.retracted.as_ref(), &self.known_at)
    }
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
    /// The date precedes every version recorded by the query's `known_at`
    /// (L3007, `unknown_before`): the law is not encoded before `earliest`,
    /// a hole in the corpus rather than a finding that no law applied.
    Before {
        /// The earliest recorded version's `from`.
        earliest: String,
    },
    /// More than one version holds the date (an unchecked L3001 overlap).
    Ambiguous(Vec<Window>),
    /// No version holds the date, which is before the date the
    /// declaration's law is encoded from (`unknown_before`, L3010).
    NotEncoded {
        /// The `unknown_before` date.
        before: String,
    },
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
    /// [`Miss::Before`] when `as_of` precedes every recorded version,
    /// [`Miss::NotInForce`] when none applies otherwise, [`Miss::Ambiguous`]
    /// when the set overlaps on that date.
    /// [`Miss::NotEncoded`] when none applies before the declaration's
    /// `unknown_before`, [`Miss::NotInForce`] when none applies otherwise,
    /// [`Miss::Ambiguous`] when the set overlaps on that date.
    pub fn as_of(&self, as_of: &str, known_at: Option<&str>) -> Result<&Version<T>, Miss> {
        self.as_of_in(as_of, known_at, as_of)
    }

    /// [`Self::as_of`] for a proceeding commenced on `commenced`.
    ///
    /// # Errors
    /// As [`Self::as_of`].
    pub fn as_of_in(
        &self,
        as_of: &str,
        known_at: Option<&str>,
        commenced: &str,
    ) -> Result<&Version<T>, Miss> {
        let (visible, hidden): (Vec<&Version<T>>, Vec<&Version<T>>) = self
            .versions
            .iter()
            .filter(|v| v.in_force.contains(as_of) && v.applies(commenced))
            .partition(|v| v.held(known_at));
        match visible.as_slice() {
            [one] => Ok(one),
            [] => match self.unknown_before().filter(|d| as_of < d.as_str()) {
                Some(before) => Err(Miss::NotEncoded { before }),
                None if hidden.is_empty() => Err(self.before(as_of, known_at).map_or_else(
                    || Miss::NotInForce { hidden: Vec::new() },
                    |earliest| Miss::Before { earliest },
                )),
                None => Err(Miss::NotInForce {
                    hidden: hidden.iter().map(|v| v.in_force.display()).collect(),
                }),
            },
            many => Err(Miss::Ambiguous(
                many.iter().map(|v| v.in_force.clone()).collect(),
            )),
        }
    }

    /// The earliest `from` among versions visible at `known_at` before
    /// which the law was not recorded in the corpus.
    fn before(&self, as_of: &str, known_at: Option<&str>) -> Option<String> {
        self.versions
            .iter()
            .filter(|v| known_at.is_none_or(|k| v.known_at.as_str() <= k))
            .map(|v| v.in_force.from.as_str())
            .min()
            .filter(|earliest| as_of < *earliest)
            .map(str::to_owned)
    }

    /// The latest explicit `unknown_before` any version declares.
    #[must_use]
    pub fn unknown_before(&self) -> Option<String> {
        self.versions
            .iter()
            .filter_map(|v| v.unknown_before.clone())
            .max()
    }

    /// The version whose window starts on `from` (the `@date` selector).
    #[must_use]
    pub fn starting(&self, from: &str) -> Option<&Version<T>> {
        self.versions
            .iter()
            .find(|v| v.in_force.from == from && v.held(None))
    }

    /// Index of the version that ends exactly where the one at `index`
    /// begins: the version an amendment replaces, as the latest corpus
    /// holds it.
    #[must_use]
    pub fn predecessor(&self, index: usize) -> Option<usize> {
        let from = &self.versions.get(index)?.in_force.from;
        self.versions
            .iter()
            .position(|v| v.in_force.to.as_ref() == Some(from) && v.held(None))
    }

    /// Every pair of versions whose windows overlap while the corpus held
    /// both, for a proceeding both apply to (`effective-disjoint`:
    /// disjointness is per transaction-time slice and per proceeding, so a
    /// correction may restate the window it corrects and two versions may
    /// split one window by commencement date), ordered by the later
    /// version's position.
    #[must_use]
    pub fn overlaps(&self) -> Vec<Overlap> {
        let mut out = Vec::new();
        for (j, b) in self.versions.iter().enumerate() {
            for (i, a) in self.versions.iter().enumerate().take(j) {
                if a.recorded_apart(b) || a.apply_apart(b) {
                    continue;
                }
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
            .filter(|v| v.held(None))
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
            let cover: Vec<Window> = self
                .versions
                .iter()
                .filter(|v| v.held(None))
                .map(|v| v.in_force.clone())
                .collect();
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
            retracted: None,
            unknown_before: None,
            commenced: None,
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
        // Before the earliest version the law is not encoded
        // (`unknown_before`); with no version recorded yet, nothing is.
        assert_eq!(
            set.as_of("1990-01-01", None),
            Err(Miss::Before {
                earliest: "1993-12-01".into()
            })
        );
        assert_eq!(
            set.as_of("1990-01-01", Some("1990-01-01")),
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
        assert_eq!(answer_period().predecessor(2), None);
        assert!(answer_period().gaps(&pkg).is_empty());
    }

    #[test]
    fn overlaps_order_by_start_not_by_source() {
        let set = VersionSet::new(vec![
            v("2016-12-01", None, "2016-04-28", 21),
            v("2009-12-01", Some("2017-01-01"), "2009-03-26", 20),
        ]);
        assert_eq!(
            set.overlaps(),
            vec![Overlap {
                first: 1,
                second: 0,
                common: Window::new("2016-12-01", Some("2017-01-01")),
            }]
        );
        assert_eq!(set.starting("2009-12-01").map(|v| v.decl), Some(20));
        assert_eq!(set.starting("2010-01-01"), None);
    }

    #[test]
    fn a_correction_restates_its_window_in_a_later_transaction_slice() {
        let mut original = v("2009-12-01", None, "2009-03-26", 1);
        original.retracted = Some("2020-01-01".into());
        let corrected = v("2009-12-01", None, "2020-01-01", 2);
        let set = VersionSet::new(vec![original, corrected]);
        assert!(set.overlaps().is_empty());
        assert_eq!(
            set.as_of("2012-01-01", Some("2015-01-01")).map(|v| v.decl),
            Ok(1)
        );
        assert_eq!(
            set.as_of("2012-01-01", Some("2020-01-01")).map(|v| v.decl),
            Ok(2)
        );
        assert_eq!(set.as_of("2012-01-01", None).map(|v| v.decl), Ok(2));
        assert_eq!(set.starting("2009-12-01").map(|v| v.decl), Some(2));
        assert!(set
            .gaps(&Window::new("2009-12-01", None::<String>))
            .is_empty());
        // Recorded while the original was still held: they overlap.
        let mut early = v("2009-12-01", None, "2010-01-01", 3);
        early.retracted = None;
        let mut held = v("2009-12-01", None, "2009-03-26", 4);
        held.retracted = Some("2020-01-01".into());
        assert_eq!(VersionSet::new(vec![held, early]).overlaps().len(), 1);
    }

    #[test]
    fn before_unknown_before_a_miss_is_a_hole() {
        let mut first = v("2009-12-01", None, "2009-03-26", 1);
        first.unknown_before = Some("2009-12-01".into());
        let set = VersionSet::new(vec![first]);
        assert_eq!(
            set.as_of("2005-01-01", None),
            Err(Miss::NotEncoded {
                before: "2009-12-01".into()
            })
        );
        assert_eq!(set.as_of("2010-01-01", None).map(|v| v.decl), Ok(1));
        assert_eq!(
            set.as_of("2010-01-01", Some("2009-01-01")),
            Err(Miss::NotInForce {
                hidden: vec!["[2009-12-01, ..)".into()]
            })
        );
    }

    #[test]
    fn an_empty_set_has_no_lifetime() {
        let empty = VersionSet::<u32>::default();
        let pkg = Window::new("2009-12-01", None::<String>);
        assert_eq!(empty.lifetime(&pkg), None);
        assert!(empty.gaps(&pkg).is_empty());
        assert_eq!(
            empty.as_of("2010-01-01", None),
            Err(Miss::NotInForce { hidden: vec![] })
        );
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
                    retracted: None,
                    unknown_before: None,
                    commenced: None,
                    decl: i,
                })
                .collect();
            if let (true, Some(last)) = (open, cuts.last()) {
                versions.push(Version {
                    in_force: Window::new(last.clone(), None::<String>),
                    known_at: last.clone(),
                    retracted: None,
                    unknown_before: None,
                    commenced: None,
                    decl: versions.len(),
                });
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
                    prop_assert!(holding.is_empty());
                    let first = cuts.first().cloned().unwrap_or_default();
                    let covered_end = set.versions.last().and_then(|v| v.in_force.to.clone());
                    if d < first {
                        prop_assert_eq!(miss, Miss::Before { earliest: first });
                    } else {
                        prop_assert_eq!(miss, Miss::NotInForce { hidden: vec![] });
                        prop_assert!(covered_end.is_some_and(|e| d >= e));
                    }
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
            grown.versions.push(Version { in_force: Window::new(start.clone(), None::<String>), known_at: start.clone(), retracted: None, unknown_before: None, commenced: None, decl: 99 });
            prop_assert!(grown.overlaps().is_empty());
            if d < start {
                let before = set.as_of(&d, known.as_deref()).map(|v| v.decl);
                let after = grown.as_of(&d, known.as_deref()).map(|v| v.decl);
                prop_assert_eq!(before, after);
            }
        }
    }
}
