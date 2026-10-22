//! Graph obligations on the lowered pack: `graph-exits`, `graph-reachable`,
//! and the structural half of `dist-exact`.
//!
//! Reachability runs litgraph's own `algo::structure::reachable` over the
//! compiled graph, so it sees exactly the edges the engine does; a pack
//! that does not compile alone (an edge into another pack) falls back to a
//! walk over its raw edges.
use crate::diag::{emit, note};
use crate::Context;
use litgraph::algo::structure;
use litgraph::model::{CompileOptions, Graph, LinkFile, NodeKind, Pack};
use litgraph::scenario::{Scenario, View};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_sema::hir;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The `entry` tag: an extra root for reachability (SPEC §5.1).
const ENTRY: &str = "entry";

/// L5004 for every non-terminal node with no outgoing edge. `requires` and
/// `forbids` are ignored: any authored edge is an exit.
pub(crate) fn exits(cx: &Context<'_>, pack: &Pack) -> Vec<Diagnostic> {
    let sources: BTreeSet<&str> = pack.edges.iter().map(|e| e.from.as_str()).collect();
    pack.nodes
        .iter()
        .filter(|n| n.kind.unwrap_or(NodeKind::State) != NodeKind::Terminal)
        .filter(|n| !sources.contains(n.id.as_str()))
        .map(|n| {
            let d = emit(
                C::NoExit,
                cx.profile,
                cx.node_span(&n.id),
                format!("`{}` is not a terminal and has no outgoing edge", n.id),
            );
            note(
                d,
                None,
                "add an edge out of it, or declare it `terminal` if the matter ends there",
            )
        })
        .collect()
}

/// Node ids reachable from the start or any `entry` node.
pub(crate) fn reached(pack: &Pack) -> BTreeSet<String> {
    let roots: Vec<&str> = std::iter::once(pack.start_node_id.as_str())
        .chain(
            pack.nodes
                .iter()
                .filter(|n| n.tags.iter().any(|t| t == ENTRY))
                .map(|n| n.id.as_str()),
        )
        .collect();
    compiled_reach(pack, &roots).unwrap_or_else(|| raw_reach(pack, &roots))
}

/// L5003 for every node outside `reached`.
pub(crate) fn unreachable(
    cx: &Context<'_>,
    pack: &Pack,
    reached: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    pack.nodes
        .iter()
        .filter(|n| !reached.contains(&n.id))
        .map(|n| {
            let d = emit(
                C::UnreachableNode,
                cx.profile,
                cx.node_span(&n.id),
                format!(
                    "`{}` is not reachable from start `{}` or any `entry` node",
                    n.id, pack.start_node_id
                ),
            );
            note(
                d,
                None,
                "route an edge to it, tag it `entry` if a matter can begin there, or remove it",
            )
        })
        .collect()
}

fn compiled_reach(pack: &Pack, roots: &[&str]) -> Option<BTreeSet<String>> {
    let options = CompileOptions {
        no_flags: true,
        ..CompileOptions::default()
    };
    let graph = Graph::compile(std::slice::from_ref(pack), &LinkFile::default(), &options).ok()?;
    let view = View::new(&graph, &Scenario::default()).ok()?;
    let qualify = |id: &str| format!("{}::{id}", pack.id);
    let mut seen = vec![false; graph.nodes.len()];
    for root in roots {
        let ix = graph.node(&qualify(root)).ok()?;
        for (s, r) in seen.iter_mut().zip(structure::reachable(&view, ix)) {
            *s |= r;
        }
    }
    let mut out = BTreeSet::new();
    for n in &pack.nodes {
        let ix = graph.node(&qualify(&n.id)).ok()?;
        if seen.get(ix).copied().unwrap_or(false) {
            out.insert(n.id.clone());
        }
    }
    Some(out)
}

fn raw_reach(pack: &Pack, roots: &[&str]) -> BTreeSet<String> {
    let mut out: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in &pack.edges {
        out.entry(e.from.as_str()).or_default().push(e.to.as_str());
    }
    let known: BTreeSet<&str> = pack.nodes.iter().map(|n| n.id.as_str()).collect();
    let mut seen: BTreeSet<&str> = roots
        .iter()
        .copied()
        .filter(|r| known.contains(r))
        .collect();
    let mut queue: VecDeque<&str> = seen.iter().copied().collect();
    while let Some(id) = queue.pop_front() {
        for to in out.get(id).into_iter().flatten() {
            if known.contains(to) && seen.insert(to) {
                queue.push_back(to);
            }
        }
    }
    seen.into_iter().map(str::to_owned).collect()
}

/// L2003 for a chance node with no edges: its distribution is empty, so it
/// sums to 0, not 1. Sema checks every chance node that has edges.
pub(crate) fn empty_distributions(cx: &Context<'_>) -> Vec<Diagnostic> {
    cx.nodes()
        .filter(|n| matches!(n.kind, hir::NodeKind::Chance { .. }) && n.edges.is_empty())
        .map(|n| {
            emit(
                C::InvalidDistribution,
                cx.profile,
                cx.span_or_manifest(n.meta.span.as_ref()),
                format!(
                    "chance node `{}` has no edges, so its probabilities sum to 0, not exactly 1",
                    n.meta.id
                ),
            )
        })
        .collect()
}
