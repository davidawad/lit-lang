//! A small independent oracle for deadline computation (SPEC §8.2;
//! design review 2026-10-23 §3 item 7, translation validation).
//!
//! Generated counting rules vary the worked example's holiday calendar;
//! each is lowered to clock IR and counted by the engine. The oracle
//! counts the same period over plain day numbers: FRCP 6(a)(1) excludes
//! the trigger day and rolls a closed last day in the counting direction,
//! and the 6(d) mail days are added after that roll and rolled again. It
//! never sees HIR, the lowered fragment or the engine clock.
mod support;

use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{self, DayUnit, Direction};
use proptest::prelude::*;
use support::{calendar_mut, counting_mut, example, render_diagnostics};

/// A generated holiday: `fixed(month, day)`, observed on the nearest
/// weekday when `observed`.
#[derive(Clone, Debug)]
struct Holiday {
    month: u8,
    day: u8,
    observed: bool,
}

/// One generated period.
#[derive(Clone, Debug)]
struct Period {
    trigger: i64,
    amount: u64,
    court_days: bool,
    backward: bool,
    mail: bool,
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Hinnant's
/// `days_from_civil`).
fn days(year: i64, month: u8, day: u8) -> i64 {
    let (month, day) = (i64::from(month), i64::from(day));
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of day number `z` (Hinnant's `civil_from_days`).
fn civil(z: i64) -> (i64, u8, u8) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        u8::try_from(month).unwrap_or_default(),
        u8::try_from(day).unwrap_or_default(),
    )
}

fn iso(z: i64) -> String {
    let (year, month, day) = civil(z);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Saturday or Sunday (1970-01-01 was a Thursday).
fn weekend(z: i64) -> bool {
    matches!((z + 4).rem_euclid(7), 0 | 6)
}

/// Whether the forum is closed on day `z` under `holidays`.
fn closed(holidays: &[Holiday], z: i64) -> bool {
    let (year, _, _) = civil(z);
    weekend(z)
        || holidays.iter().any(|h| {
            (year - 1..=year + 1).any(|y| {
                let date = days(y, h.month, h.day);
                let shifted = match (h.observed, (date + 4).rem_euclid(7)) {
                    (true, 6) => date - 1,
                    (true, 0) => date + 1,
                    _ => date,
                };
                date == z || shifted == z
            })
        })
}

/// Reference semantics: the due date of `period` under `holidays`.
fn reference(holidays: &[Holiday], period: &Period) -> String {
    let step = if period.backward { -1 } else { 1 };
    let roll = |mut z: i64| {
        while closed(holidays, z) {
            z += step;
        }
        z
    };
    let mut end = period.trigger;
    let mut left = period.amount;
    while left > 0 {
        end += step;
        if !(period.court_days && closed(holidays, end)) {
            left -= 1;
        }
    }
    end = roll(end);
    if period.mail && !period.backward {
        end = roll(end + 3);
    }
    iso(end)
}

fn holidays() -> impl Strategy<Value = Vec<Holiday>> {
    // Day 28 at most: every generated date exists in every year.
    prop::collection::vec((1_u8..=12, 1_u8..=28, any::<bool>()), 0..12).prop_map(|days| {
        days.into_iter()
            .map(|(month, day, observed)| Holiday {
                month,
                day,
                observed,
            })
            .collect()
    })
}

fn period() -> impl Strategy<Value = Period> {
    (
        days(2000, 1, 1)..days(2040, 1, 1),
        1_u64..=60,
        prop::bool::weighted(0.25),
        prop::bool::weighted(0.25),
        any::<bool>(),
    )
        .prop_map(|(trigger, amount, court_days, backward, mail)| Period {
            trigger,
            amount,
            court_days,
            backward,
            mail,
        })
}

/// The worked example with its calendar replaced by `holidays`, lowered
/// into an engine clock, and the `from` of its counting version.
fn clock(holidays: &[Holiday]) -> (EngineClock, String) {
    let mut package = example();
    let calendar = calendar_mut(&mut package);
    let template = calendar.holidays[0].clone();
    assert!(template.observed.is_some(), "new-year is observed");
    calendar.holidays = holidays
        .iter()
        .enumerate()
        .map(|(index, h)| hir::Holiday {
            id: format!("h{index}"),
            date: hir::HolidayDate::Fixed {
                month: h.month,
                day: h.day,
            },
            observed: if h.observed {
                template.observed.clone()
            } else {
                None
            },
            ..template.clone()
        })
        .collect();
    let from = counting_mut(&mut package).meta.effective.from.clone();
    let clock = EngineClock::lowered(&Lowerer::new(&[&package]), &[&package])
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    (clock, from)
}

fn due(clock: &EngineClock, from: &str, period: &Period) -> Result<String, String> {
    clock.due(&DueRequest {
        counting: "frcp6".into(),
        version: Some(from.to_owned()),
        trigger: iso(period.trigger),
        amount: period.amount,
        unit: if period.court_days {
            DayUnit::Court
        } else {
            DayUnit::Calendar
        },
        direction: if period.backward {
            Direction::Backward
        } else {
            Direction::Forward
        },
        service: if period.mail {
            vec!["mail".into()]
        } else {
            vec!["personal".into()]
        },
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn generated_counting_rules_match_the_reference_interpreter(
        holidays in holidays(), periods in prop::collection::vec(period(), 1..16)
    ) {
        let (clock, from) = clock(&holidays);
        for period in &periods {
            prop_assert_eq!(due(&clock, &from, period), Ok(reference(&holidays, period)), "{:?}", period);
        }
    }
}

/// Hand-picked differential cases over public-calendar facts: weekend and
/// holiday rolls in both directions, an observed Saturday holiday closing
/// the Friday before (across a year end), court days stepping over a
/// holiday, and mail days added after the roll.
#[test]
fn differential_cases_match_the_reference_interpreter() {
    let fixed = |month, day, observed| Holiday {
        month,
        day,
        observed,
    };
    let calendar = [fixed(1, 1, true), fixed(7, 4, true), fixed(12, 25, true)];
    let at = |date: &str, amount, court_days, backward, mail| {
        let parts: Vec<i64> = date.split('-').map(|p| p.parse().unwrap()).collect();
        let (month, day) = (
            u8::try_from(parts[1]).unwrap(),
            u8::try_from(parts[2]).unwrap(),
        );
        Period {
            trigger: days(parts[0], month, day),
            amount,
            court_days,
            backward,
            mail,
        }
    };
    let cases = [
        // 21 days from Monday 2025-03-03: Monday 2025-03-24, no roll.
        (at("2025-03-03", 21, false, false, false), "2025-03-24"),
        // 23 days from Thursday 2025-03-06 ends Saturday: rolled to Monday.
        (at("2025-03-06", 23, false, false, false), "2025-03-31"),
        // Ends Friday 2026-12-25: rolled over the weekend to Monday.
        (at("2026-12-04", 21, false, false, false), "2026-12-28"),
        // 2022-01-01 is a Saturday observed Friday 2021-12-31.
        (at("2021-12-10", 21, false, false, false), "2022-01-03"),
        // Backward from 2026-07-27 ends Saturday 2026-07-04, whose
        // observed Friday is closed too: rolled back to Thursday.
        (at("2026-07-27", 23, false, true, false), "2026-07-02"),
        // Five court days from Wednesday 2026-12-23 skip Christmas.
        (at("2026-12-23", 5, true, false, false), "2026-12-31"),
        // Mail: 21 days to Monday 2025-03-24, plus three to Thursday.
        (at("2025-03-03", 21, false, false, true), "2025-03-27"),
        // Mail: three days after Thursday 2026-07-02 is Sunday: Monday.
        (at("2026-06-11", 21, false, false, true), "2026-07-06"),
    ];
    let (engine, from) = clock(&calendar);
    for (period, want) in &cases {
        assert_eq!(reference(&calendar, period), *want, "oracle {period:?}");
        assert_eq!(
            due(&engine, &from, period).as_deref(),
            Ok(*want),
            "engine {period:?}"
        );
    }
}
