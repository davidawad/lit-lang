// SPDX-License-Identifier: GPL-3.0-or-later
//! [`View`]: a scenario resolved against a graph.

use std::collections::{BTreeMap, HashSet};

use super::belief::{self, Belief};
use super::opponent::Models;
use super::plan::PlanInputs;
use super::resolve::{
    active_edges, dist_to_terminal, edge_env, params_of, probabilities, ViewParts,
};
use super::{fee, limits, resolve_spec, NodePlan, Objective, Scenario, Warning};
use crate::error::{Error, Result};
use crate::expr::{self, Expr};
use crate::metrics::{self, EdgeEnv, PathVars, TerminalEnv};
use crate::model::{Graph, Hole, NodeIx, PackMeta, PayoffSource, Role};

/// A scenario resolved into dense arrays. Every algorithm consumes a view.
#[derive(Clone)]
pub struct View<'g> {
    /// The compiled graph.
    pub g: &'g Graph,
    /// The scenario this view resolves.
    pub sc: Scenario,
    /// Parameters after defaults.
    pub params: BTreeMap<String, f64>,
    /// Edge survives masks/removals.
    pub active: Vec<bool>,
    /// Role of each edge under the scenario's perspective.
    pub role: Vec<Role>,
    /// Probability of each edge within its node's draw (None for pure choices).
    pub prob: Vec<Option<f64>>,
    /// Payoff per node after overrides.
    pub payoff: Vec<f64>,
    /// Terminal utility per node (0 off-terminal).
    pub utility: Vec<f64>,
    /// The resolved utility expression, for re-evaluation with real
    /// path-dependent variables (`simulate` uses this; `utility` above is the
    /// Markov (path vars = 0) value `solve`/`chain` use).
    pub utility_expr: Expr,
    /// Fee-shift fraction per node: the share of our accumulated cost a fee
    /// award recovers when the matter ends there (0 off-terminal; see
    /// `scenario::fee`).
    pub fee: Vec<f64>,
    /// Scenario cost metric per edge (NaN if inactive).
    pub cost: Vec<f64>,
    /// Expected elapsed days per edge.
    pub elapsed: Vec<f64>,
    /// Who acts at each node.
    pub plan: Vec<NodePlan>,
    /// Unweighted steps to the nearest terminal over active edges (`u32::MAX` if none).
    pub dist_to_terminal: Vec<u32>,
    /// Forced choices.
    pub forced: BTreeMap<NodeIx, usize>,
    /// Start node.
    pub start: NodeIx,
    /// Everything the engine had to assume.
    pub warnings: Vec<Warning>,
    /// The Dirichlet belief over every chance draw (`plan` already holds its
    /// posterior mean where `scenario.observe` updated it).
    pub belief: Belief,
}

/// `scenario.facts` is sugar over `scenario.probabilities`: each `{node ref:
/// edge ref}` resolves to that edge's canonical id forced to 1.0 (siblings
/// fall to 0 via the usual rescale). Returns the probability overrides plus
/// which nodes had an entry, so a `fact`-tagged node without one can warn
/// instead of silently taking a heuristic/uniform fallback.
///
/// A fact is authored once for the pack node it describes, not once per
/// state-flag history that node can be reached under (same reasoning as
/// calibration's `edge_family`, `docs/PACK_SCHEMA.md#state-flags`): it
/// applies to every flagged copy of `node_ref` (`Graph::node_family`),
/// forcing each copy's own instantiation of `edge_ref`'s edge
/// (`Graph::edge_family`, matched by `base_id` rather than re-resolving
/// `edge_ref` per copy, since a flagged edge's id carries a `{flag}` suffix
/// the author's string won't contain).
fn fact_overrides(g: &Graph, sc: &Scenario) -> Result<(BTreeMap<String, f64>, HashSet<NodeIx>)> {
    let mut overrides = sc.probabilities.clone();
    let mut fact_nodes: HashSet<NodeIx> = HashSet::new();
    for (node_ref, edge_ref) in &sc.facts {
        let ni = g.node(node_ref)?;
        let ei = g.edge_at(ni, edge_ref)?;
        fact_nodes.extend(g.node_family(ni));
        for ej in g.edge_family(ei) {
            overrides.insert(g.edges[ej].id.clone(), 1.0);
        }
    }
    Ok((overrides, fact_nodes))
}

/// Per-terminal scenario values (`scenario.payoffs`, `scenario.fee_shifts`:
/// a `.case` stake and its `fee_shift`) resolved onto the compiled graph,
/// as `(ref, node, value)` in the order to apply them. Like a fact, a value
/// is authored once for the pack node it describes, so a ref naming a pack
/// node reaches every state-flag copy of it (`Graph::node_family`); a ref
/// naming one flagged copy outright (`won{declined}`) comes after every
/// family, so it wins for that copy.
pub(super) fn node_overrides<'a, V>(
    g: &Graph,
    values: &'a BTreeMap<String, V>,
) -> Result<Vec<(&'a str, NodeIx, &'a V)>> {
    let mut families = Vec::new();
    let mut copies = Vec::new();
    for (r, v) in values {
        let n = g.node(r)?;
        if g.nodes[n].flags.is_empty() {
            families.extend(g.node_family(n).into_iter().map(|m| (r.as_str(), m, v)));
        } else {
            copies.push((r.as_str(), n, v));
        }
    }
    families.extend(copies);
    Ok(families)
}

/// Whether `hole` is a terminal payoff (`/nodes/i/payoff` or
/// `/nodes/i/payoffByFlag/...`) that `covered` (the nodes a scenario payoff,
/// such as a `.case` stake, sets) fills for every state-flag copy of node
/// `i`, so nothing the engine computes rests on the hole.
fn payoff_covered(g: &Graph, pack: &PackMeta, hole: &Hole, covered: &HashSet<NodeIx>) -> bool {
    let mut parts = hole.pointer.split('/').skip(1);
    let (Some("nodes"), Some(i), Some("payoff" | "payoffByFlag")) =
        (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    i.parse::<usize>()
        .ok()
        .and_then(|i| pack.node_ids.get(i))
        .and_then(|id| g.node(id).ok())
        .is_some_and(|n| g.node_family(n).iter().all(|m| covered.contains(m)))
}

/// One `authored-hole` warning per pack whose `lowered.holes` leave a
/// number open: a dev build lowered each such `?` to an absent field, so
/// whatever the engine computes from it rests on a fill, not on the pack.
/// A payoff hole the scenario's payoffs (a `.case` stake) fill on every
/// copy of its terminal is not open: it is left out of the count and named
/// as covered, and a pack whose every hole is covered does not warn.
fn hole_warnings(g: &Graph, covered: &HashSet<NodeIx>) -> Vec<Warning> {
    g.packs
        .iter()
        .filter_map(|p| {
            let (filled, open): (Vec<&Hole>, Vec<&Hole>) = p
                .holes
                .iter()
                .filter(|h| h.numeric())
                .partition(|h| payoff_covered(g, p, h, covered));
            let first = open.first()?;
            let why = first
                .reason
                .as_deref()
                .map_or_else(String::new, |r| format!(" ({r})"));
            let also = if filled.is_empty() {
                String::new()
            } else {
                format!(
                    "; {} more covered by case stakes (scenario payoffs), first {}",
                    filled.len(),
                    filled[0].pointer
                )
            };
            Some(Warning {
                code: "authored-hole",
                at: None,
                message: format!(
                    "pack {}: {} explicit hole(s) left numbers unauthored, first {}{why}{also}; author them in the pack's source (a strict build rejects holes)",
                    p.id,
                    open.len(),
                    first.pointer
                ),
            })
        })
        .collect()
}

impl<'g> View<'g> {
    /// Resolve `sc` against `g`.
    ///
    /// ```
    /// use litigate_engine::model::{CompileOptions, Graph, LinkFile, Pack};
    /// use litigate_engine::scenario::{Scenario, View};
    /// let json = r#"{
    ///     "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "start",
    ///     "nodes": [
    ///         {"id": "start", "label": "Start"},
    ///         {"id": "end", "label": "End", "kind": "terminal", "payoff": 100.0}
    ///     ],
    ///     "edges": [{"from": "start", "to": "end", "label": "go"}]
    /// }"#;
    /// let pack = Pack::from_json(json).unwrap();
    /// let g = Graph::compile(&[pack], &LinkFile::default(), &CompileOptions::default()).unwrap();
    /// let v = View::new(&g, &Scenario::default()).unwrap();
    /// // The lone terminal's utility is its authored payoff (the default `ev` utility).
    /// assert_eq!(v.utility[g.node("demo::end").unwrap()], 100.0);
    /// ```
    ///
    /// # Errors
    /// Unknown node/edge refs, bad expressions, out-of-range probabilities.
    pub fn new(g: &'g Graph, sc: &Scenario) -> Result<View<'g>> {
        let params = params_of(sc);
        let role: Vec<Role> = (0..g.edges.len())
            .map(|e| {
                sc.perspective
                    .get(&g.edges[e].actor)
                    .copied()
                    .unwrap_or_else(|| g.base_role(e))
            })
            .collect();
        let mut payoff: Vec<f64> = g.nodes.iter().map(|n| n.payoff).collect();
        let mut staked = HashSet::new();
        for (_, n, v) in node_overrides(g, &sc.payoffs)? {
            payoff[n] = *v;
            staked.insert(n);
        }
        let active = active_edges(g, sc, &role, &params, &payoff)?;
        let parts = ViewParts {
            active: &active,
            role: &role,
            params: &params,
            payoff: &payoff,
        };
        let (overrides, fact_nodes) = fact_overrides(g, sc)?;
        let mut authored = probabilities(g, sc, &parts, &overrides)?;
        let models = Models::resolve(g, sc)?;
        models.authored(g, &role, &mut authored);
        let mut forced = BTreeMap::new();
        for (n, e) in &sc.policy {
            let ni = g.node(n)?;
            forced.insert(ni, g.edge_at(ni, e)?);
        }
        let mut plans = PlanInputs {
            g,
            sc,
            active: &active,
            role: &role,
            authored: &authored,
            fact_nodes: &fact_nodes,
            models: &models,
        }
        .build();
        let belief = Belief::build(g, sc, &plans.plan)?;
        let observed = belief::apply_observed(g, &belief, &mut plans.plan, &mut plans.prob);
        plans.warnings.extend(observed);
        plans.warnings.extend(hole_warnings(g, &staked));
        let start = sc.start.as_deref().map_or(Ok(g.start), |s| g.node(s))?;
        let fee = fee::fractions(g, sc, &payoff, &params)?;
        let mut view = View {
            g,
            sc: sc.clone(),
            params,
            dist_to_terminal: dist_to_terminal(g, &active),
            active,
            role,
            prob: plans.prob,
            payoff,
            utility: vec![0.0; g.nodes.len()],
            utility_expr: Expr::Num(0.0),
            fee,
            cost: vec![],
            elapsed: vec![],
            plan: plans.plan,
            forced,
            start,
            warnings: plans.warnings,
            belief,
        };
        view.finish(sc)?;
        Ok(view)
    }

    /// Resolves utility/cost/elapsed and the combination warnings that need
    /// the fully-built view (`utilities()` for `path-variable-in-markov`,
    /// `fee_shift`/`cvar`/`opponent_objective` limits). Split out of `new`
    /// purely to keep it short; not meaningful to call on its own.
    fn finish(&mut self, sc: &Scenario) -> Result<()> {
        let (utility, utility_expr) = self.utilities()?;
        self.utility = utility;
        self.utility_expr = utility_expr;
        self.warnings.extend(limits::fee_shift_path_vars(sc)?);
        let cost_spec = sc.cost.clone().unwrap_or_else(|| "dollars".into());
        self.cost = self.metric(&cost_spec)?;
        self.elapsed = self.metric("elapsed")?;
        if self
            .cost
            .iter()
            .zip(&self.active)
            .any(|(c, a)| *a && *c < 0.0)
        {
            self.warnings.push(Warning {
                code: "negative-cost",
                at: None,
                message: format!(
                    "cost metric `{cost_spec}` is negative on some edges (treated as a reward)"
                ),
            });
        }
        let fee_shifting = self.fee_shifting();
        self.warnings.extend(limits::cvar_limits(sc, fee_shifting));
        if let Objective::Robust { radius: None, .. } = sc.objective {
            let uncertain = self.belief.uncertain();
            let ws =
                self.belief
                    .estimated_warnings(self.g, &uncertain, "the robust ambiguity radius");
            self.warnings.extend(ws);
        }
        self.warnings
            .extend(limits::opponent_objective_limits(sc, fee_shifting));
        Ok(())
    }

    /// True if a fee award offsets cost anywhere: the scenario sets
    /// `fee_shift`/`fee_shifts`, or a pack authors a nonzero `feeShift`.
    #[must_use]
    pub fn fee_shifting(&self) -> bool {
        self.sc.fee_shift.is_some()
            || !self.sc.fee_shifts.is_empty()
            || self.fee.iter().any(|f| *f != 0.0)
    }

    fn utilities(&mut self) -> Result<(Vec<f64>, Expr)> {
        let spec = self.sc.utility.clone().unwrap_or_else(|| "ev".into());
        let src = resolve_spec(&spec, &self.sc.utilities, metrics::UTILITIES).to_string();
        let ex = expr::parse(&src)?;
        self.warnings.extend(limits::path_var_warning_for(
            &format!("utility `{src}`"),
            &ex,
        ));
        let overridden: HashSet<NodeIx> = node_overrides(self.g, &self.sc.payoffs)?
            .into_iter()
            .map(|(_, n, _)| n)
            .collect();
        let mut out = vec![0.0; self.g.nodes.len()];
        let mut guessed = 0;
        for n in self.g.terminals() {
            let env = TerminalEnv {
                g: self.g,
                n,
                payoff: self.payoff[n],
                params: &self.params,
                path: PathVars::default(),
            };
            out[n] = ex.eval(&env).map_err(|e| {
                Error::Expr(format!("utility `{src}` at {}: {e}", self.g.nodes[n].id))
            })?;
            if self.g.nodes[n].payoff_source != PayoffSource::Authored && !overridden.contains(&n) {
                guessed += 1;
            }
        }
        if guessed > 0 {
            self.warnings.push(Warning {
                code: "payoff-not-authored",
                at: None,
                message: format!("{guessed} terminal(s) use a heuristic or zero payoff; override with scenario.payoffs or author `payoff` in the pack"),
            });
        }
        Ok((out, ex))
    }

    /// Evaluate the resolved utility expression at terminal `n` with explicit
    /// path-dependent variables. `solve`/`chain` use the precomputed
    /// `utility` field (path vars default to 0); `simulate` calls this with
    /// the actual accumulated `spent`/`elapsed_total`/`steps` for the sampled
    /// trajectory that reached `n`.
    ///
    /// # Errors
    /// Expression evaluation errors (should not occur: the same expression
    /// evaluated cleanly at every terminal when the view was constructed).
    pub fn utility_at(&self, n: NodeIx, path: PathVars) -> Result<f64> {
        self.utility_expr
            .eval(&TerminalEnv {
                g: self.g,
                n,
                payoff: self.payoff[n],
                params: &self.params,
                path,
            })
            .map_err(|e| {
                Error::Expr(format!(
                    "utility at {} with path vars: {e}",
                    self.g.nodes[n].id
                ))
            })
    }

    /// Evaluate a metric (name or expression) on every edge; inactive edges get NaN.
    ///
    /// # Errors
    /// Parse or evaluation errors, naming the edge.
    pub fn metric(&self, spec: &str) -> Result<Vec<f64>> {
        let src = resolve_spec(spec, &self.sc.metrics, metrics::METRICS);
        let ex = expr::parse(src)?;
        (0..self.g.edges.len())
            .map(|e| {
                if !self.active[e] {
                    return Ok(f64::NAN);
                }
                ex.eval(&self.edge_env(e)).map_err(|err| {
                    Error::Expr(format!(
                        "metric `{src}` on edge {}: {err}",
                        self.g.edges[e].id
                    ))
                })
            })
            .collect()
    }

    /// Evaluate a terminal expression on every node; non-terminals get NaN.
    ///
    /// # Errors
    /// Parse or evaluation errors.
    pub fn terminal_metric(&self, spec: &str) -> Result<Vec<f64>> {
        let src = resolve_spec(spec, &self.sc.utilities, metrics::UTILITIES);
        let ex = expr::parse(src)?;
        (0..self.g.nodes.len())
            .map(|n| {
                if !self.g.nodes[n].is_terminal() {
                    return Ok(f64::NAN);
                }
                ex.eval(&TerminalEnv {
                    g: self.g,
                    n,
                    payoff: self.payoff[n],
                    params: &self.params,
                    path: PathVars::default(),
                })
            })
            .collect()
    }

    /// The expression environment of edge `e` under this view.
    #[must_use]
    pub fn edge_env(&self, e: usize) -> EdgeEnv<'_> {
        edge_env(
            self.g,
            e,
            &self.role,
            self.prob[e].unwrap_or(1.0),
            &self.params,
            &self.payoff,
        )
    }

    /// Active out-edges of a node.
    pub fn outs(&self, n: NodeIx) -> impl Iterator<Item = usize> + '_ {
        self.g.out[n]
            .iter()
            .copied()
            .filter(move |&e| self.active[e])
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
