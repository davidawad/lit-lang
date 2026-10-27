// SPDX-License-Identifier: GPL-3.0-or-later
//! Holiday calendars as data: the `HolidayCalendar` IR.
//!
//! A calendar is a list of holidays, each a date rule (a fixed month/day or
//! the nth weekday of a month), an optional multi-year cycle, an
//! observed-day rule for weekend dates, and an effective interval. Nothing
//! here names a forum or a statute: the shipped calendars are `lit/core`'s
//! `calendar` declarations, lowered (see `docs/PACK_SCHEMA.md`), and a
//! counting rule names the calendars it closes on.
//!
//! Effective intervals are half-open (`from` inclusive, `until`
//! exclusive) and are tested against the *observed* date, so a holiday
//! created mid-year (for example one effective from a June date) is a
//! holiday only from its observed occurrence on that date or later.

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::date::{self, Date};
use crate::error::{Error, Result};

/// A day of the week, as authored in clock data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    /// Sunday.
    Sunday,
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
}

impl Weekday {
    /// The [`date::Weekday`] number (`Sunday` = 0).
    #[must_use]
    pub fn number(self) -> date::Weekday {
        match self {
            Weekday::Sunday => date::SUNDAY,
            Weekday::Monday => date::MONDAY,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => date::THURSDAY,
            Weekday::Friday => date::FRIDAY,
            Weekday::Saturday => date::SATURDAY,
        }
    }
}

/// When in a year a holiday falls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum HolidayDate {
    /// The same month and day every year.
    Fixed {
        /// Month, 1-12.
        month: u32,
        /// Day of month, 1-31.
        day: u32,
    },
    /// The `nth` `weekday` of `month`; `nth` is 1..=5, or -1..=-5 counting
    /// from the end of the month (-1 = last).
    NthWeekday {
        /// Month, 1-12.
        month: u32,
        /// Day of the week.
        weekday: Weekday,
        /// Occurrence: 1..=5, or -1..=-5 from the end.
        nth: i32,
    },
}

/// Which year a cyclic holiday recurs in: every `every` years counting from
/// `anchorYear` (so `{every: 4, anchorYear: 1965}` is 1965, 1969, ...).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cycle {
    /// Period in years (>= 1).
    pub every: i32,
    /// A year the holiday occurs in.
    pub anchor_year: i32,
}

/// How a holiday that falls on a weekend is observed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Observance {
    /// Observed on the date itself, weekend or not.
    #[default]
    Actual,
    /// Saturday is observed the preceding Friday; Sunday the following
    /// Monday.
    NearestWeekday,
}

/// A half-open date interval `[from, until)`; either end may be open.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Effective {
    /// First day the holiday is in force (inclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Date>,
    /// First day the holiday is no longer in force (exclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<Date>,
}

impl Effective {
    /// True if `d` lies inside the interval.
    #[must_use]
    pub fn contains(&self, d: Date) -> bool {
        self.from.is_none_or(|f| d >= f) && self.until.is_none_or(|u| d < u)
    }
}

/// One holiday.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Holiday {
    /// Stable id, unique within its calendar.
    pub id: String,
    /// Display name.
    pub name: String,
    /// When in the year it falls.
    pub date: HolidayDate,
    /// Multi-year recurrence; absent means every year.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Cycle>,
    /// Weekend observance rule.
    #[serde(default)]
    pub observed: Observance,
    /// When the holiday is in force; absent means always.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective: Option<Effective>,
    /// Authority creating the holiday or its effective date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

impl Holiday {
    /// The observed date in `year`, if the holiday occurs and is in force.
    #[must_use]
    pub fn observed_in(&self, year: i32) -> Option<Date> {
        if let Some(c) = &self.cycle {
            if (year - c.anchor_year).rem_euclid(c.every) != 0 {
                return None;
            }
        }
        let raw = match &self.date {
            HolidayDate::Fixed { month, day } => Date::from_ymd(year, *month, *day).ok()?,
            HolidayDate::NthWeekday {
                month,
                weekday,
                nth,
            } => {
                let d = Date::nth_weekday_of_month(year, *month, weekday.number(), *nth).ok()?;
                // A fifth weekday that spills into the next month does not exist.
                if d.ymd().1 != *month {
                    return None;
                }
                d
            }
        };
        let d = match (self.observed, raw.weekday()) {
            (Observance::NearestWeekday, date::SATURDAY) => raw.add_days(-1),
            (Observance::NearestWeekday, date::SUNDAY) => raw.add_days(1),
            _ => raw,
        };
        self.effective
            .as_ref()
            .is_none_or(|e| e.contains(d))
            .then_some(d)
    }

    fn validate(&self, calendar: &str) -> Result<()> {
        let bad = |why: String| {
            Err(Error::Invalid(format!(
                "calendar {calendar}: holiday {}: {why}",
                self.id
            )))
        };
        let (month, day) = match &self.date {
            HolidayDate::Fixed { month, day } => (*month, Some(*day)),
            HolidayDate::NthWeekday { month, nth, .. } => {
                if *nth == 0 || !(-5..=5).contains(nth) {
                    return bad(format!("nth {nth} must be 1..=5 or -1..=-5"));
                }
                (*month, None)
            }
        };
        // 2000 is a leap year, so every real month/day pair parses.
        if Date::from_ymd(2000, month, day.unwrap_or(1)).is_err() {
            return bad(format!(
                "month {month} / day {day:?} is not a calendar date"
            ));
        }
        if let Some(c) = &self.cycle {
            if c.every < 1 {
                return bad(format!("cycle.every {} must be >= 1", c.every));
            }
        }
        if let Some(Effective {
            from: Some(f),
            until: Some(u),
        }) = &self.effective
        {
            if f >= u {
                return bad(format!("effective interval [{f}, {u}) is empty"));
            }
        }
        Ok(())
    }
}

/// A named set of holidays (`HolidayCalendar` IR).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HolidayCalendar {
    /// Stable id that counting rules reference.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Authority for the calendar as a whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Primary source URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// The holidays.
    pub holidays: Vec<Holiday>,
}

impl HolidayCalendar {
    /// Every observed holiday date in `year`.
    #[must_use]
    pub fn observed_in(&self, year: i32) -> BTreeSet<Date> {
        self.holidays
            .iter()
            .filter_map(|h| h.observed_in(year))
            .collect()
    }

    /// Check holiday ids are unique and every date rule is well formed.
    ///
    /// # Errors
    /// `Error::Invalid` naming the calendar and holiday.
    pub fn validate(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for h in &self.holidays {
            if !seen.insert(h.id.as_str()) {
                return Err(Error::Invalid(format!(
                    "calendar {}: duplicate holiday id {}",
                    self.id, h.id
                )));
            }
            h.validate(&self.id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(month: u32, day: u32, observed: Observance) -> Holiday {
        Holiday {
            id: "h".into(),
            name: "H".into(),
            date: HolidayDate::Fixed { month, day },
            cycle: None,
            observed,
            effective: None,
            cite: None,
        }
    }

    fn d(s: &str) -> Date {
        Date::parse_iso(s).unwrap()
    }

    #[test]
    fn nearest_weekday_shifts_saturday_back_and_sunday_forward() {
        let h = fixed(12, 25, Observance::NearestWeekday);
        assert_eq!(h.observed_in(2021), Some(d("2021-12-24"))); // Sat
        assert_eq!(h.observed_in(2022), Some(d("2022-12-26"))); // Sun
        assert_eq!(h.observed_in(2026), Some(d("2026-12-25"))); // Fri
        let actual = fixed(12, 25, Observance::Actual);
        assert_eq!(actual.observed_in(2021), Some(d("2021-12-25")));
    }

    #[test]
    fn effective_interval_is_half_open_on_the_observed_date() {
        let mut h = fixed(6, 19, Observance::NearestWeekday);
        h.effective = Some(Effective {
            from: Some(d("2021-06-18")),
            until: Some(d("2023-01-01")),
        });
        assert_eq!(h.observed_in(2020), None);
        assert_eq!(h.observed_in(2021), Some(d("2021-06-18")));
        assert_eq!(h.observed_in(2022), Some(d("2022-06-20")));
        assert_eq!(h.observed_in(2023), None);
        h.effective = Some(Effective {
            from: Some(d("2021-06-19")),
            until: None,
        });
        // Observed 2021-06-18 is before `from`, so 2021 has no occurrence.
        assert_eq!(h.observed_in(2021), None);
    }

    #[test]
    fn cycle_selects_every_nth_year() {
        let mut h = fixed(1, 20, Observance::Actual);
        h.cycle = Some(Cycle {
            every: 4,
            anchor_year: 1965,
        });
        assert!(h.observed_in(1961).is_some());
        assert!(h.observed_in(2025).is_some());
        assert!(h.observed_in(2026).is_none());
    }

    #[test]
    fn a_fifth_weekday_that_does_not_exist_is_skipped() {
        let h = Holiday {
            date: HolidayDate::NthWeekday {
                month: 2,
                weekday: Weekday::Monday,
                nth: 5,
            },
            ..fixed(1, 1, Observance::Actual)
        };
        assert_eq!(h.observed_in(2026), None);
        assert_eq!(h.observed_in(2016), Some(d("2016-02-29")));
    }

    #[test]
    fn validate_rejects_malformed_holidays() {
        let cal = |h: Holiday| HolidayCalendar {
            id: "c".into(),
            title: "C".into(),
            cite: None,
            source_url: None,
            holidays: vec![h],
        };
        assert!(cal(fixed(2, 30, Observance::Actual)).validate().is_err());
        assert!(cal(fixed(2, 29, Observance::Actual)).validate().is_ok());
        let mut zero = fixed(1, 1, Observance::Actual);
        zero.date = HolidayDate::NthWeekday {
            month: 1,
            weekday: Weekday::Monday,
            nth: 0,
        };
        assert!(cal(zero).validate().is_err());
        let mut cyc = fixed(1, 1, Observance::Actual);
        cyc.cycle = Some(Cycle {
            every: 0,
            anchor_year: 2000,
        });
        assert!(cal(cyc).validate().is_err());
        let mut empty = fixed(1, 1, Observance::Actual);
        empty.effective = Some(Effective {
            from: Some(d("2020-01-01")),
            until: Some(d("2020-01-01")),
        });
        assert!(cal(empty).validate().is_err());
        let two = HolidayCalendar {
            holidays: vec![
                fixed(1, 1, Observance::Actual),
                fixed(1, 2, Observance::Actual),
            ],
            ..cal(fixed(1, 1, Observance::Actual))
        };
        assert!(two.validate().is_err(), "duplicate holiday ids");
    }
}
