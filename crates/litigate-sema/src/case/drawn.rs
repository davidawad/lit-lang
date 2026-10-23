//! Which nodes a `beliefs` observation may name (SPEC §5.6, L4006). A
//! `chance` node is a world draw by construction. A `decision` is one too
//! when nobody in the matter chooses at it: every out-edge's actor plays
//! `nature` under the matter's roles (the package's `[package.roles]`,
//! then `perspective`). That is litgraph's own rule for what `observe` can
//! name, and it covers tribunal rulings the stdlib keeps as `decision ...
//! by office` so their packs lower unchanged.
use super::graph::Graph;
use crate::hir::{ActorRole, Case, Node, NodeKind, Perspective};

/// Whether `node` (qualified `at`) is a draw the matter can observe.
pub(super) fn is_drawn(case: &Case, graph: &Graph<'_>, at: &str, node: &Node) -> bool {
    match node.kind {
        NodeKind::Chance { .. } => true,
        NodeKind::Decision => {
            node.edges.len() >= 2
                && node
                    .edges
                    .iter()
                    .all(|e| role(case, graph, at, &e.actor) == Some(ActorRole::Nature))
        }
        NodeKind::State | NodeKind::Terminal { .. } => false,
    }
}

/// The role `actor` plays at qualified node `at` in this matter.
fn role(case: &Case, graph: &Graph<'_>, at: &str, actor: &str) -> Option<ActorRole> {
    match &case.perspective {
        Some(Perspective::Actors(map)) if map.contains_key(actor) => {
            return map.get(actor).copied()
        }
        Some(Perspective::Party(id)) => {
            let own = case.parties.iter().find(|p| &p.id == id);
            if own.is_some_and(|p| p.actor == actor) {
                return Some(ActorRole::Protagonist);
            }
            if case.parties.iter().any(|p| p.actor == actor) {
                return Some(ActorRole::Opponent);
            }
        }
        _ => {}
    }
    let pack = at.split_once("::").map_or("", |(p, _)| p);
    let base = pack.split_once('@').map_or(pack, |(b, _)| b);
    graph
        .packages
        .iter()
        .find(|p| p.manifest.pack_id == base)
        .and_then(|p| p.manifest.roles.get(actor).copied())
}
