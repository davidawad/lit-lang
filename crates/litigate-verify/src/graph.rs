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
    // A pack that compiled has every one of its nodes (the start included)
    // in the graph, qualified by the pack id.
    let index = |id: &str| graph.node(&format!("{}::{id}", pack.id)).ok();
    View::new(&graph, &Scenario::default()).ok().map(|view| {
        let seen = roots.iter().filter_map(|root| index(root)).fold(
            vec![false; graph.nodes.len()],
            |seen, ix| {
                seen.into_iter()
                    .zip(structure::reachable(&view, ix))
                    .map(|(s, r)| s | r)
                    .collect()
            },
        );
        pack.nodes
            .iter()
            .filter(|n| index(&n.id).and_then(|ix| seen.get(ix).copied()) == Some(true))
            .map(|n| n.id.clone())
            .collect()
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A pack `demo` starting at `start` with `nodes` (id, kind, tags) and
    /// `edges` (from, to).
    fn pack(start: &str, nodes: &[(&str, &str, &[&str])], edges: &[(&str, &str)]) -> Pack {
        let nodes: Vec<serde_json::Value> = nodes
            .iter()
            .map(|(id, kind, tags)| {
                serde_json::json!({"id": id, "label": id, "kind": kind, "tags": tags, "payoff": 0.0})
            })
            .collect();
        let edges: Vec<serde_json::Value> = edges
            .iter()
            .map(|(from, to)| serde_json::json!({"from": from, "to": to, "label": "go"}))
            .collect();
        let json = serde_json::json!({
            "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": start,
            "nodes": nodes, "edges": edges,
        });
        Pack::from_json(&json.to_string()).expect("a valid pack")
    }

    fn ids(set: &BTreeSet<String>) -> Vec<&str> {
        set.iter().map(String::as_str).collect()
    }

    const STATE: &[&str] = &[];
    const ENTRY_TAG: &[&str] = &["entry"];

    #[test]
    fn a_compiled_pack_is_reached_from_its_start_and_entries() {
        let p = pack(
            "a",
            &[
                ("a", "state", STATE),
                ("b", "terminal", STATE),
                ("island", "state", ENTRY_TAG),
                ("c", "terminal", STATE),
                ("lone", "terminal", STATE),
            ],
            &[("a", "b"), ("island", "c")],
        );
        assert_eq!(ids(&reached(&p)), ["a", "b", "c", "island"]);
    }

    #[test]
    fn a_pack_that_does_not_compile_alone_is_walked_over_its_raw_edges() {
        // `a -> elsewhere::x` leaves the pack, so it does not compile alone;
        // the walk ignores that edge, revisits `b` once, and never reaches
        // `lone`.
        let p = pack(
            "a",
            &[
                ("a", "state", STATE),
                ("b", "state", STATE),
                ("c", "terminal", STATE),
                ("lone", "terminal", STATE),
            ],
            &[("a", "elsewhere::x"), ("a", "b"), ("b", "c"), ("b", "a")],
        );
        assert_eq!(ids(&reached(&p)), ["a", "b", "c"]);
    }

    #[test]
    fn the_raw_walk_ignores_roots_that_are_not_nodes() {
        let p = pack("ghost", &[("a", "terminal", STATE)], &[("ghost", "a")]);
        assert!(reached(&p).is_empty());
    }
}
