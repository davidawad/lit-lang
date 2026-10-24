//! Operator typing and exact constant folding (SPEC §6 permitted-operations
//! table). No implicit conversions: mixing day units or currencies is
//! L2004, anything outside the table is L2002.
use super::expr::{Checked, Checker};
use super::rational::{self as q, ArithError};
use super::refine;
use super::ty::{display, unit_name};
use crate::db::parse::tree::Expr;
use crate::db::span::{Report, TextRange};
use crate::hir::{self, BinaryOp, DayUnit, Money, Rational, Type, UnaryOp, Value};
use crate::resolve::sig::ValueKind;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::cmp::Ordering;

const fn symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Or => "||",
        BinaryOp::And => "&&",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
    }
}

/// Adds the "convert through a counting rule" note and fix to an L2004
/// report about mixing `a` and `b` day units.
pub fn counting_hint(
    c: &Checker<'_, '_>,
    report: Report,
    a: DayUnit,
    b: DayUnit,
    r: TextRange,
) -> Report {
    let rules: Vec<String> = c
        .scope
        .value_names()
        .into_iter()
        .filter(|n| {
            std::iter::once(c.scope.own)
                .chain(c.scope.bindings.iter().map(|b| b.sig))
                .filter_map(|s| s.values.get(*n))
                .any(|v| matches!(v.kind, ValueKind::Counting(_)))
        })
        .map(str::to_owned)
        .collect();
    let rule = rules
        .first()
        .cloned()
        .unwrap_or_else(|| "<counting-rule>".to_owned());
    report
        .note(format!(
            "`{}` and `{}` count different days; convert by declaring a deadline counted by a counting rule, e.g. `deadline <name>: N.<unit> from <Trigger> counted_by {rule}`, and compare its `deadline(<name>)` date",
            unit_name(a),
            unit_name(b)
        ))
        .fix(r, "deadline(<name>)", Applicability::HasPlaceholders)
}

/// `!e` / `-e`.
pub fn unary(c: &mut Checker<'_, '_>, op: UnaryOp, inner: &Expr, r: TextRange) -> Checked {
    let e = c.infer(inner);
    let Some(ty) = e.ty.clone() else {
        return c.error(r);
    };
    let ok = match op {
        UnaryOp::Not => ty == Type::Boolean,
        UnaryOp::Neg => matches!(
            ty,
            Type::Number | Type::Money(_) | Type::Days(_) | Type::Hours
        ),
    };
    if !ok {
        let sym = if op == UnaryOp::Not { "!" } else { "-" };
        c.report(Report::new(
            C::InvalidOperation,
            r,
            format!("operator `{sym}` is not defined for `{}`", display(&ty)),
        ));
        return c.error(r);
    }
    let value = match (op, e.value) {
        (UnaryOp::Not, Some(Value::Boolean(b))) => Some(Value::Boolean(!b)),
        (UnaryOp::Neg, Some(v)) => match fold(BinaryOp::Sub, &zero_like(&v), &v) {
            Ok(v) => v,
            Err(err) => return c.arith(r, err),
        },
        _ => None,
    };
    c.node(
        hir::ExprKind::Unary {
            op,
            operand: Box::new(e.hir),
        },
        Some(ty),
        value,
        r,
    )
}

fn zero_like(v: &Value) -> Value {
    match v {
        Value::Money(m) => Value::Money(Money {
            minor_units: 0,
            currency: m.currency.clone(),
        }),
        Value::Days { unit, .. } => Value::Days {
            amount: 0,
            unit: *unit,
        },
        Value::Hours(_) => Value::Hours(Rational::integer(0)),
        _ => Value::Number(Rational::integer(0)),
    }
}

/// Result type of `l op r`, or the code and message rejecting it.
#[allow(clippy::match_same_arms)] // one arm per row of the SPEC §6 table
fn result_type(op: BinaryOp, l: &Type, r: &Type) -> Result<Type, (C, String)> {
    use BinaryOp as B;
    use Type as T;
    let bad = || {
        Err((
            C::InvalidOperation,
            format!(
                "operator `{}` is not defined for `{}` and `{}`",
                symbol(op),
                display(l),
                display(r)
            ),
        ))
    };
    let mixed = match (l, r) {
        (T::Days(_), T::Days(_)) | (T::Money(_), T::Money(_)) => l != r,
        _ => false,
    };
    if mixed {
        return Err((
            C::UnitMismatch,
            format!("cannot combine `{}` with `{}`", display(l), display(r)),
        ));
    }
    // Interval arithmetic: a credal set with a credal set or a point.
    let interval = |t: &Type| matches!(t, T::Credal | T::Probability);
    if matches!(op, B::Add | B::Sub | B::Mul)
        && (*l == T::Credal || *r == T::Credal)
        && interval(l)
        && interval(r)
    {
        return Ok(T::Credal);
    }
    match (op, l, r) {
        (B::Or | B::And, T::Boolean, T::Boolean) => Ok(T::Boolean),
        (B::Eq | B::Ne, a, b) if a == b && !matches!(a, T::List(_) | T::Distribution(_) | T::Conditional(..) | T::Unit) => Ok(T::Boolean),
        (B::Lt | B::Le | B::Gt | B::Ge, a, b) if a == b && matches!(a, T::Date | T::Days(_) | T::Hours | T::Money(_) | T::Probability | T::Number) => Ok(T::Boolean),
        (B::Add | B::Sub, a, b) if a == b && matches!(a, T::Days(_) | T::Hours | T::Money(_) | T::Probability | T::Number) => Ok(a.clone()),
        (B::Mul | B::Div, T::Days(_) | T::Hours | T::Money(_), T::Number) => Ok(l.clone()),
        (B::Mul, T::Number, T::Days(_) | T::Hours | T::Money(_)) => Ok(r.clone()),
        (B::Div, T::Money(_), T::Money(_)) => Ok(T::Number),
        (B::Mul | B::Div, T::Probability, T::Probability) | (B::Mul | B::Div | B::Rem, T::Number, T::Number) => Ok(l.clone()),
        (_, T::Date, T::Days(_) | T::Hours) | (_, T::Days(_) | T::Hours, T::Date) => Err((C::InvalidOperation, format!("cannot do date arithmetic with `{}`; dates move only through a counting rule (`deadline(..)`)", display(if *l == T::Date { r } else { l })))),
        _ => bad(),
    }
}

/// Type-checks and folds `lhs op rhs`.
pub fn binary(
    c: &mut Checker<'_, '_>,
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
    r: TextRange,
) -> Checked {
    let mut l = c.infer(lhs);
    let mut rr = c.infer(rhs);
    let (Some(lt), Some(rt)) = (l.ty.clone(), rr.ty.clone()) else {
        return c.error(r);
    };
    let point = |t: &Type| matches!(t, Type::Probability | Type::Credal);
    if point(&lt) && rt == Type::Number {
        rr = c.coerce(rr, &Type::Probability, rhs.range);
    } else if lt == Type::Number && point(&rt) {
        l = c.coerce(l, &Type::Probability, lhs.range);
    }
    let (Some(lt), Some(rt)) = (l.ty.clone(), rr.ty.clone()) else {
        return c.error(r);
    };
    let ty = match result_type(op, &lt, &rt) {
        Ok(ty) => ty,
        Err((code, message)) => {
            let mut report = Report::new(code, r, message);
            if let (C::UnitMismatch, Type::Days(a), Type::Days(b)) = (code, &lt, &rt) {
                report = counting_hint(c, report, *a, *b, r);
            }
            c.report(report);
            return c.error(r);
        }
    };
    // A point beside a credal set is its one-point set.
    let widen = |v: &Value| match v {
        Value::Probability(p) if ty == Type::Credal => Value::Credal {
            lower: *p,
            upper: *p,
        },
        other => other.clone(),
    };
    let value = match (&l.value, &rr.value) {
        (Some(a), Some(b)) => match fold(op, &widen(a), &widen(b)) {
            Ok(v) => v,
            Err(err) => return c.arith(r, err),
        },
        _ => None,
    };
    if ty == Type::Credal && super::uncertain::out_of_range(c, op, value.as_ref(), r) {
        return c.error(r);
    }
    if let Some(Value::Probability(p)) = &value {
        if !q::is_probability(*p) {
            c.report(Report::new(
                C::ArithmeticError,
                r,
                format!("probability {} is outside [0, 1]", q::display(*p)),
            ));
            return c.error(r);
        }
    }
    let (lf, rf) = (refine::operand(&l), refine::operand(&rr));
    let mut out = c.node(
        hir::ExprKind::Binary {
            op,
            lhs: Box::new(l.hir),
            rhs: Box::new(rr.hir),
        },
        Some(ty),
        value,
        r,
    );
    if out.ty == Some(Type::Probability) && out.value.is_none() {
        // A term over unknowns: decide its refinement exactly (`refine`).
        let decided = refine::combine(op, lf, rf, &out.hir).and_then(|form| match form {
            Ok(form) => refine::probability(&form).map(|v| (Some(form), v)),
            Err(v) => Ok((None, Some(v))),
        });
        match decided {
            Ok((form, None)) => out.form = form,
            Ok((_, Some(v))) => {
                c.report(Report::new(C::ArithmeticError, r, v.message()).note(v.note()));
                return c.error(r);
            }
            Err(err) => return c.arith(r, err),
        }
    }
    out
}

fn int(r: Rational) -> Result<i64, ArithError> {
    if r.denominator == 1 {
        Ok(r.numerator)
    } else {
        Err(ArithError::Inexact)
    }
}

/// Folds two constants; `Ok(None)` when the operation does not fold.
///
/// # Errors
/// Overflow, division by zero, or a non-integral day/money result.
#[allow(clippy::too_many_lines)] // one arm per value pair of the SPEC §6 table
pub fn fold(op: BinaryOp, a: &Value, b: &Value) -> Result<Option<Value>, ArithError> {
    use BinaryOp as B;
    use Value as V;
    let arith = |x: Rational, y: Rational| -> Result<Rational, ArithError> {
        match op {
            B::Add => q::add(x, y),
            B::Sub => q::sub(x, y),
            B::Mul => q::mul(x, y),
            B::Div => q::div(x, y),
            _ => q::rem(x, y),
        }
    };
    let ord = |o: Ordering| match op {
        B::Eq => o == Ordering::Equal,
        B::Ne => o != Ordering::Equal,
        B::Lt => o == Ordering::Less,
        B::Le => o != Ordering::Greater,
        B::Gt => o == Ordering::Greater,
        _ => o != Ordering::Less,
    };
    let num = |v: &V| match v {
        V::Number(x) | V::Probability(x) | V::Hours(x) => Some(*x),
        V::Money(m) => Some(Rational::integer(m.minor_units)),
        V::Days { amount, .. } => Some(Rational::integer(*amount)),
        _ => None,
    };
    // A money or day result in whole units; `None` when an operand is not
    // numeric.
    let whole = || -> Result<Option<i64>, ArithError> {
        let (Some(x), Some(y)) = (num(a), num(b)) else {
            return Ok(None);
        };
        arith(x, y).and_then(int).map(Some)
    };
    if matches!(op, B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge) {
        let o = match (num(a), num(b), a, b) {
            (Some(x), Some(y), _, _) => q::cmp(x, y),
            (_, _, V::Date(x), V::Date(y)) => x.cmp(y),
            _ if matches!(op, B::Eq | B::Ne) => {
                if a == b {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            }
            _ => return Ok(None),
        };
        return Ok(Some(V::Boolean(ord(o))));
    }
    Ok(Some(match (op, a, b) {
        (_, V::Credal { lower: l, upper: u }, V::Credal { lower: m, upper: v }) => {
            super::uncertain::interval(op, (*l, *u), (*m, *v))?
        }
        (B::And, V::Boolean(x), V::Boolean(y)) => V::Boolean(*x && *y),
        (B::Or, V::Boolean(x), V::Boolean(y)) => V::Boolean(*x || *y),
        (_, V::Number(x), V::Number(y)) => V::Number(arith(*x, *y)?),
        (_, V::Probability(x), V::Probability(y)) => V::Probability(arith(*x, *y)?),
        (_, V::Hours(x), V::Hours(y) | V::Number(y)) | (_, V::Number(x), V::Hours(y)) => {
            V::Hours(arith(*x, *y)?)
        }
        (B::Div, V::Money(x), V::Money(y)) => V::Number(q::div(
            Rational::integer(x.minor_units),
            Rational::integer(y.minor_units),
        )?),
        (_, V::Money(m), _) | (_, _, V::Money(m)) => {
            let Some(v) = whole()? else {
                return Ok(None);
            };
            V::Money(Money {
                minor_units: v,
                currency: m.currency.clone(),
            })
        }
        (_, V::Days { unit, .. }, _) | (_, _, V::Days { unit, .. }) => {
            let Some(v) = whole()? else {
                return Ok(None);
            };
            V::Days {
                amount: v,
                unit: *unit,
            }
        }
        _ => return Ok(None),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_types_follow_the_table() {
        let days = Type::Days(DayUnit::Calendar);
        let court = Type::Days(DayUnit::Court);
        let usd = Type::Money("usd".into());
        assert_eq!(result_type(BinaryOp::Add, &days, &days), Ok(days.clone()));
        assert!(matches!(
            result_type(BinaryOp::Add, &days, &court),
            Err((C::UnitMismatch, _))
        ));
        assert!(matches!(
            result_type(BinaryOp::Add, &Type::Date, &days),
            Err((C::InvalidOperation, _))
        ));
        assert_eq!(result_type(BinaryOp::Div, &usd, &usd), Ok(Type::Number));
        assert_eq!(
            result_type(BinaryOp::Mul, &Type::Number, &usd),
            Ok(usd.clone())
        );
        assert!(matches!(
            result_type(BinaryOp::Lt, &Type::Cite, &Type::Cite),
            Err((C::InvalidOperation, _))
        ));
        assert!(matches!(
            result_type(BinaryOp::Rem, &Type::Probability, &Type::Probability),
            Err((C::InvalidOperation, _))
        ));
    }

    #[test]
    fn folding_is_exact_and_checked() {
        let usd = |n| {
            Value::Money(Money {
                minor_units: n,
                currency: "usd".into(),
            })
        };
        let n = |x| Value::Number(Rational::integer(x));
        assert_eq!(fold(BinaryOp::Mul, &usd(100), &n(3)), Ok(Some(usd(300))));
        assert_eq!(
            fold(BinaryOp::Div, &usd(100), &n(3)),
            Err(ArithError::Inexact)
        );
        assert_eq!(
            fold(BinaryOp::Div, &n(1), &n(0)),
            Err(ArithError::DivisionByZero)
        );
        assert_eq!(
            fold(
                BinaryOp::Lt,
                &Value::Date("2025-01-01".into()),
                &Value::Date("2025-02-01".into())
            ),
            Ok(Some(Value::Boolean(true)))
        );
    }

    #[test]
    fn currencies_do_not_mix() {
        let usd = Type::Money("usd".into());
        let eur = Type::Money("eur".into());
        assert!(matches!(
            result_type(BinaryOp::Add, &usd, &eur),
            Err((C::UnitMismatch, _))
        ));
    }
}
