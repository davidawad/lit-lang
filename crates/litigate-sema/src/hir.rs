//! Serializable high-level IR shared by semantic analysis and lowering.
use serde::{Deserialize, Serialize};

/// Top-level declaration in a resolved rule module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    /// Canonical dotted module name.
    pub name: String,
    /// Package name used to derive the litgraph pack identity.
    pub package: String,
    /// Imports resolved to canonical module names.
    pub imports: Vec<String>,
    /// Declarations in source order.
    pub declarations: Vec<Declaration>,
}

/// Resolved and replayable matter source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Case {
    /// Matter identifier.
    pub id: String,
    /// Pinned package versions and content hashes.
    pub uses: Vec<PackagePin>,
    /// Governing in-force date.
    pub as_of: String,
    /// Date the compiled library was known, when selected.
    pub known_at: Option<String>,
    /// Named party whose legal role is the protagonist.
    pub perspective: String,
    /// Declared parties.
    pub parties: Vec<Party>,
    /// Chronologically ordered source events.
    pub events: Vec<Event>,
    /// Resolved fact-node variant per node.
    pub facts: std::collections::BTreeMap<String, String>,
    /// Terminal value or explicit `unvalued` marker per terminal.
    pub stakes: std::collections::BTreeMap<String, Value>,
    /// Observed chance outcomes by node and edge.
    pub observations: Vec<Observation>,
}

/// Exact pinned package identity used by a matter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePin {
    /// Package name.
    pub name: String,
    /// Resolved semantic version.
    pub version: String,
    /// SHA-256 source content digest.
    pub content_hash: String,
}

/// Party name, forum role, and nominal party type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Party {
    /// Local party id.
    pub id: String,
    /// Forum-declared party role.
    pub role: String,
    /// Nominal entity type.
    pub entity_type: String,
}

/// Docket event replayed against posture and law in force on its date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Event date.
    pub date: String,
    /// Typed action name and arguments.
    pub action: String,
    /// Party initiating or receiving the event.
    pub party: String,
    /// Cite-backed extension/waiver/excuse, if present.
    pub authority: Option<String>,
}

/// Observed variant count at a chance node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Chance-node id.
    pub node: String,
    /// Outcome edge counts.
    pub outcomes: Vec<(String, u64)>,
}

/// Rule-language declaration represented in HIR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Declaration {
    /// Procedural posture.
    State(State),
    /// Actor-controlled move.
    Decision(Decision),
    /// Nature or tribunal draw.
    Chance(Chance),
    /// End state.
    Terminal(Terminal),
    /// Typed deadline rule.
    Deadline(Deadline),
    /// Date counting behavior.
    Counting(Counting),
    /// Holiday set.
    Calendar(Calendar),
    /// Duty, power, or liability wrapper.
    Rule(Rule),
    /// Authority-order declaration.
    Authority(Authority),
    /// Default logic declaration.
    Default(DefaultRule),
    /// Rebutting or undercutting exception.
    Exception(Exception),
    /// Cross-version transition declaration.
    Transition(Transition),
    /// Nominal type alias.
    TypeAlias(TypeAlias),
    /// Closed enum declaration.
    Enum(EnumDecl),
}

/// Domain type in a resolved expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "argument", rename_all = "snake_case")]
pub enum Type {
    /// Gregorian calendar date.
    Date,
    /// Day count with a fixed counting unit.
    Days(DayUnit),
    /// Attorney or elapsed hours.
    Hours,
    /// Integral minor-unit money in a named currency.
    Money(String),
    /// Exact rational probability in `[0,1]`.
    Probability,
    /// Exact distribution over a closed enum.
    Distribution(String),
    /// Citation resolved to an L0 span.
    Cite,
    /// Half-open date interval.
    DateInterval,
    /// Declared procedural actor role.
    Role,
    /// Boolean value.
    Boolean,
    /// Signed integer value.
    Integer,
    /// String value.
    String,
    /// User-declared nominal type.
    Named(String),
}

/// Calendar unit carried by a typed day count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayUnit {
    /// Every calendar day.
    Calendar,
    /// Court days after exclusions.
    Court,
    /// Business days after exclusions.
    Business,
}

/// Closed literal value in typed HIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Value {
    /// ISO-8601 Gregorian date.
    Date(String),
    /// Typed day count.
    Days {
        /// Signed number of days.
        amount: i64,
        /// Counting unit for the amount.
        unit: DayUnit,
    },
    /// Exact hour count.
    Hours(Rational),
    /// Integral minor-unit money.
    Money {
        /// Integral minor-unit amount.
        minor_units: i128,
        /// ISO currency code.
        currency: String,
    },
    /// Exact probability.
    Probability(Rational),
    /// Exhaustive enum-to-probability pairs.
    Distribution(Vec<(String, Rational)>),
    /// Resolved citation reference.
    Cite(String),
    /// Explicit unresolved value and optional reason.
    Hole(Option<String>),
    /// Boolean literal.
    Boolean(bool),
    /// Signed integer literal.
    Integer(i64),
    /// String literal.
    String(String),
}

/// Rule category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// Mandatory action within a duration.
    Duty,
    /// Permission to make a procedural choice.
    Power,
    /// Consequence of failing a duty.
    Liability,
}

/// Duty, power, or liability wrapper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Rule metadata.
    pub meta: Metadata,
    /// Rule category.
    pub rule_kind: RuleKind,
    /// Nested declaration ids.
    pub declarations: Vec<String>,
}

/// Authority level and its lower-ranked authorities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    /// Authority level name.
    pub name: String,
    /// Authorities strictly below this one.
    pub outranks: Vec<String>,
}

/// Default rule condition and conclusion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultRule {
    /// Stable rule id.
    pub id: String,
    /// Condition under which the default applies.
    pub condition: String,
    /// Concluded value/expression.
    pub conclusion: String,
    /// Source span.
    pub span: Option<litigate_base::Span>,
}

/// Exception effect on a referenced rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exception {
    /// Stable exception id.
    pub id: String,
    /// Rule id this exception affects.
    pub target: String,
    /// Rebutting or undercutting semantics.
    pub kind: ExceptionKind,
    /// Applicability condition.
    pub condition: String,
    /// Contrary conclusion for a rebutting exception.
    pub conclusion: Option<String>,
    /// Explicit order for overlapping exceptions.
    pub priority: Option<u32>,
}

/// Exception semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExceptionKind {
    /// Concludes the contrary result.
    Rebutting,
    /// Disables the target without a contrary result.
    Undercutting,
}

/// Transition rule between effective versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Prior rule version.
    pub prior: String,
    /// Amending rule version.
    pub amendment: String,
    /// Conditional transition arms in source order.
    pub arms: Vec<(String, String)>,
}

/// Alias to a nominal domain type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeAlias {
    /// Declared type name.
    pub name: String,
    /// Underlying type expression.
    pub target: Type,
}

/// Closed enum declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDecl {
    /// Enum name.
    pub name: String,
    /// Unique variants in declaration order.
    pub variants: Vec<String>,
}

/// Shared identity, documentation, cite, and source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// Local declaration id, preserved in kebab-case.
    pub id: String,
    /// Human label or source doc comment.
    pub label: String,
    /// Optional normalized legal citation.
    pub cite: Option<String>,
    /// Effective interval, if versioned.
    pub effective: Option<Interval>,
    /// Declared precedence/delegation relations.
    pub relations: Vec<Relation>,
    /// Declaration source span.
    pub span: Option<litigate_base::Span>,
}

/// Explicit precedence or delegation relation on a declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "relation", content = "target", rename_all = "snake_case")]
pub enum Relation {
    /// Current declaration yields to the target when both apply.
    SubjectTo(String),
    /// Current declaration controls over the target.
    Notwithstanding(String),
    /// Current declaration replaces the target for its effective interval.
    Supersedes(String),
    /// Target is explicitly reactivated after repeal/supersession.
    Revives(String),
    /// Lower authority may override this declaration.
    DelegableTo(String),
    /// Current declaration overrides the target.
    Override(String),
}

/// Half-open date interval; absent end means unbounded future.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    /// Inclusive beginning date.
    pub from: String,
    /// Exclusive end date; `None` denotes infinity.
    pub to: Option<String>,
    /// Date when the compiler's corpus learned this version.
    pub known_at: Option<String>,
}

/// Procedural posture declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Optional display group.
    pub group: Option<String>,
}

/// Actor-controlled procedural decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Actor role.
    pub actor: String,
    /// Ordered outgoing transitions.
    pub edges: Vec<Edge>,
}

/// Uncontrolled outcome draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chance {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Decision maker (`Nature` or `Court`).
    pub actor: String,
    /// Ordered outgoing transitions.
    pub edges: Vec<Edge>,
}

/// End state with outcome and payoff declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Machine outcome tags.
    pub outcomes: Vec<String>,
    /// Payoff in USD, if authored.
    pub payoff: Option<f64>,
}

/// Directed transition between graph nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Stable local edge id.
    pub id: String,
    /// Target node id.
    pub to: String,
    /// Display label.
    pub label: String,
    /// Edge source cite.
    pub cite: Option<String>,
    /// Governing authority string.
    pub authority: Option<String>,
    /// Acting role.
    pub actor: String,
    /// Exact rational probability, if authored.
    pub probability: Option<Rational>,
    /// Effort in hours.
    pub hours: Option<f64>,
    /// Out-of-pocket cost in USD.
    pub cost: Option<Rational>,
    /// Deadline reference.
    pub deadline: Option<String>,
    /// State flags set by transition.
    pub sets: Vec<String>,
    /// State flags cleared by transition.
    pub clears: Vec<String>,
    /// Tags carried into the target edge.
    pub tags: Vec<String>,
    /// Free numeric attributes.
    pub attrs: std::collections::BTreeMap<String, f64>,
}

/// Exact rational number in reduced or unreduced numerator/denominator form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    /// Signed numerator.
    pub numerator: i128,
    /// Positive denominator.
    pub denominator: u128,
}

/// Typed deadline declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deadline {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Amount and unit spelling.
    pub amount: u64,
    /// Unit such as `court_days`.
    pub unit: String,
    /// Trigger expression.
    pub trigger: String,
    /// Counting rule id.
    pub counting: String,
    /// Direction of counting.
    pub direction: Direction,
}

/// Deadline counting direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Count forward from trigger.
    Forward,
    /// Count backward from target date.
    Backward,
}

/// Data-driven day-counting rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counting {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Exclude trigger day.
    pub exclude_trigger_day: bool,
    /// Include last day.
    pub include_last_day: bool,
    /// Weekday / holiday roll convention.
    pub roll_forward: Vec<String>,
    /// Extension amounts by service method.
    pub service_extensions: Vec<(String, u64)>,
    /// Referenced holiday calendar.
    pub calendar: Option<String>,
}

/// Data-driven holiday calendar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Holiday definitions.
    pub holidays: Vec<Holiday>,
}

/// One fixed or relative holiday definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holiday {
    /// Stable holiday id.
    pub id: String,
    /// Rule expression for the date.
    pub date_rule: String,
    /// In-force interval.
    pub effective_from: Option<String>,
    /// Observance convention.
    pub observed: Option<String>,
}
