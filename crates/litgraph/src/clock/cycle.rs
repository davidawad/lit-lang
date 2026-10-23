// SPDX-License-Identifier: GPL-3.0-or-later
//! Deciding a property of every date by Gregorian-cycle enumeration.
//!
//! The proleptic Gregorian calendar repeats every 400 years, which is
//! 146 097 days, a whole number of weeks (7 x 20 871). Closed weekdays and
//! fixed-date and nth-weekday holidays, with their weekend observance,
//! therefore repeat with that period. A holiday on a `k`-year cycle repeats
//! every `lcm(400, k)` years. Only the finitely many effective-interval
//! boundaries break the periodicity. Enumerating every year from one full
//! period before the first boundary to one full period after the last
//! therefore decides a property for every date there is. Nothing is
//! sampled.
//!
//! [`Clock::closed_run`] uses this to decide whether rolling to an open day
//! terminates from every date (the roll and business-day loops of
//! [`super::interp`] run until they find an open day), and how far it can
//! go.

use std::collections::BTreeSet;

use super::date::Date;
use super::interp::Clock;
use super::library::ClockLibrary;

/// Years in one Gregorian cycle.
pub const CYCLE_YEARS: i64 = 400;

/// Days in one Gregorian cycle: a whole number of weeks.
pub const CYCLE_DAYS: i64 = 146_097;

/// Longest period enumerated, in years: one cycle times any single holiday
/// cycle of up to 100 years. Longer combined cycles are not decided.
pub const MAX_PERIOD_YEARS: i64 = CYCLE_YEARS * 100;

/// The longest run of consecutive closed days under a counting rule, over
/// every date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosedRun {
    /// Every run of closed days ends: `longest` days at most, first
    /// reached by the run starting `from`.
    Bounded {
        /// Length of the longest run, in days (0 when no day is closed).
        longest: i64,
        /// First day of one longest run.
        from: Date,
    },
    /// The run of closed days through `at` never ends: a roll from there
    /// does not terminate.
    Unbounded {
        /// A closed day in the endless run.
        at: Date,
    },
    /// The holidays' cycles combine to a period longer than
    /// [`MAX_PERIOD_YEARS`].
    Undecided,
}

const fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Clock<'_> {
    /// The period, in years, after which every calendar of this rule
    /// repeats, saturating past [`MAX_PERIOD_YEARS`].
    fn period_years(&self) -> i64 {
        self.calendars
            .iter()
            .flat_map(|c| &c.holidays)
            .filter_map(|h| h.cycle.as_ref())
            .map(|c| i64::from(c.every.max(1)))
            .fold(CYCLE_YEARS, |p, k| {
                (p / gcd(p, k)).saturating_mul(k).min(MAX_PERIOD_YEARS + 1)
            })
    }

    /// Decides the longest run of consecutive closed days over all dates
    /// (closed weekdays and the observed holidays of this rule's
    /// calendars).
    #[must_use]
    pub fn closed_run(&self) -> ClosedRun {
        let period = self.period_years();
        if period > MAX_PERIOD_YEARS {
            return ClosedRun::Undecided;
        }
        let bounds: Vec<i64> = self
            .calendars
            .iter()
            .flat_map(|c| &c.holidays)
            .filter_map(|h| h.effective.as_ref())
            .flat_map(|e| [e.from, e.until])
            .flatten()
            .map(|d| i64::from(d.ymd().0))
            .collect();
        let first = bounds.iter().copied().min().unwrap_or(2000);
        let last = bounds.iter().copied().max().unwrap_or(2000);
        // Runs start in [first - period - 1, last + period + 1]; a run as
        // long as the period is endless, so a margin of one more period
        // holds every day a walk can visit.
        let (lo, hi) = (first - period - 1, last + period + 1);
        let year = |y: i64| i32::try_from(y).unwrap_or(i32::MAX);
        let holidays: BTreeSet<Date> = (lo - period - 1..=hi + period + 1)
            .flat_map(|y| self.holidays_in(year(y)))
            .collect();
        let closed = |d: Date| {
            self.rule
                .closed_weekdays
                .iter()
                .any(|w| w.number() == d.weekday())
                || holidays.contains(&d)
        };
        let start = Date::from_ymd(year(lo), 1, 1).unwrap_or(Date::from_epoch_day(0));
        let end = Date::from_ymd(year(hi), 12, 31).unwrap_or(Date::from_epoch_day(0));
        let cap = period * 366;
        // The closed days of one week (closed weekdays) and every holiday
        // start a walk, in date order; a seed inside the last run is
        // already counted.
        let seeds: BTreeSet<Date> = (0..7)
            .map(|i| start.add_days(i))
            .chain(holidays.range(start..=end).copied())
            .filter(|d| closed(*d))
            .collect();
        let (mut longest, mut longest_from) = (0, start);
        let mut covered = start.add_days(-1);
        for seed in seeds {
            if seed <= covered {
                continue;
            }
            let mut from = seed;
            while closed(from.add_days(-1)) && seed.epoch_day() - from.epoch_day() < cap {
                from = from.add_days(-1);
            }
            let mut to = seed;
            while closed(to.add_days(1)) && to.epoch_day() - from.epoch_day() < cap {
                to = to.add_days(1);
            }
            let length = to.epoch_day() - from.epoch_day() + 1;
            if length > cap {
                return ClosedRun::Unbounded { at: seed };
            }
            covered = to;
            if length > longest {
                (longest, longest_from) = (length, from);
            }
        }
        ClosedRun::Bounded {
            longest,
            from: longest_from,
        }
    }
}

/// Why some date never rolls out of a closed run under a rule of
/// `library`, for the first such rule: counting under it would not
/// terminate, so a caller refuses it instead.
#[must_use]
pub fn endless_roll(library: &ClockLibrary) -> Option<String> {
    library.counting_rules.iter().find_map(|r| {
        let clock = library.clock(&r.id).ok()?;
        match clock.closed_run() {
            ClosedRun::Unbounded { at } => Some(format!(
                "counting rule `{}` never rolls to an open day from {}: every day from there on is closed",
                r.id,
                at.to_iso()
            )),
            _ => None,
        }
    })
}
