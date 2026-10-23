//! The `within` deadline of an edge replay takes (SPEC §5.3, §5.6): the
//! event must fall on or before the due date the clock counts from the
//! latest earlier trigger, unless a cited excusal covers it (L4002).
use super::Cx;
use crate::case::deadline;
use crate::case::graph::{display, pack_of};
use crate::case::spans::{error, Build};
use crate::hir::{DeadlineWindow, Edge};
use litigate_base::DiagnosticCode as C;

impl Cx<'_, '_> {
    /// The `within` deadline of `edge` (declared in `pack`), taken by
    /// event `k`.
    pub(super) fn deadline(&mut self, k: usize, pack: &str, edge: &Edge) {
        let Some(DeadlineWindow::Declared { deadline: id }) =
            edge.deadline.as_ref().map(|d| &d.window)
        else {
            return;
        };
        // The deadline's trigger and counting rule are written in the view
        // of the package that declares it.
        let qualified = self.graph.qualify(pack, id);
        let known_at = self.case.known_at.as_deref();
        let governing = |pack: &str, id: &str| {
            deadline::governing(self.graph, pack, id, &self.events(pack), k, known_at)
        };
        // A claimant of another package that displaces the deadline is
        // judged in its own package's view, under its own transitions.
        let found = governing(pack_of(&qualified), id).and_then(|(q, d)| {
            if pack_of(&q) == pack_of(&qualified) {
                Some((q, d))
            } else {
                governing(pack_of(&q), &q)
            }
        });
        let Some((q, d)) = found else {
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
        let pack = pack_of(&q);
        let events = self.events(pack);
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
