//! `lowered.basis` (design review 2026-10-23 §2.10): a pack or link file
//! labels each node or edge whose `basis` the source declares, at its
//! pointer; a number with no label is an estimate. A scenario carries one
//! label, at `/scenario`: the join over every number of the packs it uses,
//! each an estimate unless labelled and a hole where the source wrote `?`,
//! so an analysis is never better sourced than its worst input. The
//! probabilities of a node the matter gives a `prior` are the matter's
//! own estimate instead.
use litigate_engine::model::{Basis, BasisLabel};
use litigate_sema::hir::{self, Authored};

/// The label `b` gives the object at `pointer`.
pub(crate) fn label(pointer: String, b: &hir::Basis) -> BasisLabel {
    let (basis, source) = match b {
        hir::Basis::Sourced { cite } => (Basis::Sourced, Some(cite.clone())),
        hir::Basis::Calibrated { dataset, as_of } => {
            (Basis::Calibrated, Some(format!("{dataset}@{as_of}")))
        }
        hir::Basis::CalibratedSet { set } => (Basis::Calibrated, Some(set.clone())),
        hir::Basis::Expert { name } => (Basis::Expert, Some(name.clone())),
        hir::Basis::Estimate => (Basis::Estimate, None),
        hir::Basis::Placeholder => (Basis::Placeholder, None),
    };
    BasisLabel {
        pointer,
        basis,
        source,
        vintage: None,
    }
}

/// The labels of the objects at `{prefix}/{i}` that declare a basis.
pub(crate) fn labels<'a>(
    prefix: &str,
    declared: impl Iterator<Item = Option<&'a hir::Basis>>,
) -> Vec<BasisLabel> {
    declared
        .enumerate()
        .filter_map(|(i, b)| Some(label(format!("{prefix}/{i}"), b?)))
        .collect()
}

/// The lattice element of a declared basis (an estimate when none).
fn rank(b: Option<&hir::Basis>) -> Basis {
    b.map_or(Basis::Estimate, |b| label(String::new(), b).basis)
}

/// The label of an edge's numbers; with `primed`, its probability is the
/// matter's prior (an estimate) instead of the authored one.
pub(crate) fn edge(e: &hir::Edge, primed: bool) -> Option<Basis> {
    let p = e.probability.as_ref().filter(|_| !primed);
    let mut labels = Vec::new();
    let default = e.basis.as_ref();
    let mut push = |present: bool, provenance: Option<&hir::NumberProvenance>| {
        if present {
            labels.push(rank(provenance.and_then(|p| p.basis.as_ref()).or(default)));
        }
    };
    push(p.is_some(), e.probability_provenance.as_ref());
    push(e.cost.is_some(), e.cost_provenance.as_ref());
    push(e.hours.is_some(), e.hours_provenance.as_ref());
    push(e.duration.is_some(), e.duration_provenance.as_ref());
    if !e.attrs.is_empty() {
        labels.push(rank(default));
    }
    if matches!(e.probability, Some(Authored::Hole(_))) || matches!(e.cost, Some(Authored::Hole(_)))
    {
        labels.push(Basis::Hole);
    }
    if primed && e.probability.is_some() {
        labels.push(Basis::Estimate);
    }
    labels.into_iter().reduce(Basis::join)
}

/// The label of a node's numbers.
pub(crate) fn node(n: &hir::Node) -> Option<Basis> {
    let (payoff, others) = match &n.kind {
        hir::NodeKind::Terminal {
            payoff,
            payoff_by_flag,
            fee_shift,
            ..
        } => (
            payoff.as_ref(),
            !payoff_by_flag.is_empty() || fee_shift.is_some(),
        ),
        _ => (None, false),
    };
    let hole = matches!(payoff, Some(Authored::Hole(_)));
    let default = n.basis.as_ref();
    let mut labels = Vec::new();
    if payoff.is_some() {
        labels.push(rank(
            n.payoff_provenance
                .as_ref()
                .and_then(|p| p.basis.as_ref())
                .or(default),
        ));
    }
    if others || !n.attrs.is_empty() {
        labels.push(rank(default));
    }
    if hole {
        labels.push(Basis::Hole);
    }
    labels.into_iter().reduce(Basis::join)
}

/// The join of the labels of every node of `nodes` and its edges; a node
/// paired with `true` has a matter prior.
pub(crate) fn join<'a>(nodes: impl IntoIterator<Item = (&'a hir::Node, bool)>) -> Option<Basis> {
    nodes
        .into_iter()
        .flat_map(|(n, primed)| {
            std::iter::once(node(n)).chain(n.edges.iter().map(move |e| edge(e, primed)))
        })
        .flatten()
        .reduce(Basis::join)
}
