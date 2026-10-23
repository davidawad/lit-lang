//! Counting rules roll to an open day from every date (`well-typed`,
//! L3009), decided by Gregorian-cycle enumeration over the lowered clock;
//! a rule that never rolls fails its tests with that reason instead of
//! counting forever.
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use support::{edit, outcome, verify_one};

const CALENDAR: &str = "  new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday
  juneteenth: fixed(6, 19) effective [2021-06-17, ..) observed nearest_weekday
";

/// `BASE` with its calendar's holidays replaced by `holidays`.
fn calendar(holidays: &str) -> String {
    edit(CALENDAR, holidays)
}

fn messages(lit: &str, name: Obligation) -> (Option<ObligationStatus>, Vec<String>) {
    let checked = verify_one(lit, Profile::Strict);
    let found = outcome(&checked, name);
    let status = found.finding.map(|f| f.status);
    let messages = checked
        .verification
        .diagnostics
        .iter()
        .filter(|d| litigate_verify::obligation_of(d.code) == Some(name))
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    (status, messages)
}

#[test]
fn the_shipped_kind_of_calendar_rolls() {
    let (status, found) = messages(&calendar(CALENDAR), Obligation::WellTyped);
    assert_eq!((status, found), (Some(ObligationStatus::Passed), vec![]));
}

#[test]
fn a_calendar_closing_every_day_is_rejected_and_never_counted() {
    let days: String = (1..=12u32)
        .flat_map(|m| {
            let last = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m as usize - 1];
            (1..=last)
                .map(move |d| format!("  d{m}-{d}: fixed({m}, {d}) effective [1870-06-28, ..)\n"))
        })
        .collect();
    let lit = calendar(&days);
    let (status, found) = messages(&lit, Obligation::WellTyped);
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("L3009 counting rule `frcp6` never rolls to an open day from "),
        "{found:?}"
    );
    let (status, found) = messages(&lit, Obligation::TestsPass);
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert!(
        found
            .iter()
            .all(|m| m.contains("never rolls to an open day")),
        "{found:?}"
    );
}

#[test]
fn holiday_cycles_beyond_the_enumerated_period_are_not_decided() {
    let lit = calendar(
        "  a: every(99, 2001, 1, 2) effective [1870-06-28, ..)
  b: every(97, 2001, 1, 3) effective [1870-06-28, ..)
  c: every(89, 2001, 1, 4) effective [1870-06-28, ..)
",
    );
    let (status, found) = messages(&lit, Obligation::WellTyped);
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert_eq!(
        found,
        ["L3009 the holiday cycles of counting rule `frcp6` combine to a period longer than 40000 years"]
    );
    // Undecided is not endless: the tests still count.
    assert_eq!(
        messages(&lit, Obligation::TestsPass).0,
        Some(ObligationStatus::Passed)
    );
}
