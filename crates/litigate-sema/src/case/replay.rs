//! Event replay (SPEC §5.6): events in date order, each a legal transition
//! from the posture it finds, judged under the versions governing its date
//! (transitions applied, SPEC §5.4). The posture is derived, never asserted.
use super::deadline::{self, Clock};
use super::graph::{display, is_excusal, Graph};
use super::spans::{error, Build, CaseSpans, EventSpans};
use crate::hir::{
    Case, DeadlineWindow, Declaration, Edge, Event, EventParty, Expr, Node, NodeKind, Removal,
};
use crate::temporal::condition::matches;
use crate::temporal::{MatterEvent, Ungoverned};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C};
use std::collections::BTreeMap;

mod removal;

/// Replay outcome.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Replayed {
    /// The derived posture; `None` when replay stopped at an error.
    pub posture: Option<String>,
    /// Replay diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Options removed by the events replayed, in replay order.
    pub removals: Vec<Removal>,
}

struct Cx<'c, 'g> {
    graph: &'c Graph<'g>,
    case: &'c Case,
    spans: &'c CaseSpans,
    clock: &'c dyn Clock,
    /// `(source index, event)` sorted by date, ties in source order.
    sorted: Vec<(usize, &'c Event)>,
    /// Sorted events in each package's view, by pack id.
    views: BTreeMap<String, Vec<Expr>>,
    /// Options removed so far.
    removed: Vec<Removal>,
    out: Vec<Diagnostic>,
}

/// Events in date order, ties in source order.
pub(super) fn sorted(events: &[Event]) -> Vec<(usize, &Event)> {
    let mut v: Vec<(usize, &Event)> = events.iter().enumerate().collect();
    v.sort_by(|a, b| a.1.date.cmp(&b.1.date));
    v
}

/// Replays `case.events` from `start`.
pub(super) fn replay(
    graph: &Graph<'_>,
    case: &Case,
    spans: &CaseSpans,
    clock: &dyn Clock,
    start: &str,
) -> Replayed {
    let sorted = sorted(&case.events);
    let views = graph
        .packages
        .iter()
        .map(|p| {
            let pack = p.manifest.pack_id.clone();
            let v = sorted
                .iter()
                .map(|(_, e)| graph.view(&pack, &e.action))
                .collect();
            (pack, v)
        })
        .collect();
    let mut cx = Cx {
        graph,
        case,
        spans,
        clock,
        sorted,
        views,
        removed: Vec::new(),
        out: Vec::new(),
    };
    cx.order();
    let mut posture = start.to_owned();
    for k in 0..cx.sorted.len() {
        match cx.step(k, &posture) {
            Some(next) => posture = next,
            None => {
                return Replayed {
                    posture: None,
                    diagnostics: cx.out,
                    removals: cx.removed,
                }
            }
        }
    }
    Replayed {
        posture: Some(posture),
        diagnostics: cx.out,
        removals: cx.removed,
    }
}

fn pack_of(qualified: &str) -> &str {
    qualified.split_once("::").map_or("", |(p, _)| p)
}

impl<'g> Cx<'_, 'g> {
    fn id(&self) -> &str {
        &self.case.id
    }

    fn at(
        &self,
        k: usize,
        part: fn(&EventSpans) -> Option<&litigate_base::Span>,
    ) -> litigate_base::Span {
        self.spans.event(self.sorted[k].0, part, self.id())
    }

    /// Sorted events in `pack`'s view.
    fn events(&self, pack: &str) -> Vec<MatterEvent<'_>> {
        let view = self.views.get(pack);
        self.sorted
            .iter()
            .enumerate()
            .map(|(k, (_, e))| MatterEvent {
                date: &e.date,
                action: view.and_then(|v| v.get(k)).unwrap_or(&e.action),
            })
            .collect()
    }

    /// Out-of-order source is L4001 with a fix that sorts the block.
    fn order(&mut self) {
        let events = &self.case.events;
        let Some(i) = (1..events.len()).find(|&i| events[i].date < events[i - 1].date) else {
            return;
        };
        let mut d = error(
            C::InvalidCaseEvent,
            self.spans.event(i, |e| e.date.as_ref(), self.id()),
            format!(
                "events must be in date order: {} comes after {}",
                events[i].date,
                events[i - 1].date
            ),
        )
        .note("replay processes events sorted by date (ties keep source order)");
        if let Some((span, text)) = &self.spans.sort_events {
            d = d.fix(span.clone(), text.clone(), Applicability::MachineApplicable);
        }
        self.out.push(d);
    }

    /// The node version governing event `k` at qualified `posture`.
    fn node(&mut self, posture: &str, k: usize) -> Option<&'g Node> {
        let pack = pack_of(posture).to_owned();
        let events = self.events(&pack);
        let known_at = self.case.known_at.as_deref();
        let got = self.graph.governing(posture, &events, k, known_at);
        match got {
            Ok(Declaration::Node(n)) => Some(n),
            Ok(_) | Err(Ungoverned::Inactive(_)) => {
                let d = error(
                    C::NoVersionInForce,
                    self.at(k, |e| e.date.as_ref()),
                    format!(
                        "stage `{posture}` has no version in force on {}",
                        self.sorted[k].1.date
                    ),
                );
                self.out.push(d);
                None
            }
            Err(Ungoverned::Transition(why)) => {
                let d = error(
                    C::InvalidTransition,
                    self.at(k, |e| e.date.as_ref()),
                    format!("no version of stage `{posture}` governs this event: {why}"),
                );
                self.out.push(d);
                None
            }
        }
    }

    /// Follows `facts` out of fact nodes before event `k` (SPEC §5.6).
    fn follow_facts(&mut self, mut posture: String, k: usize) -> Option<String> {
        for _ in 0..=self.case.facts.len() {
            let node = self.node(&posture, k)?;
            let Some(edge) = self.case.facts.get(&posture) else {
                return Some(posture);
            };
            if !node.tags.iter().any(|t| t == "fact") {
                return Some(posture);
            }
            let pack = pack_of(&posture).to_owned();
            let local = edge.rsplit("::").next().unwrap_or(edge);
            let next = node
                .edges
                .iter()
                .find(|e| e.id.as_deref() == Some(local))
                .map(|e| self.graph.target(&pack, e))?;
            posture = next;
        }
        Some(posture)
    }

    /// One event; the next posture, or `None` when replay must stop.
    fn step(&mut self, k: usize, posture: &str) -> Option<String> {
        let event = self.sorted[k].1;
        if event.date > self.case.as_of {
            let d = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.date.as_ref()),
                format!("event on {} is after the matter's as_of {}", event.date, self.case.as_of),
            )
            .note("as_of selects the law the matter is analysed under; later events are not yet history");
            self.out.push(d);
        }
        let posture = self.follow_facts(posture.to_owned(), k)?;
        if let Some(arg) = removal::option(self.graph, &event.action) {
            self.remove(k, arg);
            return Some(posture);
        }
        let node = self.node(&posture, k)?;
        let edges = self.graph.edges(&posture, node);
        let (matching, removed): (Vec<(String, &Edge)>, Vec<_>) = edges
            .into_iter()
            .filter(|(pack, e)| {
                e.event
                    .as_ref()
                    .is_some_and(|p| matches(p, &self.view(pack, k)))
            })
            .partition(|(pack, e)| self.removal(pack, e).is_none());
        let actor = self.actor(event);
        let (fits, misfits): (Vec<_>, Vec<_>) = matching
            .into_iter()
            .partition(|(_, e)| e.actor == "either" || actor.is_none_or(|a| a == e.actor));
        match (fits.as_slice(), misfits.as_slice()) {
            ([(pack, edge)], _) => {
                self.deadline(k, pack, edge);
                Some(self.graph.target(pack, edge))
            }
            ([], []) if !removed.is_empty() => self.takes_removed(k, &posture, &removed[0]),
            ([], []) if self.timing_only(k) => Some(posture),
            ([], []) => {
                self.illegal(k, &posture, node);
                None
            }
            ([], [(_, edge), ..]) => {
                let d = error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.party.as_ref()),
                    format!(
                        "`{}` at stage `{posture}` is taken by {}, but `{}` plays {}",
                        display(&event.action),
                        edge.actor,
                        event.party,
                        actor.unwrap_or("?")
                    ),
                );
                self.out.push(d);
                None
            }
            (many, _) => {
                let ids: Vec<String> = many
                    .iter()
                    .map(|(_, e)| format!("`{}`", e.id.as_deref().unwrap_or(&e.to)))
                    .collect();
                let d = error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    format!(
                        "`{}` at stage `{posture}` matches {} transitions",
                        display(&event.action),
                        many.len()
                    ),
                )
                .note(format!(
                    "candidates: {}; make the event's arguments specific",
                    ids.join(", ")
                ));
                self.out.push(d);
                None
            }
        }
    }

    fn view(&self, pack: &str, k: usize) -> Expr {
        self.views
            .get(pack)
            .and_then(|v| v.get(k))
            .cloned()
            .unwrap_or_else(|| self.graph.view(pack, &self.sorted[k].1.action))
    }

    /// The actor of a `by` party (`on` events are done to the party).
    fn actor(&self, event: &Event) -> Option<&str> {
        (event.relation == EventParty::By)
            .then(|| self.case.parties.iter().find(|p| p.id == event.party))
            .flatten()
            .map(|p| p.actor.as_str())
    }

    /// An event that takes no edge: a cited excusal, history a transition
    /// reads, or the trigger of a deadline governing its date.
    fn timing_only(&self, k: usize) -> bool {
        let event = self.sorted[k].1;
        if event.authority.is_some()
            && matches!(&event.action.kind, crate::hir::ExprKind::Call { callee, .. } if is_excusal(callee))
        {
            return true;
        }
        if self
            .graph
            .transition_patterns()
            .iter()
            .any(|(pack, p)| matches(p, &self.view(pack, k)))
        {
            return true;
        }
        let known_at = self.case.known_at.as_deref();
        self.graph.deadlines().iter().any(|q| {
            let pack = pack_of(q);
            let events = self.events(pack);
            matches!(
                self.graph.governing(q, &events, k, known_at),
                Ok(Declaration::Deadline(d)) if matches(&d.trigger, &self.view(pack, k))
            )
        })
    }

    /// Typestate: the event is not legal at this stage (L4001).
    fn illegal(&mut self, k: usize, posture: &str, node: &'g Node) {
        let event = self.sorted[k].1;
        let legal: Vec<String> = self
            .graph
            .edges(posture, node)
            .iter()
            .filter_map(|(_, e)| {
                let p = e.event.as_ref()?;
                Some(format!(
                    "{} [{}]",
                    display(p),
                    e.id.as_deref().unwrap_or(&e.to)
                ))
            })
            .collect();
        let stage = format!("`{posture}` ({})", node.meta.label);
        let mut d = error(
            C::InvalidCaseEvent,
            self.at(k, |e| e.action.as_ref()),
            format!(
                "`{}` is not a legal event at stage {stage}",
                display(&event.action)
            ),
        );
        d = if legal.is_empty() {
            let why = match node.kind {
                NodeKind::Terminal { .. } => "the matter has ended here".to_owned(),
                _ => format!(
                    "no event moves this stage; it is decided by {}",
                    node.actor.as_deref().unwrap_or("either party")
                ),
            };
            d.note(format!("legal events: none ({why})"))
        } else {
            d.note(format!("legal events: {}", legal.join(", ")))
        };
        if node.tags.iter().any(|t| t == "fact") {
            d = d.note(format!(
                "`{posture}` is a matter fact: resolve it with `facts {{ {}: <edge> }}`",
                posture.rsplit("::").next().unwrap_or(posture)
            ));
        }
        if event.authority.is_none()
            && matches!(&event.action.kind, crate::hir::ExprKind::Call { callee, .. } if is_excusal(callee))
        {
            d = d.note(
                "an excusing order takes effect only when it cites its authority (`cite ...`)",
            );
        }
        self.out
            .push(d.note("replay stops at the first illegal event"));
    }

    /// The `within` deadline of the edge taken by event `k` (L4002).
    fn deadline(&mut self, k: usize, pack: &str, edge: &Edge) {
        let Some(DeadlineWindow::Declared { deadline: id }) =
            edge.deadline.as_ref().map(|d| &d.window)
        else {
            return;
        };
        let known_at = self.case.known_at.as_deref();
        let events = self.events(pack);
        let Some(d) = deadline::governing(self.graph, pack, id, &events, k, known_at) else {
            let e = error(
                C::NoVersionInForce,
                self.at(k, |e| e.date.as_ref()),
                format!(
                    "deadline `{id}` has no version governing {}",
                    self.sorted[k].1.date
                ),
            );
            self.out.push(e);
            return;
        };
        let Some(due) = deadline::due(self.graph, self.clock, pack, d, &events, k, known_at) else {
            return;
        };
        let event = self.sorted[k].1;
        let trigger = self.sorted[due.trigger].1;
        let how = deadline::explain(&due, &trigger.action);
        let trigger_at = self.at(due.trigger, |e| e.whole.as_ref());
        match &due.date {
            Err(why) => {
                let e = error(
                    C::InvalidCounting,
                    self.at(k, |e| e.date.as_ref()),
                    format!("cannot count deadline `{id}` for this event"),
                )
                .note(why.clone())
                .note_at(Some(trigger_at), how);
                self.out.push(e);
            }
            Ok(date) if event.date.as_str() > date.as_str() => {
                if deadline::excusal(&self.sorted, k, &trigger.date, &event.date).is_some() {
                    return;
                }
                let excusable = if self.graph.declares_excusal() {
                    "a cited `Waived`, `Extended`, or `Excused` event between the trigger and this event would excuse it"
                } else {
                    "no used package declares a `Waived`, `Extended`, or `Excused` event type to excuse it"
                };
                let e = error(
                    C::DeadlineMissed,
                    self.at(k, |e| e.date.as_ref()),
                    format!(
                        "`{}` on {} missed deadline `{id}`, due {date}",
                        display(&event.action),
                        event.date
                    ),
                )
                .note_at(Some(trigger_at), how)
                .note(excusable);
                self.out.push(e);
            }
            Ok(_) => {}
        }
    }
}
