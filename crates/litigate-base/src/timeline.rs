//! The timeline document (`litigate.timeline/1`): the timeline DAG of a
//! `.case` or `.lit` file, as `litigate timeline` prints it. The schema is
//! frozen in `docs/litigate/timeline.v1.schema.json` (described in
//! `docs/litigate/TIMELINE.md`); these types serialize to exactly it, so a
//! renderer (the Emacs timeline buffer, lg-z2m.2) reads one contract. A
//! nullable field is always serialized (`null`), never omitted; the one
//! optional field, a row's `replaced_by`, is omitted when unset.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The document's `schema` value.
pub const TIMELINE_SCHEMA: &str = "litigate.timeline/1";

/// A timeline: rows per world, with layer provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    /// Always [`TIMELINE_SCHEMA`].
    pub schema: String,
    /// The file the timeline was built from.
    pub source: TimelineSource,
    /// The date statuses are judged at.
    pub as_of: String,
    /// The knowledge date, or `None` for everything recorded.
    pub known_at: Option<String>,
    /// Every layer that contributed a row, enabled or not.
    pub layers: Vec<TimelineLayer>,
    /// The docket world first, then one per alternate timeline.
    pub worlds: Vec<TimelineWorld>,
    /// Events, deadlines and fork points, each in exactly one world.
    pub rows: Vec<TimelineRow>,
    /// Links between rows of one world.
    pub edges: Vec<TimelineEdge>,
    /// Errors and warnings from building the timeline.
    pub diagnostics: Vec<TimelineDiagnostic>,
}

/// The file a timeline was built from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineSource {
    /// Repository-relative path.
    pub file: String,
    /// `case` or `lit`.
    pub kind: TimelineSourceKind,
}

/// What kind of file a timeline was built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimelineSourceKind {
    /// A `.case` matter.
    Case,
    /// A `.lit` rule module.
    Lit,
}

/// A source span: byte range plus its one-based start line and column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineSpan {
    /// Repository-relative path.
    pub file: String,
    /// Start byte offset.
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
    /// One-based start line.
    pub line: u32,
    /// One-based start column (Unicode scalars).
    pub column: u32,
}

/// The closed set of authority classes a layer can belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LayerClass {
    /// A statute, regulation or court rule package.
    BaseRule,
    /// A court's local rules.
    LocalRule,
    /// A judge's or panel's standing order.
    StandingOrder,
    /// A court-wide general order.
    GeneralOrder,
    /// A closure or emergency declaration.
    EmergencyDeclaration,
    /// An order in this matter.
    CaseOrder,
    /// A decision that changes how a rule applies.
    Decision,
}

/// One rule source applied to the matter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineLayer {
    /// Layer id, named by rows' `provenance`.
    pub id: String,
    /// Authority class.
    pub class: LayerClass,
    /// Level in the package's declared authority lattice.
    pub authority: String,
    /// Display label.
    pub label: String,
    /// The layer's own citation, if any.
    pub cite: Option<String>,
    /// When the layer is in force.
    pub effective: TimelineEffective,
    /// The package that declares it.
    pub package: String,
    /// `false` when the caller switched it off (its rows are omitted).
    pub enabled: bool,
}

/// An in-force interval; `None` ends are open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineEffective {
    /// First day in force.
    pub from: Option<String>,
    /// First day no longer in force.
    pub to: Option<String>,
}

/// One timeline: the docket, or an alternate branching from a fork row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineWorld {
    /// World id.
    pub id: String,
    /// Display label.
    pub label: String,
    /// The world it branches from (`None` for the docket).
    pub parent: Option<String>,
    /// The row where it diverges (`None` for the docket).
    pub fork_row: Option<String>,
    /// The choice that defines it (`None` for the docket).
    pub choice: Option<TimelineChoice>,
}

/// The diverging choice of an alternate world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineChoice {
    /// What kind of choice.
    pub kind: ChoiceKind,
    /// Display label.
    pub label: String,
    /// The rule text it rests on.
    pub cite: Option<String>,
    /// Its probability, when the model has one.
    pub probability: Option<f64>,
}

/// What kind of choice defines an alternate world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceKind {
    /// An order assumed to issue.
    AssumedOrder,
    /// A motion granted.
    Grant,
    /// A motion denied.
    Denial,
    /// A ruling the tribunal might make.
    HypotheticalDecision,
    /// A party's election.
    Election,
    /// A world draw.
    Chance,
}

/// A row's status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineStatus {
    /// The state.
    pub state: RowState,
    /// Why, when the state needs one; always set for `unknown` and
    /// `superseded`.
    pub reason: Option<String>,
}

/// A row's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RowState {
    /// Discharged in time.
    Met,
    /// Not yet due.
    Open,
    /// Acted on after the due date.
    Late,
    /// Past due with no act, not a breach.
    Lapsed,
    /// A duty past due with no act.
    Breached,
    /// Not computable; the reason says why.
    Unknown,
    /// Replaced; the reason says by what.
    Superseded,
    /// An event or a filing limit (no status).
    None,
}

/// What a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RowKind {
    /// Something that happened (or happens in an alternate world).
    Event,
    /// A computed deadline.
    Deadline,
    /// A point where worlds diverge.
    Fork,
    /// The filing limit (SPEC §3.7) governing the paper an event files:
    /// `provenance` lists the layers selection passed through, so its
    /// last entry is the layer that set the limit.
    Limit,
}

/// One event, deadline, fork point or filing limit, in exactly one world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineRow {
    /// Row id.
    pub id: String,
    /// Stable identity across worlds and runs, derived from the row's
    /// cause: `<rule id>@<trigger key>`, `event:<Name>@<declaration>`,
    /// `fork:<node id>@<trigger key>`, `<limit id>@<event key>`. A child
    /// world's row with its parent's key replaces the parent's row.
    pub key: String,
    /// Its world.
    pub world: String,
    /// What it is.
    pub kind: RowKind,
    /// When an event happened; `None` for deadlines and unresolved forks.
    pub date: Option<String>,
    /// A deadline's due date; `None` when it cannot be computed.
    pub due: Option<String>,
    /// Display label.
    pub label: String,
    /// Status with reason.
    pub status: TimelineStatus,
    /// The rule text it rests on.
    pub cite: Option<String>,
    /// The `.lit` or `.case` span that produced it.
    pub source: Option<TimelineSpan>,
    /// The class of the layer that decided it (the last of `provenance`).
    pub layer: LayerClass,
    /// Layer ids in the order they applied, base first; never empty.
    pub provenance: Vec<String>,
    /// The row that started this one.
    pub trigger: Option<String>,
    /// The `key` of the row that replaces this one, in this world's view:
    /// set on a `superseded` row whenever a replacing row exists, omitted
    /// otherwise. The status reason is for people; this is the link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
}

/// How two rows of one world are linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Order within the world.
    Sequence,
    /// A trigger to what it starts.
    Triggers,
    /// A reset to the row it replaces.
    Supersedes,
    /// A fork row to the first row of a child world.
    Forks,
}

/// A link between rows of one world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineEdge {
    /// Source row.
    pub from: String,
    /// Target row.
    pub to: String,
    /// The world it belongs to.
    pub world: String,
    /// Link kind.
    pub kind: EdgeKind,
}

/// A diagnostic's severity in a timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimelineSeverity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// Anything milder.
    Note,
}

/// An error or warning from building the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelineDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Severity.
    pub severity: TimelineSeverity,
    /// Message.
    pub message: String,
    /// Where it points, if anywhere.
    pub source: Option<TimelineSpan>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frozen_example_round_trips_through_the_types() {
        let text = include_str!("../../../docs/litigate/examples/timeline-ttab-opposition.json");
        let example: serde_json::Value = serde_json::from_str(text).unwrap();
        let timeline: Timeline = serde_json::from_value(example.clone()).unwrap();
        assert_eq!(timeline.schema, TIMELINE_SCHEMA);
        assert_eq!(serde_json::to_value(&timeline).unwrap(), example);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let err = serde_json::from_str::<TimelineSource>(r#"{"file":"a","kind":"lit","x":1}"#);
        assert!(err.is_err());
    }
}
