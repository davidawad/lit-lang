//! `holes` (the inventory of every `?`) and the `rule-without-test` lint.
//!
//! A deadline is tested when some inline `test` evaluates `deadline(d)`
//! on it; a counting rule is tested when it counts a tested deadline. A
//! test's `cite` alone tests nothing: the assertion has to exercise the
//! rule.
use crate::diag::{emit, end_of, note};
use crate::Context;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, Fix, Profile};
use litigate_sema::hir::{self, Declaration, ExprKind};
use litigate_sema::Hole;
use std::collections::BTreeSet;

/// One diagnostic per hole: L9002 (a warning) in `dev`, L5002 (an error)
/// in `strict`.
pub(crate) fn holes(holes: &[Hole], profile: Profile) -> Vec<Diagnostic> {
    holes
        .iter()
        .map(|h| {
            let reason = h.reason.as_deref().unwrap_or("no reason given");
            match profile {
                Profile::Dev => emit(
                    C::HolePresent,
                    profile,
                    h.span.clone(),
                    format!("hole: {reason}"),
                ),
                Profile::Strict => note(
                    emit(
                        C::HoleInStrict,
                        profile,
                        h.span.clone(),
                        format!("strict does not permit holes: {reason}"),
                    ),
                    None,
                    "replace the `?` with a sourced value",
                ),
            }
        })
        .collect()
}

/// L9001 for every deadline and counting rule no test exercises.
pub(crate) fn untested(cx: &Context<'_>) -> Vec<Diagnostic> {
    let tested = tested_deadlines(cx);
    let counted: BTreeSet<String> = cx
        .declarations()
        .filter_map(|d| match d {
            Declaration::Deadline(x) if tested.contains(&x.meta.id) => {
                Some(cx.local(&x.counting).to_owned())
            }
            _ => None,
        })
        .collect();
    let mut seen = BTreeSet::new();
    cx.declarations()
        .filter_map(|d| match d {
            Declaration::Deadline(x) if !tested.contains(&x.meta.id) => Some(deadline(cx, x)),
            Declaration::Counting(c) if !counted.contains(&c.meta.id) => Some(counting(cx, c)),
            _ => None,
        })
        .filter(|(id, _)| seen.insert(id.clone()))
        .map(|(_, d)| d)
        .collect()
}

/// Local ids of deadlines some test evaluates with `deadline(d)`.
fn tested_deadlines(cx: &Context<'_>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for d in cx.declarations() {
        if let Declaration::Test(t) = d {
            for e in &t.expects {
                deadline_refs(&e.lhs, cx, &mut out);
                deadline_refs(&e.rhs, cx, &mut out);
            }
        }
    }
    out
}

fn deadline_refs(e: &hir::Expr, cx: &Context<'_>, out: &mut BTreeSet<String>) {
    if let ExprKind::Call { callee, args } = &e.kind {
        if let (true, Some(ExprKind::Ref { target })) =
            (callee == "deadline", args.first().map(|a| &a.kind))
        {
            out.insert(cx.local(target).to_owned());
        }
    }
    for child in children(e) {
        deadline_refs(child, cx, out);
    }
}

fn children(e: &hir::Expr) -> Vec<&hir::Expr> {
    match &e.kind {
        ExprKind::Call { args, .. } => args.iter().collect(),
        ExprKind::Unary { operand, .. } => vec![operand],
        ExprKind::Binary { lhs, rhs, .. } => vec![lhs, rhs],
        ExprKind::Record { fields } => fields.iter().map(|(_, v)| v).collect(),
        ExprKind::List { items } => items.iter().collect(),
        ExprKind::Field { base, .. } => vec![base],
        ExprKind::Match { scrutinee, arms } => std::iter::once(&**scrutinee)
            .chain(arms.iter().map(|(_, a)| a))
            .collect(),
        ExprKind::Literal { .. } | ExprKind::Ref { .. } => Vec::new(),
    }
}

fn deadline(cx: &Context<'_>, x: &hir::Deadline) -> (String, Diagnostic) {
    let span = cx.span_or_manifest(x.meta.span.as_ref());
    let id = &x.meta.id;
    let skeleton = format!(
        "\n\ntest \"{id} is due on the expected date\" {{\n  given <trigger event> on \
         <YYYY-MM-DD>\n  expect deadline({id}) == <YYYY-MM-DD>\n}}"
    );
    let mut d = emit(
        C::RuleWithoutTest,
        cx.profile,
        span.clone(),
        format!("deadline `{id}` has no test that evaluates `deadline({id})`"),
    );
    d.fixes.push(Fix {
        span: end_of(&span),
        replacement: skeleton,
        applicability: Applicability::HasPlaceholders,
    });
    (id.clone(), d)
}

fn counting(cx: &Context<'_>, c: &hir::Counting) -> (String, Diagnostic) {
    let id = &c.meta.id;
    let d = emit(
        C::RuleWithoutTest,
        cx.profile,
        cx.span_or_manifest(c.meta.span.as_ref()),
        format!("counting rule `{id}` counts no tested deadline"),
    );
    let d = note(
        d,
        None,
        format!("add a test that evaluates `deadline(d)` for a deadline `counted_by {id}`"),
    );
    (format!("counting:{id}"), d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{BinaryOp, Expr, Pattern, Type, UnaryOp, Value};

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: Type::Unit,
            kind,
            span: None,
        }
    }

    fn leaf(name: &str) -> Expr {
        expr(ExprKind::Ref {
            target: name.to_owned(),
        })
    }

    fn operands(e: &Expr) -> Vec<Expr> {
        children(e).into_iter().cloned().collect()
    }

    #[test]
    fn every_compound_expression_yields_its_operands() {
        let cases = [
            (
                ExprKind::Call {
                    callee: "f".into(),
                    args: vec![leaf("a"), leaf("b")],
                },
                vec!["a", "b"],
            ),
            (
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(leaf("a")),
                },
                vec!["a"],
            ),
            (
                ExprKind::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(leaf("a")),
                    rhs: Box::new(leaf("b")),
                },
                vec!["a", "b"],
            ),
            (
                ExprKind::Record {
                    fields: vec![("x".into(), leaf("a")), ("y".into(), leaf("b"))],
                },
                vec!["a", "b"],
            ),
            (
                ExprKind::List {
                    items: vec![leaf("a")],
                },
                vec!["a"],
            ),
            (
                ExprKind::Field {
                    base: Box::new(leaf("a")),
                    field: "x".into(),
                },
                vec!["a"],
            ),
            (
                ExprKind::Match {
                    scrutinee: Box::new(leaf("s")),
                    arms: vec![
                        (Pattern::Variant("E::x".into()), leaf("a")),
                        (Pattern::Wildcard, leaf("b")),
                    ],
                },
                vec!["s", "a", "b"],
            ),
            (
                ExprKind::Literal {
                    value: Value::Boolean(true),
                },
                vec![],
            ),
            (ExprKind::Ref { target: "a".into() }, vec![]),
        ];
        for (kind, want) in cases {
            let want: Vec<Expr> = want.into_iter().map(leaf).collect();
            assert_eq!(operands(&expr(kind.clone())), want, "{kind:?}");
        }
    }
}
