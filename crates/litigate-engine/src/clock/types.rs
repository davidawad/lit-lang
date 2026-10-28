// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared clock types: deadline requests, service methods, and the
//! diagnostic result shape the interpreter returns. Which counting rule
//! governs is data (`clock::rule::CountingRule`), never a type here.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::date::Date;

/// Direction a day-counted period runs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// "N days after" the trigger (the ordinary case; every pack-authored
    /// `deadline` is forward).
    #[default]
    Forward,
    /// "At least N days before" the trigger (backward counting; not
    /// currently authored on any pack edge, but a real rule shape, e.g. a
    /// pre-filing notice window).
    Backward,
}

/// The unit a period is stated in, per `Deadline.unit` in `model::schema`
/// (`docs/PACK_SCHEMA.md`). A counting rule's `count` step decides what
/// calendar, court and business days mean (`clock::rule::Period`); a week is seven
/// calendar days; months and years are calendar months, counted by the
/// rule's `months` provision (`clock::rule::Months`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// Calendar days (the authored default).
    #[default]
    Calendar,
    /// Court days: a pack-authored choice for a forum whose own rule
    /// counts court days.
    Court,
    /// Business days: a period the forum's own rule states in business
    /// days, counted by the counting rule's own `business` period.
    Business,
    /// Weeks of seven calendar days.
    Week,
    /// Calendar months.
    Month,
    /// Calendar years (twelve calendar months).
    Year,
}

/// Service methods that may add days to a period: a closed vocabulary, the
/// union of every shipped rule's methods. A counting rule's `service` step
/// is total over it: each method either adds a number of days (zero
/// included) or is one the rule does not recognize (design review
/// 2026-10-23, P8).
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceMethod {
    /// Personal delivery (handing it over, or leaving it at an office or
    /// dwelling).
    Personal,
    /// Electronic means.
    Electronic,
    /// Mail (domestic).
    Mail,
    /// Leaving with the clerk.
    Clerk,
    /// Other means consented to.
    OtherConsented,
    /// Third-party commercial carrier.
    CommercialCarrier,
    /// Prison mailing system.
    PrisonMail,
    /// Mail to a foreign address.
    MailForeign,
    /// Express delivery (domestic).
    Express,
    /// Express delivery to a foreign destination.
    ExpressForeign,
}

impl ServiceMethod {
    /// Every service method, in declaration order.
    pub const ALL: [ServiceMethod; 10] = [
        ServiceMethod::Personal,
        ServiceMethod::Electronic,
        ServiceMethod::Mail,
        ServiceMethod::Clerk,
        ServiceMethod::OtherConsented,
        ServiceMethod::CommercialCarrier,
        ServiceMethod::PrisonMail,
        ServiceMethod::MailForeign,
        ServiceMethod::Express,
        ServiceMethod::ExpressForeign,
    ];

    /// The authored (kebab-case) name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceMethod::Personal => "personal",
            ServiceMethod::Electronic => "electronic",
            ServiceMethod::Mail => "mail",
            ServiceMethod::Clerk => "clerk",
            ServiceMethod::OtherConsented => "other-consented",
            ServiceMethod::CommercialCarrier => "commercial-carrier",
            ServiceMethod::PrisonMail => "prison-mail",
            ServiceMethod::MailForeign => "mail-foreign",
            ServiceMethod::Express => "express",
            ServiceMethod::ExpressForeign => "express-foreign",
        }
    }
}

/// A day-counted (not hours-based) deadline computation request.
#[derive(Debug, Clone)]
pub struct DayDeadline {
    /// The triggering event date.
    pub trigger: Date,
    /// Length of the period in `unit`s (non-negative integer).
    pub days: i64,
    /// Forward ("after") or backward ("before") counting.
    pub direction: Direction,
    /// Authored unit (the counting rule decides what it means).
    pub unit: Unit,
    /// Service method adding days before the roll, if any.
    pub service_method: Option<ServiceMethod>,
}

/// Extra calendar configuration.
#[derive(Debug, Clone, Default)]
pub struct ClockConfig {
    /// Declared holidays beyond the rule's calendars (e.g. a state or
    /// presidential declaration).
    pub additional_holidays: Vec<Date>,
    /// Clerk's office inaccessible on the last day (or, for an hours
    /// period, during its last hour). Applied by a rule's
    /// `clerkInaccessible` step or `hours.inaccessibleText`; a rule without
    /// one ignores it.
    pub clerk_inaccessible: bool,
}

/// The computed due date, plus a step-by-step trace for teaching/audit.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedDeadline {
    /// Id of the counting rule that computed this.
    pub rule_set: String,
    /// The anchor date (the trigger itself, or an adjacent business day,
    /// per the rule's `anchor` step).
    pub anchor_date: String,
    /// The period's last day as counted, before any roll or extension
    /// (the "nominal" due date).
    pub nominal_date: String,
    /// The working date where the rule's `base` step sits in its
    /// pipeline.
    pub base_date: String,
    /// The final ("adjusted") due date, after every step.
    pub due_date: String,
    /// Whether any `roll` step moved the date.
    pub last_day_rolled: bool,
    /// Whether a service-method extension was applied.
    pub service_days_added: bool,
    /// Whether clerk-inaccessibility extension was applied.
    pub clerk_inaccessibility_applied: bool,
    /// Ordered, human-readable computation steps (each cites its rule).
    pub steps: Vec<String>,
}

/// An hours-based deadline computation request.
#[derive(Debug, Clone)]
pub struct HourDeadline {
    /// Exact trigger instant, as whole seconds since the Unix epoch
    /// (UTC-less / calendar-local, matching the TS port).
    pub trigger_epoch_seconds: i64,
    /// Number of hours (non-negative integer).
    pub hours: i64,
}

/// The computed due instant for an hours-based period.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedHourDeadline {
    /// The due date (date portion only).
    pub due_date: String,
    /// The full due instant, `"YYYY-MM-DDTHH:MM:SS"`.
    pub due_datetime: String,
    /// Whether the period rolled past a weekend/holiday.
    pub last_day_rolled: bool,
    /// Whether the rule's hours inaccessibility extension was applied.
    pub clerk_inaccessibility_applied: bool,
    /// Ordered, human-readable computation steps.
    pub steps: Vec<String>,
}
