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

/// Calendar days versus business ("court") days, per `Deadline.unit` in
/// `model::schema` (`docs/PACK_SCHEMA.md`). A counting rule's `count`
/// step decides what each unit means (`clock::rule::Period`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// Calendar days (the authored default).
    #[default]
    Calendar,
    /// Court ("business") days: a pack-authored choice for a forum whose
    /// own rule counts business days.
    Court,
}

/// Service methods that may add days to a period. The vocabulary is the
/// union of every shipped rule's methods; each counting rule's `service`
/// step lists the subset it recognizes and how many days each adds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceMethod {
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
    /// The authored (kebab-case) name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
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
    /// Length of the period in days (non-negative integer).
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
    /// The working date where the rule's `base` step sits in its
    /// pipeline.
    pub base_date: String,
    /// The final due date, after every step.
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
