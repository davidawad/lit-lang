// SPDX-License-Identifier: GPL-3.0-or-later
//! Decision ops: solve, explain, chain, simulate.

use std::collections::VecDeque;

use serde_json::{json, Map, Value};

use super::render::{
    best_line, choice_id, choice_label, convergence_warnings, edge_ref, metric_list, node_ref, r,
};
use super::Warn;
use crate::algo::{chain, cvar, equilibrium, mdp, sim};
use crate::error::Result;
use crate::model::NodeIx;
use crate::scenario::{Control, NodePlan, Objective, View, WAIT};

type Out = Result<(Value, Vec<Warn>)>;

/// The policy `chain`/`simulate`/`explain` run under: `mdp::solve` for the
/// default (`Expected`/`Cara`/`Worst`) objectives, delegating to the
/// general-sum equilibrium (itself a passthrough to `mdp::solve` when
/// `opponent_objective` is unset — the zero-sum special case) or to the
/// `CVaR`-optimal solve. The second element is the opponent's own per-edge
/// `q` (`Some` only for a general-sum equilibrium — `explain` uses it for a
/// mover-correct `regret`; `chain`/`simulate` ignore it).
fn solve(v: &View) -> Result<(mdp::Solution, Option<Vec<f64>>)> {
    if let Objective::Cvar {
        alpha,
        grid,
        y_lo,
        y_hi,
    } = v.sc.objective
    {
        let y_range = cvar::y_range_of(y_lo, y_hi)?;
        let sol = cvar::solve(v, alpha, grid, y_range, &mdp::SolveOptions::default())?.solution;
        return Ok((sol, None));
    }
    let eq = equilibrium::resolve(v, &mdp::SolveOptions::default())?;
    let opponent_q = eq.general_sum.then_some(eq.opponent_q);
    Ok((eq.solution, opponent_q))
}

pub(super) fn solve_op(
    v: &View,
    start: NodeIx,
    full_policy: bool,
    all_values: bool,
    max_steps: usize,
) -> Out {
    if let Objective::Cvar {
        alpha,
        grid,
        y_lo,
        y_hi,
    } = v.sc.objective
    {
        let y_range = cvar::y_range_of(y_lo, y_hi)?;
        let cv = cvar::solve(v, alpha, grid, y_range, &mdp::SolveOptions::default())?;
        let sol = &cv.solution;
        let mut result = solve_result(v, sol, start, full_policy, all_values, max_steps);
        result["cvar"] = r(cv.cvar);
        result["zeta"] = r(cv.zeta);
        result["alpha"] = json!(cv.alpha);
        result["grid"] = json!(cv.grid);
        result["y_range"] = json!([r(cv.y_range.0), r(cv.y_range.1)]);
        return Ok((result, convergence_warnings(v, sol)));
    }
    let eq = equilibrium::resolve(v, &mdp::SolveOptions::default())?;
    let sol = &eq.solution;
    let mut result = solve_result(v, sol, start, full_policy, all_values, max_steps);
    if matches!(v.sc.objective, Objective::Robust { .. }) && !eq.general_sum {
        result["robust"] = super::uncertainty::robust_report(v, sol, start)?;
    }
    if eq.general_sum {
        result["opponent_value"] = r(eq.opponent_value[start]);
        if all_values {
            result["opponent_values"] = json!(values_json(v, &eq.opponent_value));
        }
    }
    Ok((result, convergence_warnings(v, sol)))
}

/// The original `solve` result shape, shared by the zero-sum/general-sum and
/// `CVaR` paths; callers add their objective-specific fields on top.
fn solve_result(
    v: &View,
    sol: &mdp::Solution,
    start: NodeIx,
    full_policy: bool,
    all_values: bool,
    max_steps: usize,
) -> Value {
    let policy: Vec<Value> = sol
        .my_policy(v)
        .map(|(n, e)| json!({ "node": v.g.nodes[n].id, "edge": choice_id(v, e), "label": choice_label(v, e), "q": r(sol.option_q(v, n, e)) }))
        .collect();
    json!({
        "start": node_ref(v, start),
        "value": r(sol.value[start]),
        "converged": sol.converged,
        "iterations": sol.iterations,
        "fee_shift_rounds": sol.fee_shift_rounds,
        "best_line": best_line(v, sol, start, max_steps),
        "policy_size": policy.len(),
        "policy": full_policy.then_some(policy),
        "values": all_values.then(|| values_json(v, &sol.value)),
    })
}

fn values_json(v: &View, values: &[f64]) -> Map<String, Value> {
    (0..v.g.nodes.len())
        .map(|n| (v.g.nodes[n].id.clone(), r(values[n])))
        .collect()
}

pub(super) fn chain_op(v: &View, start: NodeIx, metrics: &[String], top: usize) -> Out {
    let (sol, _) = solve(v)?;
    let ms = metric_list(v, metrics)?;
    let c = chain::chain(v, &sol, start, &ms)?;
    let cost_total = metrics
        .first()
        .and_then(|m| c.expected.get(m))
        .copied()
        .unwrap_or(0.0);
    let absorption: Vec<Value> = c
        .absorption
        .iter()
        .map(|&(t, p)| {
            let n = &v.g.nodes[t];
            json!({ "terminal": n.id, "label": n.label, "p": r(p), "utility": r(v.utility[t]), "outcome": n.outcome })
        })
        .collect();
    let result = json!({
        "start": node_ref(v, start),
        "absorption": absorption,
        "expected": c.expected.iter().map(|(k, x)| (k.clone(), r(*x))).collect::<Map<_, _>>(),
        "expected_steps": r(c.expected_steps),
        "expected_utility": r(c.expected_utility),
        "expected_net": r(c.expected_utility - cost_total),
        "sink_mass": r(c.sink_mass),
        "top_visits": c.visits.iter().take(top).map(|&(n, x)| json!({ "node": v.g.nodes[n].id, "visits": r(x) })).collect::<Vec<_>>(),
    });
    Ok((result, convergence_warnings(v, &sol)))
}

pub(super) fn simulate_op(v: &View, start: NodeIx, metrics: &[String], o: &sim::SimOptions) -> Out {
    let (sol, _) = solve(v)?;
    let ms = metric_list(v, metrics)?;
    let res = sim::simulate(v, &sol, start, &ms, o)?;
    let result = json!({
        "start": node_ref(v, start),
        "runs": res.runs,
        "seed": res.seed,
        "net": res.net,
        "cvar": r(res.cvar),
        "alpha": res.alpha,
        "p_loss": r(res.p_loss),
        "metrics": res.metrics,
        "truncated": res.truncated,
        "terminals": res.terminals.iter().map(|&(t, p)| json!({ "terminal": v.g.nodes[t].id, "p": r(p) })).collect::<Vec<_>>(),
        "samples": res.samples.iter().map(|tr| tr.iter().map(|&e| v.g.edges[e].label.clone()).collect::<Vec<_>>()).collect::<Vec<_>>(),
    });
    Ok((result, convergence_warnings(v, &sol)))
}

/// Who acts at a node, in plain language. `Control::Opponent` is adversarial
/// (minimizes our value) unless `scenario.opponent_objective` is set, in
/// which case the opponent maximizes their own payoff instead — see
/// `equilibrium.rs`.
fn who_decides(control: Control, general_sum: bool) -> &'static str {
    match control {
        Control::Me => "you",
        Control::Opponent if general_sum => "opponent (own payoff, general-sum equilibrium)",
        Control::Opponent => "opponent (adversarial)",
        Control::Chance => "tribunal/chance",
        Control::Terminal => "nobody (terminal)",
        Control::Sink => "nobody (dead end)",
    }
}

/// `q` of option `e` at node `n` under an arbitrary per-edge `q` array (the
/// same rule as `mdp::Solution::option_q`: `WAIT`'s value is the weighted
/// sum over the world edges it lets fire).
fn option_q_of(v: &View, n: NodeIx, e: usize, q: &[f64]) -> f64 {
    if e == WAIT {
        v.plan[n].wait.iter().map(|&(w, p)| p * q[w]).sum()
    } else {
        q[e]
    }
}

/// `option_json`/`explain_op`'s `regret` for option `e` against the chosen
/// `best`: how much worse `e` is *for whoever moves at `n`*, so it is always
/// `>= 0` at the actual optimum. At an opponent's node under a general-sum
/// equilibrium (`opp_q` is `Some`) the mover maximizes their own `opp_q`, not
/// our `q` with `plan.minimize`'s adversarial sign flip — using `opp_q` here
/// is exactly the fix `NodePlan::minimize` can't express, since it stays
/// `true` at every `Control::Opponent` node regardless of
/// `scenario.opponent_objective` (see `plan.rs`).
#[allow(clippy::too_many_arguments)]
fn regret(
    v: &View,
    plan: &NodePlan,
    n: NodeIx,
    e: usize,
    best: usize,
    sol: &mdp::Solution,
    opp_q: Option<&[f64]>,
) -> f64 {
    if plan.control == Control::Opponent {
        if let Some(oq) = opp_q {
            return option_q_of(v, n, best, oq) - option_q_of(v, n, e, oq);
        }
    }
    let q = sol.option_q(v, n, e);
    let qb = sol.option_q(v, n, best);
    if plan.minimize {
        q - qb
    } else {
        qb - q
    }
}

fn option_json(v: &View, sol: &mdp::Solution, n: NodeIx, e: usize, opp_q: Option<&[f64]>) -> Value {
    let plan = &v.plan[n];
    let ed = &v.g.edges[e];
    let q = sol.q[e];
    let mut j = edge_ref(v, e);
    j["q"] = r(q);
    j["cost"] = r(v.cost[e]);
    j["hours"] = r(ed.hours);
    j["fees"] = r(ed.cost);
    if let Some(du) = &ed.duration {
        j["elapsed_days"] = json!(du);
    }
    if !ed.tags.is_empty() {
        j["tags"] = json!(ed.tags);
    }
    if let Some(nt) = &ed.note {
        j["note"] = json!(nt);
    }
    let best = sol.choice.get(&n).copied();
    j["chosen"] = json!(best == Some(e));
    let kind = if plan.choices.contains(&e) {
        "choice"
    } else if plan.wait.iter().any(|&(w, _)| w == e) {
        "draw-if-waiting"
    } else {
        "draw"
    };
    j["kind"] = json!(kind);
    if let (Some(b), "choice") = (best, kind) {
        j["regret"] = r(regret(v, plan, n, e, b, sol, opp_q));
    }
    // What happens after taking this edge: absorption from its target under the policy.
    if let Ok(after) = chain::chain(v, sol, ed.to, &[]) {
        j["then"] = json!(after
            .absorption
            .iter()
            .take(4)
            .map(|&(t, p)| json!({ "terminal": v.g.nodes[t].local_id, "p": r(p) }))
            .collect::<Vec<_>>());
    }
    j
}

/// Unweighted steps from `from` to every node along active edges (BFS),
/// `None` where unreachable.
fn steps_from(v: &View, from: NodeIx) -> Vec<Option<usize>> {
    let mut dist = vec![None; v.g.nodes.len()];
    let mut q = VecDeque::from([(from, 0)]);
    dist[from] = Some(0);
    while let Some((u, d)) = q.pop_front() {
        for e in v.outs(u) {
            let w = v.g.edges[e].to;
            if dist[w].is_none() {
                dist[w] = Some(d + 1);
                q.push_back((w, d + 1));
            }
        }
    }
    dist
}

/// The flag context `explain` details at `named` (`node`, or the start when
/// `node` was omitted), and why. A flagged copy named by its own id is
/// explained as named. A base node id resolves in the replayed context:
/// the copy carrying the start's flags when the start reaches it, else the
/// copy the start reaches first (flags set on the way), else, when the
/// start reaches no copy, the base node itself.
fn flag_context(
    v: &View,
    named: NodeIx,
    start: NodeIx,
    family: &[NodeIx],
    dist: &[Option<usize>],
) -> (NodeIx, &'static str) {
    if named == start {
        return (named, "replayed");
    }
    if !v.g.nodes[named].flags.is_empty() {
        return (named, "named");
    }
    let flags = &v.g.nodes[start].flags;
    let reached = family.iter().filter_map(|&i| dist[i].map(|d| (d, i)));
    if let Some(&i) = family
        .iter()
        .find(|&&i| dist[i].is_some() && &v.g.nodes[i].flags == flags)
    {
        (i, "replayed")
    } else if let Some((_, i)) = reached.min() {
        (i, "reached")
    } else {
        (named, "unreached")
    }
}

/// The chosen option's id and label at `n`, `None` where nobody chooses.
fn chosen_json(v: &View, sol: &mdp::Solution, n: NodeIx) -> Value {
    sol.choice.get(&n).map_or(
        Value::Null,
        |&e| json!({ "id": choice_id(v, e), "label": choice_label(v, e) }),
    )
}

/// The authored edge `n`'s chosen option instantiates (`WAIT` as itself).
fn chosen_base(v: &View, sol: &mdp::Solution, n: NodeIx) -> Option<String> {
    sol.choice.get(&n).map(|&e| {
        if e == WAIT {
            "WAIT".to_owned()
        } else {
            v.g.edges[e].base_id.clone()
        }
    })
}

/// `explain`'s `context` and `other_contexts` for a node with flagged
/// copies: the explained copy's flags and how it was picked, then the
/// copies the start reaches (nearest first) with their value and chosen
/// option, and a count of those it never reaches.
fn contexts_json(
    v: &View,
    sol: &mdp::Solution,
    n: NodeIx,
    basis: &str,
    family: &[NodeIx],
    dist: &[Option<usize>],
) -> (Value, Value) {
    let node = &v.g.nodes[n];
    let context = json!({
        "base_id": node.base_id,
        "flags": node.flags,
        "basis": basis,
        "reachable": dist[n].is_some(),
    });
    let mine = chosen_base(v, sol, n);
    let mut reached: Vec<(usize, NodeIx)> = family
        .iter()
        .filter(|&&i| i != n)
        .filter_map(|&i| dist[i].map(|d| (d, i)))
        .collect();
    reached.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| v.g.nodes[a.1].id.cmp(&v.g.nodes[b.1].id))
    });
    let reachable: Vec<Value> = reached
        .iter()
        .map(|&(d, i)| {
            json!({
                "id": v.g.nodes[i].id,
                "flags": v.g.nodes[i].flags,
                "steps": d,
                "value": r(sol.value[i]),
                "chosen": chosen_json(v, sol, i),
                "same_choice": chosen_base(v, sol, i) == mine,
            })
        })
        .collect();
    let others = json!({
        "count": family.len() - 1,
        "unreached": family.len() - 1 - reachable.len(),
        "reachable": reachable,
    });
    (context, others)
}

/// `explain` at `named`, solved from `start`. A node the state-flag
/// product copied (`docs/PACK_SCHEMA.md#state-flags`) is explained in one
/// flag context ([`flag_context`]) and the other contexts summarized
/// ([`contexts_json`]).
pub(super) fn explain_op(v: &View, named: NodeIx, start: NodeIx) -> Out {
    let (sol, opp_q) = solve(v)?;
    let family = v.g.node_family(named);
    let grouped = (family.len() > 1).then(|| {
        let dist = steps_from(v, start);
        let (n, basis) = flag_context(v, named, start, &family, &dist);
        (n, contexts_json(v, &sol, n, basis, &family, &dist))
    });
    let n = grouped.as_ref().map_or(named, |g| g.0);
    let node = &v.g.nodes[n];
    let plan = &v.plan[n];
    let mut options: Vec<(f64, Value)> = v
        .outs(n)
        .map(|e| (sol.q[e], option_json(v, &sol, n, e, opp_q.as_deref())))
        .collect();
    if !plan.wait.is_empty() {
        let qw = sol.option_q(v, n, WAIT);
        let best = sol.choice.get(&n).copied();
        let mut j = json!({
            "id": "WAIT", "label": choice_label(v, WAIT), "kind": "wait", "q": r(qw), "chosen": best == Some(WAIT),
            "draws": plan.wait.iter().map(|&(e, p)| json!({ "edge": v.g.edges[e].id, "label": v.g.edges[e].label, "p": r(p) })).collect::<Vec<_>>(),
        });
        if let Some(b) = best {
            j["regret"] = r(regret(v, plan, n, WAIT, b, &sol, opp_q.as_deref()));
        }
        options.push((qw, j));
    }
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    let here: Vec<_> = v
        .warnings
        .iter()
        .filter(|w| w.at.as_deref() == Some(node.id.as_str()))
        .collect();
    let mut result = json!({
        "node": { "id": node.id, "label": node.label, "kind": node.kind, "cite": node.cite, "note": node.note },
        "control": plan.control,
        "who_decides": who_decides(plan.control, opp_q.is_some()),
        "value": r(sol.value[n]),
        "interrupt_mass": r(1.0 - plan.choice_mass),
        "options": options.into_iter().map(|x| x.1).collect::<Vec<_>>(),
        "warnings_here": here,
    });
    if let Some((_, (context, others))) = grouped {
        result["context"] = context;
        result["other_contexts"] = others;
    }
    Ok((result, convergence_warnings(v, &sol)))
}
