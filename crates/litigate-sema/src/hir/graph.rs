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
    /// Information set (`info`): a decision its actor cannot tell apart
    /// from the other decisions naming the same set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
    /// Where the node's numbers (payoffs, attrs) come from (`basis`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<Basis>,
    /// Outgoing edges in source order.
    pub edges: Vec<Edge>,
    /// Ids this node had in earlier releases (`formerly`, SPEC §4.3): the
    /// node's identity is its id together with these, so a matter or a
    /// link written against an old id still resolves to it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formerly: Vec<String>,
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
        /// `fee_shift`: the fraction of our accumulated cost a fee award
        /// recovers when the matter ends here, as the governing statute
        /// sets it by default (`feeShift`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fee_shift: Option<Rational>,
    },
}

/// Where a node's or edge's numbers come from (`basis`; design review
/// 2026-10-23 §2.10). A number with none is an estimate; a `?` is a hole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "basis", rename_all = "snake_case")]
pub enum Basis {
    /// `sourced("cite")`: read from a cited primary source.
    Sourced {
        /// The cite text.
        cite: String,
    },
    /// `calibrated("dataset", date)`: measured on a dataset as of a date.
    Calibrated {
        /// The dataset.
        dataset: String,
        /// Its date.
        as_of: String,
    },
    /// `expert("name")`: a named expert's judgment.
    Expert {
        /// Who.
        name: String,
    },
    /// `estimate`: a working estimate, said so.
    Estimate,
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
    /// Nodes where taking the edge starts a concurrent track each (`spawn`):
    /// the matter's marking gains a token there (SPEC §5.6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spawn: Vec<String>,
    /// Nodes whose tracks the edge joins (`joins`): it is enabled only
    /// while each holds a token, and taking it consumes them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joins: Vec<String>,
    /// Guard of a fact edge (`when`): a condition over the matter's
    /// history that, when it holds, derives the fact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<super::Expr>,
    /// Where the edge's numbers (`p`, effort, cost, duration, attrs) come
    /// from (`basis`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<Basis>,
    /// Ids this edge had in earlier releases (`formerly`, SPEC §4.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formerly: Vec<String>,
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
    /// `counting "id"` on a literal window: the clock-library rule it is
    /// counted under when not the pack's (`Deadline.counting`). A `within`
    /// window is counted by its declaration's own rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<String>,
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
    /// `payoff_transform (e)`: a typed `Money<usd>` expression over the
    /// locals `payoff` and `stake` (SPEC §8.4).
    pub payoff_transform: Option<super::Expr>,
    /// The value of `stake` in `payoff_transform`, or a hole when only the
    /// matter can know it.
    pub stake: Option<Authored<Money>>,
    /// Local edge id to its `patch`: typed edge fields replacing the base
    /// edge's.
    pub patch_edges: BTreeMap<String, EdgePatch>,
}

/// `patch edge { ... }` (SPEC §8.4): the edge fields a patch writes, typed
/// as on an edge declaration. Each written field replaces the base edge's
/// whole field (`within`/`deadline` its whole window); a field the patch
/// does not write is `None` and the base edge keeps it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgePatch {
    /// `authority`.
    pub authority: Option<String>,
    /// `effort`, exact hours.
    pub hours: Option<Rational>,
    /// `cost`.
    pub cost: Option<Authored<Money>>,
    /// `within` or `deadline`.
    pub deadline: Option<EdgeDeadline>,
    /// `duration`.
    pub duration: Option<TriangularDuration>,
    /// `valence`.
    pub valence: Option<Valence>,
    /// `court_listener_url`.
    pub court_listener_url: Option<String>,
    /// `action_id`.
    pub action_id: Option<String>,
    /// `note`.
    pub note: Option<String>,
    /// `tags`.
    pub tags: Option<Vec<String>>,
    /// `attrs`, exact.
    pub attrs: Option<BTreeMap<String, Rational>>,
    /// `sets`.
    pub sets: Option<Vec<String>>,
    /// `clears`.
    pub clears: Option<Vec<String>>,
    /// `requires`.
    pub requires: Option<Vec<String>>,
    /// `forbids`.
    pub forbids: Option<Vec<String>>,
    /// Source span of the `patch`.
    pub span: Option<litigate_base::Span>,
    /// Explicit holes in the written fields, by dotted field path, with
    /// their reasons.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub holes: BTreeMap<String, Option<String>>,
    /// The typed operations on the base edge's guard (SPEC §3.6), in
    /// source order; [`crate::guard::compose`] applies them in the order
    /// their authorities and effective dates fix. Only operations with
    /// their whole authority are kept (the rest are reported).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guard_ops: Vec<GuardOp>,
}

/// What a guard operation does to the guard `g` in force (SPEC §3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardOpKind {
    /// `when and C`: `g && C`.
    Narrow,
    /// `when or C`: `g || C`.
    Widen,
    /// `supersedes when C`: `C`.
    Replace,
}

impl GuardOpKind {
    /// The name (`narrow`, `widen`, `replace`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Narrow => "narrow",
            Self::Widen => "widen",
            Self::Replace => "replace",
        }
    }

    /// The source spelling (`when and`).
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Narrow => "when and",
            Self::Widen => "when or",
            Self::Replace => "supersedes when",
        }
    }
}

/// A typed operation on a patched edge's guard with its authority (SPEC
/// §3.6): the instrument's level, `cite` and `effective` interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardOp {
    /// Narrow, widen or replace.
    pub kind: GuardOpKind,
    /// The `Bool` condition, typed as a `when` guard (SPEC §5.1), in the
    /// view of the package that writes it.
    pub condition: super::Expr,
    /// The instrument's authority level (`by Level`).
    pub by: String,
    /// The instrument (`cite`); a hole leaves it unverified and the
    /// operation then does not apply.
    pub cite: Authored<String>,
    /// When it is in force (`effective`), and when it became known
    /// (`known_at`, else the package's).
    pub effective: super::Interval,
    /// Source span of the operation.
    pub span: Option<litigate_base::Span>,
}
