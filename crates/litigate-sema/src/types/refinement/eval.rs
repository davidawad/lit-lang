//! Tier 1: exact evaluation of a refinement where every value it names is
//! known, and the source spelling of predicates and values for
//! diagnostics.
use crate::hir::{self, BinaryOp, DayUnit, ExprKind, Rational, Type, UnaryOp, Value};
use crate::types::{date, rational as q};
use std::cmp::Ordering;

/// A value of the qualifier grammar: every ordered domain is exact
/// rational (a date its day number, money its minor units, a duration its
/// amount), an enum value its variant's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// A number, probability, duration, amount of money, or date.
    Num(Rational),
    /// A `Bool`.
    Bool(bool),
    /// An enum variant, by its own name (the parameter's type fixes the
    /// enum, so the views of two packages compare).
    Variant(String),
}

/// `v` as a qualifier value; `None` for a hole or a value outside the
/// grammar.
#[must_use]
pub fn of(v: &Value) -> Option<Val> {
    Some(match v {
        Value::Number(r) | Value::Probability(r) | Value::Hours(r) => Val::Num(*r),
        Value::Days { amount, .. } => Val::Num(Rational::integer(*amount)),
        Value::Money(m) => Val::Num(Rational::integer(m.minor_units)),
        Value::Date(d) => Val::Num(Rational::integer(date::day_number(d)?)),
        Value::Boolean(b) => Val::Bool(*b),
        Value::Variant(v) => Val::Variant(v.rsplit("::").next().unwrap_or(v).to_owned()),
        _ => return None,
    })
}

/// Whether `pred` holds with each name's value given by `env`; `None`
/// when it depends on a value `env` does not know (a hole, a bound date
/// not yet counted) or exact arithmetic overflows. `&&` and `||` decide
/// when one side does (`false && ?` is false).
#[must_use]
pub fn eval(pred: &hir::Expr, env: &dyn Fn(&str) -> Option<Value>) -> Option<bool> {
    match term(pred, env)? {
        Val::Bool(b) => Some(b),
        _ => None,
    }
}

/// The value of `e` under `env`; `None` as for [`eval`].
#[must_use]
pub fn term(e: &hir::Expr, env: &dyn Fn(&str) -> Option<Value>) -> Option<Val> {
    match &e.kind {
        ExprKind::Literal { value } => of(value),
        ExprKind::Ref { target } => of(&env(target)?),
        ExprKind::Unary { op, operand } => match (op, term(operand, env)?) {
            (UnaryOp::Not, Val::Bool(b)) => Some(Val::Bool(!b)),
            (UnaryOp::Neg, Val::Num(n)) => q::neg(n).ok().map(Val::Num),
            _ => None,
        },
        ExprKind::Binary { op, lhs, rhs } => binary(*op, lhs, rhs, env),
        _ => None,
    }
}

fn binary(
    op: BinaryOp,
    lhs: &hir::Expr,
    rhs: &hir::Expr,
    env: &dyn Fn(&str) -> Option<Value>,
) -> Option<Val> {
    if matches!(op, BinaryOp::And | BinaryOp::Or) {
        // `false && ?` is false and `true || ?` true: the side that decides
        // decides alone.
        let decides = op == BinaryOp::Or;
        let (a, b) = (term(lhs, env), term(rhs, env));
        return match (a, b) {
            (Some(Val::Bool(x)), _) | (_, Some(Val::Bool(x))) if x == decides => {
                Some(Val::Bool(decides))
            }
            (Some(Val::Bool(_)), Some(Val::Bool(_))) => Some(Val::Bool(!decides)),
            _ => None,
        };
    }
    let (a, b) = (term(lhs, env)?, term(rhs, env)?);
    let ordering = |a: &Val, b: &Val| match (a, b) {
        (Val::Num(x), Val::Num(y)) => Some(q::cmp(*x, *y)),
        _ => None,
    };
    let (Val::Num(x), Val::Num(y)) = (&a, &b) else {
        return match op {
            BinaryOp::Eq => Some(Val::Bool(a == b)),
            BinaryOp::Ne => Some(Val::Bool(a != b)),
            _ => None,
        };
    };
    let (x, y) = (*x, *y);
    let holds = |want: &[Ordering]| ordering(&a, &b).map(|o| Val::Bool(want.contains(&o)));
    match op {
        BinaryOp::Eq => holds(&[Ordering::Equal]),
        BinaryOp::Ne => holds(&[Ordering::Less, Ordering::Greater]),
        BinaryOp::Lt => holds(&[Ordering::Less]),
        BinaryOp::Le => holds(&[Ordering::Less, Ordering::Equal]),
        BinaryOp::Gt => holds(&[Ordering::Greater]),
        BinaryOp::Ge => holds(&[Ordering::Greater, Ordering::Equal]),
        BinaryOp::Add => q::add(x, y).ok().map(Val::Num),
        BinaryOp::Sub => q::sub(x, y).ok().map(Val::Num),
        BinaryOp::Mul => q::mul(x, y).ok().map(Val::Num),
        BinaryOp::Div => q::div(x, y).ok().map(Val::Num),
        _ => None,
    }
}

/// The source spelling of `v`, a value of type `ty` (for a model or a
/// counterexample).
#[must_use]
pub fn show(ty: &Type, v: &Val) -> String {
    match (ty, v) {
        (Type::Date, Val::Num(n)) if n.denominator == 1 => {
            date::from_day_number(n.numerator).unwrap_or_else(|| format!("day {}", n.numerator))
        }
        (Type::Days(u), Val::Num(n)) => format!("{}.{}", q::display(*n), suffix(*u)),
        (Type::Hours, Val::Num(n)) => format!("{}.hours", q::display(*n)),
        (Type::Money(c), Val::Num(n)) => money(*n, c),
        (_, Val::Num(n)) => q::display(*n),
        (_, Val::Bool(b)) => b.to_string(),
        (_, Val::Variant(v)) => v.clone(),
    }
}

/// `minor` units of currency `c` as a money literal (`1250.50.usd`).
fn money(minor: Rational, c: &str) -> String {
    if minor.denominator != 1 {
        return format!("{} minor units of {c}", q::display(minor));
    }
    let sign = if minor.numerator < 0 { "-" } else { "" };
    let n = minor.numerator.unsigned_abs();
    match n % 100 {
        0 => format!("{sign}{}.{c}", n / 100),
        cents => format!("{sign}{}.{cents:02}.{c}", n / 100),
    }
}

/// The literal suffix of a day unit.
#[must_use]
pub const fn suffix(u: DayUnit) -> &'static str {
    match u {
        DayUnit::Calendar => "days",
        DayUnit::Court => "court_days",
        DayUnit::Business => "business_days",
        DayUnit::Week => "weeks",
        DayUnit::Month => "months",
        DayUnit::Year => "years",
    }
}

/// The source spelling of a typed predicate or term.
#[must_use]
pub fn render(e: &hir::Expr) -> String {
    match &e.kind {
        ExprKind::Literal { value } => match value {
            Value::Date(d) => d.clone(),
            Value::Variant(v) => v.rsplit("::").next().unwrap_or(v).to_owned(),
            other => of(other).map_or_else(|| "?".to_owned(), |v| show(&e.ty, &v)),
        },
        ExprKind::Ref { target } => target.clone(),
        ExprKind::Unary { op, operand } => {
            let sym = if *op == UnaryOp::Not { "!" } else { "-" };
            format!("{sym}{}", operand_text(operand, u8::MAX, false))
        }
        ExprKind::Binary { op, lhs, rhs } => format!(
            "{} {} {}",
            operand_text(lhs, precedence(*op), false),
            crate::types::ops::symbol(*op),
            operand_text(rhs, precedence(*op), true)
        ),
        _ => "?".to_owned(),
    }
}

/// Binding strength of `op` (SPEC §3.3).
const fn precedence(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            3
        }
        BinaryOp::Add | BinaryOp::Sub => 4,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => 5,
    }
}

/// An operand of an operator binding with strength `outer`, in
/// parentheses when it binds more loosely (or as tightly, on the right:
/// operators associate left).
fn operand_text(e: &hir::Expr, outer: u8, right: bool) -> String {
    match &e.kind {
        ExprKind::Binary { op, .. }
            if precedence(*op) < outer || (right && precedence(*op) == outer) =>
        {
            format!("({})", render(e))
        }
        _ => render(e),
    }
}
