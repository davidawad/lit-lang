// SPDX-License-Identifier: GPL-3.0-or-later
//! Fee shifting per terminal: the fraction of our accumulated cost a fee
//! award recovers when the matter ends there.
//!
//! A terminal's payoff is the gross amount; a fee award offsets what it
//! cost to get there, the fee motion's own hours included. The fraction
//! comes from, in order: the scenario's `fee_shifts` entry for that
//! terminal (a `.case` stake's `fee_shift`, reaching every state-flag
//! copy of it), the scenario's `fee_shift`
//! (its `fraction` at every terminal its `eligible` expression selects, 0
//! elsewhere), and the pack's `feeShift` (the statute's default).

use std::collections::BTreeMap;

use super::Scenario;
use crate::error::{Error, Result};
use crate::expr;
use crate::metrics::{PathVars, TerminalEnv};
use crate::model::Graph;

/// The `eligible` expression a `fee_shift` without one uses.
pub const DEFAULT_ELIGIBLE: &str = "tag('fee-eligible')";

/// The fee-shift fraction per node (0 off-terminal).
///
/// # Errors
/// A bad `fee_shift.eligible` expression, an unknown or non-terminal
/// `fee_shifts` ref, or a fraction outside `[0, 1]`.
pub(super) fn fractions(
    g: &Graph,
    sc: &Scenario,
    payoff: &[f64],
    params: &BTreeMap<String, f64>,
) -> Result<Vec<f64>> {
    let mut out: Vec<f64> = g
        .nodes
        .iter()
        .map(|n| {
            if n.is_terminal() {
                n.fee_shift.unwrap_or(0.0)
            } else {
                0.0
            }
        })
        .collect();
    if let Some(fs) = &sc.fee_shift {
        check("fee_shift.fraction", fs.fraction)?;
        let ex = expr::parse(fs.eligible.as_deref().unwrap_or(DEFAULT_ELIGIBLE))?;
        for n in g.terminals() {
            let x = ex.eval(&TerminalEnv {
                g,
                n,
                payoff: payoff[n],
                params,
                path: PathVars::default(),
            })?;
            out[n] = if x == 0.0 { 0.0 } else { fs.fraction };
        }
    }
    for (r, n, f) in super::view::node_overrides(g, &sc.fee_shifts)? {
        check(&format!("fee_shifts.{r}"), *f)?;
        if !g.nodes[n].is_terminal() {
            return Err(Error::Invalid(format!(
                "fee_shifts.{r}: {r} is not a terminal; only a terminal takes a fee award"
            )));
        }
        out[n] = *f;
    }
    Ok(out)
}

fn check(what: &str, f: f64) -> Result<()> {
    if (0.0..=1.0).contains(&f) {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "{what}: {f} is not a fraction in [0, 1]"
        )))
    }
}
