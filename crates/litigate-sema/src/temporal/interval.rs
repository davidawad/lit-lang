//! Half-open date windows `[from, to)` (SPEC §5.4).
//!
//! Dates are validated ISO `YYYY-MM-DD` text, so lexicographic order is
//! chronological order and no conversion is needed to compare them. `to =
//! None` is open-ended.
use crate::hir::Interval;
use std::cmp::Ordering;

/// A half-open in-force window; `to = None` never ends.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Window {
    /// Inclusive first day.
    pub from: String,
    /// Exclusive end; `None` is open-ended.
    pub to: Option<String>,
}

impl Window {
    /// `[from, to)`.
    #[must_use]
    pub fn new(from: impl Into<String>, to: Option<impl Into<String>>) -> Self {
        Self {
            from: from.into(),
            to: to.map(Into::into),
        }
    }

    /// The in-force part of a HIR interval.
    #[must_use]
    pub fn of(interval: &Interval) -> Self {
        Self {
            from: interval.from.clone(),
            to: interval.to.clone(),
        }
    }

    /// True when the window holds no day.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to
            .as_deref()
            .is_some_and(|to| to <= self.from.as_str())
    }

    /// True when `date` lies in the window.
    #[must_use]
    pub fn contains(&self, date: &str) -> bool {
        self.from.as_str() <= date && self.to.as_deref().is_none_or(|to| date < to)
    }

    /// The common part of two windows, if any.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let from = self.from.clone().max(other.from.clone());
        let to = min_end(self.to.as_deref(), other.to.as_deref()).map(str::to_owned);
        let w = Self { from, to };
        (!w.is_empty()).then_some(w)
    }

    /// True when the windows share at least one day.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.intersect(other).is_some()
    }

    /// True when every day of `self` lies in `other`.
    #[must_use]
    pub fn within(&self, other: &Self) -> bool {
        self.is_empty()
            || (other.from <= self.from && cmp_end(self.to.as_deref(), other.to.as_deref()).is_le())
    }

    /// Length in days; `None` for an open-ended window.
    #[must_use]
    pub fn days(&self) -> Option<i64> {
        let to = self.to.as_deref()?;
        Some(day_number(to)? - day_number(&self.from)?)
    }

    /// Source spelling, `[from, to)` or `[from, ..)`.
    #[must_use]
    pub fn display(&self) -> String {
        format!("[{}, {})", self.from, self.to.as_deref().unwrap_or(".."))
    }
}

/// Orders window ends; `None` (open) is latest.
#[must_use]
pub fn cmp_end(a: Option<&str>, b: Option<&str>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => a.cmp(b),
    }
}

fn min_end<'a>(a: Option<&'a str>, b: Option<&'a str>) -> Option<&'a str> {
    if cmp_end(a, b).is_le() {
        a
    } else {
        b
    }
}

/// Parts of `span` that no window of `cover` contains, in order.
#[must_use]
pub fn gaps(span: &Window, cover: &[Window]) -> Vec<Window> {
    let mut parts: Vec<Window> = cover.iter().filter_map(|w| w.intersect(span)).collect();
    parts.sort();
    let mut out = Vec::new();
    let mut cursor = Some(span.from.clone());
    for w in parts {
        let Some(at) = cursor.clone() else {
            break;
        };
        if w.from > at {
            out.push(Window::new(at.clone(), Some(w.from.clone())));
        }
        if cmp_end(w.to.as_deref(), Some(&at)).is_gt() {
            cursor.clone_from(&w.to);
        }
    }
    if let Some(at) = cursor {
        let tail = Window {
            from: at,
            to: span.to.clone(),
        };
        if !tail.is_empty() {
            out.push(tail);
        }
    }
    out
}

/// Days since 0000-03-01 for a valid ISO date (proleptic Gregorian).
#[must_use]
pub fn day_number(date: &str) -> Option<i64> {
    let (y, m, d) = crate::types::date::parse(date)?;
    let (y, m, d) = (i64::from(y), i64::from(m), i64::from(d));
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe)
}

#[cfg(kani)]
mod proofs;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn w(from: &str, to: Option<&str>) -> Window {
        Window::new(from, to)
    }

    #[test]
    fn half_open_membership_and_intersection() {
        let a = w("2000-01-01", Some("2009-12-01"));
        assert!(a.contains("2000-01-01"));
        assert!(a.contains("2009-11-30"));
        assert!(!a.contains("2009-12-01"));
        let b = w("2009-12-01", None);
        assert!(!a.overlaps(&b), "adjacent windows share no day");
        assert_eq!(
            w("2005-01-01", None).intersect(&a),
            Some(w("2005-01-01", Some("2009-12-01")))
        );
        assert!(w("2001-01-01", Some("2002-01-01")).within(&a));
        assert!(!b.within(&a));
        assert!(w("2001-01-01", Some("2001-01-01")).is_empty());
    }

    #[test]
    fn gaps_list_uncovered_days_in_order() {
        let span = w("2000-01-01", None);
        let cover = [
            w("2001-01-01", Some("2002-01-01")),
            w("2003-01-01", None),
            w("1990-01-01", Some("2000-06-01")),
        ];
        assert_eq!(
            gaps(&span, &cover),
            vec![
                w("2000-06-01", Some("2001-01-01")),
                w("2002-01-01", Some("2003-01-01"))
            ]
        );
        assert_eq!(gaps(&span, &[]), vec![span.clone()]);
        assert!(gaps(&span, &[w("1999-01-01", None)]).is_empty());
        let nested = [
            w("2000-01-01", Some("2010-01-01")),
            w("2001-01-01", Some("2002-01-01")),
        ];
        assert_eq!(gaps(&span, &nested), vec![w("2010-01-01", None)]);
        let after_open = [w("2000-01-01", None), w("2005-01-01", Some("2006-01-01"))];
        assert!(gaps(&span, &after_open).is_empty());
    }

    #[test]
    fn within_needs_every_day_inside() {
        let outer = w("2000-01-01", Some("2010-01-01"));
        assert!(w("2001-01-01", Some("2002-01-01")).within(&outer));
        assert!(
            w("2020-01-01", Some("2020-01-01")).within(&outer),
            "an empty window is within anything"
        );
        assert!(!w("1999-01-01", Some("2002-01-01")).within(&outer));
        assert!(!w("2001-01-01", Some("2011-01-01")).within(&outer));
        assert!(!w("2001-01-01", None).within(&outer));
        assert!(w("2001-01-01", None).within(&w("2000-01-01", None)));
    }

    #[test]
    fn invalid_dates_have_no_day_number() {
        assert_eq!(day_number("2009-02-30"), None);
        assert_eq!(w("2009-12-01", Some("not-a-date")).days(), None);
        assert_eq!(w("not-a-date", Some("2009-12-01")).days(), None);
        assert_eq!(
            w("2009-12-01", Some("2009-12-01")).display(),
            "[2009-12-01, 2009-12-01)"
        );
    }

    #[test]
    fn day_numbers_count_real_days() {
        let n = |d| day_number(d).unwrap_or_default();
        assert_eq!(n("2009-12-01") - n("2009-11-30"), 1);
        assert_eq!(n("2001-01-01") - n("2000-01-01"), 366);
        assert_eq!(n("1900-03-01") - n("1900-02-28"), 1);
        assert_eq!(w("2009-12-01", Some("2009-12-22")).days(), Some(21));
        assert_eq!(w("2009-12-01", None).days(), None);
    }

    fn date() -> impl Strategy<Value = String> {
        (1990_u32..2030, 1_u32..=12, 1_u32..=28)
            .prop_map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"))
    }

    fn window() -> impl Strategy<Value = Window> {
        (date(), proptest::option::of(date())).prop_map(|(a, b)| match b {
            Some(b) if b < a => Window::new(b, Some(a)),
            b => Window::new(a, b),
        })
    }

    proptest! {
        #[test]
        fn prop_intersection_is_exactly_common_membership(a in window(), b in window(), d in date()) {
            let both = a.contains(&d) && b.contains(&d);
            prop_assert_eq!(a.intersect(&b).is_some_and(|i| i.contains(&d)), both);
        }

        #[test]
        fn prop_gaps_are_exactly_the_uncovered_days(span in window(), cover in proptest::collection::vec(window(), 0..5), d in date()) {
            let uncovered = span.contains(&d) && !cover.iter().any(|c| c.contains(&d));
            prop_assert_eq!(gaps(&span, &cover).iter().any(|g| g.contains(&d)), uncovered);
        }
    }
}
