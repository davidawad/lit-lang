// SPDX-License-Identifier: GPL-3.0-or-later
//! Proleptic Gregorian calendar arithmetic, exact and dependency-free.
//!
//! A [`Date`] is stored as a single signed day count relative to the Unix
//! epoch (1970-01-01), so `add_days`/comparisons are plain integer ops and
//! there is no month/day overflow to normalize. Conversion to/from
//! `(year, month, day)` uses Howard Hinnant's `days_from_civil` /
//! `civil_from_days` algorithms — a well-known, exact, allocation-free
//! civil-calendar<->day-count mapping valid over the full `i32` year range.
//! Reference: <http://howardhinnant.github.io/date_algorithms.html> (public
//! domain). This is an algorithm citation, not a legal source.
//!
//! Deliberate departure from the `civ-pro-the-gathering` TypeScript port
//! (`src/engine/clock/date-math.ts`): that code silently normalizes
//! out-of-range components (e.g. day 32) through `Date.UTC` coercion.
//! Rust's `Date::from_ymd` instead rejects an invalid calendar date with an
//! error, matching this repo's "no silent guesses" / strict-parsing
//! commitment (see `AGENTS.md` rule 5 and `docs/ARCHITECTURE.md` item 2).
//! Nothing in the clock interpreter ever needs to construct an
//! out-of-range date; every date we build starts from a validated ISO
//! string or from `first_of_month`/`add_days` arithmetic on an already-valid
//! `Date`.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

/// A calendar date, stored as days since the Unix epoch (1970-01-01).
/// May be negative (dates before 1970) or beyond; the algorithms are exact
/// for the full proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(i64);

/// Day of week, `Sunday` = 0 .. `Saturday` = 6 (the civ-pro TS port's
/// convention).
pub type Weekday = u8;

/// Sunday.
pub const SUNDAY: Weekday = 0;
/// Monday.
pub const MONDAY: Weekday = 1;
/// Friday.
pub const FRIDAY: Weekday = 5;
/// Saturday.
pub const SATURDAY: Weekday = 6;
/// Thursday.
pub const THURSDAY: Weekday = 4;

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Howard Hinnant's `days_from_civil`: exact for `month` in `1..=12` and
/// `day` in `1..=days_in_month(year, month)`.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y: i64 = if month <= 2 {
        i64::from(year) - 1
    } else {
        i64::from(year)
    };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = i64::from((month + 9) % 12); // Mar=0 .. Feb=11
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u32; // [1, 12]
    let year = if month <= 2 { y + 1 } else { y };
    #[allow(clippy::cast_possible_truncation)]
    let year = year as i32;
    (year, month, day)
}

impl Date {
    /// Build a date from civil components, rejecting an invalid calendar
    /// date (unlike the TS port's silent `Date.UTC` normalization).
    ///
    /// # Errors
    /// `Error::Invalid` for `month` outside `1..=12` or `day` outside the
    /// valid range for that year/month.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Date> {
        if !(1..=12).contains(&month) {
            return Err(Error::Invalid(format!("month {month} out of range 1..=12")));
        }
        let max = days_in_month(year, month);
        if day < 1 || day > max {
            return Err(Error::Invalid(format!(
                "day {day} out of range 1..={max} for {year}-{month:02}"
            )));
        }
        Ok(Date(days_from_civil(year, month, day)))
    }

    /// Parse `"YYYY-MM-DD"`.
    ///
    /// # Errors
    /// `Error::Invalid` on malformed input or an out-of-range calendar date.
    pub fn parse_iso(s: &str) -> Result<Date> {
        let bytes = s.as_bytes();
        let ok = bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[0..4].iter().all(u8::is_ascii_digit)
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[8..10].iter().all(u8::is_ascii_digit);
        if !ok {
            return Err(Error::Invalid(format!(
                "invalid ISO date {s:?}; expected YYYY-MM-DD"
            )));
        }
        let year: i32 = s[0..4]
            .parse()
            .map_err(|_| Error::Invalid(format!("invalid year in {s:?}")))?;
        let month: u32 = s[5..7]
            .parse()
            .map_err(|_| Error::Invalid(format!("invalid month in {s:?}")))?;
        let day: u32 = s[8..10]
            .parse()
            .map_err(|_| Error::Invalid(format!("invalid day in {s:?}")))?;
        Date::from_ymd(year, month, day)
    }

    /// Civil year/month/day.
    #[must_use]
    pub fn ymd(self) -> (i32, u32, u32) {
        civil_from_days(self.0)
    }

    /// Format as `"YYYY-MM-DD"`.
    #[must_use]
    pub fn to_iso(self) -> String {
        let (y, m, d) = self.ymd();
        format!("{y:04}-{m:02}-{d:02}")
    }

    /// Add (or, for negative `n`, subtract) calendar days.
    #[must_use]
    pub fn add_days(self, n: i64) -> Date {
        Date(self.0 + n)
    }

    /// Days since the Unix epoch (1970-01-01), for datetime arithmetic that
    /// needs an integer day count (e.g. hours-based deadlines).
    #[must_use]
    pub fn epoch_day(self) -> i64 {
        self.0
    }

    /// The date `n` days since the Unix epoch (inverse of [`Self::epoch_day`]).
    #[must_use]
    pub fn from_epoch_day(n: i64) -> Date {
        Date(n)
    }

    /// Day of week, `SUNDAY`..=`SATURDAY`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 (day 0) was a Thursday.
        ((self.0 + 4).rem_euclid(7)) as Weekday
    }

    /// True for Saturday or Sunday.
    #[must_use]
    pub fn is_weekend(self) -> bool {
        matches!(self.weekday(), SUNDAY | SATURDAY)
    }

    /// The first day of this date's month.
    #[must_use]
    pub fn first_of_month(self) -> Date {
        let (y, m, _) = self.ymd();
        Date::from_ymd(y, m, 1).unwrap_or(self)
    }

    /// The last day of this date's month.
    #[must_use]
    pub fn last_of_month(self) -> Date {
        let (y, m, _) = self.ymd();
        let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
        Date::from_ymd(ny, nm, 1).unwrap_or(self).add_days(-1)
    }

    /// The date `n` calendar months away (earlier for negative `n`) on the
    /// same day of the month, and whether the target month is too short to
    /// have that day; then the date is the target month's last day (January
    /// 31 plus one month is February 28 or 29, `true`).
    ///
    /// # Errors
    /// `Error::Invalid` when the target year is out of range.
    pub fn add_months(self, n: i64) -> Result<(Date, bool)> {
        let (y, m, d) = self.ymd();
        let total = n
            .checked_add(i64::from(y) * 12 + i64::from(m) - 1)
            .ok_or_else(|| Error::Invalid(format!("{n} months is out of range")))?;
        let year = i32::try_from(total.div_euclid(12))
            .map_err(|_| Error::Invalid(format!("{n} months from {self} is out of range")))?;
        // `rem_euclid(12)` is in 0..12.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let month = total.rem_euclid(12) as u32 + 1;
        let last = days_in_month(year, month);
        Ok((Date(days_from_civil(year, month, d.min(last))), d > last))
    }

    /// The `n`th occurrence of `dow` in `(year, month)`. `n` is 1-indexed;
    /// use `n = -1` for "last".
    ///
    /// # Errors
    /// `Error::Invalid` for `n == 0` or an out-of-range `year`/`month`.
    pub fn nth_weekday_of_month(year: i32, month: u32, dow: Weekday, n: i32) -> Result<Date> {
        if n == 0 {
            return Err(Error::Invalid(
                "nth_weekday_of_month: n must not be 0".into(),
            ));
        }
        let first = Date::from_ymd(year, month, 1)?;
        if n > 0 {
            let first_dow = first.weekday();
            let diff = i64::from((dow + 7 - first_dow) % 7);
            Ok(first.add_days(diff + i64::from(n - 1) * 7))
        } else {
            let last = first.last_of_month();
            let last_dow = last.weekday();
            let diff = i64::from((last_dow + 7 - dow) % 7);
            Ok(last.add_days(-diff - i64::from(-n - 1) * 7))
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso())
    }
}

/// Serialized as its ISO `YYYY-MM-DD` string (clock data files author
/// dates this way).
impl Serialize for Date {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_iso())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Date, D::Error> {
        let s = String::deserialize(d)?;
        Date::parse_iso(&s).map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for Date {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Date".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "format": "date",
            "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_iso() {
        for s in [
            "2026-01-01",
            "2026-12-31",
            "2024-02-29",
            "1969-12-31",
            "2000-02-29",
        ] {
            assert_eq!(Date::parse_iso(s).unwrap().to_iso(), s);
        }
    }

    #[test]
    fn rejects_invalid_calendar_dates() {
        assert!(Date::from_ymd(2026, 2, 30).is_err());
        assert!(Date::from_ymd(2025, 2, 29).is_err()); // not a leap year
        assert!(Date::from_ymd(2026, 13, 1).is_err());
        assert!(Date::parse_iso("2026-1-1").is_err());
        assert!(Date::parse_iso("not-a-date").is_err());
    }

    #[test]
    fn epoch_is_thursday() {
        assert_eq!(Date::from_ymd(1970, 1, 1).unwrap().weekday(), THURSDAY);
    }

    #[test]
    fn add_days_crosses_month_and_year() {
        assert_eq!(
            Date::parse_iso("2025-12-28").unwrap().add_days(14).to_iso(),
            "2026-01-11"
        );
        assert_eq!(
            Date::parse_iso("2026-01-01").unwrap().add_days(-1).to_iso(),
            "2025-12-31"
        );
    }

    #[test]
    fn nth_weekday_forward_and_last() {
        // MLK 2026: 3rd Monday in January = Jan 19.
        assert_eq!(
            Date::nth_weekday_of_month(2026, 1, MONDAY, 3)
                .unwrap()
                .to_iso(),
            "2026-01-19"
        );
        // Memorial Day 2026: last Monday in May = May 25.
        assert_eq!(
            Date::nth_weekday_of_month(2026, 5, MONDAY, -1)
                .unwrap()
                .to_iso(),
            "2026-05-25"
        );
    }

    #[test]
    fn serde_roundtrips_as_an_iso_string() {
        let d: Date = serde_json::from_str("\"2021-06-17\"").unwrap();
        assert_eq!(serde_json::to_string(&d).unwrap(), "\"2021-06-17\"");
        assert!(serde_json::from_str::<Date>("\"2021-02-30\"").is_err());
    }

    #[test]
    fn add_months_keeps_the_day_or_reports_a_short_month() {
        let d = |s: &str| Date::parse_iso(s).unwrap();
        let shift = |s: &str, n: i64| {
            let (date, short) = d(s).add_months(n).unwrap();
            (date.to_iso(), short)
        };
        assert_eq!(shift("2025-03-04", 9), ("2025-12-04".into(), false));
        assert_eq!(shift("2024-11-30", 3), ("2025-02-28".into(), true));
        assert_eq!(shift("2023-11-30", 3), ("2024-02-29".into(), true));
        assert_eq!(shift("2025-02-28", 3), ("2025-05-28".into(), false));
        assert_eq!(shift("2024-02-29", 12), ("2025-02-28".into(), true));
        assert_eq!(shift("2025-01-15", -1), ("2024-12-15".into(), false));
        assert_eq!(shift("2025-03-31", -1), ("2025-02-28".into(), true));
        assert!(d("2025-01-01").add_months(i64::MAX).is_err());
        assert!(d("2025-01-01")
            .add_months(i64::from(i32::MAX) * 12)
            .is_err());
    }

    #[test]
    fn last_of_month_handles_december() {
        assert_eq!(
            Date::parse_iso("2026-12-15")
                .unwrap()
                .last_of_month()
                .to_iso(),
            "2026-12-31"
        );
    }
}
