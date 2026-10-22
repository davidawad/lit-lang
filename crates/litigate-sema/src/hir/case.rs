//! Resolved `.case` matter.
use super::{ActorRole, Expr, Money};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Resolved and replayable matter (lowers to one `NamedScenario`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Case {
    /// Scenario id (kebab-case).
    pub id: String,
    /// One-line summary.
    pub summary: String,
    /// Free-text notes.
    pub notes: Option<String>,
    /// Scenario-level sources.
    pub sources: Vec<CaseSource>,
    /// Pinned packages in `uses` order; the first is primary.
    pub uses: Vec<PackagePin>,
    /// Governing in-force date.
    pub as_of: String,
    /// Corpus date, when selected.
    pub known_at: Option<String>,
    /// Explicit start node (qualified), else derived from replay.
    pub start: Option<String>,
    /// Analysis perspective.
    pub perspective: Option<Perspective>,
    /// Declared parties.
    pub parties: Vec<Party>,
    /// Chronologically ordered events.
    pub events: Vec<Event>,
    /// Qualified fact node to qualified edge.
    pub facts: BTreeMap<String, String>,
    /// Qualified terminal to stake.
    pub stakes: BTreeMap<String, Stake>,
    /// Observed chance outcomes.
    pub observations: Vec<Observation>,
    /// Remaining litgraph `Scenario` fields.
    pub analysis: Analysis,
    /// Options removed by cited option-removing events, in replay order.
    /// Derived by the checker from `events` (SPEC §5.6), never authored:
    /// checking replaces whatever is here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removals: Vec<Removal>,
}

/// An option a replayed event removed (lowers to `scenario.remove_edges`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Removal {
    /// Qualified edge `pack::edge` no longer available to the matter.
    pub edge: String,
    /// Date of the removing event.
    pub date: String,
    /// Index of the removing event in [`Case::events`] (source order).
    pub event: usize,
    /// The removing event's cite.
    pub authority: String,
}

/// Scenario source (`NamedScenario.sources[]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseSource {
    /// Title.
    pub title: String,
    /// URL.
    pub url: Option<String>,
    /// Version date.
    pub as_of: Option<String>,
}

/// Exact pinned package identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePin {
    /// Package name.
    pub name: String,
    /// Pack id it lowers to.
    pub pack_id: String,
    /// Resolved exact version.
    pub version: String,
    /// SHA-256 source digest.
    pub content_hash: String,
}

/// Perspective selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "perspective", content = "value", rename_all = "snake_case")]
pub enum Perspective {
    /// `perspective <party>`: that party's actor is `self` (SPEC §8.3).
    Party(String),
    /// `perspective { actor: role, ... }`: explicit actor-role override.
    Actors(BTreeMap<String, ActorRole>),
}

/// Party: id, actor key it plays, nominal entity type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Party {
    /// Local party id.
    pub id: String,
    /// Actor key declared in the primary package's `[package.roles]`.
    pub actor: String,
    /// Nominal entity type.
    pub entity_type: String,
}

/// Docket event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Event date.
    pub date: String,
    /// Typed event constructor call.
    pub action: Expr,
    /// Whether the party acted (`by`) or was acted upon (`on`).
    pub relation: EventParty,
    /// Party id.
    pub party: String,
    /// Cite-backed extension/waiver/excuse, if any.
    pub authority: Option<String>,
}

/// Event-party relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventParty {
    /// `by party`.
    By,
    /// `on party`.
    On,
}

/// Terminal valuation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stake", content = "value", rename_all = "snake_case")]
pub enum Stake {
    /// Authored value.
    Value(Money),
    /// Explicitly unvalued (strict error, dev warning).
    Unvalued,
}

/// Observed outcome counts at a chance node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Qualified chance node.
    pub node: String,
    /// Qualified edge to count, in source order.
    pub outcomes: Vec<(String, u64)>,
}

/// `analysis { ... }`: litgraph `Scenario` fields without a dedicated
/// `.case` block, already converted to their JSON values (SPEC §8.3).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// Scenario field name to JSON value.
    pub fields: BTreeMap<String, serde_json::Value>,
}
