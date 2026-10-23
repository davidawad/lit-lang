// SPDX-License-Identifier: GPL-3.0-or-later
//! `Clock::closed_run`: the longest run of closed days over every date,
//! decided by Gregorian-cycle enumeration (one 400-year cycle on either
//! side of every effective-date boundary), not by sampling.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::clock::cycle::{ClosedRun, CYCLE_DAYS};
use litgraph::clock::rule::CountingRule;
use litgraph::clock::{Clock, ClockLibrary, Date, HolidayCalendar};

/// Every shipped rule's closed runs end; the witness is a real maximal run
/// (each day closed, both neighbours open), and no run in a 60-year window
/// counted day by day through the interpreter's own `is_closed` is longer.
#[test]
fn every_shipped_rule_rolls_to_an_open_day_from_every_date() {
    let library = &ClockLibrary::embedded().0;
    assert!(!library.counting_rules.is_empty());
    for rule in &library.counting_rules {
        let clock = library.clock(&rule.id).unwrap();
        let ClosedRun::Bounded { longest, from } = clock.closed_run() else {
            panic!("{}: {:?}", rule.id, clock.closed_run());
        };
        assert!((2..=5).contains(&longest), "{}: {longest}", rule.id);
        assert!(!clock.is_closed(from.add_days(-1), &[]), "{}", rule.id);
        assert!(!clock.is_closed(from.add_days(longest), &[]), "{}", rule.id);
        assert!(
            (0..longest).all(|i| clock.is_closed(from.add_days(i), &[])),
            "{}",
            rule.id
        );
        assert!(
            brute_longest(&clock, "1990-01-01", 60 * 366) <= longest,
            "{}",
            rule.id
        );
    }
}

fn brute_longest(clock: &Clock<'_>, from: &str, days: i64) -> i64 {
    let start = Date::parse_iso(from).unwrap();
    let (mut run, mut best) = (0, 0);
    for i in 0..days {
        run = if clock.is_closed(start.add_days(i), &[]) {
            run + 1
        } else {
            0
        };
        best = best.max(run);
    }
    best
}

/// A copy of the shipped `frcp6` rule closed on `closed` weekdays and the
/// holidays `holidays` (JSON objects).
struct Custom {
    rule: CountingRule,
    calendar: HolidayCalendar,
}

impl Custom {
    fn clock(&self) -> Clock<'_> {
        Clock {
            rule: &self.rule,
            calendars: vec![&self.calendar],
            forward_calendars: Vec::new(),
        }
    }
}

fn library(holidays: &str, closed: &str) -> Custom {
    let mut rule = ClockLibrary::embedded()
        .0
        .clock("frcp6")
        .unwrap()
        .rule
        .clone();
    rule.closed_weekdays = serde_json::from_str(&format!("[{closed}]")).unwrap();
    let calendar = serde_json::from_str(&format!(
        r#"{{"id": "k", "title": "K", "holidays": [{holidays}]}}"#
    ))
    .unwrap();
    Custom { rule, calendar }
}

fn fixed(id: &str, month: u32, day: u32, extra: &str) -> String {
    format!(
        r#"{{"id": "{id}", "name": "{id}", "date": {{"fixed": {{"month": {month}, "day": {day}}}}}{extra}}}"#
    )
}

#[test]
fn a_rule_closing_every_weekday_never_rolls() {
    let lib = library(
        "",
        r#""sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday""#,
    );
    let clock = lib.clock();
    assert!(matches!(clock.closed_run(), ClosedRun::Unbounded { .. }));
}

#[test]
fn holidays_next_to_a_weekend_lengthen_the_run() {
    // Fixed holidays on 24, 25 and 26 December beside a weekend: in some
    // year of the cycle they meet Saturday and Sunday on both sides.
    let days = [24, 25, 26]
        .iter()
        .map(|d| fixed(&format!("h{d}"), 12, *d, ""))
        .collect::<Vec<_>>()
        .join(",");
    let lib = library(&days, r#""saturday", "sunday""#);
    let clock = lib.clock();
    assert!(matches!(
        clock.closed_run(),
        ClosedRun::Bounded { longest: 5, .. }
    ));
    // No calendar day closed at all: the longest run is 0.
    let lib = library("", "");
    assert!(matches!(
        lib.clock().closed_run(),
        ClosedRun::Bounded { longest: 0, .. }
    ));
}

#[test]
fn an_effective_window_is_enumerated_between_its_boundaries() {
    // Every day of 2030 is a holiday (366 fixed dates in force for that
    // year only); with no weekday closed, the longest run is the year
    // itself, found between the boundaries, not in a periodic tail.
    let mut days = Vec::new();
    for month in 1..=12u32 {
        let last = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month as usize - 1];
        for day in 1..=last {
            days.push(fixed(
                &format!("d{month}-{day}"),
                month,
                day,
                r#", "effective": {"from": "2030-01-01", "until": "2031-01-01"}"#,
            ));
        }
    }
    let lib = library(&days.join(","), "");
    let ClosedRun::Bounded { longest, from } = lib.clock().closed_run() else {
        panic!("bounded");
    };
    assert_eq!((longest, from.to_iso().as_str()), (365, "2030-01-01"));
}

#[test]
fn a_cycle_longer_than_the_bound_is_not_decided() {
    let every = |id: &str, k: u32| {
        fixed(
            id,
            1,
            2,
            &format!(r#", "cycle": {{"every": {k}, "anchorYear": 2001}}"#),
        )
    };
    let lib = library(
        &[every("a", 99), every("b", 97), every("c", 89)].join(","),
        "",
    );
    assert_eq!(lib.clock().closed_run(), ClosedRun::Undecided);
    // One 100-year cycle is decided: lcm(400, 100) = 400.
    let lib = library(&every("a", 100), "");
    assert!(matches!(
        lib.clock().closed_run(),
        ClosedRun::Bounded { longest: 1, .. }
    ));
}

#[test]
fn the_cycle_is_a_whole_number_of_weeks() {
    let a = Date::parse_iso("2001-01-01").unwrap();
    let b = Date::parse_iso("2401-01-01").unwrap();
    assert_eq!(b.epoch_day() - a.epoch_day(), CYCLE_DAYS);
    assert_eq!(CYCLE_DAYS % 7, 0);
    assert_eq!(a.weekday(), b.weekday());
}

#[test]
fn a_forward_only_calendar_lengthens_the_run() {
    // A forward-only holiday on Friday 2027-01-01 (FRCP 6(a)(6)(C)) meets
    // the weekend after it: a period counted forward can roll 3 days, so
    // the run is decided over the forward closed set.
    let lib = library("", r#""saturday", "sunday""#);
    let forward: HolidayCalendar = serde_json::from_str(&format!(
        r#"{{"id": "state", "title": "State", "holidays": [{}]}}"#,
        fixed("new-year", 1, 1, "")
    ))
    .unwrap();
    let clock = Clock {
        forward_calendars: vec![&forward],
        ..lib.clock()
    };
    assert!(matches!(
        clock.closed_run(),
        ClosedRun::Bounded { longest: 3, .. }
    ));
    assert!(matches!(
        lib.clock().closed_run(),
        ClosedRun::Bounded { longest: 2, .. }
    ));
}
