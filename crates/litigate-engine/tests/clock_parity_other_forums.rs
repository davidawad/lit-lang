// SPDX-License-Identifier: GPL-3.0-or-later
//! Parity/known-answer cases for the forums the TS engine never modeled as
//! a single unit: `federalHolidays`/`isLegalHoliday` re-export parity
//! (still FRCP, still from `clock.test.ts`), then known-answer cases for
//! `RCFC 6`, `FRAP 26`, and `19 CFR 210.6(a)` (ITC). Split out of
//! `clock_parity.rs` to keep both files under the repo's 500-line cap.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::clock::{
    compute_due_date, Clock, ClockConfig, ClockLibrary, Date, DayDeadline, Direction,
    ServiceMethod, Unit,
};

/// A shipped counting rule from the embedded clock library (`lit/core` lowered).
fn rule(id: &str) -> Clock<'static> {
    ClockLibrary::embedded().0.clock(id).unwrap()
}

fn forward_frcp(trigger: &str, days: i64) -> String {
    compute_due_date(
        &DayDeadline {
            trigger: Date::parse_iso(trigger).unwrap(),
            days,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: None,
        },
        &rule("frcp6"),
        &ClockConfig::default(),
    )
    .unwrap()
    .due_date
}

// ---------------------------------------------------------------------------
// federalHolidays / isLegalHoliday parity (still FRCP, from clock.test.ts)
// ---------------------------------------------------------------------------

#[test]
fn christmas_2026_is_a_legal_holiday() {
    assert!(rule("frcp6").is_legal_holiday(Date::parse_iso("2026-12-25").unwrap(), &[]));
}

#[test]
fn a_random_weekday_is_not_a_legal_holiday() {
    assert!(!rule("frcp6").is_legal_holiday(Date::parse_iso("2026-03-16").unwrap(), &[]));
}

#[test]
fn federal_holidays_returns_eleven_for_a_year() {
    assert_eq!(rule("frcp6").holidays_in(2026).len(), 11);
}

// ---------------------------------------------------------------------------
// Known-answer cases beyond the TS engine's scope: RCFC 6, FRAP 26, ITC 210.
// ---------------------------------------------------------------------------

#[test]
fn rcfc_ordinary_case_matches_frcp() {
    let frcp = forward_frcp("2026-07-01", 14);
    let rcfc = compute_due_date(
        &DayDeadline {
            trigger: Date::parse_iso("2026-07-01").unwrap(),
            days: 14,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: None,
        },
        &rule("rcfc6"),
        &ClockConfig::default(),
    )
    .unwrap()
    .due_date;
    assert_eq!(frcp, rcfc);
}

#[test]
fn frap_notice_of_appeal_thirty_days() {
    // Fed. R. App. P. 4(a)(1)(A): 30 days from entry of judgment. Uses
    // FRAP 26(a) day mechanics (identical to FRCP's).
    let r = compute_due_date(
        &DayDeadline {
            trigger: Date::parse_iso("2026-01-05").unwrap(),
            days: 30,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: None,
        },
        &rule("frap26"),
        &ClockConfig::default(),
    )
    .unwrap();
    assert_eq!(r.due_date, "2026-02-04");
}

#[test]
fn frap_service_by_commercial_carrier_adds_three_days() {
    // Base (no service) is 2026-02-04 (a Wednesday); +3 raw = 2026-02-07,
    // a Saturday, so it rolls forward to Monday 2026-02-09.
    let r = compute_due_date(
        &DayDeadline {
            trigger: Date::parse_iso("2026-01-05").unwrap(),
            days: 30,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: Some(ServiceMethod::CommercialCarrier),
        },
        &rule("frap26"),
        &ClockConfig::default(),
    )
    .unwrap();
    assert_eq!(r.due_date, "2026-02-09");
    assert!(r.service_days_added);
    assert!(r.last_day_rolled);
}

#[test]
fn itc_response_period_twenty_days() {
    // 19 CFR 210.13(a): 20 days from service of the complaint. Trigger a
    // Monday so the ITC business-day anchor and FRCP's calendar-day anchor
    // coincide (both fall on the next calendar day), isolating the >= 7 day
    // general rule's forward-roll-only behavior.
    let r = compute_due_date(
        &DayDeadline {
            trigger: Date::parse_iso("2026-06-01").unwrap(),
            days: 20,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: None,
        },
        &rule("itc210"),
        &ClockConfig::default(),
    )
    .unwrap();
    assert_eq!(r.due_date, "2026-06-22");
}
