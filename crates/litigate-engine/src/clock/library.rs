// SPDX-License-Identifier: GPL-3.0-or-later
//! The clock library: holiday calendars and counting rules loaded from
//! `clock/*.json` fragments (the shared library, `lit/core` lowered, is
//! embedded at build time), merged into one namespace that a pack's
//! `counting` field resolves against.
//!
//! Each file is a fragment `{ "schemaVersion": 3, "calendars": [...],
//! "countingRules": [...] }`. Fragments merge in file-name order; a
//! fragment that fails to parse or validate, or that redefines an id
//! already loaded, is skipped and reported, never silently merged. A rule
//! whose holiday calendars do not resolve is dropped and reported.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::calendar::HolidayCalendar;
use super::interp::Clock;
use super::rule::CountingRule;
use crate::error::{Error, Result};
use crate::model::{check_header, Lowered};

include!(concat!(env!("OUT_DIR"), "/embedded_clock.rs"));

/// The newest clock-library `schemaVersion`: `4` adds the `lowered`
/// header to `3`. Both are read.
pub const CLOCK_SCHEMA_VERSION: u32 = 4;

/// A set of holiday calendars and counting rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClockLibrary {
    /// `3`, or `4` (`lowered`).
    pub schema_version: u32,
    /// Holiday calendars.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calendars: Vec<HolidayCalendar>,
    /// Counting rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counting_rules: Vec<CountingRule>,
    /// Build provenance of a compiled fragment (v4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowered: Option<Lowered>,
}

impl ClockLibrary {
    /// Parse, version-check and validate one fragment (calendar references
    /// are checked after merging, since they may cross fragments).
    ///
    /// # Errors
    /// `Error::Parse` on malformed JSON, unknown fields or a wrong
    /// `schemaVersion`; `Error::Invalid` from calendar/rule validation or
    /// a duplicate id within the fragment.
    pub fn from_json(text: &str) -> Result<ClockLibrary> {
        let lib: ClockLibrary =
            serde_json::from_str(text).map_err(|e| Error::Parse(e.to_string()))?;
        if !(3..=CLOCK_SCHEMA_VERSION).contains(&lib.schema_version) {
            return Err(Error::Parse(format!(
                "unsupported clock schemaVersion {}; expected 3 or {CLOCK_SCHEMA_VERSION}",
                lib.schema_version
            )));
        }
        check_header("clock", lib.schema_version, 4, lib.lowered.as_ref(), &lib)?;
        lib.calendars
            .iter()
            .try_for_each(HolidayCalendar::validate)?;
        lib.counting_rules
            .iter()
            .try_for_each(CountingRule::validate)?;
        let empty = ClockLibrary::default();
        if let Some(dup) = empty.collisions(&lib).first() {
            return Err(Error::Invalid(format!("duplicate id {dup}")));
        }
        Ok(lib)
    }

    /// Ids in `other` that repeat within `other` itself, or collide with a
    /// different definition already here. A calendar `other` restates
    /// unchanged is no collision: two packages' rules may close on one
    /// shared calendar, and each fragment carries it.
    fn collisions(&self, other: &ClockLibrary) -> Vec<String> {
        let mut cals: BTreeSet<&str> = BTreeSet::new();
        let mut rules: BTreeSet<&str> = self.counting_rules.iter().map(|r| r.id.as_str()).collect();
        let mut out = vec![];
        for c in &other.calendars {
            let differs = self.calendars.iter().any(|k| k.id == c.id && k != c);
            if !cals.insert(&c.id) || differs {
                out.push(format!("calendar {}", c.id));
            }
        }
        for r in &other.counting_rules {
            if !rules.insert(&r.id) {
                out.push(format!("counting rule {}", r.id));
            }
        }
        out
    }

    /// Merge `(file name, JSON text)` fragments in file-name order. Returns
    /// the library and `(file, message)` for every fragment or rule that
    /// was skipped.
    #[must_use]
    pub fn merge(
        files: impl IntoIterator<Item = (String, String)>,
    ) -> (ClockLibrary, Vec<(String, String)>) {
        let mut files: Vec<(String, String)> = files.into_iter().collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut lib = ClockLibrary {
            schema_version: CLOCK_SCHEMA_VERSION,
            ..ClockLibrary::default()
        };
        let mut errors = vec![];
        for (name, text) in files {
            let frag = match ClockLibrary::from_json(&text) {
                Ok(f) => f,
                Err(e) => {
                    errors.push((name.clone(), format!("{name}: {e}")));
                    continue;
                }
            };
            let dups = lib.collisions(&frag);
            if dups.is_empty() {
                let new: Vec<HolidayCalendar> = frag
                    .calendars
                    .into_iter()
                    .filter(|c| lib.calendars.iter().all(|k| k.id != c.id))
                    .collect();
                lib.calendars.extend(new);
                lib.counting_rules.extend(frag.counting_rules);
            } else {
                errors.push((
                    name.clone(),
                    format!("{name}: redefines {}; fragment skipped", dups.join(", ")),
                ));
            }
        }
        let (kept, dangling): (Vec<CountingRule>, Vec<CountingRule>) =
            lib.counting_rules.drain(..).partition(|r| {
                r.holidays
                    .iter()
                    .chain(&r.forward_holidays)
                    .all(|h| lib.calendars.iter().any(|c| &c.id == h))
            });
        lib.counting_rules = kept;
        errors.extend(dangling.into_iter().map(|r| {
            (
                "clock".to_string(),
                format!(
                    "counting rule {} names unknown holiday calendar(s) {}; rule skipped",
                    r.id,
                    r.holidays
                        .iter()
                        .chain(&r.forward_holidays)
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        }));
        (lib, errors)
    }

    /// The library built from the embedded shared clock library, with any
    /// fragment errors (none for a well-formed build; the test suite checks).
    #[must_use]
    pub fn embedded() -> &'static (ClockLibrary, Vec<(String, String)>) {
        static LIB: OnceLock<(ClockLibrary, Vec<(String, String)>)> = OnceLock::new();
        LIB.get_or_init(|| {
            ClockLibrary::merge(
                EMBEDDED_CLOCK
                    .iter()
                    .map(|(n, t)| ((*n).to_string(), (*t).to_string())),
            )
        })
    }

    /// Resolve counting rule `id` against its holiday calendars.
    ///
    /// # Errors
    /// `Error::NotFound` listing the available rule ids, or naming a
    /// calendar that is missing.
    pub fn clock(&self, id: &str) -> Result<Clock<'_>> {
        let rule = self
            .counting_rules
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| {
                let ids: Vec<&str> = self.counting_rules.iter().map(|r| r.id.as_str()).collect();
                Error::NotFound(format!(
                    "counting rule {id:?}; available: {}",
                    ids.join(", ")
                ))
            })?;
        let resolve = |ids: &[String]| {
            ids.iter()
                .map(|h| {
                    self.calendars
                        .iter()
                        .find(|c| &c.id == h)
                        .ok_or_else(|| Error::NotFound(format!("holiday calendar {h:?}")))
                })
                .collect::<Result<Vec<_>>>()
        };
        Ok(Clock {
            rule,
            calendars: resolve(&rule.holidays)?,
            forward_calendars: resolve(&rule.forward_holidays)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_library_loads_cleanly_and_every_rule_resolves() {
        let (lib, errors) = ClockLibrary::embedded();
        assert!(errors.is_empty(), "{errors:?}");
        assert!(!lib.counting_rules.is_empty());
        for r in &lib.counting_rules {
            assert!(lib.clock(&r.id).is_ok(), "{}", r.id);
        }
        assert!(lib.clock("no-such-rule").is_err());
    }

    fn frag(cal: &str, rule_cal: &str) -> String {
        serde_json::json!({
            "schemaVersion": 3,
            "calendars": [{"id": cal, "title": "C", "holidays": []}],
            "countingRules": [{
                "id": format!("r-{cal}"), "label": "R", "sourceUrl": "u",
                "closedWeekdays": ["sunday"], "holidays": [rule_cal],
                "steps": [
                    {"op": "anchor", "kind": "trigger", "text": "t"},
                    {"op": "count", "periods": [{"count": "calendar", "text": "c"}]},
                    {"op": "base"},
                    {"op": "service", "text": "s", "backwardText": "b", "methods": [],
                     "unrecognized": crate::clock::ServiceMethod::ALL}
                ]
            }]
        })
        .to_string()
    }

    #[test]
    fn merge_skips_redefinitions_bad_fragments_and_dangling_rules() {
        let mut other_a: serde_json::Value = serde_json::from_str(&frag("a", "a")).unwrap();
        other_a["calendars"][0]["title"] = serde_json::json!("another a");
        other_a["countingRules"][0]["id"] = serde_json::json!("r-other");
        let (lib, errors) = ClockLibrary::merge([
            ("clock/a.json".to_string(), frag("a", "a")),
            ("clock/b.json".to_string(), other_a.to_string()),
            ("clock/c.json".to_string(), "{ not json".to_string()),
            ("clock/d.json".to_string(), frag("d", "missing")),
            (
                "clock/e.json".to_string(),
                r#"{"schemaVersion": 2}"#.to_string(),
            ),
        ]);
        assert_eq!(lib.calendars.len(), 2, "a and d");
        assert_eq!(lib.counting_rules.len(), 1, "only r-a resolves");
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors[0].1.contains("redefines calendar a"));
        assert!(errors
            .iter()
            .any(|e| e.1.contains("unknown holiday calendar")));
        assert!(lib.clock("r-a").is_ok());
    }

    #[test]
    fn merge_shares_a_calendar_two_fragments_restate_unchanged() {
        let mut b: serde_json::Value = serde_json::from_str(&frag("a", "a")).unwrap();
        b["countingRules"][0]["id"] = serde_json::json!("r-b");
        b["countingRules"][0]["forwardHolidays"] = serde_json::json!(["a"]);
        let (lib, errors) = ClockLibrary::merge([
            ("clock/a.json".to_string(), frag("a", "a")),
            ("clock/b.json".to_string(), b.to_string()),
        ]);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(lib.calendars.len(), 1);
        assert_eq!(lib.clock("r-b").unwrap().forward_calendars.len(), 1);
        b["countingRules"][0]["forwardHolidays"] = serde_json::json!(["missing"]);
        let (lib, errors) = ClockLibrary::merge([("clock/b.json".to_string(), b.to_string())]);
        assert!(lib.counting_rules.is_empty());
        assert!(errors[0].1.contains("a, missing"), "{errors:?}");
    }

    #[test]
    fn a_fragment_with_internal_duplicates_is_rejected() {
        let mut v: serde_json::Value = serde_json::from_str(&frag("a", "a")).unwrap();
        let cal = v["calendars"][0].clone();
        v["calendars"].as_array_mut().unwrap().push(cal);
        assert!(ClockLibrary::from_json(&v.to_string()).is_err());
    }
}
