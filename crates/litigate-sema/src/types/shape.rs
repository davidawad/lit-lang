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
use std::fmt::Write as _;

/// The fields of `deadline(d)` (design review 2026-10-23 §2.1, `Due`).
pub const DUE_FIELDS: [&str; 2] = ["nominal", "adjusted"];

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
    let due = matches!(&b.hir.kind, hir::ExprKind::Call { callee, .. } if callee == "deadline");
    let ty = match &b.ty {
        // `deadline(d)` is a `Due`: bare, its adjusted date; `.nominal` is
        // the last day as counted, before any roll or extension.
        Some(Type::Date) if due => {
            if DUE_FIELDS.contains(&name.text.as_str()) {
                Some(Type::Date)
            } else {
                let mut report = Report::new(
                    C::UnresolvedName,
                    name.range,
                    format!("a due date has no field `{}`", name.text),
                )
                .note("`deadline(d)` has the fields `nominal` and `adjusted`");
                if let Some(best) = suggest::nearest(&name.text, DUE_FIELDS) {
                    report = report.fix(name.range, best, Applicability::MaybeIncorrect);
                }
                c.report(report);
                None
            }
        }
        Some(Type::Record(fields)) => {
            if let Some((_, t)) = fields.iter().find(|(n, _)| *n == name.text) {
                Some(t.clone())
            } else {
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
        }
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

/// The variants a `match` scrutinee ranges over: an enum or `Bool`.
fn domain(c: &mut Checker<'_, '_>, s: &Checked, at: TextRange) -> Option<(String, Vec<String>)> {
    match &s.ty {
        Some(Type::Named(n)) if c.scope.enum_variants(n).is_some() => Some((
            n.clone(),
            c.scope
                .enum_variants(n)
                .map(<[String]>::to_vec)
                .unwrap_or_default(),
        )),
        Some(Type::Boolean) => Some(("Bool".into(), vec!["false".into(), "true".into()])),
        Some(other) => {
            c.report(Report::new(
                C::InvalidOperation,
                at,
                format!(
                    "`match` needs an enum or `Bool`, found `{}`",
                    display(other)
                ),
            ));
            None
        }
        None => None,
    }
}

/// Resolves an arm pattern to a variant (`None` for `_`); `Err` after a
/// report.
fn arm_variant(
    c: &mut Checker<'_, '_>,
    pattern: &Pattern,
    en: &str,
    variants: &[String],
) -> Result<(hir::Pattern, TextRange, Option<String>), ()> {
    let p = match pattern {
        Pattern::Wildcard(range) => return Ok((hir::Pattern::Wildcard, *range, None)),
        Pattern::Ref(p) => p,
    };
    let local = en.rsplit("::").next().unwrap_or(en);
    // `Variant` or `Enum.Variant`; any other path shape names no variant.
    let qualified =
        p.rest.len() == 1 && p.head.text == local && p.instance.is_none() && p.version.is_none();
    let name = if qualified { &p.rest[0].1 } else { &p.head };
    if !(p.is_simple() || qualified) || !variants.contains(&name.text) {
        let mut report = Report::new(
            C::UnresolvedName,
            p.range,
            format!("`{en}` has no variant `{}`", p.display()),
        );
        if let Some(best) = suggest::nearest(&name.text, variants.iter().map(String::as_str)) {
            report = report.fix(name.range, best, Applicability::MaybeIncorrect);
        }
        c.report(report);
        return Err(());
    }
    Ok((
        hir::Pattern::Variant(format!("{en}::{}", name.text)),
        p.range,
        Some(name.text.clone()),
    ))
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
    let Some((en, variants)) = domain(c, &s, scrutinee.range) else {
        return c.error(r);
    };
    let scrutinee_variant = match &s.value {
        Some(Value::Variant(v)) => v.rsplit("::").next().map(str::to_owned),
        Some(Value::Boolean(b)) => Some(b.to_string()),
        _ => None,
    };
    let (mut covered, mut wildcard, mut ok) = (Vec::<String>::new(), false, true);
    let mut result = expected.cloned();
    let (mut out, mut chosen) = (Vec::new(), None);
    for (pattern, body) in arms {
        let Ok((pat, range, variant)) = arm_variant(c, pattern, &en, &variants) else {
            ok = false;
            continue;
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
        if chosen.is_none()
            && scrutinee_variant
                .as_ref()
                .is_some_and(|sv| variant.as_ref().is_none_or(|v| v == sv))
        {
            chosen = Some(b.value.clone());
        }
        out.push((pat, b.hir));
    }
    let missing: Vec<&String> = variants.iter().filter(|v| !covered.contains(v)).collect();
    if !wildcard && !missing.is_empty() {
        let arms = missing.iter().fold(String::new(), |mut s, v| {
            let _ = write!(s, ", {v} => ?");
            s
        });
        let at = r.at_last();
        let names = missing
            .iter()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<_>>()
            .join(", ");
        c.report(
            Report::new(
                C::NonExhaustiveMatch,
                r,
                format!("non-exhaustive match: missing {names}"),
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

/// Checks the weights' exact sum: 1 when complete, at most 1 with holes.
fn check_sum(
    c: &mut Checker<'_, '_>,
    weights: &[(String, Option<Rational>)],
    r: TextRange,
) -> bool {
    let holes = weights.iter().any(|(_, w)| w.is_none());
    let message = match q::sum(weights.iter().filter_map(|(_, w)| *w)) {
        Ok(total) if !holes && total != Rational::integer(1) => {
            format!("weights sum to {}, not exactly 1", q::display(total))
        }
        Ok(total) if holes && q::cmp(total, Rational::integer(1)).is_gt() => format!(
            "known weights already sum to {}, more than 1",
            q::display(total)
        ),
        Ok(_) => return true,
        Err(e) => {
            c.report(Report::new(C::ArithmeticError, r, e.message()));
            return false;
        }
    };
    c.report(Report::new(C::InvalidDistribution, r, message));
    false
}

/// Every variant has a weight (L2003 otherwise).
fn all_variants(
    c: &mut Checker<'_, '_>,
    en: &str,
    variants: &[String],
    weights: &[(String, Option<Rational>)],
    r: TextRange,
) -> bool {
    let missing: Vec<String> = variants
        .iter()
        .filter(|v| !weights.iter().any(|(n, _)| n == *v))
        .map(|v| format!("`{v}`"))
        .collect();
    if !missing.is_empty() {
        c.report(
            Report::new(
                C::InvalidDistribution,
                r,
                format!("`Dist<{en}>` is missing {}", missing.join(", ")),
            )
            .note("give every variant a weight, `0` included"),
        );
    }
    missing.is_empty()
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
    ok &= all_variants(c, en, &variants, &weights, r);
    ok = ok && check_sum(c, &weights, r);
    let complete = ok && weights.iter().all(|(_, w)| w.is_some());
    if complete {
        let ordered = variants
            .iter()
            .filter_map(|v| {
                weights
                    .iter()
                    .find(|(n, _)| n == v)
                    .and_then(|(n, w)| w.map(|w| (n.clone(), w)))
            })
            .collect();
        return c.literal(
            Value::Distribution(ordered),
            Type::Distribution(en.to_owned()),
            r,
        );
    }
    hir_fields.sort_by(|a, b| a.0.cmp(&b.0));
    c.node(
        hir::ExprKind::Record { fields: hir_fields },
        ok.then(|| Type::Distribution(en.to_owned())),
        None,
        r,
    )
}
