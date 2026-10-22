//! Graph obligations over a rule package's nodes in force (SPEC §7):
//! `graph-exits` (L5004), `graph-reachable` (L5003), anonymous edges
//! (L9003), and `deadline-satisfiable` (L5005).
use crate::hirx;
use litigate_base::{Diagnostic, DiagnosticCode as C, Severity, Span};
use litigate_sema::hir::{self, DeadlineWindow, Declaration, NodeKind, RuleKind};
use std::collections::{BTreeMap, BTreeSet};

fn at(span: Option<&Span>, fallback: &Span) -> Span {
    span.cloned().unwrap_or_else(|| fallback.clone())
}

/// L9003 for every `_` edge (every version).
#[must_use]
pub fn anonymous_edges(pkg: &hir::Package, fallback: &Span) -> Vec<Diagnostic> {
    hirx::nodes_at(pkg, None)
        .into_iter()
        .flat_map(|n| n.edges.iter().map(move |e| (n, e)))
        .filter(|(_, e)| e.id.is_none())
        .map(|(n, e)| Diagnostic {
            severity: Severity::Info,
            ..Diagnostic::error(
                C::AnonymousEdge,
                at(e.span.as_ref(), fallback),
                format!(
                    "the edge `{} -> {}` has no id, so cases and links cannot name it",
                    n.meta.id,
                    hirx::local(&e.to)
                ),
            )
        })
        .collect()
}

/// L5004 for non-terminals without an out-edge, L5003 for nodes neither
/// reachable from `start` nor from an `entry`-tagged node, on `date`.
#[must_use]
pub fn shape(pkg: &hir::Package, date: &str, fallback: &Span) -> Vec<Diagnostic> {
    let nodes = hirx::nodes_at(pkg, Some(date));
    let by_id: BTreeMap<&str, &hir::Node> =
        nodes.iter().map(|n| (n.meta.id.as_str(), *n)).collect();
    let mut out: Vec<Diagnostic> = nodes
        .iter()
        .filter(|n| !matches!(n.kind, NodeKind::Terminal { .. }) && n.edges.is_empty())
        .map(|n| {
            Diagnostic::error(
                C::NoExit,
                at(n.meta.span.as_ref(), fallback),
                format!("`{}` is not terminal but has no outgoing edge", n.meta.id),
            )
        })
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = std::iter::once(pkg.start.as_str())
        .chain(
            nodes
                .iter()
                .filter(|n| n.tags.iter().any(|t| t == "entry"))
                .map(|n| n.meta.id.as_str()),
        )
        .collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(n) = by_id.get(id) {
            stack.extend(
                n.edges
                    .iter()
                    .filter(|e| !e.to.contains("::"))
                    .map(|e| e.to.as_str()),
            );
        }
    }
    out.extend(
        nodes
            .iter()
            .filter(|n| !seen.contains(n.meta.id.as_str()))
            .map(|n| Diagnostic {
                notes: vec![litigate_base::Note {
                    span: None,
                    message: "tag it `entry` if another package's link enters here".to_owned(),
                }],
                ..Diagnostic::error(
                    C::UnreachableNode,
                    at(n.meta.span.as_ref(), fallback),
                    format!(
                        "`{}` is not reachable from `start {}`",
                        n.meta.id, pkg.start
                    ),
                )
            }),
    );
    out
}

/// Deadline ids that belong to `duty` rules.
fn duties(pkg: &hir::Package) -> BTreeSet<String> {
    hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Rule(r) if r.rule_kind == RuleKind::Duty => Some(r.declarations.iter()),
            _ => None,
        })
        .flatten()
        .map(|d| hirx::local(d).to_owned())
        .collect()
}

/// L5005: a node whose out-edges carry mandatory deadlines with the same
/// trigger counted in opposite directions. A forward window `[T, T+a]` and
/// a backward one `(.., T-b]` from one trigger `T` never intersect.
#[must_use]
pub fn satisfiable(pkg: &hir::Package, date: &str, fallback: &Span) -> Vec<Diagnostic> {
    let duty = duties(pkg);
    hirx::nodes_at(pkg, Some(date))
        .into_iter()
        .filter_map(|n| {
            let windows: Vec<&hir::Deadline> = n
                .edges
                .iter()
                .filter_map(|e| match &e.deadline.as_ref()?.window {
                    DeadlineWindow::Declared { deadline } if duty.contains(hirx::local(deadline)) => {
                        hirx::deadline_at(pkg, deadline, date).ok()
                    }
                    _ => None,
                })
                .collect();
            let clash = windows.iter().find_map(|a| {
                windows.iter().find(|b| {
                    a.direction != b.direction
                        && hirx::event_parts(&a.trigger) == hirx::event_parts(&b.trigger)
                })
                .map(|b| (*a, *b))
            })?;
            Some(Diagnostic::error(
                C::DeadlineUnsatisfiable,
                at(n.meta.span.as_ref(), fallback),
                format!(
                    "`{}` must meet `{}` and `{}`, counted in opposite directions from {}; no date satisfies both",
                    n.meta.id,
                    clash.0.meta.id,
                    clash.1.meta.id,
                    hirx::event_text(&clash.0.trigger)
                ),
            ))
        })
        .collect()
}
