// SPDX-License-Identifier: GPL-3.0-or-later
//! Named scenario library: matter profiles (`scenarios/*.json`) loadable by
//! name from a request's `scenario` field, alone or composed with inline
//! overrides via `extends`. See `docs/PACK_SCHEMA.md` for the shape a file
//! must have and `api::Catalog::scenarios`/`scenario` for how it is loaded.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{check_header, Lowered};
use crate::scenario::Scenario;

/// The newest named-scenario `schema_version`: `2` adds `lowered`,
/// `parties` and `events` (absent is `1`).
pub const SCENARIO_SCHEMA_VERSION: u32 = 2;

/// A party to the matter (v2).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScenarioParty {
    /// The matter's id for the party.
    pub id: String,
    /// The pack actor key it plays.
    pub actor: String,
    /// Its entity type, as the matter declares it.
    pub entity_type: String,
}

/// Whether a docket event's party acted or was acted upon.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EventRelation {
    /// `by party`: the party acted.
    By,
    /// `on party`: the party was acted upon.
    On,
}

/// One docket event of the matter, in docket order (v2). The matter's
/// compiler replayed these to derive `scenario.start` and
/// `scenario.remove_edges`; the engine reads them as record only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocketEvent {
    /// Event date (`YYYY-MM-DD`).
    pub date: String,
    /// Docket sequence number (`docket N`), ordering the event among
    /// others of its date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket: Option<u64>,
    /// The typed event, spelled as in the source (`Served(complaint, Mail)`).
    pub action: String,
    /// `by` or `on`.
    pub relation: EventRelation,
    /// The party's id.
    pub party: String,
    /// The authority cited for it, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
}

/// A primary source backing one fact or number in a named scenario (a rate,
/// a stake, a probability, a deadline). Mirrors a pack's `sources` entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScenarioSource {
    /// What this source backs, e.g. "bid-protest sustain rate".
    pub title: String,
    /// Official URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Effective / retrieval date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
}

/// One named matter profile: which packs it needs, a human summary and
/// notes (facts, and the basis for every number), its sourced facts, and the
/// engine [`Scenario`] itself (rates, stakes, perspective, payoffs,
/// probabilities, policy, modeling modes).
///
/// A request references one by name (`"scenario": "<id>"`) or composes it
/// with inline overrides (`"scenario": {"extends": "<id>", ...overrides}`,
/// deep-merged per RFC 7386). See `crate::api::scenario_ref::resolve`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NamedScenario {
    /// `1` when absent, or `2` (`lowered`, `parties`, `events`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    /// Stable name a request references this scenario by; by convention
    /// matches the file stem (`scenarios/<id>.json`), though lookup also
    /// accepts the file stem when it differs.
    pub id: String,
    /// One-line summary; `describe` lists every scenario by this.
    pub summary: String,
    /// Free-text notes: facts, the basis for every rate/stake/probability
    /// number (sourced vs illustrative), and any modeling judgment calls.
    #[serde(default)]
    pub notes: String,
    /// Pack refs this scenario needs. A request that names this scenario and
    /// supplies no `packs` of its own uses these; an explicit `packs` on the
    /// request always wins.
    #[serde(default)]
    pub packs: Vec<String>,
    /// Sourced facts backing this scenario's numbers.
    #[serde(default)]
    pub sources: Vec<ScenarioSource>,
    /// The engine scenario: params, perspective, masks, payoffs,
    /// probabilities, policy, modeling modes.
    #[serde(default)]
    pub scenario: Scenario,
    /// The matter's parties (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parties: Vec<ScenarioParty>,
    /// The matter's docket events (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<DocketEvent>,
    /// Build provenance and explicit holes of a compiled matter (v2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowered: Option<Lowered>,
}

impl NamedScenario {
    /// Parse and sanity-check a scenario-library file.
    ///
    /// # Errors
    /// `Error::Parse` on malformed JSON, an unknown field, an empty
    /// `id`/`summary`, an unsupported `schema_version`, v2 fields in a v1
    /// file, or a `lowered` header that does not match.
    pub fn from_json(text: &str) -> Result<NamedScenario> {
        let def: NamedScenario =
            serde_json::from_str(text).map_err(|e| Error::Parse(e.to_string()))?;
        def.check()?;
        Ok(def)
    }

    /// Sanity-check a scenario-library file already read.
    ///
    /// # Errors
    /// As [`NamedScenario::from_json`].
    pub fn check(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::Parse("scenario: `id` must not be empty".into()));
        }
        let what = format!("scenario {}", self.id);
        if self.summary.trim().is_empty() {
            return Err(Error::Parse(format!("{what}: `summary` must not be empty")));
        }
        let version = self.schema_version.unwrap_or(1);
        if !(1..=SCENARIO_SCHEMA_VERSION).contains(&version) {
            return Err(Error::Parse(format!(
                "{what}: unsupported schema_version {version}"
            )));
        }
        if version < 2 && !(self.parties.is_empty() && self.events.is_empty()) {
            return Err(Error::Parse(format!(
                "{what}: `parties` and `events` need schema_version 2"
            )));
        }
        check_header(&what, version, 2, self.lowered.as_ref(), self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_minimal_named_scenario() -> Result<()> {
        let text = r#"{"id":"t","summary":"a test scenario","packs":["cofc"],"scenario":{"params":{"rate":900}}}"#;
        let def = NamedScenario::from_json(text)?;
        assert_eq!(def.id, "t");
        assert_eq!(def.packs, vec!["cofc".to_string()]);
        assert_eq!(def.scenario.params.get("rate"), Some(&900.0));
        Ok(())
    }

    #[test]
    fn rejects_unknown_fields() {
        let text = r#"{"id":"t","summary":"s","bogus":1}"#;
        assert!(NamedScenario::from_json(text).is_err());
    }

    #[test]
    fn rejects_empty_id_or_summary() {
        assert!(NamedScenario::from_json(r#"{"id":"","summary":"s"}"#).is_err());
        assert!(NamedScenario::from_json(r#"{"id":"t","summary":""}"#).is_err());
    }
}
