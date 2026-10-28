//! Static matter checks around replay (SPEC §5.6): pins against `Lit.lock`
//! and the loaded packages, typed parties and perspective, facts, stakes
//! exhaustiveness, and observations.
use super::graph::Graph;
use super::spans::{error, CaseSpans};
use crate::hir::{Authored, Case, Node, NodeKind, Package, Perspective, Stake};
use crate::pkg::lock::LockFile;
use crate::resolve::suggest::nearest;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, Profile};
use std::collections::BTreeSet;

/// L3004 for each order of the matter's packages that displaces a
/// declaration under a capability exercised on motion (`delegable { ...,
/// on M() }`) when the matter records no such motion on or before the
/// order's date: the order has no effect in this matter (SPEC §5.5), so
/// replay keeps the delegating declaration.
pub(super) fn motions(
    case: &Case,
    graph: &Graph<'_>,
    spans: &CaseSpans,
    out: &mut Vec<Diagnostic>,
) {
    let world = &graph.world;
    let mut seen = BTreeSet::new();
    for (by, target, motion) in crate::precedence::delegation::unmoved_claims(world) {
        let (name, order) = (world.name(target, by), world.decls[by].meta.id.as_str());
        if !seen.insert((by, name.clone())) {
            continue;
        }
        let pack = world.pack_of(by);
        let at = case.uses.iter().position(|u| u.pack_id == pack);
        out.push(error(
            C::MissingDelegation,
            spans.at(at.and_then(|i| spans.uses.get(i)).and_then(Option::as_ref), &case.id),
            format!(
                "`{pack}::{order}` displaces `{name}` under a capability exercised on motion `{}`, which the matter does not record on or before {}: the order has no effect here",
                super::graph::display(&motion),
                world.decls[by].meta.effective.from
            ),
        ));
    }
}

/// The packages `case.uses` pins, in `uses` order, when every pin agrees
/// with `Lit.lock` and with the loaded package (version and content hash).
pub(super) fn pins<'a>(
    case: &Case,
    packages: &[&'a Package],
    lock: Option<&LockFile>,
    spans: &CaseSpans,
    out: &mut Vec<Diagnostic>,
) -> Option<Vec<&'a Package>> {
    let at = |i: usize| spans.at(spans.uses.get(i).and_then(Option::as_ref), &case.id);
    let mut used = Vec::new();
    let mut seen = BTreeSet::new();
    let before = out.len();
    for (i, pin) in case.uses.iter().enumerate() {
        if !seen.insert(pin.name.as_str()) {
            out.push(error(
                C::DuplicateName,
                at(i),
                format!("`{}` is used twice", pin.name),
            ));
            continue;
        }
        let named = |p: &&&Package| p.manifest.name == pin.name;
        let Some(pkg) = packages
            .iter()
            .filter(named)
            .find(|p| p.manifest.version == pin.version)
            .or_else(|| packages.iter().find(named))
        else {
            out.push(error(
                C::UndeclaredDependency,
                at(i),
                format!("package `{}` is not loaded", pin.name),
            ));
            continue;
        };
        let locked = lock.and_then(|l| l.packages.iter().find(|p| p.name == pin.name));
        let problem = match locked {
            None if lock.is_none() => Some("there is no Lit.lock".to_owned()),
            None => Some(format!("Lit.lock has no entry for `{}`", pin.name)),
            Some(l)
                if l.version != pin.version
                    || (!l.is_path() && l.source != format!("sha256:{}", pin.content_hash)) =>
            {
                Some(format!("Lit.lock pins {} {}", l.name, l.shown()))
            }
            Some(_) if pkg.manifest.version != pin.version || pkg.digest != pin.content_hash => {
                Some(format!(
                    "Lit.lock pins {} {} sha256:{}, but the loaded package is {} sha256:{}",
                    pin.name, pin.version, pin.content_hash, pkg.manifest.version, pkg.digest
                ))
            }
            Some(_) => None,
        };
        match problem {
            Some(why) => out.push(
                error(
                    C::StaleLock,
                    at(i),
                    format!("`uses {}` does not match its pin", pin.name),
                )
                .note(why)
                .note("run `litigate lock` to pin the packages this matter uses"),
            ),
            None => used.push(*pkg),
        }
    }
    (out.len() == before && !used.is_empty()).then_some(used)
}

/// Parties, perspective, and event parties (L4005).
pub(super) fn parties(
    case: &Case,
    graph: &Graph<'_>,
    spans: &CaseSpans,
    out: &mut Vec<Diagnostic>,
) {
    let roles = &graph.packages[0].manifest.roles;
    let mut ids = BTreeSet::new();
    for (i, party) in case.parties.iter().enumerate() {
        let (line, actor) = spans.parties.get(i).cloned().unwrap_or_default();
        if !ids.insert(party.id.as_str()) {
            out.push(error(
                C::InvalidParty,
                spans.at(line.as_ref(), &case.id),
                format!(
                    "party `{}` is declared twice; a party holds one role per proceeding",
                    party.id
                ),
            ));
        }
        if !roles.contains_key(&party.actor) {
            let span = spans.at(actor.as_ref(), &case.id);
            let mut d = error(
                C::InvalidParty,
                span.clone(),
                format!(
                    "`{}` is not an actor of `{}`",
                    party.actor, graph.packages[0].manifest.name
                ),
            )
            .note(format!(
                "actors: {}",
                roles.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
            if let Some(best) = nearest(&party.actor, roles.keys().map(String::as_str)) {
                d = d.fix(span, best, Applicability::MaybeIncorrect);
            }
            out.push(d);
        }
    }
    let perspective = spans.at(spans.perspective.as_ref(), &case.id);
    match &case.perspective {
        Some(Perspective::Party(p)) if !ids.contains(p.as_str()) => out.push(error(
            C::InvalidParty,
            perspective,
            format!("perspective names `{p}`, which is not a party"),
        )),
        Some(Perspective::Actors(map)) => {
            for actor in map.keys().filter(|a| !roles.contains_key(*a)) {
                out.push(error(
                    C::InvalidParty,
                    perspective.clone(),
                    format!(
                        "perspective sets `{actor}`, which is not an actor of the primary package"
                    ),
                ));
            }
        }
        _ => {}
    }
    for (i, e) in case.events.iter().enumerate() {
        if !ids.contains(e.party.as_str()) {
            out.push(error(
                C::InvalidParty,
                spans.event(i, |s| s.party.as_ref(), &case.id),
                format!("`{}` is not a declared party", e.party),
            ));
        }
    }
}

fn own_edge(node: &Node, edge: &str) -> bool {
    let local = edge.rsplit("::").next().unwrap_or(edge);
    node.edges.iter().any(|e| e.id.as_deref() == Some(local))
}

/// `facts` name fact nodes and their own edges (L4003); `beliefs` name
/// world draws and their own edges (L4006). Judged at `as_of`.
pub(super) fn facts_and_beliefs(
    case: &Case,
    graph: &Graph<'_>,
    spans: &CaseSpans,
    out: &mut Vec<Diagnostic>,
) {
    let known_at = case.known_at.as_deref();
    let removed = removed_edges(case);
    duplicate_analysis_removals(case, spans, out);
    for (node_id, edge) in &case.facts {
        let (key, value) = spans
            .facts
            .get(node_id)
            .cloned()
            .map_or((None, None), |(k, v)| (Some(k), Some(v)));
        validate_fact(
            case,
            graph,
            spans,
            &removed,
            (node_id, edge, key.as_ref(), value.as_ref()),
            out,
        );
    }
    for (i, obs) in case.observations.iter().enumerate() {
        let (node_span, outcome_spans) = spans.observations.get(i).cloned().unwrap_or_default();
        match graph.node_at(&obs.node, &case.as_of, known_at) {
            Some(n) if super::drawn::is_drawn(case, graph, &obs.node, n) => {
                for (j, (edge, _)) in obs.outcomes.iter().enumerate() {
                    if !own_edge(n, edge) {
                        out.push(
                            error(
                                C::InvalidBelief,
                                spans.at(outcome_spans.get(j).or(node_span.as_ref()), &case.id),
                                format!("`{edge}` is not an outcome of `{}`", obs.node),
                            )
                            .note(format!("edges: {}", edge_ids(n))),
                        );
                    }
                }
            }
            _ => out.push(error(
                C::InvalidBelief,
                spans.at(node_span.as_ref(), &case.id),
                format!(
                    "`{}` is not a chance node in force on {}",
                    obs.node, case.as_of
                ),
            )
            .note("only world draws can be observed: a `chance` node, or a `decision` whose every edge is taken by an actor playing nature")),
        }
    }
}

fn validate_fact(
    case: &Case,
    graph: &Graph<'_>,
    spans: &CaseSpans,
    removed: &BTreeSet<String>,
    (node_id, edge, key, value): (
        &str,
        &str,
        Option<&litigate_base::Span>,
        Option<&litigate_base::Span>,
    ),
    out: &mut Vec<Diagnostic>,
) {
    match graph.node_at(node_id, &case.as_of, case.known_at.as_deref()) {
        Some(n) if n.tags.iter().any(|t| t == "fact") => {
            if !own_edge(n, edge) {
                out.push(
                    error(
                        C::InvalidCaseFact,
                        spans.at(value, &case.id),
                        format!("`{edge}` is not an edge of fact `{node_id}`"),
                    )
                    .note(format!("edges: {}", edge_ids(n))),
                );
            } else if let Some(removal) = case
                .removals
                .iter()
                .find(|r| r.edge == qualified_edge(graph, node_id, edge))
            {
                out.push(
                    error(
                        C::InvalidCaseFact,
                        spans.at(value, &case.id),
                        format!("`{edge}` was removed and cannot resolve fact `{node_id}`"),
                    )
                    .note_at(
                        Some(spans.event(removal.event, |s| s.whole.as_ref(), &case.id)),
                        format!("removed here on {}", removal.date),
                    ),
                );
            } else if removed.contains(&qualified_edge(graph, node_id, edge)) {
                out.push(error(
                    C::InvalidCaseFact,
                    spans.at(value, &case.id),
                    format!("`{edge}` was removed and cannot resolve fact `{node_id}`"),
                ));
            }
        }
        _ => out.push(
            error(
                C::InvalidCaseFact,
                spans.at(key, &case.id),
                format!("`{node_id}` is not a fact node in force on {}", case.as_of),
            )
            .note("facts resolve nodes tagged `fact` (SPEC §5.1)"),
        ),
    }
}

fn edge_ids(n: &Node) -> String {
    n.edges
        .iter()
        .filter_map(|e| e.id.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Terminals reachable from `start` at `as_of`, pruned by `facts`.
pub(super) fn reachable_terminals(case: &Case, graph: &Graph<'_>, start: &str) -> BTreeSet<String> {
    let known_at = case.known_at.as_deref();
    let removed = removed_edges(case);
    let mut seen = BTreeSet::new();
    let mut terminals = BTreeSet::new();
    let mut todo = vec![start.to_owned()];
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some(node) = graph.node_at(&id, &case.as_of, known_at) else {
            continue;
        };
        if matches!(node.kind, NodeKind::Terminal { .. }) {
            terminals.insert(id.clone());
        }
        let fact = node
            .tags
            .iter()
            .any(|t| t == "fact")
            .then(|| case.facts.get(&id))
            .flatten();
        // Every edge leaving `id`, link edges included, is in its pack.
        for (edge_pack, e) in graph.edges(&id, node) {
            if e.id
                .as_deref()
                .is_some_and(|edge| removed.contains(&format!("{edge_pack}::{edge}")))
            {
                continue;
            }
            if fact.is_some_and(|f| !own_edge_is(e, f)) {
                continue;
            }
            todo.push(graph.target(&edge_pack, e));
        }
    }
    terminals
}

fn qualified_edge(graph: &Graph<'_>, node: &str, edge: &str) -> String {
    let pack = node
        .split_once("::")
        .map_or(graph.primary(), |(pack, _)| pack);
    format!("{pack}::{}", edge.rsplit("::").next().unwrap_or(edge))
}

/// Edges removed by replay events or the explicit analysis option.
fn removed_edges(case: &Case) -> BTreeSet<String> {
    let mut removed: BTreeSet<String> = case.removals.iter().map(|r| r.edge.clone()).collect();
    removed.extend(case.analysis.remove_edges.iter().cloned());
    removed
}

fn duplicate_analysis_removals(case: &Case, spans: &CaseSpans, out: &mut Vec<Diagnostic>) {
    let mut seen: BTreeSet<String> = case.removals.iter().map(|r| r.edge.clone()).collect();
    for qualified in &case.analysis.remove_edges {
        if !seen.insert(qualified.clone()) {
            out.push(error(
                C::InvalidCaseEvent,
                spans.at(None, &case.id),
                format!("option `{qualified}` is removed more than once"),
            ));
        }
    }
}

fn own_edge_is(e: &crate::hir::Edge, fact: &str) -> bool {
    e.id.as_deref() == Some(fact.rsplit("::").next().unwrap_or(fact))
}

/// Stakes are exhaustive over reachable terminals (L4004); `unvalued` is
/// an error in `strict`; a stake (or its `fee_shift`) must name a terminal.
pub(super) fn stakes(
    case: &Case,
    graph: &Graph<'_>,
    start: &str,
    profile: Profile,
    spans: &CaseSpans,
    out: &mut Vec<Diagnostic>,
) {
    let known_at = case.known_at.as_deref();
    let named: std::collections::BTreeSet<&String> =
        case.stakes.keys().chain(case.fee_shifts.keys()).collect();
    for id in named {
        let span = spans.at(spans.stakes.get(id), &case.id);
        match graph.node_at(id, &case.as_of, known_at) {
            Some(n) if matches!(n.kind, NodeKind::Terminal { .. }) => {
                if case.stakes.get(id) == Some(&Stake::Unvalued) && profile == Profile::Strict {
                    out.push(error(
                        C::MissingStake,
                        span,
                        format!("terminal `{id}` is `unvalued`, which `strict` does not accept"),
                    ));
                }
            }
            _ => out.push(error(
                C::MissingStake,
                span,
                format!(
                    "`{id}` is not a terminal in force on {}; only terminals take stakes",
                    case.as_of
                ),
            )),
        }
    }
    for id in reachable_terminals(case, graph, start) {
        // A terminal whose pack authors its payoff already has a value.
        let authored = graph.node_at(&id, &case.as_of, known_at).is_some_and(|n| {
            matches!(
                n.kind,
                NodeKind::Terminal {
                    payoff: Some(Authored::Known(_)),
                    ..
                }
            )
        });
        if authored || case.stakes.contains_key(&id) {
            continue;
        }
        let local = id
            .strip_prefix(&format!("{}::", graph.primary()))
            .unwrap_or(&id);
        let mut d = error(
            C::MissingStake,
            spans.at(spans.stakes_end.as_ref().map(|(s, _, _)| s), &case.id),
            format!("terminal `{id}` is reachable from `{start}` but has no stake"),
        )
        .note("give it a value, or `unvalued` while drafting (an error in strict)");
        if let Some((span, before, after)) = &spans.stakes_end {
            d = d.fix(
                span.clone(),
                format!("{before}{local}: unvalued{after}"),
                Applicability::HasPlaceholders,
            );
        }
        out.push(d);
    }
}

/// An explicit `start` must agree with the replayed posture (L4008).
pub(super) fn start(case: &Case, derived: &str, spans: &CaseSpans, out: &mut Vec<Diagnostic>) {
    let Some(explicit) = &case.start else {
        return;
    };
    if explicit == derived {
        return;
    }
    let (line, reference) = spans
        .start
        .clone()
        .map_or((None, None), |(l, r)| (Some(l), Some(r)));
    let mut d = error(
        C::StartConflict,
        spans.at(reference.as_ref(), &case.id),
        format!("`start {explicit}` disagrees with the replayed posture `{derived}`"),
    )
    .note("posture is replayed from the events, never asserted");
    if let Some(line) = line {
        d = d.fix(line, "", Applicability::MaybeIncorrect);
    }
    out.push(d);
}
