//! Records, lists, projections, exhaustive `match`, and exact `Dist`
//! literals (SPEC §5.2, §6).
use super::expr::{Checked, Checker};
use super::rational as q;
use super::ty::display;
use crate::db::parse::tree::{Expr, Name, Pattern};
use crate::db::span::{Report, TextRange};
use crate::hir::{self, Rational, Type, Value};
use crate::resolve::suggest;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeSet;

/// `{ f: e, ... }` as a record value.
pub fn record(c: &mut Checker<'_, '_>, fields: &[(Name, Expr)], r: TextRange) -> Checked {
    let mut seen = BTreeSet::new();
    let mut out: Vec<(String, Checked)> = Vec::new();
    for (name, e) in fields {
        if !seen.insert(name.text.clone()) {
            c.report(Report::new(
                C::InvalidField,
                name.range,
                format!("field `{}` is given twice", name.text),
            ));
            continue;
        }
        out.push((name.text.clone(), c.infer(e)));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    let ty: Option<Vec<(String, Type)>> = out
        .iter()
        .map(|(n, ch)| ch.ty.clone().map(|t| (n.clone(), t)))
        .collect();
    let hir_fields = out.into_iter().map(|(n, ch)| (n, ch.hir)).collect();
    c.node(
        hir::ExprKind::Record { fields: hir_fields },
        ty.map(Type::Record),
        None,
        r,
    )
}

/// `[e, ...]`, checked against `expected` element type when known.
pub fn list(
    c: &mut Checker<'_, '_>,
    items: &[Expr],
    expected: Option<&Type>,
    r: TextRange,
) -> Checked {
    let mut elem = expected.cloned();
    let mut ok = true;
    let mut hir_items = Vec::new();
    for item in items {
        let ch = match &elem {
            Some(t) => c.check(item, t),
            None => c.infer(item),
        };
        ok &= ch.ty.is_some();
        if elem.is_none() {
            elem.clone_from(&ch.ty);
        }
        hir_items.push(ch.hir);
    }
    if elem.is_none() && items.is_empty() {
        c.report(Report::new(
            C::TypeMismatch,
            r,
            "cannot infer the element type of an empty list here",
        ));
    }
    let ty = elem.filter(|_| ok).map(|t| Type::List(Box::new(t)));
    c.node(hir::ExprKind::List { items: hir_items }, ty, None, r)
}

/// `base.field` on a record.
pub fn field(c: &mut Checker<'_, '_>, base: &Expr, name: &Name, r: TextRange) -> Checked {
    let b = c.infer(base);
    let ty = match &b.ty {
        Some(Type::Record(fields)) => match fields.iter().find(|(n, _)| *n == name.text) {
            Some((_, t)) => Some(t.clone()),
            None => {
                let best = suggest::nearest(&name.text, fields.iter().map(|(n, _)| n.as_str()));
                let mut report = Report::new(
                    C::UnresolvedName,
                    name.range,
                    format!("record has no field `{}`", name.text),
                );
                if let Some(best) = best {
                    report = report.fix(name.range, best, Applicability::MaybeIncorrect);
                }
                c.report(report);
                None
            }
        },
        Some(other) => {
            c.report(Report::new(
                C::InvalidOperation,
                r,
                format!("`{}` has no fields", display(other)),
            ));
            None
        }
        None => None,
    };
    c.node(
        hir::ExprKind::Field {
            base: Box::new(b.hir),
            field: name.text.clone(),
        },
        ty,
        None,
        r,
    )
}

/// Exhaustive `match` with no unreachable arms (L2005).
pub fn matching(
    c: &mut Checker<'_, '_>,
    scrutinee: &Expr,
    arms: &[(Pattern, Expr)],
    expected: Option<&Type>,
    r: TextRange,
) -> Checked {
    let s = c.infer(scrutinee);
    let (en, variants): (String, Vec<String>) = match &s.ty {
        Some(Type::Named(n)) if c.scope.enum_variants(n).is_some() => (
            n.clone(),
            c.scope
                .enum_variants(n)
                .map(<[String]>::to_vec)
                .unwrap_or_default(),
        ),
        Some(Type::Boolean) => ("Bool".into(), vec!["false".into(), "true".into()]),
        Some(other) => {
            c.report(Report::new(
                C::InvalidOperation,
                scrutinee.range,
                format!(
                    "`match` needs an enum or `Bool`, found `{}`",
                    display(other)
                ),
            ));
            return c.error(r);
        }
        None => return c.error(r),
    };
    let mut covered: Vec<String> = Vec::new();
    let mut wildcard = false;
    let mut result = expected.cloned();
    let mut ok = true;
    let mut out = Vec::new();
    let mut chosen = None;
    let scrutinee_variant = match &s.value {
        Some(Value::Variant(v)) => v.rsplit("::").next().map(str::to_owned),
        Some(Value::Boolean(b)) => Some(b.to_string()),
        _ => None,
    };
    for (pattern, body) in arms {
        let (pat, range, variant) = match pattern {
            Pattern::Wildcard(range) => (hir::Pattern::Wildcard, *range, None),
            Pattern::Ref(p) => {
                let name =
                    if p.rest.len() == 1 && p.head.text == en.rsplit("::").next().unwrap_or(&en) {
                        &p.rest[0].1
                    } else {
                        &p.head
                    };
                if !variants.contains(&name.text) || (!p.is_simple() && p.rest.len() != 1) {
                    let mut report = Report::new(
                        C::UnresolvedName,
                        p.range,
                        format!("`{en}` has no variant `{}`", p.display()),
                    );
                    if let Some(best) =
                        suggest::nearest(&name.text, variants.iter().map(String::as_str))
                    {
                        report = report.fix(name.range, best, Applicability::MaybeIncorrect);
                    }
                    c.report(report);
                    ok = false;
                    continue;
                }
                (
                    hir::Pattern::Variant(format!("{en}::{}", name.text)),
                    p.range,
                    Some(name.text.clone()),
                )
            }
        };
        let exhausted = wildcard || variants.iter().all(|v| covered.contains(v));
        if exhausted || variant.as_ref().is_some_and(|v| covered.contains(v)) {
            c.report(
                Report::new(C::NonExhaustiveMatch, range, "unreachable match arm")
                    .note("an earlier arm already covers it"),
            );
            ok = false;
        }
        match &variant {
            Some(v) => covered.push(v.clone()),
            None => wildcard = true,
        }
        let b = match &result {
            Some(t) => c.check(body, t),
            None => c.infer(body),
        };
        if result.is_none() {
            result.clone_from(&b.ty);
        }
        ok &= b.ty.is_some();
        let hits = scrutinee_variant
            .as_ref()
            .is_some_and(|sv| variant.as_ref().is_none_or(|v| v == sv));
        if chosen.is_none() && hits {
            chosen = Some(b.value.clone());
        }
        out.push((pat, b.hir));
    }
    let missing: Vec<&String> = variants.iter().filter(|v| !covered.contains(v)).collect();
    if !wildcard && !missing.is_empty() {
        let arms: String = missing.iter().map(|v| format!(", {v} => ?")).collect();
        let at = TextRange::new(r.end.saturating_sub(1), r.end.saturating_sub(1));
        c.report(
            Report::new(
                C::NonExhaustiveMatch,
                r,
                format!(
                    "non-exhaustive match: missing {}",
                    missing
                        .iter()
                        .map(|v| format!("`{v}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
            .fix(at, arms, Applicability::HasPlaceholders),
        );
        ok = false;
    }
    let ty = result.filter(|_| ok);
    c.node(
        hir::ExprKind::Match {
            scrutinee: Box::new(s.hir),
            arms: out,
        },
        ty,
        chosen.flatten(),
        r,
    )
}

/// A record checked as `Dist<en>`: every variant exactly once, constant
/// probabilities summing to exactly 1 (holes leave the sum open, ≤ 1).
pub fn distribution(
    c: &mut Checker<'_, '_>,
    fields: &[(Name, Expr)],
    en: &str,
    r: TextRange,
) -> Checked {
    let variants = c
        .scope
        .enum_variants(en)
        .map(<[String]>::to_vec)
        .unwrap_or_default();
    let mut weights: Vec<(String, Option<Rational>)> = Vec::new();
    let mut hir_fields = Vec::new();
    let mut ok = true;
    for (name, e) in fields {
        if !variants.contains(&name.text) || weights.iter().any(|(n, _)| *n == name.text) {
            let message = if variants.contains(&name.text) {
                format!("variant `{}` appears twice", name.text)
            } else {
                format!("`{en}` has no variant `{}`", name.text)
            };
            c.report(Report::new(C::InvalidDistribution, name.range, message));
            ok = false;
            continue;
        }
        let ch = c.check(e, &Type::Probability);
        let w = match &ch.value {
            Some(Value::Probability(p)) => Some(*p),
            Some(Value::Hole(_)) => None,
            _ => {
                if ch.ty.is_some() {
                    c.report(Report::new(
                        C::InvalidDistribution,
                        e.range,
                        "distribution weights must be constant",
                    ));
                }
                ok = false;
                None
            }
        };
        weights.push((name.text.clone(), w));
        hir_fields.push((name.text.clone(), ch.hir));
    }
    let missing: Vec<&String> = variants
        .iter()
        .filter(|v| !weights.iter().any(|(n, _)| n == *v))
        .collect();
    if !missing.is_empty() {
        let list = missing
            .iter()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<_>>()
            .join(", ");
        c.report(
            Report::new(
                C::InvalidDistribution,
                r,
                format!("`Dist<{en}>` is missing {list}"),
            )
            .note("give every variant a weight, `0` included"),
        );
        ok = false;
    }
    let known: Vec<Rational> = weights.iter().filter_map(|(_, w)| *w).collect();
    let holes = weights.iter().any(|(_, w)| w.is_none());
    match q::sum(known) {
        Ok(total) if ok && !holes && total != Rational::integer(1) => {
            c.report(Report::new(
                C::InvalidDistribution,
                r,
                format!("weights sum to {}, not exactly 1", q::display(total)),
            ));
            ok = false;
        }
        Ok(total) if ok && holes && q::cmp(total, Rational::integer(1)).is_gt() => {
            c.report(Report::new(
                C::InvalidDistribution,
                r,
                format!(
                    "known weights already sum to {}, more than 1",
                    q::display(total)
                ),
            ));
            ok = false;
        }
        Ok(_) => {}
        Err(e) => return c.arith(r, e),
    }
    hir_fields.sort_by(|a, b| a.0.cmp(&b.0));
    let value = (ok && !holes).then(|| {
        let ordered = variants
            .iter()
            .filter_map(|v| {
                weights
                    .iter()
                    .find(|(n, _)| n == v)
                    .and_then(|(n, w)| w.map(|w| (n.clone(), w)))
            })
            .collect();
        Value::Distribution(ordered)
    });
    let ty = ok.then(|| Type::Distribution(en.to_owned()));
    match value {
        Some(v) => c.literal(v, Type::Distribution(en.to_owned()), r),
        None => c.node(hir::ExprKind::Record { fields: hir_fields }, ty, None, r),
    }
}
