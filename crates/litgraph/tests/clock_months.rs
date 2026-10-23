// SPDX-License-Identifier: GPL-3.0-or-later
//! Calendar periods (design review 2026-10-23 §2.1): weeks are seven
//! calendar days; months and years run to the corresponding day of the
//! later month under the rule's `months` provision and its end-of-month
//! policy (MPEP 710.01(a) for `clamp`), then roll like any period. The
//! result reports the nominal (as-counted) date beside the adjusted one.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::clock::rule::{EndOfMonth, Months};
use litgraph::clock::{
    compute_due_date, Clock, ClockConfig, ClockLibrary, ComputedDeadline, CountingRule, Date,
    DayDeadline, Direction, Unit,
};
use litgraph::model::Deadline;

fn rule(id: &str) -> CountingRule {
    ClockLibrary::embedded()
        .0
        .counting_rules
        .iter()
        .find(|r| r.id == id)
        .unwrap()
        .clone()
}

fn count(
    rule: &CountingRule,
    trigger: &str,
    n: i64,
    unit: Unit,
    direction: Direction,
) -> litgraph::Result<ComputedDeadline> {
    let library = &ClockLibrary::embedded().0;
    let calendars = rule
        .holidays
        .iter()
        .map(|h| library.calendars.iter().find(|c| &c.id == h).unwrap())
        .collect();
    let clock = Clock {
        rule,
        calendars,
        forward_calendars: Vec::new(),
    };
    let dl = DayDeadline {
        trigger: Date::parse_iso(trigger).unwrap(),
        days: n,
        direction,
        unit,
        service_method: None,
    };
    compute_due_date(&dl, &clock, &ClockConfig::default())
}

fn due(rule: &CountingRule, trigger: &str, n: i64, unit: Unit) -> (String, String) {
    let c = count(rule, trigger, n, unit, Direction::Forward).unwrap();
    (c.nominal_date, c.due_date)
}

fn pair(nominal: &str, adjusted: &str) -> (String, String) {
    (nominal.to_owned(), adjusted.to_owned())
}

#[test]
fn months_run_to_the_corresponding_day() {
    let r = rule("rcfc6");
    // 35 U.S.C. § 321(c): 9 months from 2025-03-04, not 270 days.
    assert_eq!(
        due(&r, "2025-03-04", 9, Unit::Month),
        pair("2025-12-04", "2025-12-04")
    );
    // MPEP 710.01(a): from November 30, three months end on February 28.
    assert_eq!(
        due(&r, "2024-11-30", 3, Unit::Month),
        pair("2025-02-28", "2025-02-28")
    );
    // A year from February 29 ends on February 28.
    assert_eq!(
        due(&r, "2024-02-29", 1, Unit::Year),
        pair("2025-02-28", "2025-02-28")
    );
    assert_eq!(
        due(&r, "2025-05-31", 9, Unit::Month),
        pair("2026-02-28", "2026-03-02")
    );
    let c = count(&r, "2025-03-04", 2, Unit::Year, Direction::Forward).unwrap();
    assert_eq!(
        c.steps[1],
        "RCFC 6: count 2 years after the anchor to the corresponding day, else the last day of \
         that month -> 2027-03-04"
    );
    let c = count(&r, "2025-03-04", 1, Unit::Month, Direction::Forward).unwrap();
    assert!(c.steps[1].contains("count 1 month after"), "{:?}", c.steps);
    let c = count(&r, "2025-03-04", 1, Unit::Year, Direction::Forward).unwrap();
    assert!(c.steps[1].contains("count 1 year after"), "{:?}", c.steps);
}

#[test]
fn a_rolling_policy_moves_to_the_next_month() {
    let mut r = rule("frcp6");
    r.months = Some(Months {
        end_of_month: EndOfMonth::Roll,
        text: "count {n} {unit} -> {date}".into(),
    });
    assert_eq!(
        due(&r, "2025-01-31", 1, Unit::Month),
        pair("2025-03-01", "2025-03-03")
    );
    assert_eq!(
        due(&r, "2025-01-15", 1, Unit::Month),
        pair("2025-02-15", "2025-02-18")
    );
}

#[test]
fn backward_months_count_earlier() {
    let r = rule("frcp6");
    let c = count(&r, "2025-03-31", 1, Unit::Month, Direction::Backward).unwrap();
    assert_eq!(
        (c.nominal_date.as_str(), c.due_date.as_str()),
        ("2025-02-28", "2025-02-28")
    );
}

#[test]
fn weeks_are_seven_calendar_days() {
    let r = rule("frcp6");
    let c = count(&r, "2025-03-03", 2, Unit::Week, Direction::Forward).unwrap();
    assert_eq!(c.due_date, "2025-03-17");
    assert!(c.steps[1].contains("14 calendar day(s)"), "{:?}", c.steps);
}

#[test]
fn month_periods_need_a_provision_and_a_trigger_anchor() {
    let frap26 = rule("frap26");
    let err = count(&frap26, "2025-03-03", 1, Unit::Month, Direction::Forward).unwrap_err();
    assert!(
        err.to_string().contains("no calendar-month provision"),
        "{err}"
    );
    let mut itc = rule("itc210");
    itc.months = rule("frcp6").months;
    let err = count(&itc, "2025-03-03", 1, Unit::Year, Direction::Forward).unwrap_err();
    assert!(
        err.to_string().contains("counts from the trigger date"),
        "{err}"
    );
    let r = rule("frcp6");
    let err = count(&r, "2025-03-03", i64::MAX, Unit::Year, Direction::Forward).unwrap_err();
    assert!(err.to_string().contains("out of range"), "{err}");
    let err = count(&r, "2025-03-03", i64::MAX, Unit::Month, Direction::Forward).unwrap_err();
    assert!(err.to_string().contains("out of range"), "{err}");
}

#[test]
fn a_deadline_reads_its_length_in_days() {
    let days = |length, unit: Option<&str>| {
        Deadline {
            length,
            unit: unit.map(str::to_owned),
            ..Deadline::default()
        }
        .days()
    };
    assert_eq!(days(21.0, None), 21.0);
    assert_eq!(days(5.0, Some("court")), 5.0);
    assert_eq!(days(2.0, Some("week")), 14.0);
    assert_eq!(days(12.0, Some("month")), 365.2425);
    assert_eq!(days(2.0, Some("year")), 730.485);
}
