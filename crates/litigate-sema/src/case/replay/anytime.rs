//! `anytime` edges (SPEC §5.6): an act a package declares legal from any
//! posture while its own deadline is open. An event that takes no edge of
//! any track takes the one `anytime` edge its pattern matches whose
//! `within` deadline has an instance open for the event's party (or
//! unbound): it discharges that instance under the edge's deadline, as an
//! edge taken on its track would, and the marking does not move. A track
//! that later reaches the edge's source takes the edge at once, so a
//! serial package keeps one track and still lowers.
use super::obligations::logical;
use super::Cx;
use crate::case::graph::display;
use crate::case::{Effect, Step};
use crate::hir::{DeadlineWindow, Edge};
use crate::temporal::condition::matches;

/// An `anytime` edge an event took off its track, awaiting the track.
#[derive(Debug, Clone)]
pub(super) struct Early {
    /// The edge's qualified source.
    from: String,
    /// Its qualified target.
    to: String,
    /// Its qualified id, if it has one.
    edge: Option<String>,
    /// The event that took it, as written.
    event: String,
}

impl<'c> Cx<'c, '_> {
    /// The `anytime` edges in force on event `k`'s date whose pattern
    /// matches it, not removed, with each edge's source and pack; `open`:
    /// only those whose actor the event's party plays and whose deadline
    /// has an instance open for it.
    fn anytime_edges(&self, k: usize, open: bool) -> Vec<(String, String, &'c Edge)> {
        let event = self.sorted[k].1;
        let actor = self.actor(event);
        let party = event.party.as_str();
        let graph: &'c super::Graph<'_> = self.graph;
        graph
            .anytime(&event.date, self.case.known_at.as_deref())
            .into_iter()
            .filter(|(_, pack, e)| {
                e.event
                    .as_ref()
                    .is_some_and(|p| matches(p, self.view(pack, k)))
                    && self.removal(pack, e).is_none()
            })
            .filter(|(_, pack, e)| {
                !open
                    || (actor.is_none_or(|a| e.actor == "either" || a == e.actor)
                        && within(e).is_some_and(|d| {
                            let q = logical(&graph.qualify(pack, d));
                            !self.open(&q, |b| b.is_none_or(|b| b == party)).is_empty()
                        }))
            })
            .collect()
    }

    /// Whether event `k` matches an `anytime` edge it may take.
    pub(super) fn anytime(&self, k: usize) -> bool {
        !self.anytime_edges(k, true).is_empty()
    }

    /// Event `k` takes the `anytime` edge it matches off every track of
    /// `marked` (`primary` first): its deadline is checked and the step
    /// recorded, the edge awaits a track at its source, and the marking
    /// does not move; two candidates are ambiguous (L4001).
    pub(super) fn off_track(
        &mut self,
        k: usize,
        primary: &str,
        marked: Vec<String>,
    ) -> Option<Vec<String>> {
        let found = self.anytime_edges(k, true);
        if let [_, _, ..] = found.as_slice() {
            let many: Vec<(String, &Edge)> = found.into_iter().map(|(_, p, e)| (p, e)).collect();
            self.ambiguous(k, primary, &many);
            return None;
        }
        let (from, pack, edge) = found.into_iter().next()?;
        self.deadline(k, &pack, edge);
        let id = edge.id.as_ref().map(|id| format!("{pack}::{id}"));
        self.record(k, Effect::Anytime, primary, primary, id.clone());
        self.early.push(Early {
            from,
            to: self.graph.target(&pack, edge),
            edge: id,
            event: display(&self.sorted[k].1.action),
        });
        self.triggers(k);
        Some(marked)
    }

    /// `posture`, reached at event `k`, after every `anytime` edge an
    /// earlier event took off its track from there: each is taken now.
    pub(super) fn caught_up(&mut self, k: usize, mut posture: String) -> String {
        while let Some(i) = self.early.iter().position(|e| e.from == posture) {
            let e = self.early.remove(i);
            self.trace.push(Step {
                date: self.sorted[k].1.date.clone(),
                event: e.event,
                effect: Effect::Took,
                from: posture,
                to: e.to.clone(),
                edge: e.edge,
                guard_ops: Vec::new(),
            });
            posture = e.to;
        }
        posture
    }

    /// A note for event `k`, illegal at every track, that matches an
    /// `anytime` edge it may not take: no instance of its deadline is open
    /// for the party, or the party does not play its actor.
    pub(super) fn anytime_note(&self, k: usize) -> Option<String> {
        let (_, pack, e) = self.anytime_edges(k, false).into_iter().next()?;
        let id = e.id.as_deref().unwrap_or(&e.to);
        Some(format!(
            "`{pack}::{id}` is `anytime`: it is legal off its track only for {} while an instance of `{}` is open for the party",
            e.actor,
            within(e).unwrap_or_default()
        ))
    }
}

/// The deadline an edge is `within`.
fn within(e: &Edge) -> Option<&str> {
    match e.deadline.as_ref().map(|d| &d.window) {
        Some(DeadlineWindow::Declared { deadline }) => Some(deadline),
        _ => None,
    }
}
