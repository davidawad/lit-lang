//! Boolean conditions of transitions, defaults, and exceptions: connectives
//! over opaque atoms (SPEC §5.4-§5.5).
//!
//! An atom is any boolean sub-expression that is not `!`, `&&`, `||`, a
//! boolean `==`/`!=`, or a boolean literal; in practice `has_event(P)`.
//! Atoms compare structurally, ignoring spans.
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
/// unknown.
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
            let (a, b) = (eval(lhs, atom)?, eval(rhs, atom)?);
            match op {
                BinaryOp::And => Some(a && b),
                BinaryOp::Or => Some(a || b),
                BinaryOp::Eq => Some(a == b),
                BinaryOp::Ne => Some(a != b),
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

/// Whether all of `exprs` can be true together for some assignment of
/// their atoms; `None` when there are more than [`MAX_ATOMS`] atoms.
#[must_use]
pub fn jointly_satisfiable(exprs: &[&Expr]) -> Option<bool> {
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
        exprs.iter().all(|e| eval(e, &mut lookup) == Some(true))
    });
    Some(found)
}

/// The first sub-expression of `e` that is not a connective, a boolean
/// literal, or `has_event(...)`.
#[must_use]
pub fn first_foreign(e: &Expr) -> Option<&Expr> {
    atoms(&[e]).into_iter().find(|a| has_event(a).is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(b: bool) -> Expr {
        Expr {
            ty: Type::Boolean,
            kind: ExprKind::Literal {
                value: Value::Boolean(b),
            },
            span: None,
        }
    }

    fn event(name: &str, args: &[&str]) -> Expr {
        Expr {
            ty: Type::Named(name.into()),
            kind: ExprKind::Call {
                callee: name.into(),
                args: args
                    .iter()
                    .map(|a| Expr {
                        ty: Type::Named("E".into()),
                        kind: ExprKind::Literal {
                            value: Value::Variant((*a).into()),
                        },
                        span: None,
                    })
                    .collect(),
            },
            span: None,
        }
    }

    fn has(name: &str, args: &[&str]) -> Expr {
        Expr {
            ty: Type::Boolean,
            kind: ExprKind::Call {
                callee: "has_event".into(),
                args: vec![event(name, args)],
            },
            span: None,
        }
    }

    fn bin(op: BinaryOp, a: Expr, b: Expr) -> Expr {
        Expr {
            ty: Type::Boolean,
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(a),
                rhs: Box::new(b),
            },
            span: None,
        }
    }

    fn not(a: Expr) -> Expr {
        Expr {
            ty: Type::Boolean,
            kind: ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(a),
            },
            span: None,
        }
    }

    #[test]
    fn patterns_match_by_prefix() {
        assert!(matches(
            &event("Served", &["c"]),
            &event("Served", &["c", "Mail"])
        ));
        assert!(!matches(
            &event("Served", &["a"]),
            &event("Served", &["c", "Mail"])
        ));
        assert!(!matches(&event("Filed", &[]), &event("Served", &["c"])));
        assert!(!matches(
            &event("Served", &["c", "Mail", "x"]),
            &event("Served", &["c", "Mail"])
        ));
    }

    #[test]
    fn satisfiability_over_has_event_atoms() {
        let a = has("Ordered", &[]);
        let b = has("Served", &["c"]);
        let not_a = not(a.clone());
        assert_eq!(jointly_satisfiable(&[&a, &not_a]), Some(false));
        assert_eq!(jointly_satisfiable(&[&a, &b]), Some(true));
        let both = bin(BinaryOp::And, a.clone(), b.clone());
        let neither = not(bin(BinaryOp::Or, a.clone(), b));
        assert_eq!(jointly_satisfiable(&[&both, &neither]), Some(false));
        assert_eq!(jointly_satisfiable(&[&lit(false)]), Some(false));
        assert_eq!(atoms(&[&both, &a]).len(), 2);
        assert!(first_foreign(&both).is_none());
    }
}
