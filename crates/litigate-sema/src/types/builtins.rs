//! The closed builtin list (SPEC §6): `deadline`, `has_event`, `prob`,
//! `given`, `in`, `intersect`, `disjoint`, `len`, `contains`.
use super::expr::{Checked, Checker};
use super::rational;
use super::ty::display;
use crate::db::parse::tree::{Expr, ExprKind};
use crate::db::span::{Report, TextRange};
use crate::hir::{self, Type, Value};
use crate::resolve::sig::ValueKind;
use litigate_base::DiagnosticCode as C;

/// Builtin function names.
pub const NAMES: [&str; 9] = [
    "deadline",
    "has_event",
    "prob",
    "given",
    "in",
    "intersect",
    "disjoint",
    "len",
    "contains",
];

fn arity(c: &mut Checker<'_, '_>, name: &str, args: &[Expr], n: usize, r: TextRange) -> bool {
    let ok = args.len() == n;
    if !ok {
        c.report(Report::new(
            C::TypeMismatch,
            r,
            format!(
                "`{name}` takes {n} argument{}, found {}",
                if n == 1 { "" } else { "s" },
                args.len()
            ),
        ));
    }
    ok
}

/// Checks a builtin call; `None` when `name` is not a builtin.
pub fn call(c: &mut Checker<'_, '_>, name: &str, args: &[Expr], r: TextRange) -> Option<Checked> {
    if !NAMES.contains(&name) {
        return None;
    }
    let n = match name {
        "deadline" | "has_event" | "len" => 1,
        _ => 2,
    };
    if !arity(c, name, args, n, r) {
        return Some(c.error(r));
    }
    let mk = |c: &Checker<'_, '_>, args: Vec<hir::Expr>, ty: Option<Type>, value: Option<Value>| {
        c.node(
            hir::ExprKind::Call {
                callee: name.to_owned(),
                args,
            },
            ty,
            value,
            r,
        )
    };
    Some(match name {
        "deadline" => {
            let arg = deadline_ref(c, &args[0]);
            let ty = arg.as_ref().map(|_| Type::Date);
            let args = arg.into_iter().collect();
            mk(c, args, ty, None)
        }
        "has_event" => {
            let arg = match &args[0].kind {
                ExprKind::Call(callee, inner) => c.event_call(callee, inner, false, args[0].range),
                _ => {
                    c.report(Report::new(
                        C::TypeMismatch,
                        args[0].range,
                        "`has_event` takes an event pattern such as `Served(complaint, Mail)`",
                    ));
                    c.error(args[0].range)
                }
            };
            let ty = arg.ty.as_ref().map(|_| Type::Boolean);
            mk(c, vec![arg.hir], ty, None)
        }
        "prob" | "given" => dist_call(c, name, args, r),
        "in" | "intersect" | "disjoint" => {
            let first = if name == "in" {
                Type::Date
            } else {
                Type::DateInterval
            };
            let a = c.check(&args[0], &first);
            let b = c.check(&args[1], &Type::DateInterval);
            let ok = a.ty.is_some() && b.ty.is_some();
            let ty = if name == "intersect" {
                Type::DateInterval
            } else {
                Type::Boolean
            };
            mk(c, vec![a.hir, b.hir], ok.then_some(ty), None)
        }
        _ => {
            let list = c.infer(&args[0]);
            let elem = match &list.ty {
                Some(Type::List(t)) => Some((**t).clone()),
                Some(other) => {
                    c.report(Report::new(
                        C::TypeMismatch,
                        args[0].range,
                        format!("`{name}` needs a list, found `{}`", display(other)),
                    ));
                    None
                }
                None => None,
            };
            if name == "len" {
                let value = match &list.hir.kind {
                    hir::ExprKind::List { items } => Some(Value::Number(hir::Rational::integer(
                        i64::try_from(items.len()).unwrap_or(i64::MAX),
                    ))),
                    _ => None,
                };
                mk(c, vec![list.hir], elem.map(|_| Type::Number), value)
            } else {
                let x = match &elem {
                    Some(t) => c.check(&args[1], t),
                    None => c.infer(&args[1]),
                };
                let ok = elem.is_some() && x.ty.is_some();
                mk(c, vec![list.hir, x.hir], ok.then_some(Type::Boolean), None)
            }
        }
    })
}

fn deadline_ref(c: &mut Checker<'_, '_>, arg: &Expr) -> Option<hir::Expr> {
    let ExprKind::Ref(path) = &arg.kind else {
        c.report(Report::new(
            C::TypeMismatch,
            arg.range,
            "`deadline` takes the name of a deadline declaration",
        ));
        return None;
    };
    match c.scope.value(path) {
        Ok((target, found)) if matches!(found.item.kind, ValueKind::Deadline) => Some(
            c.node(
                hir::ExprKind::Ref { target },
                Some(Type::Unit),
                None,
                arg.range,
            )
            .hir,
        ),
        Ok((_, found)) => {
            c.report(Report::new(
                C::TypeMismatch,
                arg.range,
                format!(
                    "`{}` is a {}, not a deadline",
                    path.display(),
                    found.item.kind.noun()
                ),
            ));
            None
        }
        Err(report) => {
            c.report(report);
            None
        }
    }
}

/// Infers a `Dist` argument; a record literal is typed by the unique
/// visible enum whose variants include all its keys.
fn dist_arg(c: &mut Checker<'_, '_>, arg: &Expr) -> Checked {
    let ExprKind::Record(fields) = &arg.kind else {
        return c.infer(arg);
    };
    let keys: Vec<&str> = fields.iter().map(|(n, _)| n.text.as_str()).collect();
    match c.scope.enums_covering(&keys).as_slice() {
        [en] => super::shape::distribution(c, fields, en, arg.range),
        [] => {
            c.report(
                Report::new(
                    C::InvalidDistribution,
                    arg.range,
                    "no visible enum has these variants",
                )
                .note("a `Dist` literal names every variant of one enum"),
            );
            c.error(arg.range)
        }
        many => {
            c.report(Report::new(
                C::TypeMismatch,
                arg.range,
                format!(
                    "ambiguous distribution: enums {} all have these variants",
                    many.join(", ")
                ),
            ));
            c.error(arg.range)
        }
    }
}

fn dist_call(c: &mut Checker<'_, '_>, name: &str, args: &[Expr], r: TextRange) -> Checked {
    let d = dist_arg(c, &args[0]);
    let en = match &d.ty {
        Some(Type::Distribution(en)) => en.clone(),
        Some(other) => {
            c.report(Report::new(
                C::TypeMismatch,
                args[0].range,
                format!("`{name}` needs a `Dist`, found `{}`", display(other)),
            ));
            return c.error(r);
        }
        None => return c.error(r),
    };
    let weights = match &d.value {
        Some(Value::Distribution(w)) => Some(w.clone()),
        _ => None,
    };
    let variant = |v: &Value| match v {
        Value::Variant(q) => q.rsplit("::").next().map(str::to_owned),
        _ => None,
    };
    if name == "prob" {
        let v = c.check(&args[1], &Type::Named(en));
        let value = weights
            .zip(v.value.as_ref().and_then(variant))
            .and_then(|(w, v)| {
                w.into_iter()
                    .find(|(k, _)| *k == v)
                    .map(|(_, p)| Value::Probability(p))
            });
        let ty = v.ty.as_ref().map(|_| Type::Probability);
        return c.node(
            hir::ExprKind::Call {
                callee: name.to_owned(),
                args: vec![d.hir, v.hir],
            },
            ty,
            value,
            r,
        );
    }
    let list = c.check(&args[1], &Type::List(Box::new(Type::Named(en.clone()))));
    let chosen: Option<Vec<String>> = match &list.hir.kind {
        hir::ExprKind::List { items } => items
            .iter()
            .map(|i| match &i.kind {
                hir::ExprKind::Literal { value } => variant(value),
                _ => None,
            })
            .collect(),
        _ => None,
    };
    let mut value = None;
    if let (Some(w), Some(chosen)) = (weights, chosen) {
        let kept: Vec<_> = w.iter().filter(|(k, _)| chosen.contains(k)).collect();
        match rational::sum(kept.iter().map(|(_, p)| *p)) {
            Ok(mass) if mass.numerator == 0 => {
                c.report(Report::new(
                    C::InvalidDistribution,
                    args[1].range,
                    "`given` conditions on a subset with zero probability mass",
                ));
                return c.error(r);
            }
            Ok(mass) => {
                let renorm: Result<Vec<_>, _> = w
                    .iter()
                    .map(|(k, p)| {
                        if chosen.contains(k) {
                            rational::div(*p, mass).map(|q| (k.clone(), q))
                        } else {
                            Ok((k.clone(), hir::Rational::integer(0)))
                        }
                    })
                    .collect();
                match renorm {
                    Ok(v) => value = Some(Value::Distribution(v)),
                    Err(e) => return c.arith(r, e),
                }
            }
            Err(e) => return c.arith(r, e),
        }
    }
    let ok = list.ty.is_some();
    c.node(
        hir::ExprKind::Call {
            callee: name.to_owned(),
            args: vec![d.hir, list.hir],
        },
        ok.then(|| Type::Distribution(en)),
        value,
        r,
    )
}
