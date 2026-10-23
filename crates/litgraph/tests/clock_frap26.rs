// SPDX-License-Identifier: GPL-3.0-or-later
//! The shipped `frap26` counting rule against the vendored FRAP 26 text
//! (`sources/frap.txt`, "## FRAP 26"): hours periods (26(a)(2)) and the
//! inaccessible clerk's office (26(a)(3)), one test per clause.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::clock::{
    compute_due_date, compute_hour_deadline, Clock, ClockConfig, ClockLibrary,
    ComputedHourDeadline, Date, DayDeadline, Direction, HourDeadline, ServiceMethod, Unit,
};

fn frap26() -> Clock<'static> {
    ClockLibrary::embedded().0.clock("frap26").unwrap()
}

fn d(s: &str) -> Date {
    Date::parse_iso(s).unwrap()
}

fn closed() -> ClockConfig {
    ClockConfig {
        clerk_inaccessible: true,
        ..ClockConfig::default()
    }
}

/// `hours` from `date` at `hour`:00.
fn hours(date: &str, hour: i64, hours: i64, cfg: &ClockConfig) -> ComputedHourDeadline {
    let dl = HourDeadline {
        trigger_epoch_seconds: d(date).epoch_day() * 86_400 + hour * 3_600,
        hours,
    };
    compute_hour_deadline(&dl, &frap26(), cfg).unwrap()
}

fn days(
    trigger: &str,
    n: i64,
    direction: Direction,
    service_method: Option<ServiceMethod>,
    cfg: &ClockConfig,
) -> litgraph::clock::ComputedDeadline {
    let dl = DayDeadline {
        trigger: d(trigger),
        days: n,
        direction,
        unit: Unit::Calendar,
        service_method,
    };
    compute_due_date(&dl, &frap26(), cfg).unwrap()
}

#[test]
fn hours_begin_immediately_and_count_every_hour() {
    // 26(a)(2)(A)-(B): Wed 2026-07-22 15:00 + 48 h = Fri 15:00, and the
    // intermediate weekend hours count: Thu 2026-07-23 09:00 + 96 h = Mon
    // 09:00.
    let r = hours("2026-07-22", 15, 48, &ClockConfig::default());
    assert_eq!(r.due_datetime, "2026-07-24T15:00:00");
    assert!(!r.last_day_rolled);
    assert!(r.steps[0].contains("26(a)(2)(A)-(B)"), "{:?}", r.steps);
    let r = hours("2026-07-23", 9, 96, &ClockConfig::default());
    assert_eq!(r.due_datetime, "2026-07-27T09:00:00");
    assert!(!r.last_day_rolled);
}

#[test]
fn an_hours_period_ending_on_a_closed_day_runs_to_the_same_time_next_open_day() {
    // 26(a)(2)(C): Fri 2026-07-24 12:00 + 24 h ends Saturday -> Mon 12:00.
    let r = hours("2026-07-24", 12, 24, &ClockConfig::default());
    assert_eq!(r.due_datetime, "2026-07-27T12:00:00");
    assert!(r.last_day_rolled);
    assert!(r.steps[1].contains("26(a)(2)(C)"), "{:?}", r.steps);
    // A legal holiday: Thu 2026-12-24 10:00 + 24 h ends on Christmas (Fri),
    // then the weekend -> Mon 2026-12-28 10:00.
    let r = hours("2026-12-24", 10, 24, &ClockConfig::default());
    assert_eq!(r.due_datetime, "2026-12-28T10:00:00");
}

#[test]
fn inaccessible_during_the_last_hour_extends_to_the_same_time_next_accessible_day() {
    // 26(a)(3)(B): due Mon 2026-07-27 12:00; inaccessible -> Tue 12:00.
    let r = hours("2026-07-24", 12, 24, &closed());
    assert_eq!(r.due_datetime, "2026-07-28T12:00:00");
    assert!(r.clerk_inaccessibility_applied);
    assert!(r.steps.last().unwrap().contains("26(a)(3)(B)"));
    // Not a Saturday, Sunday, or legal holiday: due Fri 2026-07-24 15:00
    // -> Mon 07-27 15:00.
    let r = hours("2026-07-22", 15, 48, &closed());
    assert_eq!(r.due_datetime, "2026-07-27T15:00:00");
    // A declared closure the request adds is skipped too.
    let cfg = ClockConfig {
        additional_holidays: vec![d("2026-07-27")],
        clerk_inaccessible: true,
    };
    assert_eq!(
        hours("2026-07-22", 15, 48, &cfg).due_datetime,
        "2026-07-28T15:00:00"
    );
    assert!(!hours("2026-07-22", 15, 48, &ClockConfig::default()).clerk_inaccessibility_applied);
}

#[test]
fn inaccessible_on_the_last_day_extends_to_the_first_accessible_open_day() {
    // 26(a)(3)(A): 14 days from Mon 2026-07-20 end Mon 08-03 -> Tue 08-04.
    let r = days("2026-07-20", 14, Direction::Forward, None, &closed());
    assert_eq!(r.due_date, "2026-08-04");
    assert!(r.clerk_inaccessibility_applied);
    assert!(r.steps.last().unwrap().contains("26(a)(3)(A)"));
    // Last day Fri 2026-07-31 (11 days): the next open day is Mon 08-03.
    assert_eq!(
        days("2026-07-20", 11, Direction::Forward, None, &closed()).due_date,
        "2026-08-03"
    );
    // Last day Thu 2026-12-24: Christmas and the weekend -> Mon 12-28.
    assert_eq!(
        days("2026-12-10", 14, Direction::Forward, None, &closed()).due_date,
        "2026-12-28"
    );
}

#[test]
fn inaccessibility_follows_the_26c_extension_onto_the_true_last_day() {
    // 30 days from 2026-01-05 end Wed 02-04; 26(c) mail +3 = Sat 02-07 ->
    // Mon 02-09 (26(a)(1)(C)); inaccessible that day -> Tue 02-10.
    let r = days(
        "2026-01-05",
        30,
        Direction::Forward,
        Some(ServiceMethod::Mail),
        &closed(),
    );
    assert_eq!(r.base_date, "2026-02-04");
    assert_eq!(r.due_date, "2026-02-10");
    assert!(r.service_days_added && r.last_day_rolled && r.clerk_inaccessibility_applied);
}

#[test]
fn inaccessibility_extends_a_backward_period_to_the_first_accessible_day() {
    let r = days("2026-07-27", 2, Direction::Backward, None, &closed());
    // 26(a)(3)(A) says the first accessible day, without limiting the
    // extension to periods measured after an event.
    assert_eq!(r.due_date, "2026-07-27");
    assert!(r.clerk_inaccessibility_applied);
    assert!(r.steps.last().unwrap().contains("26(a)(3)(A)"));
}

#[test]
fn without_inaccessibility_day_counts_are_unchanged() {
    let open = ClockConfig::default();
    let r = days("2026-07-20", 14, Direction::Forward, None, &open);
    assert_eq!(r.due_date, "2026-08-03");
    assert!(!r.clerk_inaccessibility_applied);
    assert_eq!(r.steps.len(), 2);
}
