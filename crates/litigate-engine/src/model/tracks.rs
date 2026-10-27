// SPDX-License-Identifier: GPL-3.0-or-later
//! Concurrent tracks on edges (`spawn`/`joins`) compiled into a product
//! graph over markings.
//!
//! A posture is a marking: one token per concurrent track (SPEC §5.6). An
//! edge that `spawn`s starts a track at each node it names; an edge that
//! `joins` nodes exists only where each holds another track's token, and
//! taking it consumes them. This module expands the reachable markings at
//! compile time into plain nodes, so every downstream algorithm sees one
//! walk and needs no changes, as `flags` does for state flags.
//!
//! The expansion is an interleaving with a fixed scheduler: at a marking
//! of several tokens one track moves, the first (in canonical node order)
//! among the readiest: a track whose every out-edge is enabled, then one
//! with only some enabled (a join still waiting), then one whose enabled
//! edges only loop back to its own node. A track that has reached a
//! terminal has ended and never moves. The marking's node is a copy of the
//! moving track's node (its kind, so who chooses or draws) offering its
//! enabled edges; once every track has ended it is a terminal paying the
//! sum of the tracks' payoffs. Costs add along the walk, so tracks that do
//! not interact solve to the value of running them one after the other.
//!
//! A marking of one token is its node: every base node keeps its id and
//! index, and only markings of several tokens are new (`a+b`, the tokens'
//! local ids in canonical order). A pack that declares no track compiles to
//! an untouched graph (`Graph::compile` only calls [`Graph::expand_tracks`]
//! when some edge declares one). See `docs/PACK_SCHEMA.md#concurrent-tracks`.

use std::collections::{HashMap, VecDeque};

use super::entities::{Edge, Node, PayoffSource};
use super::graph::{qualify, EdgeIx, Graph, NodeIx};
use super::schema::NodeKind;
use crate::error::{Error, Result};

/// A marking: one node per token, sorted (a multiset).
type Marking = Vec<NodeIx>;

/// The base edges with their tracks resolved to node indices.
struct Base {
    edges: Vec<Edge>,
    out: Vec<Vec<EdgeIx>>,
    spawn: Vec<Vec<NodeIx>>,
    joins: Vec<Vec<NodeIx>>,
}

impl Base {
    /// `m` without one token at each of `nodes`; `None` unless all are held.
    fn without(m: &[NodeIx], nodes: &[NodeIx]) -> Option<Marking> {
        let mut rest = m.to_vec();
        for n in nodes {
            let i = rest.iter().position(|x| x == n)?;
            rest.remove(i);
        }
        Some(rest)
    }

    /// The track that moves at marking `m` of several tokens and its
    /// enabled edges (module docs); `None` when no track can move.
    fn schedule(&self, nodes: &[Node], m: &[NodeIx]) -> Option<(NodeIx, Vec<EdgeIx>)> {
        let mut best: Option<(u8, NodeIx, Vec<EdgeIx>)> = None;
        for (i, &n) in m.iter().enumerate() {
            if nodes[n].is_terminal() || (i > 0 && m[i - 1] == n) {
                continue;
            }
            let rest = Self::without(m, &[n]).unwrap_or_default();
            let outs = &self.out[n];
            let enabled: Vec<EdgeIx> = outs
                .iter()
                .copied()
                .filter(|&e| Self::without(&rest, &self.joins[e]).is_some())
                .collect();
            if enabled.is_empty() {
                continue;
            }
            let leaves = enabled.iter().any(|&e| {
                self.edges[e].to != n || !self.spawn[e].is_empty() || !self.joins[e].is_empty()
            });
            let rank = match (leaves, enabled.len() == outs.len()) {
                (true, true) => 0,
                (true, false) => 1,
                (false, _) => 2,
            };
            if best.as_ref().is_none_or(|(r, _, _)| rank < *r) {
                best = Some((rank, n, enabled));
            }
        }
        best.map(|(_, n, enabled)| (n, enabled))
    }

    /// The marking after the token at `n` takes `e` at `m`.
    fn fire(&self, m: &[NodeIx], n: NodeIx, e: EdgeIx) -> Marking {
        let mut next = Self::without(m, &[n])
            .and_then(|rest| Self::without(&rest, &self.joins[e]))
            .unwrap_or_default();
        next.push(self.edges[e].to);
        next.extend(&self.spawn[e]);
        next.sort_unstable();
        next
    }
}

/// The ids of a marking of several tokens: `pack::a+b` when every token is
/// in one pack, else the qualified ids joined.
fn marking_ids(nodes: &[Node], m: &[NodeIx]) -> (String, String) {
    let pack = &nodes[m[0]].pack;
    if m.iter().all(|&n| &nodes[n].pack == pack) {
        let local = join(m.iter().map(|&n| nodes[n].local_id.as_str()), "+");
        (qualify(pack, &local), local)
    } else {
        let id = join(m.iter().map(|&n| nodes[n].id.as_str()), "+");
        (id.clone(), id)
    }
}

fn join<'a>(parts: impl Iterator<Item = &'a str>, sep: &str) -> String {
    parts.collect::<Vec<_>>().join(sep)
}

impl Graph {
    /// Expand `spawn`/`joins` into a product graph over markings.
    ///
    /// A worklist walks markings starting from every base node's marking of
    /// one token (so any node remains a valid entry point and keeps its id
    /// and index). There an edge that joins a track is impossible and
    /// omitted; elsewhere the scheduled track's enabled edges are copied,
    /// each to the marking it fires into, materialized and queued the first
    /// time it is reached. A copied edge is `edge@a+b`, its `base_id` the
    /// edge an author wrote, and a marking's node has the moving track's
    /// node as its `base_id`, so facts and calibration authored on a pack
    /// node or edge reach every copy.
    ///
    /// # Errors
    /// `Invalid` if a track names an unknown node, if the compiled nodes
    /// would exceed `cap`, or if a reachable marking deadlocks (a track has
    /// not ended and none can move) or ends on a terminal whose payoff
    /// depends on flags or that shifts fees.
    pub(super) fn expand_tracks(&mut self, cap: usize) -> Result<()> {
        let tracks = std::mem::take(&mut self.tracks);
        let edges = std::mem::take(&mut self.edges);
        let resolve = |e: &Edge, which: &str, ns: &[String]| -> Result<Vec<NodeIx>> {
            ns.iter()
                .map(|n| {
                    self.node_ix.get(n).copied().ok_or_else(|| {
                        Error::Invalid(format!("edge {}: {which} names unknown node {n}", e.id))
                    })
                })
                .collect()
        };
        let mut spawn = Vec::with_capacity(edges.len());
        let mut joins = Vec::with_capacity(edges.len());
        for e in &edges {
            let (s, j) = tracks.get(&e.id).cloned().unwrap_or_default();
            spawn.push(resolve(e, "spawn", &s)?);
            joins.push(resolve(e, "joins", &j)?);
        }
        let mut out = vec![vec![]; self.nodes.len()];
        for (i, e) in edges.iter().enumerate() {
            out[e.from].push(i);
        }
        let base = Base {
            edges,
            out,
            spawn,
            joins,
        };
        let base_n = self.nodes.len();
        let mut state_ix: HashMap<Marking, NodeIx> = (0..base_n).map(|i| (vec![i], i)).collect();
        let mut queue: VecDeque<Marking> = (0..base_n).map(|i| vec![i]).collect();
        let mut new_edges: Vec<Edge> = Vec::new();
        while let Some(m) = queue.pop_front() {
            let from = state_ix[&m];
            let (n, enabled) = match m.as_slice() {
                [n] => {
                    let enabled = base.out[*n]
                        .iter()
                        .copied()
                        .filter(|&e| base.joins[e].is_empty())
                        .collect();
                    (*n, enabled)
                }
                _ => match base.schedule(&self.nodes, &m) {
                    Some(s) => s,
                    None => continue,
                },
            };
            for e in enabled {
                let next = base.fire(&m, n, e);
                let to = self.marking_node(&base, &mut state_ix, &mut queue, next, cap)?;
                let mut ne = base.edges[e].clone();
                if m.len() > 1 {
                    ne.id = format!("{}@{}", ne.id, self.nodes[from].local_id);
                }
                ne.from = from;
                ne.to = to;
                new_edges.push(ne);
            }
        }
        self.edge_ix = new_edges
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect();
        self.edges = new_edges;
        if self.nodes.len() > base_n {
            self.notes.push(format!(
                "concurrent tracks expanded the graph: {} additional node(s) for reachable markings",
                self.nodes.len() - base_n
            ));
        }
        Ok(())
    }

    /// The (possibly new) node for marking `m`: reuses an existing one, or
    /// materializes and queues it, subject to `cap`.
    fn marking_node(
        &mut self,
        base: &Base,
        state_ix: &mut HashMap<Marking, NodeIx>,
        queue: &mut VecDeque<Marking>,
        m: Marking,
        cap: usize,
    ) -> Result<NodeIx> {
        if let Some(&ix) = state_ix.get(&m) {
            return Ok(ix);
        }
        let (id, local_id) = marking_ids(&self.nodes, &m);
        if self.nodes.len() >= cap {
            return Err(Error::Invalid(format!(
                "concurrent tracks exceed max_product_nodes ({cap}) at marking {id}: reachable markings (one token per track) exceed the cap. Join tracks sooner, spawn fewer, or raise max_product_nodes"
            )));
        }
        let label = join(m.iter().map(|&n| self.nodes[n].label.as_str()), " | ");
        let marking = m.iter().map(|&n| self.nodes[n].id.clone()).collect();
        let node = match base.schedule(&self.nodes, &m) {
            Some((n, _)) => Node {
                id: id.clone(),
                local_id,
                label,
                base_id: self.nodes[n].id.clone(),
                marking,
                ..self.nodes[n].clone()
            },
            None => Node {
                marking,
                ..self.ended(&m, id.clone(), local_id, label)?
            },
        };
        let ix = self.nodes.len();
        self.node_ix.insert(id, ix);
        self.nodes.push(node);
        state_ix.insert(m.clone(), ix);
        queue.push_back(m);
        Ok(ix)
    }

    /// The terminal of marking `m` where every track has ended: the sum of
    /// their payoffs, the union of their outcomes and tags.
    fn ended(&self, m: &[NodeIx], id: String, local_id: String, label: String) -> Result<Node> {
        let tokens: Vec<&Node> = m.iter().map(|&n| &self.nodes[n]).collect();
        if let Some(t) = tokens.iter().find(|t| !t.is_terminal()) {
            return Err(Error::Invalid(format!(
                "concurrent tracks deadlock at marking {id}: the track at {} has not ended and no track can move (a join waits on a track that has ended elsewhere)",
                t.id
            )));
        }
        if let Some(t) = tokens
            .iter()
            .find(|t| !t.payoff_by_flag.is_empty() || t.fee_shift.is_some())
        {
            return Err(Error::Invalid(format!(
                "marking {id}: terminal {} ends one of several tracks, and a flag-dependent payoff or a fee shift has no sum over tracks",
                t.id
            )));
        }
        let mut outcome: Vec<String> = Vec::new();
        let mut tags: Vec<String> = Vec::new();
        let mut attrs = std::collections::BTreeMap::new();
        for t in &tokens {
            for o in &t.outcome {
                if !outcome.contains(o) {
                    outcome.push(o.clone());
                }
            }
            for x in &t.tags {
                if !tags.contains(x) {
                    tags.push(x.clone());
                }
            }
            for (k, v) in &t.attrs {
                *attrs.entry(k.clone()).or_insert(0.0) += v;
            }
        }
        let source = |s: PayoffSource| tokens.iter().any(|t| t.payoff_source == s);
        let payoff_source = if source(PayoffSource::Authored) {
            PayoffSource::Authored
        } else if source(PayoffSource::Heuristic) {
            PayoffSource::Heuristic
        } else {
            PayoffSource::Default
        };
        Ok(Node {
            id: id.clone(),
            pack: tokens[0].pack.clone(),
            local_id,
            kind: NodeKind::Terminal,
            label,
            cite: None,
            note: None,
            group: tokens[0].group.clone(),
            valence: None,
            payoff: tokens.iter().map(|t| t.payoff).sum(),
            payoff_source,
            outcome,
            tags,
            attrs,
            payoff_by_flag: std::collections::BTreeMap::new(),
            fee_shift: None,
            synthetic: false,
            flags: vec![],
            base_id: id,
            marking: vec![],
        })
    }
}
