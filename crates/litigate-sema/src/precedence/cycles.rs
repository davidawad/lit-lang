//! `precedence-acyclic` over declarations (SPEC §5.2, §5.5, §7): declared
//! wins (`notwithstanding`, `override`, `subject_to`), supersession, and
//! default/exception references must each form a DAG (L3006).
use crate::hir::{Declaration, Expr, ExprKind, Relation};
use crate::temporal::check::relation_key;
use crate::temporal::world::{error, span_of, DiagnosticExt, Key, Locate, World};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use std::collections::{BTreeMap, BTreeSet};

/// One directed edge `from -> to`, declared by declaration `by` with
/// clause `clause`.
#[derive(Debug, Clone)]
struct Edge {
    to: Key,
    by: usize,
    clause: &'static str,
}

type Graph = BTreeMap<Key, Vec<Edge>>;

/// The three relation graphs, with the reading each edge has.
fn graphs(world: &World<'_>) -> [(Graph, &'static str); 3] {
    let mut wins = Graph::new();
    let mut supersession = Graph::new();
    let mut defeats = Graph::new();
    for (i, d) in world.decls.iter().enumerate() {
        let me = world.key(i);
        let pack = world.pack_of(i).to_owned();
        for r in &d.meta.relations {
            let clause = relation_key(r);
            let (graph, from, to) = match r {
                Relation::Notwithstanding(t) | Relation::Override(t) => {
                    (&mut wins, me.clone(), world.target(&pack, t))
                }
                Relation::SubjectTo(t) => {
                    let t = world.target(&pack, t);
                    let edge = Edge {
                        to: me.clone(),
                        by: i,
                        clause,
                    };
                    wins.entry(t.key).or_default().push(edge);
                    continue;
                }
                Relation::Supersedes(t) => {
                    let t = world.target(&pack, t);
                    if t.key == me {
                        continue;
                    }
                    (&mut supersession, me.clone(), t)
                }
                Relation::Revives(_) | Relation::DelegableTo(_) => continue,
            };
            graph.entry(from).or_default().push(Edge {
                to: to.key,
                by: i,
                clause,
            });
        }
        let mut refs = Vec::new();
        match d.decl {
            Declaration::Exception(e) => {
                defeats.entry(me.clone()).or_default().push(Edge {
                    to: world.target(&pack, &e.target).key,
                    by: i,
                    clause: "when",
                });
                collect_refs(&e.condition, &mut refs);
                if let Some(c) = &e.conclusion {
                    collect_refs(c, &mut refs);
                }
            }
            Declaration::Default(x) => {
                collect_refs(&x.condition, &mut refs);
                collect_refs(&x.conclusion, &mut refs);
            }
            _ => {}
        }
        for target in refs {
            let t = world.target(&pack, target).key;
            let is_logic = world.versions(&t).versions.iter().any(|v| {
                matches!(
                    world.decls[v.decl].decl,
                    Declaration::Default(_) | Declaration::Exception(_)
                )
            });
            if is_logic {
                defeats.entry(me.clone()).or_default().push(Edge {
                    to: t,
                    by: i,
                    clause: "when",
                });
            }
        }
    }
    [
        (
            wins,
            "declared precedence (`notwithstanding`, `override`, `subject_to`)",
        ),
        (supersession, "supersession"),
        (defeats, "default/exception references"),
    ]
}

fn collect_refs<'e>(e: &'e Expr, out: &mut Vec<&'e str>) {
    match &e.kind {
        ExprKind::Ref { target } => out.push(target),
        ExprKind::Call { args: items, .. } | ExprKind::List { items } => {
            for a in items {
                collect_refs(a, out);
            }
        }
        ExprKind::Record { fields } => {
            for (_, a) in fields {
                collect_refs(a, out);
            }
        }
        ExprKind::Unary { operand, .. } => collect_refs(operand, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_refs(lhs, out);
            collect_refs(rhs, out);
        }
        ExprKind::Field { base, .. } => collect_refs(base, out),
        ExprKind::Match { scrutinee, arms } => {
            collect_refs(scrutinee, out);
            for (_, a) in arms {
                collect_refs(a, out);
            }
        }
        ExprKind::Literal { .. } => {}
    }
}

/// A shortest cycle through `start`, as the edges taken.
fn cycle_through(graph: &Graph, start: &Key) -> Option<Vec<Edge>> {
    let mut prev: BTreeMap<Key, (Key, Edge)> = BTreeMap::new();
    let mut queue = std::collections::VecDeque::from([start.clone()]);
    while let Some(n) = queue.pop_front() {
        for e in graph.get(&n).into_iter().flatten() {
            if e.to == *start {
                let mut path = vec![e.clone()];
                let mut at = n.clone();
                while at != *start {
                    let (p, edge) = prev.get(&at)?.clone();
                    path.push(edge);
                    at = p;
                }
                path.reverse();
                return Some(path);
            }
            if !prev.contains_key(&e.to) {
                prev.insert(e.to.clone(), (n.clone(), e.clone()));
                queue.push_back(e.to.clone());
            }
        }
    }
    None
}

/// L3006 for every relation cycle through a declaration of pack `pack`,
/// reported once per cycle.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let own = &world.packs[pack].pack;
    let mut out = Vec::new();
    for (graph, reading) in graphs(world) {
        let mut reported: BTreeSet<BTreeSet<Key>> = BTreeSet::new();
        for start in graph.keys().filter(|k| k.pack == *own) {
            let Some(path) = cycle_through(&graph, start) else {
                continue;
            };
            let members: BTreeSet<Key> = path.iter().map(|e| e.to.clone()).collect();
            if !reported.insert(members) {
                continue;
            }
            let first = &path[0];
            let names: Vec<String> = std::iter::once(start.display(own))
                .chain(path.iter().map(|e| e.to.display(own)))
                .collect();
            let mut d = error(
                C::PrecedenceCycle,
                span_of(loc, &world.decls[first.by], first.clause),
                format!("{reading} forms a cycle: {}", names.join(" -> ")),
            );
            for e in path.iter().skip(1) {
                d = d.with_note(
                    Some(span_of(loc, &world.decls[e.by], e.clause)),
                    format!("`{}` continues the cycle here", world.name(e.by, first.by)),
                );
            }
            out.push(d.with_note(None, "remove one of the relations so the order is acyclic"));
        }
    }
    out
}
