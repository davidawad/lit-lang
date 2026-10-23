//! The `within` deadline of an edge replay takes (SPEC §5.3, §5.6): the
//! act discharges the oldest open obligation instance of that deadline
//! borne by the acting party (or unbound). It must fall on or before the
//! instance's due date (L4002), counted from the instance's own trigger
//! or moved by a licensed extension. Another party's obligation cannot be
//! discharged by this act (L4001).
use super::obligations::{effective, logical};
use super::Cx;
use crate::case::deadline;
use crate::case::graph::{display, pack_of};
use crate::case::spans::error;
use crate::case::Status;
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
        let q = logical(&qualified);
        let Some((w, d)) = self.governing(&q, k, false) else {
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
        let (source, event) = self.sorted[k];
        let party = event.party.as_str();
        let Some(&i) = self.open(&q, |b| b.is_none_or(|b| b == party)).first() else {
            self.foreign(k, &q);
            return;
        };
        let pack = pack_of(&w);
        let due = self.count(i, pack, d, &self.events(pack), k);
        let trigger = self.sorted[due.trigger].1;
        let how = deadline::explain(&due, &trigger.action);
        let trigger_at = self.at(due.trigger, |e| e.whole.as_ref());
        let date = effective(&self.obligations[i], &due);
        let late = matches!(&date, Ok(d) if event.date.as_str() > d.as_str());
        self.close(
            &[i],
            if late {
                Status::Late
            } else {
                Status::Discharged
            },
            source,
        );
        match date {
            Err(why) => {
                let e = error(
                    C::InvalidCounting,
                    self.at(k, |e| e.date.as_ref()),
                    format!("cannot count deadline `{id}` for this event"),
                )
                .note(why)
                .note_at(Some(trigger_at), how);
                self.out.push(e);
            }
            Ok(date) if late => {
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
                .note(self.moved(i));
                self.out.push(e);
            }
            Ok(_) => {}
        }
    }

    /// Event `k` takes a `within q` edge while only other parties'
    /// obligations of `q` are open (L4001); nothing when none is.
    fn foreign(&mut self, k: usize, q: &str) {
        let Some(&o) = self.open(q, |_| true).first() else {
            return;
        };
        let event = self.sorted[k].1;
        let bearer = self.obligations[o].bearer.clone().unwrap_or_default();
        let trigger_at = self.at(self.obligations[o].at, |e| e.whole.as_ref());
        let e = error(
            C::InvalidCaseEvent,
            self.at(k, |e| e.party.as_ref()),
            format!(
                "`{}` cannot discharge `{q}` for `{}`: the open obligation is borne by `{bearer}`",
                display(&event.action),
                event.party
            ),
        )
        .note_at(Some(trigger_at), "created here");
        self.out.push(e);
    }

    /// What moved instance `i`'s due date, for a missed-deadline note.
    fn moved(&self, i: usize) -> String {
        let o = &self.obligations[i];
        match (&o.extended_to, &o.floor) {
            (Some(x), _) => format!("an order extended it to {x}"),
            (None, Some(f)) => format!("it may not end before {f} (`later_of`)"),
            (None, None) => "only a cited event of a type that `extends` this deadline, within its `extend` capability, moves the due date".to_owned(),
        }
    }
}
