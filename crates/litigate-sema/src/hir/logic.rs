//! Rule metadata, authority, defeasibility, nominal types, and tests.
use super::{Expr, Interval, Type};
use serde::{Deserialize, Serialize};

/// Shared identity, documentation, cite, and source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// Local declaration id, preserved in kebab-case.
    pub id: String,
    /// Human label (string after `:`), else the joined `///` doc comment.
    pub label: String,
    /// Citation text of the first `cite` clause (`cite "..."`).
    pub cite: Option<String>,
    /// That cite is marked `historical`: it cites an expression of the work
    /// dated outside `effective` on purpose (SPEC §4.3).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub historical: bool,
    /// Resolved in-force interval and corpus date (never absent after sema).
    pub effective: Interval,
    /// Authority level (declared, else enclosing rule's, else package's).
    pub authority: String,
    /// Declared precedence relations.
    pub relations: Vec<Relation>,
    /// Typed delegation capabilities (`delegable { ... }`, design review
    /// §2.4): which lower authority may change the declaration, by which
    /// operations, within which bound.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegations: Vec<Delegation>,
    /// Declaration source span.
    pub span: Option<litigate_base::Span>,
    /// Written by `litigate import` (a `/// imported` doc line, SPEC §8.7):
    /// in `dev`, its `cite-resolves` failures are warnings (SPEC §7).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub imported: bool,
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
    /// Target is explicitly reactivated after the superseding interval.
    Revives(String),
    /// Current declaration overrides the target (needs its delegation).
    Override(String),
}

/// One typed delegation capability (SPEC §5.5, design review §2.4): the
/// lower authority `to` (principal) may change the declaration by the
/// operations `ops`, an extension reaching no further than `max` past the
/// declared period (bound), for cause `standard` (condition), on motion
/// `on` (procedure). A displacing relation from `to` is licensed only by
/// a capability covering the operation it performs; in a matter, one
/// with a procedure only once the matter records the motion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    /// The principal: an authority level strictly below the declaration's.
    pub to: String,
    /// The operations delegated, in declared order.
    pub ops: Vec<DelegatedOp>,
    /// The most an extension may add to the declared period (`max N.unit`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<(u64, crate::hir::DayUnit)>,
    /// The cause standard the lower authority applies (`discretion S`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<crate::hir::CauseStandard>,
    /// The motion an exercise requires (`on M()`): an order under the
    /// capability takes effect in a matter only when the matter records a
    /// matching event on or before the order's effective date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on: Option<Expr>,
    /// The rule text delegating (`cite`); `None`: the declaration's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

/// An operation a delegation grants (design review §2.4). A displacing
/// relation performs the operations its effect on the target amounts to
/// ([`crate::precedence::delegation::operations`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegatedOp {
    /// Lengthen a period.
    Extend,
    /// Shorten a period.
    Shorten,
    /// Replace the declaration's value: covers `extend` and `shorten`,
    /// and any other change of the value but its counting method.
    Set,
    /// Suspend the declaration (`supersedes`/`revives` of another one).
    Suspend,
    /// Change how a period is counted (another counting rule).
    ComputeMethod,
}

impl DelegatedOp {
    /// Every operation, in declaration order.
    pub const ALL: [Self; 5] = [
        Self::Extend,
        Self::Shorten,
        Self::Set,
        Self::Suspend,
        Self::ComputeMethod,
    ];

    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Extend => "extend",
            Self::Shorten => "shorten",
            Self::Set => "set",
            Self::Suspend => "suspend",
            Self::ComputeMethod => "compute_method",
        }
    }

    /// Whether a capability granting `self` licenses operation `op`.
    #[must_use]
    pub fn covers(self, op: Self) -> bool {
        self == op || (self == Self::Set && matches!(op, Self::Extend | Self::Shorten))
    }
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
    /// Ids of nested declarations (hoisted into the module's namespace).
    pub declarations: Vec<String>,
}

/// Authority chain `A > B > C`: each name outranks every later one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    /// Authority level name.
    pub name: String,
    /// Authorities strictly below this one, nearest first.
    pub outranks: Vec<String>,
}

/// Default rule: when `condition` holds, `conclusion` follows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultRule {
    /// Rule metadata.
    pub meta: Metadata,
    /// Boolean condition.
    pub condition: Expr,
    /// Concluded value.
    pub conclusion: Expr,
}

/// Exception to a default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exception {
    /// Rule metadata.
    pub meta: Metadata,
    /// Default or exception id this affects.
    pub target: String,
    /// Rebutting or undercutting semantics.
    pub kind: ExceptionKind,
    /// Boolean applicability condition.
    pub condition: Expr,
    /// Contrary conclusion (rebutting only; `None` for undercutting).
    pub conclusion: Option<Expr>,
    /// Explicit order; larger wins.
    pub priority: Option<u32>,
}

/// A new characterization of another package's deadline (docs/litigate/
/// CHANGE.md §2.8; Harrow v. Department of Defense, 601 U.S. 480 (2024)):
/// the deadline's text stands, and in a world holding this declaration
/// its character is `characterization` over this declaration's window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recharacterization {
    /// Its metadata (its window, its authority's `cite`).
    pub meta: Metadata,
    /// The deadline, `pack::id`.
    pub target: String,
    /// The character it gives the deadline.
    pub characterization: super::CharacterKind,
}

/// The withdrawal of another package's declaration over this
/// declaration's window (docs/litigate/CHANGE.md §2.8, §2.10): an
/// annulment, or a precedent invalidating a rule or a doctrine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invalidation {
    /// Its metadata (its window, its authority's `cite`).
    pub meta: Metadata,
    /// The declaration withdrawn, `pack::id`.
    pub target: String,
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

/// Alias to a nominal domain type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeAlias {
    /// Declared type name.
    pub name: String,
    /// Underlying type expression.
    pub target: Type,
    /// The predicates refining it (SPEC §3.5): `type T = {x: B | p}` or
    /// `type T = B where p`; every value of `T` satisfies each.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refinements: Vec<Refinement>,
}

/// A refinement (SPEC §3.5): a `Bool` predicate in the qualifier grammar
/// (linear comparisons of numbers, durations, money and dates; enum and
/// boolean tests; `&&`, `||`, `!`) over the value it refines and, on an
/// event, its other parameters. Checked by exact evaluation where the
/// values are known, and by the SMT tier where they are not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refinement {
    /// The name the predicate gives the refined value: the parameter of an
    /// event, the binder `x` of an alias `{x: B | p}`, or `it`.
    pub binder: String,
    /// The predicate, typed `Bool`; a parameter is a [`ExprKind::Ref`] to
    /// its name, a bound date of an `extends` event (SPEC §3.5) a `Ref` to
    /// `prescribed_due`, `current_due` or `grant_order_entered`.
    ///
    /// [`ExprKind::Ref`]: super::ExprKind::Ref
    pub predicate: Expr,
    /// The alias the predicate comes from, when a parameter has a refined
    /// alias type (in the event's package view).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

/// Closed enum declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDecl {
    /// Enum name.
    pub name: String,
    /// Unique variants in declaration order.
    pub variants: Vec<String>,
}

/// Inline `test` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Test {
    /// Test name (string literal).
    pub name: String,
    /// Optional cite of the tested provision.
    pub cite: Option<String>,
    /// `given <event> on <date>` facts in order.
    pub givens: Vec<(Expr, String)>,
    /// The actor doing each `given <event> by <actor> on <date>`, by its
    /// index in `givens`, as a matter's `by` party's actor: only it holds
    /// the deadline grants made `by` that actor (SPEC §5.3).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub actors: std::collections::BTreeMap<usize, String>,
    /// `expect` assertions in order.
    pub expects: Vec<Expectation>,
    /// Source span.
    pub span: Option<litigate_base::Span>,
}

/// One `expect lhs <op> rhs` assertion; `op` is a comparison operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expectation {
    /// Left operand.
    pub lhs: Expr,
    /// Comparison.
    pub op: super::BinaryOp,
    /// Right operand.
    pub rhs: Expr,
}

/// Event type `event Served(document: Document, method: ServiceMethod)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventType {
    /// Constructor name.
    pub name: String,
    /// Parameters in order.
    pub params: Vec<(String, Type)>,
    /// The declared effect on a matter, if any (SPEC §5.2). Replay gives
    /// an event's name no meaning: only this clause makes an event remove
    /// or elect an option, extend an obligation, or waive one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<EventEffect>,
    /// The kind of paper the event files (`paper k`, SPEC §3.7): the
    /// `limit` declaring its filing requirements, by canonical name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paper: Option<String>,
    /// The predicates its arguments satisfy (SPEC §3.5), from refined
    /// parameter types, `where` clauses and refined aliases, in parameter
    /// order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refinements: Vec<Refinement>,
}

/// What an event does to a matter besides taking an edge (SPEC §5.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum EventEffect {
    /// `removes p`: the `String` parameter `p` names an edge the matter
    /// loses for the rest of it.
    Removes {
        /// Index of the parameter in [`EventType::params`].
        param: usize,
    },
    /// `elects p`: the `String` parameter `p` names the option chosen;
    /// every alternative at each fork offering it leaves the matter.
    Elects {
        /// Index of the parameter in [`EventType::params`].
        param: usize,
    },
    /// `extends d to p`: moves the due date of an open obligation of
    /// deadline `d` to the `Date` parameter `p`, within `d`'s `extend`
    /// capability.
    Extends {
        /// The deadline, as written in the declaring package's view.
        deadline: String,
        /// Index of the `Date` parameter in [`EventType::params`].
        param: usize,
    },
    /// `waives d`: releases an open obligation of deadline `d`, under
    /// `d`'s `waive` capability.
    Waives {
        /// The deadline, as written in the declaring package's view.
        deadline: String,
    },
}
