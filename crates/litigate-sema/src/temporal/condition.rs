//! Boolean conditions of transitions, defaults, exceptions, and fact
//! guards: connectives over opaque atoms (SPEC §5.1, §5.4-§5.5).
//!
//! An atom is any boolean sub-expression that is not `!`, `&&`, `||`, a
//! boolean `==`/`!=`, or a boolean literal; in practice `has_event(P)`,
//! and in a fact guard a comparison of dates. Atoms compare structurally,
//! ignoring spans.
use crate::hir::{BinaryOp, Expr, ExprKind, Type, UnaryOp, Value};

/// Structural equality ignoring source spans.
#[must_use]
pub fn same(a: &Expr, b: &Expr) -> bool {
    a.ty == b.ty && same_kind(&a.kind, &b.kind)
}

fn all_same(a: &[Expr], b: &[Expr]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y))
}

fn same_kind(a: &ExprKind, b: &ExprKind) -> bool {
    use ExprKind as K;
    match (a, b) {
        (K::Literal { value: x }, K::Literal { value: y }) => x == y,
        (K::Ref { target: x }, K::Ref { target: y }) => x == y,
        (K::Call { callee: f, args: x }, K::Call { callee: g, args: y }) => {
            f == g && all_same(x, y)
        }
        (K::Unary { op: o, operand: x }, K::Unary { op: p, operand: y }) => o == p && same(x, y),
        (
            K::Binary {
                op: o,
                lhs: a1,
                rhs: b1,
            },
            K::Binary {
                op: p,
                lhs: a2,
                rhs: b2,
            },
        ) => o == p && same(a1, a2) && same(b1, b2),
        (K::Record { fields: x }, K::Record { fields: y }) => {
            x.len() == y.len() && x.iter().zip(y).all(|((n, e), (m, f))| n == m && same(e, f))
        }
        (K::List { items: x }, K::List { items: y }) => all_same(x, y),
        (K::Field { base: x, field: f }, K::Field { base: y, field: g }) => f == g && same(x, y),
        (
            K::Match {
                scrutinee: x,
                arms: a,
            },
            K::Match {
                scrutinee: y,
                arms: b,
            },
        ) => {
            same(x, y)
                && a.len() == b.len()
                && a.iter().zip(b).all(|((p, e), (q, f))| p == q && same(e, f))
        }
        _ => false,
    }
}

/// True when event `event` (a complete constructor call) matches `pattern`,
/// which may omit trailing arguments.
#[must_use]
pub fn matches(pattern: &Expr, event: &Expr) -> bool {
    match (&pattern.kind, &event.kind) {
        (
            ExprKind::Call {
                callee: p,
                args: pa,
            },
            ExprKind::Call {
                callee: e,
                args: ea,
            },
        ) => p == e && pa.len() <= ea.len() && pa.iter().zip(ea).all(|(x, y)| same(x, y)),
        _ => false,
    }
}

/// The event pattern of `has_event(P)`.
#[must_use]
pub fn has_event(e: &Expr) -> Option<&Expr> {
    match &e.kind {
        ExprKind::Call { callee, args } if callee == "has_event" && args.len() == 1 => args.first(),
        _ => None,
    }
}

fn is_bool(e: &Expr) -> bool {
    e.ty == Type::Boolean
}

/// Evaluates `e`, asking `atom` for every atom; `None` when an atom is
/// unknown and the result depends on it (Kleene logic: `false && ?` is
/// `false`, `true || ?` is `true`).
pub fn eval(e: &Expr, atom: &mut dyn FnMut(&Expr) -> Option<bool>) -> Option<bool> {
    match &e.kind {
        ExprKind::Literal {
            value: Value::Boolean(b),
        } => Some(*b),
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand,
        } => eval(operand, atom).map(|b| !b),
        ExprKind::Binary { op, lhs, rhs } if is_bool(lhs) && is_bool(rhs) => {
            let (a, b) = (eval(lhs, atom), eval(rhs, atom));
            match op {
                BinaryOp::And if a == Some(false) || b == Some(false) => Some(false),
                BinaryOp::Or if a == Some(true) || b == Some(true) => Some(true),
                BinaryOp::And | BinaryOp::Or => a.and(b),
                BinaryOp::Eq => Some(a? == b?),
                BinaryOp::Ne => Some(a? != b?),
                _ => atom(e),
            }
        }
        _ => atom(e),
    }
}

/// The distinct atoms of `exprs`, in first-seen order.
#[must_use]
pub fn atoms<'e>(exprs: &[&'e Expr]) -> Vec<&'e Expr> {
    fn walk<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
        match &e.kind {
            ExprKind::Literal {
                value: Value::Boolean(_),
            } => {}
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            } => walk(operand, out),
            ExprKind::Binary {
                op: BinaryOp::And | BinaryOp::Or | BinaryOp::Eq | BinaryOp::Ne,
                lhs,
                rhs,
            } if is_bool(lhs) && is_bool(rhs) => {
                walk(lhs, out);
                walk(rhs, out);
            }
            _ => {
                if !out.iter().any(|a| same(a, e)) {
                    out.push(e);
                }
            }
        }
    }
    let mut out = Vec::new();
    for e in exprs {
        walk(e, &mut out);
    }
    out
}

/// Most atoms a satisfiability check enumerates (2^n assignments).
pub const MAX_ATOMS: usize = 16;

/// Whether some assignment of the atoms of `exprs` satisfies `holds`;
/// `None` when there are more than [`MAX_ATOMS`] atoms.
fn exists(
    exprs: &[&Expr],
    holds: impl Fn(&mut dyn FnMut(&Expr) -> Option<bool>) -> bool,
) -> Option<bool> {
    let atoms = atoms(exprs);
    if atoms.len() > MAX_ATOMS {
        return None;
    }
    let found = (0_u32..1 << atoms.len()).any(|bits| {
        let mut lookup = |a: &Expr| {
            atoms
                .iter()
                .position(|x| same(x, a))
                .map(|i| bits & (1 << i) != 0)
        };
        holds(&mut lookup)
    });
    Some(found)
}

/// Whether all of `exprs` can be true together for some assignment of
/// their atoms; `None` when there are more than [`MAX_ATOMS`] atoms.
#[must_use]
pub fn jointly_satisfiable(exprs: &[&Expr]) -> Option<bool> {
    exists(exprs, |lookup| {
        exprs.iter().all(|e| eval(e, lookup) == Some(true))
    })
}

/// Whether `a` entails `b`: no assignment of their atoms makes `a` true
/// and `b` false. `None` when there are more than [`MAX_ATOMS`] atoms.
#[must_use]
pub fn entails(a: &Expr, b: &Expr) -> Option<bool> {
    exists(&[a, b], |lookup| {
        eval(a, lookup) == Some(true) && eval(b, lookup) == Some(false)
    })
    .map(|counterexample| !counterexample)
}

/// Whether `e` is a date a fact guard may read from the matter: a date
/// literal, `date_of(P)`, or `deadline(d)`.
fn date_term(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Literal {
            value: Value::Date(_),
        } => true,
        ExprKind::Call { callee, .. } => callee == "date_of" || callee == "deadline",
        _ => false,
    }
}

/// The first atom of fact guard `e` that is neither `has_event(...)` nor a
/// comparison of two date terms ([`date_term`]).
#[must_use]
pub fn first_foreign_guard(e: &Expr) -> Option<&Expr> {
    atoms(&[e]).into_iter().find(|a| {
        has_event(a).is_none()
            && !matches!(
                &a.kind,
                ExprKind::Binary { op: BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge, lhs, rhs }
                    if date_term(lhs) && date_term(rhs)
            )
    })
}

/// The first sub-expression of `e` that is not a connective, a boolean
/// literal, or `has_event(...)`.
#[must_use]
pub fn first_foreign(e: &Expr) -> Option<&Expr> {
    atoms(&[e]).into_iter().find(|a| has_event(a).is_none())
}

#[cfg(test)]
mod tests;
