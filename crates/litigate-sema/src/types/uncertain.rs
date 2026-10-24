//! Epistemic uncertainty in expressions (design review 2026-10-23 §2.11):
//! `Credal` intervals of probabilities, checked by interval arithmetic and
//! used as a point only through an explicit projection (`lower`, `upper`,
//! `mean`: the decision rule), and conditional distributions `Dist<E, C>`,
//! total over the conditioning enum `C` and read with `at(d, c)`.
use super::expr::{Checked, Checker};
use super::rational::{self as q, ArithError};
use super::ty::display;
use crate::db::parse::tree::{Expr, ExprKind, Name};
use crate::db::span::{Report, TextRange};
use crate::hir::{self, BinaryOp, Rational, Type, Value};
use litigate_base::DiagnosticCode as C;

/// The builtins this module checks, with their arities.
pub const NAMES: [(&str, usize); 5] = [
    ("credal", 2),
    ("lower", 1),
    ("upper", 1),
    ("mean", 1),
    ("at", 2),
];

/// The note on a `Credal` used where a point probability is required.
pub const PROJECT: &str = "a credal set is not a point: project it with a decision rule, \
                           `mean(..)`, `lower(..)` or `upper(..)`";

fn call_node(name: &str, args: Vec<hir::Expr>) -> hir::ExprKind {
    hir::ExprKind::Call {
        callee: name.to_owned(),
        args,
    }
}

/// Checks one of [`NAMES`] (arity already checked).
pub fn call(c: &mut Checker<'_, '_>, name: &str, args: &[Expr], r: TextRange) -> Checked {
    match name {
        "credal" => credal(c, args, r),
        "at" => at(c, args, r),
        _ => projection(c, name, &args[0], r),
    }
}

/// `credal(lo, hi)`: the probabilities from `lo` to `hi`.
fn credal(c: &mut Checker<'_, '_>, args: &[Expr], r: TextRange) -> Checked {
    let lo = c.check(&args[0], &Type::Probability);
    let hi = c.check(&args[1], &Type::Probability);
    let ok = lo.ty.is_some() && hi.ty.is_some();
    let value = match (&lo.value, &hi.value) {
        (Some(Value::Probability(a)), Some(Value::Probability(b))) => {
            if q::cmp(*a, *b).is_gt() {
                c.report(Report::new(
                    C::ArithmeticError,
                    r,
                    format!(
                        "credal set [{}, {}] is empty: its lower bound exceeds its upper bound",
                        q::display(*a),
                        q::display(*b)
                    ),
                ));
                return c.error(r);
            }
            Some(Value::Credal {
                lower: *a,
                upper: *b,
            })
        }
        _ => None,
    };
    let kind = call_node("credal", vec![lo.hir, hi.hir]);
    c.node(kind, ok.then_some(Type::Credal), value, r)
}

/// `lower(s)`, `upper(s)`, `mean(s)`: a credal set's point under a
/// decision rule (the midpoint for `mean`).
fn projection(c: &mut Checker<'_, '_>, name: &str, arg: &Expr, r: TextRange) -> Checked {
    let s = c.check(arg, &Type::Credal);
    let value = match &s.value {
        Some(Value::Credal { lower, upper }) => {
            let point = match name {
                "lower" => Ok(*lower),
                "upper" => Ok(*upper),
                _ => q::add(*lower, *upper).and_then(|t| q::div(t, Rational::integer(2))),
            };
            match point {
                Ok(p) => Some(Value::Probability(p)),
                Err(e) => return c.arith(r, e),
            }
        }
        _ => None,
    };
    let ty = s.ty.as_ref().map(|_| Type::Probability);
    let kind = call_node(name, vec![s.hir]);
    c.node(kind, ty, value, r)
}

/// `at(d, c)`: the distribution `d: Dist<E, C>` gives under condition `c`.
fn at(c: &mut Checker<'_, '_>, args: &[Expr], r: TextRange) -> Checked {
    let d = conditional_arg(c, &args[0]);
    let (outcome, given) = match &d.ty {
        Some(Type::Conditional(e, g)) => (e.clone(), g.clone()),
        Some(other) => {
            c.report(Report::new(
                C::TypeMismatch,
                args[0].range,
                format!(
                    "`at` needs a conditional `Dist<E, C>`, found `{}`",
                    display(other)
                ),
            ));
            return c.error(r);
        }
        None => return c.error(r),
    };
    let v = c.check(&args[1], &Type::Named(given));
    let value = match (&d.value, &v.value) {
        (Some(Value::Conditional(rows)), Some(Value::Variant(q))) => {
            let variant = q.rsplit("::").next().unwrap_or_default();
            rows.iter()
                .find(|(k, _)| k == variant)
                .map(|(_, w)| Value::Distribution(w.clone()))
        }
        _ => None,
    };
    let ty = v.ty.as_ref().map(|_| Type::Distribution(outcome));
    let kind = call_node("at", vec![d.hir, v.hir]);
    c.node(kind, ty, value, r)
}

/// The unique visible enum covering `keys`, else a report naming `what`.
fn covering(c: &mut Checker<'_, '_>, keys: &[&str], what: &str, r: TextRange) -> Option<String> {
    match c.scope.enums_covering(keys).as_slice() {
        [en] => Some(en.clone()),
        [] => {
            c.report(
                Report::new(
                    C::InvalidDistribution,
                    r,
                    format!("no visible enum has these {what}"),
                )
                .note("a `Dist<E, C>` literal names every variant of `C`, each with a `Dist<E>`"),
            );
            None
        }
        many => {
            c.report(Report::new(
                C::TypeMismatch,
                r,
                format!(
                    "ambiguous conditional distribution: enums {} all have these {what}",
                    many.join(", ")
                ),
            ));
            None
        }
    }
}

/// Infers `at`'s first argument; a record literal of records is typed by
/// the enums covering its keys and its first row's keys.
fn conditional_arg(c: &mut Checker<'_, '_>, arg: &Expr) -> Checked {
    let ExprKind::Record(fields) = &arg.kind else {
        return c.infer(arg);
    };
    let keys: Vec<&str> = fields.iter().map(|(n, _)| n.text.as_str()).collect();
    let Some(given) = covering(c, &keys, "conditions", arg.range) else {
        return c.error(arg.range);
    };
    let Some(ExprKind::Record(row)) = fields.first().map(|(_, v)| &v.kind) else {
        c.report(Report::new(
            C::TypeMismatch,
            arg.range,
            "cannot infer the outcome enum: write the first condition's `Dist` as a record",
        ));
        return c.error(arg.range);
    };
    let outcomes: Vec<&str> = row.iter().map(|(n, _)| n.text.as_str()).collect();
    let Some(outcome) = covering(c, &outcomes, "outcomes", arg.range) else {
        return c.error(arg.range);
    };
    conditional(c, fields, &outcome, &given, arg.range)
}

/// A record checked as `Dist<outcome, given>`: every variant of `given`
/// exactly once, each with an exact `Dist<outcome>` (L2003 otherwise).
pub fn conditional(
    c: &mut Checker<'_, '_>,
    fields: &[(Name, Expr)],
    outcome: &str,
    given: &str,
    r: TextRange,
) -> Checked {
    let variants = c
        .scope
        .enum_variants(given)
        .map(<[String]>::to_vec)
        .unwrap_or_default();
    let dist = Type::Distribution(outcome.to_owned());
    let (mut ok, mut rows, mut hir_fields) = (true, Vec::new(), Vec::new());
    for (name, e) in fields {
        if !variants.contains(&name.text) || hir_fields.iter().any(|(n, _)| *n == name.text) {
            let message = if variants.contains(&name.text) {
                format!("condition `{}` appears twice", name.text)
            } else {
                format!("`{given}` has no variant `{}`", name.text)
            };
            c.report(Report::new(C::InvalidDistribution, name.range, message));
            ok = false;
            continue;
        }
        let ch = c.check(e, &dist);
        ok &= ch.ty.is_some();
        if let Some(Value::Distribution(w)) = ch.value {
            rows.push((name.text.clone(), w));
        }
        hir_fields.push((name.text.clone(), ch.hir));
    }
    let missing: Vec<String> = variants
        .iter()
        .filter(|v| !hir_fields.iter().any(|(n, _)| n == *v))
        .map(|v| format!("`{v}`"))
        .collect();
    if !missing.is_empty() {
        c.report(
            Report::new(
                C::InvalidDistribution,
                r,
                format!(
                    "`Dist<{outcome}, {given}>` is missing {}",
                    missing.join(", ")
                ),
            )
            .note("a conditional distribution gives a `Dist` under every condition"),
        );
        ok = false;
    }
    let ty = Type::Conditional(outcome.to_owned(), given.to_owned());
    if ok && rows.len() == variants.len() {
        rows.sort_by_key(|(k, _)| variants.iter().position(|v| v == k));
        return c.literal(Value::Conditional(rows), ty, r);
    }
    hir_fields.sort_by(|a, b| a.0.cmp(&b.0));
    c.node(
        hir::ExprKind::Record { fields: hir_fields },
        ok.then_some(ty),
        None,
        r,
    )
}

/// Interval arithmetic on credal sets `[al, au]` and `[bl, bu]` (a
/// probability is the one-point set): `+`, `-`, and otherwise `*`.
///
/// # Errors
/// Exact arithmetic overflow.
pub fn interval(
    op: BinaryOp,
    (al, au): (Rational, Rational),
    (bl, bu): (Rational, Rational),
) -> Result<Value, ArithError> {
    let (lower, upper) = match op {
        BinaryOp::Add => (q::add(al, bl), q::add(au, bu)),
        BinaryOp::Sub => (q::sub(al, bu), q::sub(au, bl)),
        // Every bound is non-negative, so the product's bounds are the
        // products of the bounds.
        _ => (q::mul(al, bl), q::mul(au, bu)),
    };
    let (lower, upper) = lower.and_then(|l| upper.map(|u| (l, u)))?;
    Ok(Value::Credal { lower, upper })
}

/// Reports a credal result that leaves `[0, 1]`, or a sum or difference
/// of credal sets that does not fold (so cannot be shown to stay inside);
/// true when it was reported.
pub fn out_of_range(
    c: &mut Checker<'_, '_>,
    op: BinaryOp,
    value: Option<&Value>,
    r: TextRange,
) -> bool {
    let message = match value {
        Some(Value::Credal { lower, upper })
            if lower.numerator < 0 || q::cmp(*upper, Rational::integer(1)).is_gt() =>
        {
            format!(
                "credal set [{}, {}] leaves [0, 1]",
                q::display(*lower),
                q::display(*upper)
            )
        }
        None if matches!(op, BinaryOp::Add | BinaryOp::Sub) => {
            "a sum or difference of credal sets that does not fold is not provably within [0, 1]"
                .to_owned()
        }
        _ => return false,
    };
    c.report(Report::new(C::ArithmeticError, r, message));
    true
}
