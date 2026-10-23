//! Event replay (SPEC §5.6): events in date order, each a legal transition
//! from the posture it finds, judged under the versions governing its date
//! (transitions applied, SPEC §5.4). The posture is derived, never asserted.
//!
//! An event that takes no edge must have a declared effect: it fires the
//! trigger of an in-force deadline (creating an obligation instance), is
//! history a transition reads, or is an event type declaring `removes`,
//! `extends`, or `waives`. No such event is accepted where the matter has
//! ended. Event names mean nothing.
use super::deadline::Clock;
use super::graph::{base, display, pack_of, Graph};
use super::spans::{error, Build, CaseSpans, EventSpans};
use super::{Effect, Obligation, Step};
use crate::hir::{
    Case, Declaration, Edge, Event, EventEffect, EventParty, Expr, ExprKind, Node, NodeKind,
    Removal, Type, Value,
};
use crate::temporal::condition::matches;
use crate::temporal::{MatterEvent, Ungoverned};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C};
use std::collections::BTreeMap;

mod obligations;
mod removal;

use obligations::Transform;
mod within;

/// Replay outcome.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Replayed {
    /// The derived posture; `None` when replay stopped at an error.
    pub posture: Option<String>,
    /// Replay diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Options removed by the events replayed, in replay order.
    pub removals: Vec<Removal>,
    /// The transitions replay made, in order.
    pub trace: Vec<Step>,
    /// Obligation instances, in creation order.
    pub obligations: Vec<Obligation>,
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
    trace: Vec<Step>,
    /// Obligation instances so far.
    obligations: Vec<Obligation>,
    /// The `as_of` sentinel: a non-event dated `as_of`, read only for the
    /// versions governing that date.
    sentinel: Expr,
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
        trace: Vec::new(),
        obligations: Vec::new(),
        sentinel: Expr {
            ty: Type::Unit,
            kind: ExprKind::Literal { value: Value::Unit },
            span: None,
        },
    };
    cx.order();
    let mut posture = Some(start.to_owned());
    for k in 0..cx.sorted.len() {
        posture = posture.and_then(|p| cx.step(k, &p));
    }
    if posture.is_some() {
        cx.at_as_of();
    }
    Replayed {
        posture,
        diagnostics: cx.out,
        removals: cx.removed,
        trace: cx.trace,
        obligations: cx.obligations,
    }
}

impl<'c, 'g> Cx<'c, 'g> {
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

    /// Sorted event `k` in `pack`'s view; an instance `base@origin` reads
    /// in `base`'s view. Replay only asks for used packs and their
    /// instances (postures are qualified in one); any other reads as written.
    fn view(&self, pack: &str, k: usize) -> &Expr {
        self.views
            .get(base(pack))
            .and_then(|v| v.get(k))
            .unwrap_or(&self.sorted[k].1.action)
    }

    /// Sorted events in `pack`'s view.
    fn events(&self, pack: &str) -> Vec<MatterEvent<'_>> {
        self.sorted
            .iter()
            .enumerate()
            .map(|(k, (_, e))| MatterEvent {
                date: &e.date,
                action: self.view(pack, k),
            })
            .collect()
    }

    /// [`Self::events`] followed by the `as_of` sentinel, which is last.
    fn events_to_as_of(&self, pack: &str) -> Vec<MatterEvent<'_>> {
        let mut events = self.events(pack);
        events.push(MatterEvent {
            date: &self.case.as_of,
            action: &self.sentinel,
        });
        events
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
            Ok((q, Declaration::Node(n))) if Graph::key(&q) == Graph::key(posture) => Some(n),
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
            Err(Ungoverned::Unresolved(pairs)) => {
                let d = error(
                    C::UnresolvedPrecedence,
                    self.at(k, |e| e.date.as_ref()),
                    format!(
                        "no single declaration governs stage `{posture}` on {}",
                        self.sorted[k].1.date
                    ),
                )
                .note(self.graph.unordered(&pairs));
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

    /// Follows `facts` out of fact nodes before event `k` (SPEC §5.6), at
    /// most once per entry and once more: the posture the event finds, with
    /// its node version.
    fn follow_facts(&mut self, mut posture: String, k: usize) -> Option<(String, &'g Node)> {
        let mut followed = 0;
        loop {
            let node = self.node(&posture, k)?;
            let edge = match self.case.facts.get(&posture) {
                Some(edge)
                    if followed <= self.case.facts.len()
                        && node.tags.iter().any(|t| t == "fact") =>
                {
                    edge
                }
                _ => return Some((posture, node)),
            };
            let pack = pack_of(&posture).to_owned();
            let local = edge.rsplit("::").next().unwrap_or(edge);
            let next = node
                .edges
                .iter()
                .find(|e| e.id.as_deref() == Some(local))
                .map(|e| self.graph.target(&pack, e))?;
            self.trace.push(Step {
                date: String::new(),
                event: "fact".to_owned(),
                effect: Effect::Fact,
                from: posture,
                to: next.clone(),
                edge: Some(edge.clone()),
            });
            posture = next;
            followed += 1;
        }
    }

    /// The edges leaving `posture` (version `node`) whose event pattern
    /// matches event `k`, less those an earlier event removed, with the
    /// first such removal.
    fn matching(
        &self,
        k: usize,
        posture: &str,
        node: &'g Node,
    ) -> (Vec<(String, &'c Edge)>, Option<Removal>) {
        let graph: &'c Graph<'g> = self.graph;
        let mut removed: Option<Removal> = None;
        let matching = graph
            .edges(posture, node)
            .into_iter()
            .filter(|(pack, e)| {
                e.event
                    .as_ref()
                    .is_some_and(|p| matches(p, self.view(pack, k)))
            })
            .filter(|(pack, e)| match self.removal(pack, e) {
                Some(r) => {
                    removed.get_or_insert_with(|| r.clone());
                    false
                }
                None => true,
            })
            .collect();
        (matching, removed)
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
        let (posture, node) = self.follow_facts(posture.to_owned(), k)?;
        let ended = self.ended(&posture, node);
        if let Some((pack, effect, args)) = self.effect(k) {
            if ended {
                self.illegal(k, &posture, node);
                return None;
            }
            let applied = match effect {
                EventEffect::Removes { param } => self.remove(k, args.get(*param)),
                EventEffect::Elects { param } => self.elect(k, args.get(*param)),
                EventEffect::Extends { deadline, param } => {
                    self.transform(k, &pack, deadline, Transform::Extend(args.get(*param)))
                }
                EventEffect::Waives { deadline } => {
                    self.transform(k, &pack, deadline, Transform::Waive)
                }
            };
            if let Some(effect) = applied {
                self.record(k, effect, &posture, &posture, None);
            }
            self.triggers(k);
            return Some(posture);
        }
        let (matching, removed) = self.matching(k, &posture, node);
        let actor = self.actor(event);
        let (fits, misfits): (Vec<_>, Vec<_>) = matching
            .into_iter()
            .partition(|(_, e)| e.actor == "either" || actor.is_none_or(|a| a == e.actor));
        match (fits.as_slice(), misfits.as_slice(), removed) {
            ([(pack, edge)], _, _) => {
                let next = self.take(k, &posture, pack, edge);
                self.triggers(k);
                Some(next)
            }
            ([], [], Some(r)) => {
                self.takes_removed(k, &posture, &r);
                None
            }
            ([], [], None) if !ended && self.triggers(k) => {
                self.record(k, Effect::Triggered, &posture, &posture, None);
                Some(posture)
            }
            ([], [], None) if !ended && self.history(k) => {
                self.record(k, Effect::History, &posture, &posture, None);
                Some(posture)
            }
            ([], [], None) => {
                self.illegal(k, &posture, node);
                None
            }
            ([], [(_, edge), ..], _) => {
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
            (many, _, _) => {
                self.ambiguous(k, &posture, many);
                None
            }
        }
    }

    /// Event `k` at `posture` matches `many` edges (L4001).
    fn ambiguous(&mut self, k: usize, posture: &str, many: &[(String, &Edge)]) {
        let ids: Vec<String> = many
            .iter()
            .map(|(_, e)| format!("`{}`", e.id.as_deref().unwrap_or(&e.to)))
            .collect();
        let d = error(
            C::InvalidCaseEvent,
            self.at(k, |e| e.action.as_ref()),
            format!(
                "`{}` at stage `{posture}` matches {} transitions",
                display(&self.sorted[k].1.action),
                many.len()
            ),
        )
        .note(format!(
            "candidates: {}; make the event's arguments specific",
            ids.join(", ")
        ));
        self.out.push(d);
    }

    /// Whether the matter has ended at `posture` (version `node`): a
    /// terminal with no outgoing edge, links included.
    fn ended(&self, posture: &str, node: &'g Node) -> bool {
        matches!(node.kind, NodeKind::Terminal { .. }) && self.graph.edges(posture, node).is_empty()
    }

    /// The declared effect of event `k`'s type, with the pack declaring
    /// the type and the event's arguments (SPEC §5.2).
    fn effect(&self, k: usize) -> Option<(String, &'g EventEffect, &'c [Expr])> {
        let ExprKind::Call { callee, args } = &self.sorted[k].1.action.kind else {
            return None;
        };
        let (pack, name) = callee
            .split_once("::")
            .unwrap_or((self.graph.primary(), callee.as_str()));
        let effect = self.graph.event_type(pack, name)?.effect.as_ref()?;
        Some((pack.to_owned(), effect, args))
    }

    /// Event `k` takes `edge` (written in `pack`) from `posture`: its
    /// `within` deadline is checked and the step recorded; the next posture.
    fn take(&mut self, k: usize, posture: &str, pack: &str, edge: &Edge) -> String {
        self.deadline(k, pack, edge);
        let to = self.graph.target(pack, edge);
        let id = edge.id.as_ref().map(|id| format!("{pack}::{id}"));
        self.record(k, Effect::Took, posture, &to, id);
        to
    }

    /// Records event `k` moving the matter from `from` to `to`.
    fn record(&mut self, k: usize, effect: Effect, from: &str, to: &str, edge: Option<String>) {
        let event = self.sorted[k].1;
        self.trace.push(Step {
            date: event.date.clone(),
            event: display(&event.action),
            effect,
            from: from.to_owned(),
            to: to.to_owned(),
            edge,
        });
    }

    /// The actor of a `by` party (`on` events are done to the party).
    fn actor(&self, event: &Event) -> Option<&'c str> {
        (event.relation == EventParty::By)
            .then(|| self.case.parties.iter().find(|p| p.id == event.party))
            .flatten()
            .map(|p| p.actor.as_str())
    }

    /// Whether event `k` is history a transition reads (an order keeping
    /// the prior rules, FRCP 86(a)(2)(B)).
    fn history(&self, k: usize) -> bool {
        self.graph
            .transition_patterns()
            .iter()
            .any(|(pack, p)| matches(p, self.view(pack, k)))
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
        if !self.ended(posture, node) {
            d = d.note(
                "an event that takes no edge must fire the trigger of an in-force deadline, be history a transition reads, or have a type declaring `removes`, `extends`, or `waives`",
            );
        }
        if node.tags.iter().any(|t| t == "fact") {
            d = d.note(format!(
                "`{posture}` is a matter fact: resolve it with `facts {{ {}: <edge> }}`",
                posture.rsplit("::").next().unwrap_or(posture)
            ));
        }
        self.out
            .push(d.note("replay stops at the first illegal event"));
    }
}
