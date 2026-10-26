// SPDX-License-Identifier: GPL-3.0-or-later
//! Fee shifting under a fixed policy: what a fee award recovers, and what
//! a choice is worth for the spend already behind it.
//!
//! A fee award recovers `f` (per terminal, `View::fee`) of everything we
//! spent on the way, not only what we spend after a choice. So choosing at
//! node `u` between options that lead to different expected fractions
//! `F(e.to)` changes the recovery of the spend `S̄(u)` already incurred on
//! arrival at `u`: by `S̄(u)·F(e.to)`. That term is exact policy
//! improvement (the future is independent of the past given `u`), and it
//! is what makes a fee motion worth the award rather than the award less
//! its own hours when the litigation before it was costly.

use std::collections::BTreeMap;

use crate::algo::chain::step_dist;
use crate::algo::mdp::absorb_prob;
use crate::algo::structure::{is_cyclic, scc};
use crate::model::{NodeIx, Role};
use crate::scenario::{Control, View};

/// Edge costs after recovery, the per-edge bonus our choices weigh for the
/// spend behind them, and the expected fraction recovered from each node,
/// all under `choice`.
pub(crate) struct Adjusted {
    /// `c(e)·(1 − F(e.to))` on our active edges, `c(e)` elsewhere.
    pub cost: Vec<f64>,
    /// `S̄(from)·F(e.to)` on every edge.
    pub bonus: Vec<f64>,
}

/// Fee-shift adjustment of `base` under `choice`.
pub(crate) fn adjust(v: &View, choice: &BTreeMap<NodeIx, usize>, base: &[f64]) -> Adjusted {
    let recovered = absorb_prob(v, choice, &v.fee);
    let own = |e: usize| v.role[e] == Role::Me && v.active[e];
    let spent = arrival_spend(v, choice, base);
    let edges = &v.g.edges;
    Adjusted {
        cost: (0..base.len())
            .map(|e| {
                if own(e) {
                    base[e] * (1.0 - recovered[edges[e].to])
                } else {
                    base[e]
                }
            })
            .collect(),
        bonus: (0..base.len())
            .map(|e| spent[edges[e].from] * recovered[edges[e].to])
            .collect(),
    }
}

/// Expected own spend on arrival at each node (per visit), from the start,
/// under `choice`: the occupation measure's accumulated cost over its
/// visits (0 where the node is never reached).
fn arrival_spend(v: &View, choice: &BTreeMap<NodeIx, usize>, base: &[f64]) -> Vec<f64> {
    let n = v.g.nodes.len();
    let mut incoming: Vec<Vec<(usize, f64)>> = vec![vec![]; n];
    for u in 0..n {
        if matches!(v.plan[u].control, Control::Terminal | Control::Sink) {
            continue;
        }
        for (e, p) in step_dist(v, choice, u) {
            incoming[v.g.edges[e].to].push((e, p));
        }
    }
    let own = |e: usize| {
        if v.role[e] == Role::Me && v.active[e] {
            base[e]
        } else {
            0.0
        }
    };
    // Visits and accumulated spend over visits, forward in topological order.
    let mut visits = vec![0.0; n];
    let mut mass = vec![0.0; n];
    for comp in scc(v).into_iter().rev() {
        let cyclic = is_cyclic(v, &comp);
        for it in 1.. {
            let mut delta: f64 = 0.0;
            for &u in &comp {
                let mut p = if u == v.start { 1.0 } else { 0.0 };
                let mut m = 0.0;
                for &(e, pr) in &incoming[u] {
                    let from = v.g.edges[e].from;
                    p += pr * visits[from];
                    m += pr * (mass[from] + visits[from] * own(e));
                }
                let moved = (m - mass[u]).abs() / m.abs().max(1.0);
                delta = delta.max((p - visits[u]).abs()).max(moved);
                visits[u] = p;
                mass[u] = m;
            }
            if !cyclic || delta < 1e-12 || it >= 100_000 {
                break;
            }
        }
    }
    visits
        .iter()
        .zip(&mass)
        .map(|(p, m)| if *p > 0.0 { m / p } else { 0.0 })
        .collect()
}
