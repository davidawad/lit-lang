//! Resolved `.case` matter.
use super::{ActorRole, Expr, Money, Rational};
use litigate_base::Span;
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
    /// Declared priors the observations update (`beliefs { prior .. }`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priors: Vec<Prior>,
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
    /// Version date, or a hole's reason.
    pub as_of: Option<String>,
    /// `as_of` is a hole `?("reason")`, not a date.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub as_of_hole: bool,
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
    /// Docket sequence number (`docket N`): orders the event among others
    /// of its date (SPEC §5.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket: Option<u64>,
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

/// A Dirichlet prior over a draw's outcomes (`prior node: { edge: α }`):
/// the matter's own judgment, as pseudo-counts. Its mean `α / Σα` replaces
/// the authored probabilities and `Σα` is its strength; with two outcomes
/// it is `Beta(α, β)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prior {
    /// Qualified draw node.
    pub node: String,
    /// Qualified outcome edge and its positive pseudo-count, in source
    /// order.
    pub pseudo_counts: Vec<(String, Rational)>,
}

/// `analysis { ... }` (SPEC §8.3): the litgraph `Scenario` fields a matter
/// sets beyond its dedicated blocks, each typed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// `params`: typed scenario parameters by name (SPEC §8.3 `Params`).
    pub params: BTreeMap<String, Param>,
    /// `metrics`: named edge expressions.
    pub metrics: BTreeMap<String, EngineExpr>,
    /// `utilities`: named terminal expressions.
    pub utilities: BTreeMap<String, EngineExpr>,
    /// `cost`, `utility`, `mask`, `probability_fn` and
    /// `opponent_objective`, by field name.
    pub expressions: BTreeMap<String, EngineExpr>,
    /// `calibration`: calibration set names, in order.
    pub calibration: Vec<String>,
    /// `discount_annual`, exact.
    pub discount_annual: Option<Rational>,
    /// `remove_edges`: resolved edges (`pack::edge`), in order.
    pub remove_edges: Vec<String>,
    /// The engine's option fields (`mixed`, `opponent`, `prob_fill`,
    /// `objective`, `fee_shift`, `uncertainty`), each checked against the
    /// engine's own schema when the matter lowers.
    pub options: BTreeMap<String, EngineOption>,
}

/// One scenario parameter (`analysis.params`), typed by the `Params`
/// record (SPEC §8.3): a dimensionless number or a money rate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Param {
    /// An exact dimensionless `Number`.
    Number(Rational),
    /// A `Rate<Cur, Per>`: money per unit of time.
    Rate(Rate),
}

/// Money per unit of time (`650.usd / 1.hours`), exact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rate {
    /// Minor units of `currency` per one `per`.
    pub minor_units: Rational,
    /// Lowercase ISO 4217 code.
    pub currency: String,
    /// The time unit the money is spent per.
    pub per: RateUnit,
}

/// The time unit of a [`Rate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateUnit {
    /// Per attorney hour (`Rate<usd, Hours>`).
    Hour,
    /// Per calendar day (`Rate<usd, CalendarDays>`).
    CalendarDay,
}

/// An expression in the engine's expression language (`analysis.metrics`,
/// `utility`, ...), parsed by the engine when the matter lowers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineExpr {
    /// The expression text.
    pub text: String,
    /// Where it is written.
    pub span: Option<Span>,
}

/// An engine option's value, with where it is written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineOption {
    /// The value as the engine reads it.
    pub value: serde_json::Value,
    /// Where it is written.
    pub span: Option<Span>,
}
