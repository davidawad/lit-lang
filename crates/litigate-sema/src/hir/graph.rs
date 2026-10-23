//! Graph nodes, edges, links, and instances: the litgraph-facing part of HIR.
use super::{ActorRole, Authored, DayUnit, Metadata, Money, Rational};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One graph node (`RawNode`) with its outgoing edges (`RawEdge`s).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Id, label, cite, effective interval, relations, span.
    pub meta: Metadata,
    /// Kind plus kind-specific data.
    pub kind: NodeKind,
    /// Group id (`group` field).
    pub group: Option<String>,
    /// Commentary (`note`).
    pub note: Option<String>,
    /// Display valence.
    pub valence: Option<Valence>,
    /// Filed example URL (`courtListenerUrl`).
    pub court_listener_url: Option<String>,
    /// Tags in authored order (`entry`, `fact`, `router`, ...).
    pub tags: Vec<String>,
    /// Numeric attributes, exact.
    pub attrs: BTreeMap<String, Rational>,
    /// Default actor for edges that name none (`by` on the node).
    pub actor: Option<String>,
    /// Outgoing edges in source order.
    pub edges: Vec<Edge>,
    /// Explicit holes in the node's header and fields, by dotted field
    /// path (`cite`, `payoff`, `attrs.weight`), with their reasons.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub holes: BTreeMap<String, Option<String>>,
}

/// Node kind. `Chance` is sugar that lowers to litgraph kind `state`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// Procedural posture (`state`).
    State,
    /// Actor-controlled choice (`decision`).
    Decision,
    /// Pure world draw: every edge has `p` and they sum to one (`state`).
    Chance {
        /// Tribunal discretion marker (verification metadata only).
        discretion: bool,
    },
    /// End of the matter for this pack (`terminal`).
    Terminal {
        /// Outcome tags in authored order.
        outcomes: Vec<String>,
        /// Protagonist payoff.
        payoff: Option<Authored<Money>>,
        /// Payoff override keyed by state flag (`payoffByFlag`).
        payoff_by_flag: BTreeMap<String, Money>,
    },
}

/// Display valence of a node or edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Valence {
    /// Favourable.
    Good,
    /// Risky.
    Caution,
    /// Unfavourable.
    Bad,
    /// Neither.
    Neutral,
}

/// Directed transition (`RawEdge`); `from` is the enclosing node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Stable local id; `None` for an anonymous `_` edge (no `id` emitted).
    pub id: Option<String>,
    /// Target node: local id, or `<pack-id>::<id>` across packages.
    pub to: String,
    /// Display label.
    pub label: String,
    /// Resolved actor: edge `by`, else node `by`, else `either`.
    pub actor: String,
    /// Governing authority text (`authority`).
    pub authority: Option<String>,
    /// Exact probability, if authored.
    pub probability: Option<Authored<Rational>>,
    /// Attorney hours, exact.
    pub hours: Option<Rational>,
    /// Out-of-pocket cost.
    pub cost: Option<Authored<Money>>,
    /// Event pattern that takes this edge during case replay (`event`).
    pub event: Option<super::Expr>,
    /// Deadline window.
    pub deadline: Option<EdgeDeadline>,
    /// Expected elapsed calendar days (triangular).
    pub duration: Option<TriangularDuration>,
    /// Display valence.
    pub valence: Option<Valence>,
    /// Filed example URL.
    pub court_listener_url: Option<String>,
    /// Rulepack action cross-reference (`actionId`).
    pub action_id: Option<String>,
    /// Commentary.
    pub note: Option<String>,
    /// Tags in authored order.
    pub tags: Vec<String>,
    /// Numeric attributes, exact.
    pub attrs: BTreeMap<String, Rational>,
    /// Flags set when taken.
    pub sets: Vec<String>,
    /// Flags cleared when taken.
    pub clears: Vec<String>,
    /// Flags that must all be set.
    pub requires: Vec<String>,
    /// Flags that must all be absent.
    pub forbids: Vec<String>,
    /// Source span.
    pub span: Option<litigate_base::Span>,
    /// Explicit holes in the edge's clauses and fields, by dotted field
    /// path (`p`, `effort`, `duration.min`, `authority`), with their
    /// reasons.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub holes: BTreeMap<String, Option<String>>,
}

/// Edge deadline (`RawEdge.deadline`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeDeadline {
    /// Where the window length comes from.
    pub window: DeadlineWindow,
    /// `extendable`.
    pub extendable: Option<bool>,
    /// `extensionAuthority`.
    pub extension_authority: Option<String>,
    /// `note`.
    pub note: Option<String>,
}

/// Deadline window source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "window", rename_all = "snake_case")]
pub enum DeadlineWindow {
    /// `within <deadline-ref>`: amount and unit from the declaration in force.
    Declared {
        /// Fully qualified deadline declaration id.
        deadline: String,
    },
    /// `deadline 30.days`: a literal window with no trigger semantics.
    Inline {
        /// Length in days.
        length: u64,
        /// Unit (`business` has no litgraph lowering: L6001).
        unit: DayUnit,
    },
}

/// Triangular duration in calendar days (`RawEdge.duration`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriangularDuration {
    /// Optimistic.
    pub min: Option<Rational>,
    /// Most likely.
    pub mode: Rational,
    /// Pessimistic.
    pub max: Option<Rational>,
}

/// Cross-package link (`LinkFile.links[]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// Qualified source node `<pack-or-instance>::<node>`.
    pub from: String,
    /// The edge; `to` is qualified.
    pub edge: Edge,
    /// Qualified edge ids this link supersedes.
    pub replaces: Vec<String>,
}

/// Namespaced pack copy (`LinkFile.instances`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    /// Instance name `base@origin`.
    pub name: String,
    /// Base pack id.
    pub pack: String,
    /// Why the instance exists.
    pub note: Option<String>,
    /// Local edge ids removed.
    pub remove_edges: Vec<String>,
    /// Local edge id to probability.
    pub probabilities: BTreeMap<String, Rational>,
    /// Actor role overrides.
    pub roles: BTreeMap<String, ActorRole>,
    /// litgraph expression over `payoff` and `stake`, validated at lowering.
    pub payoff_transform: Option<String>,
    /// The value of `stake` in `payoff_transform`, or a hole when only the
    /// matter can know it.
    pub stake: Option<Authored<Money>>,
    /// Local edge id to RFC 7386 merge patch over `RawEdge` JSON.
    pub patch_edges: BTreeMap<String, serde_json::Value>,
}
