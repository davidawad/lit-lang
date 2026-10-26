// SPDX-License-Identifier: GPL-3.0-or-later
//! Per-decision opponent models (`scenario.opponent_models`): at one node
//! where only the opponent chooses, how they decide. A stipulation the
//! opponent may accept is not always refused: it can be a calibrated draw
//! or the opponent's own best response, not a move against us.

use std::collections::{BTreeMap, HashMap};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Scenario;
use crate::error::{Error, Result};
use crate::model::{Graph, NodeIx, Role};

/// How the opponent decides at one node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "model", rename_all = "kebab-case", deny_unknown_fields)]
pub enum OpponentModel {
    /// The opponent minimizes our value (zero-sum), whatever `opponent`
    /// mode says and, at this node, even under `opponent_objective`.
    Adversarial,
    /// The opponent's response is a draw: `probabilities` (edge ref →
    /// probability, e.g. a calibrated acceptance rate) replace the authored
    /// ones on the edges they name; the opponent's other edges split the
    /// rest (`prob_fill`). Empty: the authored probabilities, filled.
    Chance {
        /// Edge ref → probability in `[0, 1]`.
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
    /// The opponent takes the edge with the highest payoff to them
    /// (`payoffs`: edge ref → their value in USD of responding that way,
    /// continuation included). Edges it does not list are not taken; ties
    /// among the best go against us.
    OwnObjective {
        /// Edge ref → the opponent's payoff.
        payoffs: BTreeMap<String, f64>,
    },
}

/// Which [`OpponentModel`] a node's plan follows, without its numbers.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ModelKind {
    /// [`OpponentModel::Adversarial`].
    Adversarial,
    /// [`OpponentModel::Chance`].
    Chance,
    /// [`OpponentModel::OwnObjective`].
    OwnObjective,
}

/// `scenario.opponent_models` resolved against a graph: the model of every
/// node (every flagged copy of the node named), and per edge the number its
/// model gives it (a probability or a payoff).
#[derive(Debug, Default)]
pub(super) struct Models {
    pub kind: HashMap<NodeIx, ModelKind>,
    pub value: Vec<Option<f64>>,
}

impl Models {
    /// Resolves every entry; like `scenario.facts`, one entry reaches each
    /// state-flag copy of its node and edges (`Graph::node_family`,
    /// `Graph::edge_family`).
    ///
    /// # Errors
    /// An unknown node or edge ref, a probability outside `[0, 1]`, or a
    /// payoff that is not finite.
    pub fn resolve(g: &Graph, sc: &Scenario) -> Result<Models> {
        let mut out = Models {
            kind: HashMap::new(),
            value: vec![None; g.edges.len()],
        };
        for (node_ref, model) in &sc.opponent_models {
            let ni = g.node(node_ref)?;
            let (kind, values) = match model {
                OpponentModel::Adversarial => (ModelKind::Adversarial, None),
                OpponentModel::Chance { probabilities } => (ModelKind::Chance, Some(probabilities)),
                OpponentModel::OwnObjective { payoffs } => (ModelKind::OwnObjective, Some(payoffs)),
            };
            for (edge_ref, &x) in values.into_iter().flatten() {
                let at = format!("opponent_models.{node_ref}.{edge_ref}");
                let ok = match kind {
                    ModelKind::Chance => (0.0..=1.0).contains(&x),
                    _ => x.is_finite(),
                };
                if !ok {
                    let want = if kind == ModelKind::Chance {
                        "a probability in [0,1]"
                    } else {
                        "a finite payoff"
                    };
                    return Err(Error::Invalid(format!("{at} must be {want}")));
                }
                for e in g.edge_family(g.edge_at(ni, edge_ref)?) {
                    out.value[e] = Some(x);
                }
            }
            out.kind
                .extend(g.node_family(ni).into_iter().map(|n| (n, kind)));
        }
        Ok(out)
    }

    /// `authored` with each chance model's probabilities: an edge it names
    /// takes its number and, once it names any, the node's other opponent
    /// edges none (the fill splits what the named ones leave). A model
    /// naming no edge keeps the authored probabilities.
    pub fn authored(&self, g: &Graph, role: &[Role], authored: &mut [Option<f64>]) {
        for (&n, _) in self.kind.iter().filter(|(_, &k)| k == ModelKind::Chance) {
            if g.out[n].iter().all(|&e| self.value[e].is_none()) {
                continue;
            }
            for &e in &g.out[n] {
                if self.value[e].is_some() || role[e] == Role::Opponent {
                    authored[e] = self.value[e];
                }
            }
        }
    }

    /// An own-objective node's best edges for the opponent among `opp`: the
    /// listed ones with the highest payoff (empty if none is listed).
    pub fn best(&self, opp: &[usize]) -> Vec<usize> {
        let top = opp
            .iter()
            .filter_map(|&e| self.value[e])
            .fold(f64::NEG_INFINITY, f64::max);
        opp.iter()
            .copied()
            .filter(|&e| {
                self.value[e].is_some_and(|x| (x - top).abs() <= 1e-9 * top.abs().max(1.0))
            })
            .collect()
    }
}
