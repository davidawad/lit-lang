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
    /// Normalized citation text (`cite ...`).
    pub cite: Option<String>,
    /// Resolved in-force interval and corpus date (never absent after sema).
    pub effective: Interval,
    /// Authority level (declared, else enclosing rule's, else package's).
    pub authority: String,
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
    /// Target is explicitly reactivated after the superseding interval.
    Revives(String),
    /// The named lower authority may override this declaration.
    DelegableTo(String),
    /// Current declaration overrides the target (needs its delegation).
    Override(String),
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
}
