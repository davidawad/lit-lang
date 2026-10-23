//! Deadlines, counting rules, holiday calendars, versions, and transitions.
use super::{DayUnit, Expr, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Half-open in-force interval `[from, to)` plus corpus (`known_at`) date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    /// Inclusive beginning date.
    pub from: String,
    /// Exclusive end date; `None` is open-ended.
    pub to: Option<String>,
    /// Date the corpus recorded this version.
    pub known_at: String,
}

/// Typed deadline declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deadline {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Period length.
    pub amount: u64,
    /// Period unit.
    pub unit: DayUnit,
    /// Counting direction (`from`/`after` forward, `before` backward).
    pub direction: Direction,
    /// Trigger event expression.
    pub trigger: Expr,
    /// Qualified counting-rule id.
    pub counting: String,
    /// Whether extensions are available (`Deadline.extendable`).
    pub extendable: Option<bool>,
    /// Extension authority text.
    pub extension_authority: Option<String>,
    /// Lowered `Deadline.note`.
    pub note: Option<String>,
}

/// Deadline counting direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Count forward from the trigger.
    Forward,
    /// Count backward from the trigger.
    Backward,
}

/// Data-driven day-counting rule (lowered to `Pack.counting`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counting {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Exclude the trigger day.
    pub exclude_trigger_day: bool,
    /// Include the last day (subject to rolling).
    pub include_last_day: bool,
    /// Excluded last days that roll in the counting direction.
    pub roll_on: Vec<RollOn>,
    /// Service method to added days, forward counting only.
    pub service_extensions: BTreeMap<String, (u64, DayUnit)>,
    /// Qualified holiday-calendar id.
    pub calendar: String,
    /// Where counting starts.
    pub anchor: Anchor,
    /// Periods shorter than `threshold` count in `unit` instead.
    pub short_period: Option<ShortPeriod>,
    /// Whether hour periods (FRCP 6(a)(2)) are permitted.
    pub hours: bool,
    /// End-of-month policy; month and year periods need one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_of_month: Option<EndOfMonth>,
}

/// Where a month or year period ends when its last month has no day
/// corresponding to the trigger's (January 31 plus one month).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndOfMonth {
    /// The last day of that month (MPEP 710.01(a)).
    Clamp,
    /// The first day of the following month.
    Roll,
}

/// Day kinds that cannot be a last day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RollOn {
    /// Saturday or Sunday.
    Weekend,
    /// A holiday of the referenced calendar.
    LegalHoliday,
    /// A day the clerk's office is inaccessible (case-supplied dates).
    Inaccessible,
}

/// Counting anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Anchor {
    /// Count from the trigger day itself.
    TriggerDay,
    /// Count from the first business day on or after the trigger.
    NextBusinessDay,
}

/// Short-period rule (e.g. 19 CFR 201.14: under 7 days count business days).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortPeriod {
    /// Periods with `amount < threshold` are affected.
    pub threshold: u64,
    /// Unit used for affected periods.
    pub unit: DayUnit,
}

/// Holiday calendar (lowered to `Pack.calendars[]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Holidays in source order.
    pub holidays: Vec<Holiday>,
}

/// One holiday definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holiday {
    /// Stable holiday id.
    pub id: String,
    /// Date rule.
    pub date: HolidayDate,
    /// In-force interval.
    pub in_force: Interval,
    /// Weekend observance shift.
    pub observed: Option<Observance>,
}

/// Holiday date rule (SPEC §8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum HolidayDate {
    /// `fixed(month, day)`.
    Fixed {
        /// Month 1-12.
        month: u8,
        /// Day 1-31.
        day: u8,
    },
    /// `nth(n, weekday, month)`: the n-th (1-4) weekday of the month.
    Nth {
        /// Ordinal 1-4.
        n: u8,
        /// Lowercase English weekday.
        weekday: String,
        /// Month 1-12.
        month: u8,
    },
    /// `last(weekday, month)`.
    Last {
        /// Lowercase English weekday.
        weekday: String,
        /// Month 1-12.
        month: u8,
    },
    /// `every(years, from_year, month, day)`: years `from_year + k*years`, k >= 0.
    Every {
        /// Period in years.
        years: u16,
        /// First year.
        from_year: u16,
        /// Month 1-12.
        month: u8,
        /// Day 1-31.
        day: u8,
    },
    /// `once(date)`: a single proclaimed day.
    Once {
        /// ISO date.
        date: String,
    },
}

/// Observance shift for a holiday falling on a weekend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Observance {
    /// Saturday to Friday, Sunday to Monday.
    NearestWeekday,
    /// Sunday to Monday only.
    NextDay,
}

/// Version-selection rule for proceedings pending across an amendment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Amended declaration, version-qualified (`id@from-date`).
    pub amendment: String,
    /// Arms in source order; the last arm is `otherwise`.
    pub arms: Vec<TransitionArm>,
}

/// One transition arm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionArm {
    /// `unless` condition; `None` for `otherwise`.
    pub unless: Option<Expr>,
    /// Version-qualified declaration selected.
    pub selects: String,
}
