// SPDX-License-Identifier: GPL-3.0-or-later
//! Pack file schema (v1 compatible, v2 additive). See `docs/PACK_SCHEMA.md`.
//!
//! Every struct rejects unknown fields: a misspelled key in a pack is an
//! error, not a silently ignored value.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::cite_syntax::CiteSyntax;
use super::lowered::{check_header, Lowered};
use crate::error::{Error, Result};

/// The newest pack `schemaVersion`: v3 plus the `lowered` header.
pub const PACK_SCHEMA_VERSION: u32 = 4;

/// Structural kind of a node.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    /// A stable procedural posture.
    State,
    /// A fork someone must decide.
    Decision,
    /// The matter ends here (for this pack).
    Terminal,
}

/// A window within which a transition must happen.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Deadline {
    /// Length of the window in `unit`s.
    pub length: f64,
    /// `calendar` (default) or `court` days, or `week`, `month`, `year`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Whether extensions are available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extendable: Option<bool>,
    /// Authority for extensions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_authority: Option<String>,
    /// Free-text note (traps, US-party variants, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Deadline {
    /// The window's length in days, for metrics and estimates: the length
    /// itself for calendar and court days, seven per week, and the mean
    /// Gregorian month (146097/4800 days) and year (365.2425 days) for
    /// months and years, whose exact length depends on the trigger date.
    #[must_use]
    pub fn days(&self) -> f64 {
        self.length
            * match self.unit.as_deref() {
                Some("week") => 7.0,
                Some("month") => 146_097.0 / 4800.0,
                Some("year") => 365.2425,
                _ => 1.0,
            }
    }
}

/// Expected elapsed calendar time of a transition, in days (triangular).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Duration {
    /// Optimistic duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Most likely duration.
    pub mode: f64,
    /// Pessimistic duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// A group (visual/hierarchical cluster) of nodes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// Group id referenced by `RawNode::group`.
    pub id: String,
    /// Display label.
    pub label: String,
}

/// A node as authored in a pack.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawNode {
    /// Unique within the pack (kebab-case).
    pub id: String,
    /// Structural kind (default `state`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<NodeKind>,
    /// Human label.
    pub label: String,
    /// Primary legal citation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Commentary (traps, basis for numbers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Group id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// `good` / `caution` / `bad` / `neutral`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valence: Option<String>,
    /// A real filed example of this posture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court_listener_url: Option<String>,
    /// Terminal value in USD from the protagonist's perspective (v2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payoff: Option<f64>,
    /// Outcome tags for terminals, e.g. `win`, `fee-eligible` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome: Vec<String>,
    /// Free-form node tags, e.g. `entry`, `router` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Free numeric attributes visible to custom functions (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, f64>,
    /// Terminal payoff override keyed by flag name (v2): if the compiled
    /// product-graph copy of this node carries one of these flags, its
    /// payoff is this value instead of `payoff`. Author with mutually
    /// exclusive flags; if more than one matches, the first key in sorted
    /// order wins. See `docs/PACK_SCHEMA.md#state-flags`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub payoff_by_flag: BTreeMap<String, f64>,
}

fn default_actor() -> String {
    "either".into()
}

/// An edge as authored in a pack, or a cross-pack link in `links.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawEdge {
    /// Stable id; derived as `from->to#n` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Source node id (qualified `pack::node` in links).
    pub from: String,
    /// Target node id (qualified `pack::node` in links).
    pub to: String,
    /// Human label (the move or ruling).
    pub label: String,
    /// `applicant` / `examiner` / `office` / `either`, read through `roles`.
    #[serde(default = "default_actor")]
    pub actor: String,
    /// Rule or statute authorizing the transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Window within which the transition must occur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<Deadline>,
    /// Expected elapsed time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    /// Cash fees in USD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// Attorney hours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    /// Probability (world edges only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probability: Option<f64>,
    /// `good` / `caution` / `bad` / `neutral`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valence: Option<String>,
    /// A real filed example of this transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court_listener_url: Option<String>,
    /// Cross-reference to a rulepack action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    /// Commentary (basis for numbers, traps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Machine tags, e.g. `waiver-trap`, `appeal` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Free numeric attributes visible to custom functions (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, f64>,
    /// Links only: qualified edge ids this link supersedes when it applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replaces: Vec<String>,
    /// State flags this edge sets when taken (v2). Compiled into a product
    /// graph over `(node, flag-set)`; see `docs/PACK_SCHEMA.md#state-flags`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets: Vec<String>,
    /// State flags this edge clears when taken (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clears: Vec<String>,
    /// Flags that must ALL be set for this edge to exist in the compiled
    /// graph (v2). A flag no edge ever sets makes a `requires` on it always
    /// fail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Flags that must ALL be absent for this edge to exist in the compiled
    /// graph (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbids: Vec<String>,
}

/// A primary source a pack was authored from.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    /// Short id referenced from notes.
    pub id: String,
    /// Document title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Official URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Repo-relative path (e.g. a sibling pack). Never an absolute/local path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// SHA-256 of the document as retrieved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Effective / amendment date of the version used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
}

/// Who controls an edge, from the analysis perspective.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The protagonist whose policy is optimized.
    #[serde(rename = "self")]
    Me,
    /// An adversary with its own choices.
    Opponent,
    /// Tribunal / agency / chance.
    Nature,
}

/// One forum's procedure graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pack {
    /// `1` (civ-pro statechart format), `2`, `3` (`counting`,
    /// `citeSyntax`), or `4` (`lowered`).
    pub schema_version: u32,
    /// Pack id; the namespace prefix of every node and edge id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Jurisdiction or rule body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    /// Short forum key, as authored (a lowercase identifier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum: Option<String>,
    /// Id of the deadline-counting rule used by this pack (schema v3).
    /// The id is resolved from authored pack data, never inferred from the
    /// forum name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<String>,
    /// Where walks and default analyses begin.
    pub start_node_id: String,
    /// Node groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    /// actor → role mapping (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub roles: BTreeMap<String, Role>,
    /// Primary sources (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    /// Citation syntax this pack's `cite`/`authority` strings use (v3):
    /// rule-body prefixes and shorthands, each bound to a source.
    #[serde(default, skip_serializing_if = "CiteSyntax::is_empty")]
    pub cite_syntax: CiteSyntax,
    /// Nodes.
    pub nodes: Vec<RawNode>,
    /// Edges.
    pub edges: Vec<RawEdge>,
    /// Build provenance and explicit holes of a compiled pack (v4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowered: Option<Lowered>,
}

impl Pack {
    /// Parse and version-check a pack.
    ///
    /// # Errors
    /// `Error::Parse` on malformed JSON, unknown fields, an unsupported
    /// `schemaVersion`, an invalid `citeSyntax`, or a `lowered` header that
    /// does not match the pack.
    pub fn from_json(text: &str) -> Result<Pack> {
        let pack: Pack = serde_json::from_str(text).map_err(|e| Error::Parse(e.to_string()))?;
        if !(1..=PACK_SCHEMA_VERSION).contains(&pack.schema_version) {
            return Err(Error::Parse(format!(
                "pack {}: unsupported schemaVersion {}",
                pack.id, pack.schema_version
            )));
        }
        pack.check_cite_syntax()?;
        pack.check_lowered()?;
        if let Some(e) = pack.edges.iter().find(|e| !e.replaces.is_empty()) {
            return Err(Error::Parse(format!(
                "pack {}: edge {}->{} uses `replaces`, which is only valid in links.json",
                pack.id, e.from, e.to
            )));
        }
        Ok(pack)
    }

    /// Check the `lowered` header (v4) against the pack it heads.
    ///
    /// # Errors
    /// `Error::Parse` naming what does not hold.
    pub fn check_lowered(&self) -> Result<()> {
        let what = format!("pack {}", self.id);
        check_header(&what, self.schema_version, 4, self.lowered.as_ref(), self)
    }

    /// Check `citeSyntax` against this pack's own `sources` (see
    /// [`CiteSyntax::validate`]).
    ///
    /// # Errors
    /// `Error::Parse` naming the offending `citeSyntax` entry.
    pub fn check_cite_syntax(&self) -> Result<()> {
        let source_ids: Vec<&str> = self.sources.iter().map(|s| s.id.as_str()).collect();
        self.cite_syntax.validate(&self.id, &source_ids)
    }
}

/// The v1 semantics: only the applicant chooses; everything else is a draw.
#[must_use]
pub fn default_roles() -> BTreeMap<String, Role> {
    BTreeMap::from([
        ("applicant".to_string(), Role::Me),
        ("examiner".to_string(), Role::Nature),
        ("office".to_string(), Role::Nature),
        ("either".to_string(), Role::Nature),
    ])
}
