//! Extension refinements within their delegation bounds (SPEC §3.5;
//! design review §2.2 tier 3, §2.4). An event that `extends d to p` and
//! refines `p` claims every extension it can record lies within `d`'s
//! extension bounds. The SMT tier checks the claim for every matter at
//! once: the event's refinements, with the bound dates open, together
//! with `p` past every `extend { .. }` ceiling of each version of `d`. A
//! model is a counterexample (L2013); `unknown` is a hole (L2015).
//!
//! A version with a discretionary grant (no ceiling) bounds nothing, and
//! a version with no bounded grant has no bound to stay within: neither
//! is checked. Replay still refuses each extension past its bound.
use crate::diag::emit;
use crate::Context;
use litigate_base::{Diagnostic, DiagnosticCode as C, Span};
use litigate_sema::hir::{
    self, BinaryOp, Bound, DayUnit, Declaration, EventEffect, EventType, ExprKind, Extension, Type,
    Value,
};
use litigate_sema::types::refinement::{self as refine, smt};

/// L2013 (or L2015) for each refined `extends` event of the package whose
/// refinements admit an extension past its deadline's bounds.
pub(crate) fn bounds(cx: &Context<'_>) -> Vec<Diagnostic> {
    cx.declarations()
        .filter_map(|d| match d {
            Declaration::Event(e) => Some(e),
            _ => None,
        })
        .filter_map(|e| check(cx, e))
        .collect()
}

fn check(cx: &Context<'_>, e: &EventType) -> Option<Diagnostic> {
    let Some(EventEffect::Extends { deadline, param }) = &e.effect else {
        return None;
    };
    let (to, _) = &e.params[*param];
    // Only a refinement of the new due date claims a bound.
    let claim = e.refinements.iter().find(|r| {
        let mut names = false;
        refine::visit_refs(&r.predicate, &mut |n| names |= n == to);
        names
    })?;
    let span = cx.span_or_manifest(claim.predicate.span.as_ref());
    for version in cx.deadline_versions(deadline) {
        let ceilings: Option<Vec<&Bound>> = version
            .extend
            .iter()
            .filter(|x| x.lock().is_none())
            .map(|x| match x {
                Extension::Bounded { until, .. } => Some(until),
                _ => None,
            })
            .collect();
        let Some(ceilings) = ceilings.filter(|c| !c.is_empty()) else {
            continue;
        };
        if let Some(d) = exceeds(cx, e, to, deadline, &ceilings, &span) {
            return Some(d);
        }
    }
    None
}

/// The question for one version: `e`'s refinements, the instance's dates
/// ordered (`current_due >= prescribed_due`), and `to` past every ceiling.
fn exceeds(
    cx: &Context<'_>,
    e: &EventType,
    to: &str,
    deadline: &str,
    ceilings: &[&Bound],
    span: &Span,
) -> Option<Diagnostic> {
    let mut vars: Vec<(String, Type)> = e.params.clone();
    vars.extend(
        refine::bound_dates()
            .into_iter()
            .filter(|(d, _)| e.params.iter().all(|(p, _)| p != d)),
    );
    let variants: Vec<Option<&[String]>> = vars
        .iter()
        .map(|(_, t)| match t {
            Type::Named(en) => enum_variants(cx, en),
            _ => None,
        })
        .collect();
    let smt_vars: Vec<smt::Var<'_>> = vars
        .iter()
        .zip(&variants)
        .map(|((name, ty), variants)| smt::Var {
            name,
            ty,
            variants: *variants,
        })
        .collect();
    let mut preds: Vec<hir::Expr> = e.refinements.iter().map(|r| r.predicate.clone()).collect();
    preds.push(cmp(
        BinaryOp::Ge,
        date("current_due"),
        date("prescribed_due"),
    ));
    let within: Vec<hir::Expr> = ceilings.iter().map(|b| below(to, b)).collect();
    preds.push(not(join(BinaryOp::Or, false, within)));
    match smt::decide(&smt_vars, &preds) {
        smt::Decision::Unsat(_) => None,
        smt::Decision::Sat(model) => Some(counterexample(cx, e, to, deadline, ceilings, span, &model)),
        smt::Decision::Unknown(why) => Some(
            emit(
                C::RefinementUndecided,
                cx.profile,
                span.clone(),
                format!(
                    "the SMT tier did not decide whether `{}` stays within the bound of `{deadline}`: {why}",
                    e.name
                ),
            )
            .note("an undecided refinement is a hole, never a pass"),
        ),
    }
}

/// L2013 for model `model` of the question: the values the claim reads,
/// `to`'s, and each ceiling's date there.
fn counterexample(
    cx: &Context<'_>,
    e: &EventType,
    to: &str,
    deadline: &str,
    ceilings: &[&Bound],
    span: &Span,
    model: &[(String, String)],
) -> Diagnostic {
    let value = |n: &str| model.iter().find(|(m, _)| m == n).map(|(_, v)| v.as_str());
    let dates = hir::BoundDates {
        prescribed_due: value("prescribed_due").unwrap_or_default(),
        current_due: value("current_due").unwrap_or_default(),
        grant_order_entered: value("grant_order_entered").unwrap_or_default(),
    };
    let given: Vec<String> = model
        .iter()
        .filter(|(n, _)| n != to && (refine_reads(e, n) || is_bound_date(n)))
        .map(|(n, v)| format!("{n} = {v}"))
        .collect();
    let past: Vec<String> = ceilings
        .iter()
        .map(|b| format!("{} = {}", b.display(), b.date(dates).unwrap_or_default()))
        .collect();
    emit(
                    C::RefinementViolated,
                    cx.profile,
                    span.clone(),
                    format!(
                        "`{}` admits an extension of `{deadline}` past its bound: when {}, {to} = {} exceeds {}",
                        e.name,
                        given.join(", "),
                        value(to).unwrap_or_default(),
                        past.join(" and ")
                    ),
                )
                .note(format!(
                    "the refinements of `{}` claim every extension it records is within `{deadline}`'s `extend` bound; tighten them to entail `{to} <= {}`",
                    e.name, ceilings[0].display()
                ))
                .note("replay refuses an extension past its bound (L4001), so a refinement admitting one is a declaration error")
}

/// Whether `name` is a bound date.
fn is_bound_date(name: &str) -> bool {
    hir::DateRef::ALL.iter().any(|d| d.name() == name)
}

/// Whether a refinement of `e` reads `name` (a counterexample shows the
/// values the claim depends on).
fn refine_reads(e: &EventType, name: &str) -> bool {
    e.refinements.iter().any(|r| {
        let mut found = false;
        refine::visit_refs(&r.predicate, &mut |n| found |= n == name);
        found
    })
}

/// The variants of enum `name` (as the package spells it).
fn enum_variants<'a>(cx: &Context<'a>, name: &str) -> Option<&'a [String]> {
    let (pkg, local) = cx.owner(name);
    pkg.modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Enum(x) if x.name == local => Some(x.variants.as_slice()),
            _ => None,
        })
}

fn expr(ty: Type, kind: ExprKind) -> hir::Expr {
    hir::Expr {
        ty,
        kind,
        span: None,
    }
}

fn date(name: &str) -> hir::Expr {
    expr(
        Type::Date,
        ExprKind::Ref {
            target: name.to_owned(),
        },
    )
}

fn days(n: u64) -> hir::Expr {
    expr(
        Type::Days(DayUnit::Calendar),
        ExprKind::Literal {
            value: Value::Days {
                amount: i64::try_from(n).unwrap_or(i64::MAX),
                unit: DayUnit::Calendar,
            },
        },
    )
}

fn binary(ty: Type, op: BinaryOp, lhs: hir::Expr, rhs: hir::Expr) -> hir::Expr {
    expr(
        ty,
        ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    )
}

fn cmp(op: BinaryOp, lhs: hir::Expr, rhs: hir::Expr) -> hir::Expr {
    binary(Type::Boolean, op, lhs, rhs)
}

fn not(e: hir::Expr) -> hir::Expr {
    expr(
        Type::Boolean,
        ExprKind::Unary {
            op: hir::UnaryOp::Not,
            operand: Box::new(e),
        },
    )
}

/// `es` joined by `op` onto its unit `unit` (`false ||`, `true &&`).
fn join(op: BinaryOp, unit: bool, es: impl IntoIterator<Item = hir::Expr>) -> hir::Expr {
    let unit = expr(
        Type::Boolean,
        ExprKind::Literal {
            value: Value::Boolean(unit),
        },
    );
    es.into_iter().fold(unit, |a, b| cmp(op, a, b))
}

/// `to <= b`: `max` is a disjunction, `min` a conjunction.
fn below(to: &str, b: &Bound) -> hir::Expr {
    match b {
        Bound::Offset { from, days: n } => cmp(
            BinaryOp::Le,
            date(to),
            binary(Type::Date, BinaryOp::Add, date(from.name()), days(*n)),
        ),
        Bound::Before { from, days: n } => cmp(
            BinaryOp::Le,
            date(to),
            binary(Type::Date, BinaryOp::Sub, date(from.name()), days(*n)),
        ),
        Bound::Max(xs) => join(BinaryOp::Or, false, xs.iter().map(|x| below(to, x))),
        Bound::Min(xs) => join(BinaryOp::And, true, xs.iter().map(|x| below(to, x))),
    }
}
