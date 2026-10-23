// SPDX-License-Identifier: GPL-3.0-or-later
//! Counting rules as data: the `CountingRule` IR.
//!
//! A counting rule is an ordered pipeline of [`Step`]s over a working date
//! (anchor, count, service extension, base marker, last-day roll,
//! clerk-inaccessibility extension), plus an optional hours provision.
//! Every step carries its own cited `text` template, so the trace the
//! interpreter emits is authored alongside the arithmetic it describes.
//! The shipped rules live in `packs/clock/*.json`; the interpreter in
//! `clock::interp` knows no forum.
//!
//! Text templates may use these placeholders (anything else is rejected
//! when the library loads):
//!
//! | placeholder | value |
//! |---|---|
//! | `{trigger}` | the trigger date |
//! | `{anchor}` | the anchor date (after the `anchor` step) |
//! | `{date}` | the date this step produced |
//! | `{n}` | the period length (days, or hours for the hours provision) |
//! | `{after}` | `after` / `before`, by counting direction |
//! | `{forward}` | `forward` / `backward`, by counting direction |
//! | `{method}` | the service method's name (e.g. `Mail`, `CommercialCarrier`) |
//! | `{days}` | days added by the service method |
//! | `{unit}` | `month(s)` / `year(s)`, for the months provision |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::calendar::Weekday;
use super::types::{ServiceMethod, Unit};
use crate::error::{Error, Result};

/// The placeholders a step `text` template may use.
pub const PLACEHOLDERS: [&str; 9] = [
    "trigger", "anchor", "date", "n", "after", "forward", "method", "days", "unit",
];

/// Where day 1 of a period is anchored.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorKind {
    /// The trigger day itself is the anchor and is excluded: day 1 is the
    /// next calendar day.
    Trigger,
    /// The first business day after (forward) or before (backward) the
    /// trigger is the anchor, and it counts as day 1.
    AdjacentBusinessDay,
}

/// Which days a count step advances over.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CountUnit {
    /// Every day counts.
    Calendar,
    /// Only business days (not a closed weekday, not a holiday) count.
    Business,
}

/// Which way a roll moves a closed day.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Toward {
    /// To the next business day.
    Later,
    /// To the previous business day.
    Earlier,
}

/// One way of counting a period. The first [`Period`] whose conditions
/// hold for a deadline is used.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Period {
    /// Applies only to deadlines authored with this `unit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    /// Applies only to periods shorter than this many days (a
    /// short-period rule with an alternate unit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below: Option<i64>,
    /// Which days the count advances over.
    pub count: CountUnit,
    /// Cited trace text.
    pub text: String,
}

/// A last-day roll for one counting direction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Roll {
    /// Direction a closed day moves.
    pub toward: Toward,
    /// Cited trace text, emitted only when the date actually moves.
    pub text: String,
}

/// Days a service method adds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceExtension {
    /// The service method.
    pub method: ServiceMethod,
    /// Calendar days added.
    pub days: i64,
}

/// One operation in a counting rule's pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Step {
    /// Fix the anchor (must be the first step).
    Anchor {
        /// Anchor rule.
        kind: AnchorKind,
        /// Cited trace text.
        text: String,
    },
    /// Count the period from the anchor (exactly one).
    Count {
        /// Candidate counting modes, first match wins.
        periods: Vec<Period>,
    },
    /// Add a service-method extension to forward periods.
    Service {
        /// Recognized methods, in the order error messages list them.
        methods: Vec<ServiceExtension>,
        /// Cited trace text for an applied extension.
        text: String,
        /// Trace text when a method is given for a backward period.
        backward_text: String,
    },
    /// Record the working date as the result's `base_date` (exactly one).
    Base {},
    /// Move a closed last day.
    Roll {
        /// Roll for forward periods.
        forward: Roll,
        /// Roll for backward periods.
        backward: Roll,
    },
    /// The clerk's-office-inaccessible extension: the next business day
    /// after the last day.
    ClerkInaccessible {
        /// Cited trace text; absent when the rule has no such extension
        /// (the request is then ignored with `ignoredText`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// Trace text when the extension is requested but does not apply.
        ignored_text: String,
    },
}

/// A minimum period length with its own cited error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MinLength {
    /// Smallest allowed period, in days.
    pub days: i64,
    /// Error message for a shorter period.
    pub message: String,
}

/// An hours-based period provision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hours {
    /// Cited trace text for the count.
    pub text: String,
    /// Cited trace text when the end lands on a closed day.
    pub roll_text: String,
    /// Cited trace text for the clerk's-office-inaccessible extension of
    /// an hours period (to the same time on the next accessible day);
    /// absent when the rule has no such extension, and the request is
    /// then ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inaccessible_text: Option<String>,
}

/// Where a month or year period ends when its last month has no day
/// corresponding to the anchor's (January 31 plus one month).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EndOfMonth {
    /// On the last day of that month (MPEP 710.01(a): a 3-month period
    /// from November 30 ends February 28, or 29).
    Clamp,
    /// On the first day of the following month.
    Roll,
}

/// A calendar-month period provision: months and years (twelve months)
/// count from the anchor to the corresponding day of the later month.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Months {
    /// Where a period ends when its last month is too short.
    pub end_of_month: EndOfMonth,
    /// Cited trace text for the count.
    pub text: String,
}

/// A deadline-counting rule (`CountingRule` IR).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CountingRule {
    /// Stable id that a pack's `counting` field references.
    pub id: String,
    /// Short citation used in traces and errors.
    pub label: String,
    /// The primary source the rule was authored from.
    pub source_url: String,
    /// Weekdays the forum is closed (a last day on one rolls).
    pub closed_weekdays: Vec<Weekday>,
    /// Ids of the holiday calendars the forum closes on (unioned).
    pub holidays: Vec<String>,
    /// Smallest allowed period, if larger than zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<MinLength>,
    /// The day-counting pipeline.
    pub steps: Vec<Step>,
    /// Hours provision; absent means hours-based periods are rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<Hours>,
    /// Calendar-month provision; absent means month and year periods are
    /// rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub months: Option<Months>,
}

fn check_template(rule: &str, t: &str) -> Result<()> {
    let mut rest = t;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let close = after.find('}').ok_or_else(|| {
            Error::Invalid(format!("counting rule {rule}: unclosed '{{' in {t:?}"))
        })?;
        let name = &after[..close];
        if !PLACEHOLDERS.contains(&name) {
            return Err(Error::Invalid(format!(
                "counting rule {rule}: unknown placeholder {{{name}}} in {t:?}; allowed: {}",
                PLACEHOLDERS.join(", ")
            )));
        }
        rest = &after[close + 1..];
    }
    Ok(())
}

impl CountingRule {
    /// The anchor kind (validated rules start with an `anchor` step).
    #[must_use]
    pub fn anchor_kind(&self) -> Option<AnchorKind> {
        match self.steps.first() {
            Some(Step::Anchor { kind, .. }) => Some(*kind),
            _ => None,
        }
    }

    fn templates(&self) -> Vec<&str> {
        let mut out: Vec<&str> = vec![];
        for s in &self.steps {
            match s {
                Step::Anchor { text, .. } => out.push(text),
                Step::Count { periods } => out.extend(periods.iter().map(|p| p.text.as_str())),
                Step::Service {
                    text,
                    backward_text,
                    ..
                } => out.extend([text.as_str(), backward_text.as_str()]),
                Step::Base {} => {}
                Step::Roll { forward, backward } => {
                    out.extend([forward.text.as_str(), backward.text.as_str()]);
                }
                Step::ClerkInaccessible { text, ignored_text } => {
                    out.extend(text.as_deref());
                    out.push(ignored_text);
                }
            }
        }
        if let Some(h) = &self.hours {
            out.extend([h.text.as_str(), h.roll_text.as_str()]);
            out.extend(h.inaccessible_text.as_deref());
        }
        out.extend(self.months.as_ref().map(|m| m.text.as_str()));
        out
    }

    /// Check the pipeline's shape and every text template. Calendar
    /// references are checked by the library, which owns the calendars.
    ///
    /// # Errors
    /// `Error::Invalid` naming the rule and the defect.
    pub fn validate(&self) -> Result<()> {
        let bad = |why: &str| Err(Error::Invalid(format!("counting rule {}: {why}", self.id)));
        let count = |f: fn(&Step) -> bool| self.steps.iter().filter(|s| f(s)).count();
        if self.anchor_kind().is_none() || count(|s| matches!(s, Step::Anchor { .. })) != 1 {
            return bad("the first step, and only that step, must be `anchor`");
        }
        if count(|s| matches!(s, Step::Count { .. })) != 1 {
            return bad("exactly one `count` step is required");
        }
        if count(|s| matches!(s, Step::Base {})) != 1 {
            return bad("exactly one `base` step is required");
        }
        if count(|s| matches!(s, Step::Service { .. })) > 1
            || count(|s| matches!(s, Step::ClerkInaccessible { .. })) > 1
        {
            return bad("at most one `service` and one `clerkInaccessible` step");
        }
        let count_pos = self
            .steps
            .iter()
            .position(|s| matches!(s, Step::Count { .. }));
        if count_pos != Some(1) {
            return bad("`count` must immediately follow `anchor`");
        }
        for s in &self.steps {
            match s {
                Step::Count { periods } if periods.is_empty() => {
                    return bad("`count` needs at least one period");
                }
                Step::Count { periods }
                    if periods.iter().any(|p| p.below.is_some_and(|b| b < 1)) =>
                {
                    return bad("a period's `below` must be >= 1");
                }
                Step::Service { methods, .. } => {
                    for (i, m) in methods.iter().enumerate() {
                        if m.days < 0 {
                            return bad("service extension days must be >= 0");
                        }
                        if methods[..i].iter().any(|o| o.method == m.method) {
                            return bad("duplicate service method");
                        }
                    }
                }
                _ => {}
            }
        }
        if self.anchor_kind() == Some(AnchorKind::AdjacentBusinessDay)
            && self.min_length.as_ref().is_none_or(|m| m.days < 1)
        {
            return bad("an `adjacent-business-day` anchor counts it as day 1, so `minLength.days` must be >= 1");
        }
        if self.holidays.is_empty() && self.closed_weekdays.is_empty() {
            return bad("a rule must close on some weekday or holiday calendar");
        }
        self.templates()
            .into_iter()
            .try_for_each(|t| check_template(&self.id, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(steps: &serde_json::Value) -> serde_json::Result<CountingRule> {
        serde_json::from_value(serde_json::json!({
            "id": "r", "label": "R", "sourceUrl": "u",
            "closedWeekdays": ["saturday", "sunday"], "holidays": [],
            "steps": steps
        }))
    }

    fn minimal() -> serde_json::Value {
        serde_json::json!([
            {"op": "anchor", "kind": "trigger", "text": "exclude {trigger}"},
            {"op": "count", "periods": [{"count": "calendar", "text": "count {n} -> {date}"}]},
            {"op": "base"}
        ])
    }

    #[test]
    fn a_minimal_pipeline_validates() {
        assert!(rule(&minimal()).unwrap().validate().is_ok());
    }

    #[test]
    fn unknown_fields_and_ops_are_rejected() {
        let mut v = minimal();
        v[2] = serde_json::json!({"op": "base", "extra": 1});
        assert!(rule(&v).is_err());
        let mut v = minimal();
        v[2] = serde_json::json!({"op": "guess"});
        assert!(rule(&v).is_err());
    }

    #[test]
    fn unknown_placeholder_is_rejected() {
        let mut v = minimal();
        v[0]["text"] = serde_json::json!("exclude {trigger_day}");
        let err = rule(&v).unwrap().validate().unwrap_err().to_string();
        assert!(err.contains("{trigger_day}"), "{err}");
        let mut v = minimal();
        v[0]["text"] = serde_json::json!("exclude {trigger");
        assert!(rule(&v).unwrap().validate().is_err());
    }

    #[test]
    fn pipeline_shape_is_enforced() {
        let v = serde_json::json!([
            {"op": "count", "periods": [{"count": "calendar", "text": "t"}]},
            {"op": "anchor", "kind": "trigger", "text": "t"},
            {"op": "base"}
        ]);
        assert!(
            rule(&v).unwrap().validate().is_err(),
            "anchor must be first"
        );
        let mut v = minimal();
        v.as_array_mut().unwrap().pop();
        assert!(rule(&v).unwrap().validate().is_err(), "base is required");
        let mut v = minimal();
        v[1]["periods"] = serde_json::json!([]);
        assert!(rule(&v).unwrap().validate().is_err(), "empty periods");
        let mut v = minimal();
        v[0]["kind"] = serde_json::json!("adjacent-business-day");
        assert!(
            rule(&v).unwrap().validate().is_err(),
            "inclusive anchor needs a minimum length"
        );
    }

    #[test]
    fn duplicate_service_methods_are_rejected() {
        let mut v = minimal();
        v.as_array_mut().unwrap().push(serde_json::json!({
            "op": "service", "text": "t", "backwardText": "b",
            "methods": [{"method": "mail", "days": 3}, {"method": "mail", "days": 5}]
        }));
        assert!(rule(&v).unwrap().validate().is_err());
    }
}
