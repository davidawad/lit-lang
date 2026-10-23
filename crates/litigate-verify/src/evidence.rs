//! Positive evidence for each obligation (SPEC §7): how many subjects the
//! check covered, counted over the HIR the check ran on, so a pass states
//! what was checked rather than resting on the absence of diagnostics.
use crate::report::{Builder, Obligation as O};
use crate::Context;
use litigate_sema::hir::{self, Declaration, NodeKind, Relation};
use litigate_sema::temporal::world::meta_of;
use std::collections::{BTreeSet, HashSet};
use std::mem::{discriminant, Discriminant};

/// Records the evidence of every obligation sema discharges (attributed in
/// `sema::aggregate`), and of `dist-exact` and `rule-without-test`, over
/// `cx`'s package.
pub(crate) fn hir(b: &mut Builder, cx: &Context<'_>) {
    let declarations: Vec<&Declaration> = cx.declarations().collect();
    // Kinds sharing an id are distinct version sets.
    let version_sets: HashSet<(Discriminant<Declaration>, &str)> = declarations
        .iter()
        .filter_map(|d| Some((discriminant(*d), meta_of(d)?.id.as_str())))
        .collect();
    let relations: Vec<&Relation> = declarations
        .iter()
        .filter_map(|d| meta_of(d))
        .flat_map(|m| &m.relations)
        .collect();
    let count = |f: fn(&Declaration) -> bool| declarations.iter().filter(|d| f(d)).count();
    let authorities = count(|d| matches!(d, Declaration::Authority(_)));
    let exceptions = count(|d| matches!(d, Declaration::Exception(_)));
    let rules = count(|d| matches!(d, Declaration::Deadline(_) | Declaration::Counting(_)));
    let displacing = relations
        .iter()
        .filter(|r| !matches!(r, Relation::SubjectTo(_)))
        .count();
    let claims = relations
        .iter()
        .filter(|r| {
            matches!(
                r,
                Relation::Override(_) | Relation::Notwithstanding(_) | Relation::SubjectTo(_)
            )
        })
        .count();
    let chance: BTreeSet<&str> = cx
        .nodes()
        .filter(|n| matches!(n.kind, NodeKind::Chance { .. }))
        .map(|n| n.meta.id.as_str())
        .collect();
    b.evidence(O::WellTyped, declarations.len(), "declarations");
    b.evidence(O::EffectiveDisjoint, version_sets.len(), "version sets");
    b.evidence(O::EffectiveCovered, version_sets.len(), "version sets");
    b.evidence(
        O::PrecedenceAcyclic,
        relations.len() + authorities,
        "relations and authority chains",
    );
    b.evidence(
        O::DelegationRespected,
        displacing,
        "displacing relations and delegations",
    );
    b.evidence(
        O::ExceptionsOrdered,
        exceptions + claims,
        "exceptions and precedence claims",
    );
    b.evidence(O::DistExact, chance.len(), "chance nodes");
    b.evidence(O::RuleWithoutTest, rules, "deadline and counting rules");
}

/// The evidence of `tests-pass`: the package's inline tests.
pub(crate) fn tests(b: &mut Builder, cx: &Context<'_>) {
    let tests = cx
        .declarations()
        .filter(|d| matches!(d, Declaration::Test(_)))
        .count();
    b.evidence(O::TestsPass, tests, "inline tests");
}

/// The evidence of the graph and citation obligations of a lowered pack:
/// its nodes, its non-terminal nodes, the `deadlines` nodes intersected,
/// and its cites.
pub(crate) fn graph(b: &mut Builder, cx: &Context<'_>, deadlines: usize) {
    let nodes: BTreeSet<(&str, bool)> = cx
        .nodes()
        .map(|n| {
            let terminal = matches!(n.kind, NodeKind::Terminal { .. });
            (n.meta.id.as_str(), terminal)
        })
        .collect();
    let open = nodes.iter().filter(|(_, terminal)| !terminal).count();
    b.evidence(O::GraphExits, open, "non-terminal nodes");
    b.evidence(O::GraphReachable, nodes.len(), "nodes");
    deadline(b, deadlines);
    b.evidence(O::CiteResolves, cx.cites().count(), "cites");
}

/// The evidence of `deadline-satisfiable`: the nodes whose mandatory
/// deadlines were intersected.
pub(crate) fn deadline(b: &mut Builder, nodes: usize) {
    b.evidence(O::DeadlineSatisfiable, nodes, "nodes");
}

/// The evidence of `holes`: the holes inventoried.
pub(crate) fn holes(b: &mut Builder, holes: usize) {
    b.evidence(O::Holes, holes, "holes");
}

/// The evidence of a matter: its events replayed (`case-replay`), and its
/// parties, events, facts, stakes, and observations typed (`well-typed`).
pub(crate) fn replay(b: &mut Builder, case: &hir::Case) {
    b.evidence(O::CaseReplay, case.events.len(), "events");
    let entries = case.parties.len()
        + case.events.len()
        + case.facts.len()
        + case.stakes.len()
        + case.observations.len();
    b.evidence(O::WellTyped, entries, "case entries");
}
