//! Event replay (SPEC §5.6): posture is derived, never asserted.
use super::Cx;
use crate::clock::{self, Clocks};
use crate::hirx;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, Fix, Span};
use litigate_sema::hir::{self, DeadlineWindow, EventParty, NodeKind};
use litigate_syntax::ast::{AstNode, CaseFile};
use serde::Serialize;
use std::collections::BTreeSet;

/// One replay step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    /// Event date (empty for a fact step).
    pub date: String,
    /// What happened (`Served(complaint, Personal)`, or `fact`).
    pub event: String,
    /// `took`, `timing`, or `fact`.
    pub effect: String,
    /// The node before.
    pub from: String,
    /// The node after.
    pub to: String,
    /// The edge taken, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
}

/// A deadline the events triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Scheduled {
    /// Qualified deadline id.
    pub deadline: String,
    /// The latest triggering event.
    pub trigger: String,
    /// Its date.
    pub trigger_date: String,
    /// Due date.
    pub due: String,
    /// Counting rule.
    pub rule: String,
    /// Counting steps.
    pub steps: Vec<String>,
}

/// The replay's result.
#[derive(Debug, Clone, Default)]
pub struct Replayed {
    /// Scenario start (`None` when it is the primary start and nothing replayed).
    pub start: Option<String>,
    /// Trace.
    pub steps: Vec<Step>,
    /// Deadline schedule as of the last event.
    pub schedule: Vec<Scheduled>,
}

fn qualified(pack: &str, to: &str) -> String {
    if to.contains("::") {
        to.to_owned()
    } else {
        format!("{pack}::{to}")
    }
}

/// L4001 with a machine-applicable fix when the source lists events out of date order.
fn order(cx: &mut Cx<'_>, file: &CaseFile) {
    let events = file.events();
    let dates: Vec<String> = events
        .iter()
        .map(|e| e.date().map(|t| t.text().to_owned()).unwrap_or_default())
        .collect();
    let Some(bad) = dates.windows(2).position(|w| w[1] < w[0]) else {
        return;
    };
    let (Some(first), Some(last)) = (events.first(), events.last()) else {
        return;
    };
    let mut sorted: Vec<(String, String)> = events
        .iter()
        .zip(&dates)
        .map(|(e, d)| (d.clone(), e.syntax().text().to_string().trim().to_owned()))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let indent = " ".repeat(
        usize::try_from(cx.node_span(first.syntax()).col)
            .unwrap_or(1)
            .saturating_sub(1),
    );
    let start = cx.node_span(first.syntax());
    let end = cx.node_span(last.syntax());
    let whole = Span {
        end_line: end.end_line,
        end_col: end.end_col,
        ..start
    };
    let at = cx.node_span(events[bad + 1].syntax());
    cx.diagnostics.push(Diagnostic {
        fixes: vec![Fix {
            span: whole,
            replacement: sorted
                .into_iter()
                .map(|(_, t)| t)
                .collect::<Vec<_>>()
                .join(&format!("\n{indent}")),
            applicability: Applicability::MachineApplicable,
        }],
        ..Diagnostic::error(
            C::InvalidCaseEvent,
            at,
            format!(
                "events must be in date order: {} comes after {}",
                dates[bad + 1],
                dates[bad]
            ),
        )
    });
}

/// True when an earlier event carrying a cite waives, extends, or excuses.
fn excused(history: &[(hir::Event, usize)]) -> bool {
    history.iter().any(|(e, _)| {
        let name = hirx::event_parts(&e.action)
            .map(|(c, _)| c.to_lowercase())
            .unwrap_or_default();
        e.authority.is_some()
            && ["waive", "extend", "excus"]
                .iter()
                .any(|k| name.starts_with(k))
    })
}

/// Every deadline (in force on `date`) of the used packages whose trigger `event` matches.
fn triggered<'p>(
    cx: &Cx<'p>,
    event: &hir::Expr,
    date: &str,
) -> Vec<(&'p hir::Package, &'p hir::Deadline)> {
    cx.packages
        .iter()
        .flat_map(|p| {
            hirx::declarations(p).filter_map(move |d| match d {
                hir::Declaration::Deadline(x)
                    if hirx::in_force(&x.meta, date, None) && hirx::matches(&x.trigger, event) =>
                {
                    Some((*p, x))
                }
                _ => None,
            })
        })
        .collect()
}

/// Due date of `d` from the latest matching trigger in `history`.
fn due(
    clocks: &Clocks,
    pkg: &hir::Package,
    d: &hir::Deadline,
    history: &[(hir::Event, usize)],
) -> Option<Result<clock::Due, String>> {
    let (trigger, _) = history
        .iter()
        .rev()
        .find(|(e, _)| hirx::matches(&d.trigger, &e.action))?;
    let version = hirx::deadline_at(pkg, &d.meta.id, &trigger.date);
    Some(version.and_then(|v| clock::due(clocks, pkg, v, &trigger.date, Some(&trigger.action))))
}

/// Replays `case`'s events from its start.
pub(super) fn replay(
    cx: &mut Cx<'_>,
    file: &CaseFile,
    case: &hir::Case,
    clocks: &Clocks,
) -> Replayed {
    order(cx, file);
    let Some(primary) = cx.primary() else {
        return Replayed::default();
    };
    let package_start = format!("{}::{}", primary.manifest.pack_id, primary.start);
    let mut current = case.start.clone().unwrap_or_else(|| package_start.clone());
    let mut steps = Vec::new();
    let mut events: Vec<(usize, &hir::Event)> = case.events.iter().enumerate().collect();
    events.sort_by(|a, b| a.1.date.cmp(&b.1.date).then(a.0.cmp(&b.0)));
    let mut history: Vec<(hir::Event, usize)> = Vec::new();
    let spans: Vec<Span> = file
        .events()
        .iter()
        .map(|e| cx.node_span(e.syntax()))
        .collect();
    for (i, event) in events {
        let at = spans
            .get(i)
            .cloned()
            .unwrap_or_else(|| cx.span(file.syntax().text_range()));
        let mut seen = BTreeSet::new();
        while let Some((pkg, node)) = cx.node(&current) {
            let fact = matches!(node.kind, NodeKind::Chance { .. })
                && node.tags.iter().any(|t| t == "fact");
            let Some(edge) = case
                .facts
                .get(&current)
                .filter(|_| fact && seen.insert(current.clone()))
            else {
                break;
            };
            let local = edge.rsplit("::").next().unwrap_or(edge);
            let Some(e) = node.edges.iter().find(|e| e.id.as_deref() == Some(local)) else {
                break;
            };
            let next = qualified(&pkg.manifest.pack_id, &e.to);
            steps.push(Step {
                date: String::new(),
                event: "fact".into(),
                effect: "fact".into(),
                from: current.clone(),
                to: next.clone(),
                edge: Some(edge.clone()),
            });
            current = next;
        }
        let text = hirx::event_text(&event.action);
        let Some((pkg, node)) = cx.node(&current) else {
            break;
        };
        let pack = pkg.manifest.pack_id.clone();
        let candidates: Vec<&hir::Edge> = node
            .edges
            .iter()
            .filter(|e| {
                e.event
                    .as_ref()
                    .is_some_and(|p| hirx::matches(p, &event.action))
            })
            .collect();
        match candidates.as_slice() {
            [edge] => {
                let edge = (*edge).clone();
                let actor = case
                    .parties
                    .iter()
                    .find(|p| p.id == event.party)
                    .map(|p| p.actor.clone());
                if event.relation == EventParty::By && actor.as_deref() != Some(edge.actor.as_str())
                {
                    cx.error(
                        C::InvalidCaseEvent,
                        at.clone(),
                        format!(
                            "`{text}` by `{}` ({}) takes `{}`, which only {} may take",
                            event.party,
                            actor.unwrap_or_default(),
                            edge.id.clone().unwrap_or_default(),
                            edge.actor
                        ),
                    );
                }
                if let Some(DeadlineWindow::Declared { deadline }) =
                    edge.deadline.as_ref().map(|d| &d.window)
                {
                    let decl = hirx::deadlines(pkg, deadline).next();
                    match decl.and_then(|d| due(clocks, pkg, d, &history)) {
                        Some(Ok(d)) if event.date > d.computed.due_date && !excused(&history) => cx.error(
                            C::DeadlineMissed,
                            at.clone(),
                            format!("`{text}` on {} is after `{deadline}`, due {} (triggered {})", event.date, d.computed.due_date, d.trigger),
                        ),
                        Some(Ok(_)) => {}
                        Some(Err(why)) => cx.error(C::DeadlineMissed, at.clone(), format!("cannot compute `{deadline}`: {why}")),
                        None if excused(&history) => {}
                        None => cx.error(C::DeadlineMissed, at.clone(), format!("`{text}` is governed by `{deadline}`, but no earlier event triggers it")),
                    }
                }
                let next = qualified(&pack, &edge.to);
                steps.push(Step {
                    date: event.date.clone(),
                    event: text,
                    effect: "took".into(),
                    from: current.clone(),
                    to: next.clone(),
                    edge: edge.id.as_ref().map(|id| format!("{pack}::{id}")),
                });
                current = next;
            }
            [] if !triggered(cx, &event.action, &event.date).is_empty() => {
                steps.push(Step {
                    date: event.date.clone(),
                    event: text,
                    effect: "timing".into(),
                    from: current.clone(),
                    to: current.clone(),
                    edge: None,
                });
            }
            [] => {
                let moves: Vec<String> = node
                    .edges
                    .iter()
                    .filter_map(|e| e.event.as_ref().map(hirx::event_text))
                    .collect();
                cx.diagnostics.push(Diagnostic {
                    notes: vec![litigate_base::Note { span: None, message: if moves.is_empty() { format!("`{current}` has no event-driven moves") } else { format!("legal moves from `{current}`: {}", moves.join(", ")) } }],
                    ..Diagnostic::error(C::InvalidCaseEvent, at, format!("`{text}` on {} is not a legal move from `{current}` and triggers no deadline in force", event.date))
                });
                break;
            }
            _ => {
                cx.error(
                    C::InvalidCaseEvent,
                    at,
                    format!(
                        "`{text}` matches {} edges of `{current}`; make the patterns distinct",
                        candidates.len()
                    ),
                );
                break;
            }
        }
        history.push(((*event).clone(), i));
    }
    if let Some(explicit) = &case.start {
        if *explicit != current {
            let span = file.start().map_or_else(
                || cx.span(file.syntax().text_range()),
                |s| cx.node_span(s.syntax()),
            );
            cx.error(
                C::StartConflict,
                span,
                format!("`start {explicit}` disagrees with the replayed posture `{current}`"),
            );
        }
    }
    let schedule = schedule(cx, clocks, &history);
    let start = (!(case.events.is_empty() && case.start.is_none() && current == package_start))
        .then_some(current);
    Replayed {
        start,
        steps,
        schedule,
    }
}

fn schedule(cx: &Cx<'_>, clocks: &Clocks, history: &[(hir::Event, usize)]) -> Vec<Scheduled> {
    let mut out: Vec<Scheduled> = Vec::new();
    for pkg in &cx.packages {
        for d in hirx::declarations(pkg) {
            let hir::Declaration::Deadline(d) = d else {
                continue;
            };
            let id = format!("{}::{}", pkg.manifest.pack_id, d.meta.id);
            if out.iter().any(|s| s.deadline == id) {
                continue;
            }
            if let Some(Ok(due)) = due(clocks, pkg, d, history) {
                let trigger = history
                    .iter()
                    .rev()
                    .find(|(e, _)| hirx::matches(&d.trigger, &e.action))
                    .map(|(e, _)| hirx::event_text(&e.action))
                    .unwrap_or_default();
                out.push(Scheduled {
                    deadline: id,
                    trigger,
                    trigger_date: due.trigger.clone(),
                    due: due.computed.due_date,
                    rule: due.rule,
                    steps: due.computed.steps,
                });
            }
        }
    }
    out
}
