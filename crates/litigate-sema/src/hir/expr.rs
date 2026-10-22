//! Typed, resolved expressions used by defaults, exceptions, tests, and
//! transition arms.
use super::{Type, Value};
use serde::{Deserialize, Serialize};

/// A typed expression; `ty` is the checked result type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expr {
    /// Checked result type.
    pub ty: Type,
    /// Expression shape.
    pub kind: ExprKind,
    /// Source span.
    pub span: Option<litigate_base::Span>,
}

/// Expression shapes (SPEC §3 `Expr`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "expr", rename_all = "snake_case")]
pub enum ExprKind {
    /// Literal value.
    Literal {
        /// The value.
        value: Value,
    },
    /// Resolved reference to a declaration, party, binding, or record field path.
    Ref {
        /// Fully qualified target.
        target: String,
    },
    /// Call of a constructor, builtin, or event constructor.
    Call {
        /// Fully qualified callee.
        callee: String,
        /// Arguments in order.
        args: Vec<Expr>,
    },
    /// Unary operation.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        operand: Box<Expr>,
    },
    /// Binary operation.
    Binary {
        /// Operator.
        op: BinaryOp,
        /// Left operand.
        lhs: Box<Expr>,
        /// Right operand.
        rhs: Box<Expr>,
    },
    /// Record literal; fields sorted by name.
    Record {
        /// Field values.
        fields: Vec<(String, Expr)>,
    },
    /// List literal.
    List {
        /// Items in order.
        items: Vec<Expr>,
    },
    /// Field projection `base.field`.
    Field {
        /// Projected record.
        base: Box<Expr>,
        /// Field name.
        field: String,
    },
    /// Exhaustive match.
    Match {
        /// Scrutinee.
        scrutinee: Box<Expr>,
        /// Arms in source order.
        arms: Vec<(Pattern, Expr)>,
    },
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "pattern", content = "variant", rename_all = "snake_case")]
pub enum Pattern {
    /// Fully qualified enum variant.
    Variant(String),
    /// Wildcard `_`.
    Wildcard,
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryOp {
    /// Boolean `!`.
    Not,
    /// Arithmetic `-`.
    Neg,
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    /// `||`.
    Or,
    /// `&&`.
    And,
    /// `==`.
    Eq,
    /// `!=`.
    Ne,
    /// `<`.
    Lt,
    /// `<=`.
    Le,
    /// `>`.
    Gt,
    /// `>=`.
    Ge,
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
    /// `%` (integers only).
    Rem,
}
