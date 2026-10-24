//! `beliefs { prior node: { edge: α } }` (SPEC §5.6, L4006; design review
//! 2026-10-23 §2.11): the Dirichlet prior a matter's observations update,
//! declared instead of mutating an authored point probability. It names a
//! world draw, as `observe` does, and gives every outcome of it a positive
//! pseudo-count, once; with two outcomes it is `Beta(α, β)`. Its strength
//! `Σα` is the node's concentration, so `analysis.uncertainty` may not set
//! that node's concentration too.
use super::graph::Graph;
use super::spans::{error, CaseSpans};
use crate::hir::{Case, Node};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use std::collections::BTreeSet;

/// The local part of a qualified id (`pack::edge` is `edge`).
fn local(id: &str) -> &str {
    id.rsplit("::").next().unwrap_or(id)
}

/// The outcomes a prior over `n` must weigh: every edge but a self edge,
/// by id (an anonymous edge by its target).
fn outcomes(n: &Node) -> Vec<String> {
    n.edges
        .iter()
        .filter(|e| e.to != n.meta.id)
        .map(|e| e.id.clone().unwrap_or_else(|| format!("_ -> {}", e.to)))
        .collect()
}

/// Whether `analysis.uncertainty.concentration` names `node`.
fn concentrated(case: &Case, node: &str) -> bool {
    case.analysis
        .options
        .get("uncertainty")
        .and_then(|o| o.value.get("concentration"))
        .and_then(serde_json::Value::as_object)
        .is_some_and(|m| m.keys().any(|k| k == node || k == local(node)))
}

/// Checks every `prior` of `case` (L4006).
pub(super) fn priors(case: &Case, graph: &Graph<'_>, spans: &CaseSpans, out: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    for (i, prior) in case.priors.iter().enumerate() {
        let (node_span, outcome_spans) = spans.priors.get(i).cloned().unwrap_or_default();
        let at = |j: Option<usize>| {
            spans.at(
                j.and_then(|j| outcome_spans.get(j)).or(node_span.as_ref()),
                &case.id,
            )
        };
        let mut fail = |at: litigate_base::Span, message: String| {
            out.push(error(C::InvalidBelief, at, message));
        };
        if !seen.insert(prior.node.as_str()) {
            fail(
                at(None),
                format!("`{}` has more than one prior", prior.node),
            );
            continue;
        }
        if concentrated(case, &prior.node) {
            fail(
                at(None),
                format!(
                    "`{}`'s strength is its prior's total; drop it from \
                     `analysis.uncertainty.concentration`",
                    prior.node
                ),
            );
        }
        let node = graph
            .node_at(&prior.node, &case.as_of, case.known_at.as_deref())
            .filter(|n| super::drawn::is_drawn(case, graph, &prior.node, n));
        let Some(node) = node else {
            fail(
                at(None),
                format!(
                    "`{}` is not a chance node in force on {}",
                    prior.node, case.as_of
                ),
            );
            continue;
        };
        if crate::types::rational::sum(prior.pseudo_counts.iter().map(|(_, a)| *a)).is_err() {
            fail(
                at(None),
                format!(
                    "the pseudo-counts of `{}` overflow exact arithmetic",
                    prior.node
                ),
            );
        }
        let wanted = outcomes(node);
        let mut given = BTreeSet::new();
        for (j, (edge, _)) in prior.pseudo_counts.iter().enumerate() {
            let edge = local(edge);
            if !wanted.iter().any(|w| w == edge) {
                fail(
                    at(Some(j)),
                    format!("`{edge}` is not an outcome of `{}`", prior.node),
                );
            } else if !given.insert(edge) {
                fail(at(Some(j)), format!("`{edge}` is given twice"));
            }
        }
        let missing: Vec<String> = wanted
            .iter()
            .filter(|w| !given.contains(w.as_str()))
            .map(|w| format!("`{w}`"))
            .collect();
        if !missing.is_empty() {
            fail(
                at(None),
                format!(
                    "the prior on `{}` gives no pseudo-count to {}",
                    prior.node,
                    missing.join(", ")
                ),
            );
        }
    }
}
