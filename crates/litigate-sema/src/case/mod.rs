//! Semantic checks for resolved `.case` matters (SPEC §5.6).
use crate::hir::{self, ExprKind, NodeKind, Package};
use crate::pkg::lock::LockFile;
use litigate_base::DiagnosticCode as C;
use std::collections::{BTreeMap, BTreeSet};

/// Result of replaying a matter against its pinned package graph.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedCase {
    /// Case with its derived posture in `start`.
    pub case: hir::Case,
    /// Stable semantic diagnostics.
    pub diagnostics: Vec<litigate_base::Diagnostic>,
}

/// Checks pins and matter semantics against already checked package HIR.
///
/// Package order must match `case.uses`; packages are expected to have passed
/// package sema. `lock` is the parsed `Lit.lock` for the primary package.
#[must_use]
pub fn check(
    mut case: hir::Case,
    packages: &[Package],
    lock: &LockFile,
    strict: bool,
) -> CheckedCase {
    let mut out = Vec::new();
    let mut pins_ok = packages.len() == case.uses.len();
    for pin in &case.uses {
        let actual = packages.iter().find(|p| p.manifest.name == pin.name);
        let locked = lock.packages.iter().find(|p| p.name == pin.name);
        if actual.is_none_or(|p| p.manifest.version != pin.version || p.digest != pin.content_hash)
            || locked.is_none_or(|p| {
                p.version != pin.version || p.digest != format!("sha256:{}", pin.content_hash)
            })
        {
            pins_ok = false;
            out.push(error(
                C::StaleLock,
                "case uses pin does not match Lit.lock version and content hash",
            ));
        }
    }
    if !pins_ok || packages.is_empty() {
        return CheckedCase {
            case,
            diagnostics: out,
        };
    }
    let nodes = graph(packages);
    let primary = &packages[0];
    let mut roles = BTreeSet::new();
    for party in &case.parties {
        if !roles.insert(party.actor.clone()) || !primary.manifest.roles.contains_key(&party.actor)
        {
            out.push(error(
                C::InvalidParty,
                format!(
                    "party `{}` has an undeclared actor or duplicates a proceeding role",
                    party.id
                ),
            ));
        }
    }
    let mut posture = case
        .start
        .clone()
        .unwrap_or_else(|| qualified(primary, &primary.start));
    let mut prior_date: Option<&str> = None;
    let mut history = Vec::new();
    for event in &case.events {
        if prior_date.is_some_and(|date| event.date.as_str() < date) {
            out.push(error(C::InvalidCaseEvent, "events must be in date order"));
        }
        prior_date = Some(&event.date);
        let Some(_node) = nodes.get(&posture) else {
            out.push(error(
                C::InvalidStart,
                format!("replayed posture `{posture}` does not exist"),
            ));
            break;
        };
        follow_facts(&mut posture, &case.facts, &nodes);
        let Some(node) = nodes.get(&posture) else {
            break;
        };
        let matches: Vec<_> = node
            .edges
            .iter()
            .filter(|edge| {
                edge.event
                    .as_ref()
                    .is_some_and(|p| event_matches(p, &event.action))
            })
            .collect();
        if matches.len() == 1 {
            let edge = matches[0];
            if edge.deadline.is_some() {
                let due = deadline_due(packages, &posture, edge, &history);
                if due
                    .as_ref()
                    .is_some_and(|d| event.date.as_str() > d.as_str())
                    && !excused(&history)
                {
                    out.push(error(
                        C::DeadlineMissed,
                        format!("event missed deadline due {due:?}"),
                    ));
                }
            }
            posture = qualify_target(packages, &posture, &edge.to);
        } else if !matches.is_empty() {
            out.push(error(
                C::InvalidCaseEvent,
                "event ambiguously matches multiple legal transitions",
            ));
        } else {
            out.push(error(
                C::InvalidCaseEvent,
                format!(
                    "event is invalid at stage `{}`; legal events: {}",
                    node.meta.id,
                    legal_events(node)
                ),
            ));
        }
        history.push(event.clone());
    }
    follow_facts(&mut posture, &case.facts, &nodes);
    if case.start.is_some() && case.events.is_empty() { /* explicit initial posture */ }
    case.start = Some(posture.clone());
    validate_facts(&case, &nodes, &mut out);
    validate_stakes(&case, &posture, packages, &nodes, strict, &mut out);
    validate_observations(&case, &nodes, &mut out);
    CheckedCase {
        case,
        diagnostics: out,
    }
}

fn error(code: C, message: impl Into<String>) -> litigate_base::Diagnostic {
    litigate_base::Diagnostic::error(
        code,
        litigate_base::Span {
            file: litigate_base::FileId("<case>".into()),
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
        },
        message,
    )
}

fn graph(packages: &[Package]) -> BTreeMap<String, hir::Node> {
    packages
        .iter()
        .flat_map(|p| {
            p.modules.iter().flat_map(move |m| {
                m.declarations.iter().filter_map(move |d| match d {
                    hir::Declaration::Node(n) => Some((qualified(p, &n.meta.id), n.clone())),
                    _ => None,
                })
            })
        })
        .collect()
}
fn qualified(p: &Package, id: &str) -> String {
    format!("{}::{id}", p.manifest.pack_id)
}
fn qualify_target(packages: &[Package], from: &str, target: &str) -> String {
    if target.contains("::") {
        return target.to_owned();
    }
    let pack = from.split_once("::").map_or("", |(p, _)| p);
    let pack_id = packages
        .iter()
        .find(|p| p.manifest.pack_id == pack)
        .map(|p| p.manifest.pack_id.as_str())
        .unwrap_or(pack);
    format!("{pack_id}::{target}")
}
fn event_matches(pattern: &hir::Expr, actual: &hir::Expr) -> bool {
    match (&pattern.kind, &actual.kind) {
        (
            ExprKind::Call {
                callee: a,
                args: pa,
            },
            ExprKind::Call {
                callee: b,
                args: aa,
            },
        ) => a == b && pa.len() <= aa.len() && pa.iter().zip(aa).all(|(x, y)| x == y),
        _ => pattern == actual,
    }
}
fn legal_events(node: &hir::Node) -> String {
    let list: Vec<_> = node
        .edges
        .iter()
        .filter_map(|e| e.event.as_ref())
        .map(|e| match &e.kind {
            ExprKind::Call { callee, .. } => {
                callee.rsplit("::").next().unwrap_or(callee).to_owned()
            }
            _ => "<event>".into(),
        })
        .collect();
    if list.is_empty() {
        "none".into()
    } else {
        list.join(", ")
    }
}
fn follow_facts(
    posture: &mut String,
    facts: &BTreeMap<String, String>,
    nodes: &BTreeMap<String, hir::Node>,
) {
    for _ in 0..nodes.len() {
        let Some(node) = nodes.get(posture) else {
            return;
        };
        if !node.tags.iter().any(|t| t == "fact") {
            return;
        }
        let Some(edge_id) = facts.get(posture) else {
            return;
        };
        let Some(edge) = node
            .edges
            .iter()
            .find(|e| e.id.as_ref() == Some(edge_id) || e.to == *edge_id)
        else {
            return;
        };
        let target = edge.to.clone();
        *posture = if target.contains("::") {
            target
        } else {
            format!(
                "{}::{target}",
                posture.split_once("::").map_or("", |(p, _)| p)
            )
        };
    }
}
fn validate_facts(
    case: &hir::Case,
    nodes: &BTreeMap<String, hir::Node>,
    out: &mut Vec<litigate_base::Diagnostic>,
) {
    for (id, edge_id) in &case.facts {
        match nodes.get(id) {
            Some(n)
                if matches!(n.kind, NodeKind::Chance { .. })
                    && n.tags.iter().any(|t| t == "fact")
                    && n.edges
                        .iter()
                        .any(|e| e.id.as_ref() == Some(edge_id) || e.to == *edge_id) =>
            {
                ()
            }
            _ => out.push(error(
                C::InvalidCaseFact,
                format!("`{id}` is not a fact node with its own edge `{edge_id}`"),
            )),
        }
    }
}
fn validate_stakes(
    case: &hir::Case,
    start: &str,
    packages: &[Package],
    nodes: &BTreeMap<String, hir::Node>,
    strict: bool,
    out: &mut Vec<litigate_base::Diagnostic>,
) {
    let mut seen = BTreeSet::new();
    let mut todo = vec![start.to_owned()];
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(n) = nodes.get(&id) {
            if matches!(n.kind, NodeKind::Terminal { .. }) && !case.stakes.contains_key(&id) {
                out.push(error(
                    C::MissingStake,
                    format!("reachable terminal `{id}` has no stake"),
                ));
            }
            if n.edges.is_empty()
                && matches!(n.kind, NodeKind::Terminal { .. })
                && strict
                && matches!(case.stakes.get(&id), Some(hir::Stake::Unvalued))
            {
                out.push(error(
                    C::MissingStake,
                    format!("terminal `{id}` is unvalued in strict mode"),
                ));
            }
            todo.extend(n.edges.iter().map(|e| qualify_target(packages, &id, &e.to)));
        }
    }
}
fn validate_observations(
    case: &hir::Case,
    nodes: &BTreeMap<String, hir::Node>,
    out: &mut Vec<litigate_base::Diagnostic>,
) {
    for obs in &case.observations {
        match nodes.get(&obs.node) {
            Some(n)
                if matches!(n.kind, NodeKind::Chance { .. })
                    && obs.outcomes.iter().all(|(edge, _)| {
                        n.edges
                            .iter()
                            .any(|e| e.id.as_ref() == Some(edge) || e.to == *edge)
                    }) =>
            {
                ()
            }
            _ => out.push(error(
                C::InvalidBelief,
                format!(
                    "observation `{}` must name a chance node and its edges",
                    obs.node
                ),
            )),
        }
    }
}
fn deadline_due(
    _packages: &[Package],
    _posture: &str,
    _edge: &hir::Edge,
    _history: &[hir::Event],
) -> Option<String> {
    None
}
fn excused(history: &[hir::Event]) -> bool {
    history.iter().any(|e| e.authority.is_some() && matches!(&e.action.kind, ExprKind::Call { callee, .. } if ["waived", "extended", "excused"].iter().any(|x| callee.ends_with(x))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{Type, Value};

    fn call(callee: &str, args: Vec<hir::Expr>) -> hir::Expr {
        hir::Expr {
            ty: Type::Unit,
            kind: ExprKind::Call {
                callee: callee.into(),
                args,
            },
            span: None,
        }
    }

    #[test]
    fn event_patterns_allow_omitted_trailing_arguments() {
        let argument = hir::Expr {
            ty: Type::Named("thing".into()),
            kind: ExprKind::Literal {
                value: Value::String("complaint".into()),
            },
            span: None,
        };
        assert!(event_matches(
            &call("frcp::Filed", vec![argument.clone()]),
            &call("frcp::Filed", vec![argument.clone(), argument.clone()])
        ));
        assert!(!event_matches(
            &call("frcp::Served", vec![argument.clone()]),
            &call("frcp::Filed", vec![argument])
        ));
    }

    proptest::proptest! {
        #[test]
        fn any_prefix_of_matching_history_replays(prefix_len in 0usize..32) {
            let pattern = call("frcp::Filed", Vec::new());
            let history = vec![call("frcp::Filed", Vec::new()); 32];
            proptest::prop_assert!(history.iter().take(prefix_len).all(|event| event_matches(&pattern, event)));
        }
    }
}
