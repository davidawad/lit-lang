//! The `within` deadline of an edge replay takes (SPEC §5.3, §5.6): the
//! act discharges the oldest open obligation instance of that deadline
//! borne by the acting party (or unbound). It must fall on or before the
//! instance's due date (L4002), counted from the instance's own trigger
//! or moved by a licensed extension. Another party's obligation cannot be
//! discharged by this act (L4001).
use super::obligations::{effective, logical};
use super::Cx;
use crate::case::deadline::{self, Due};
use crate::case::graph::{display, pack_of};
use crate::case::spans::error;
use crate::case::Status;
use crate::hir::{CharacterKind, Deadline, DeadlineWindow, Edge};
use litigate_base::{Diagnostic, DiagnosticCode as C, Severity};

/// Late-act diagnostic `e` for deadline `id` (version `d`) by its
/// character: a claim-processing limit is enforced when the opponent
/// objects, and the objection can be forfeited, so the act is a warning
/// (L4013); a jurisdictional one admits no exception (L4002, noted); an
/// uncharacterized one stays L4002.
fn characterized(mut e: Diagnostic, d: &Deadline, id: &str) -> Diagnostic {
    match d.characterization.as_ref().map(|c| c.kind) {
        Some(CharacterKind::ClaimProcessing) => {
            e.code = C::LateClaimProcessing;
            e.severity = Severity::Warning;
            e.note(format!("`{id}` is a claim-processing rule: the act is untimely if the opponent objects, and the objection can be forfeited"))
        }
        Some(CharacterKind::Jurisdictional) => e.note(format!("`{id}` is jurisdictional: no waiver, forfeiture or equitable exception excuses it (Bowles v. Russell, 551 U.S. 205 (2007))")),
        None => e,
    }
}

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
            self.ungoverned(k, id, &q);
            return;
        };
        let (source, event) = self.sorted[k];
        let party = event.party.as_str();
        let Some(&i) = self.open(&q, |b| b.is_none_or(|b| b == party)).first() else {
            self.foreign(k, &q);
            return;
        };
        if let Some(earliest) = self.obligations[i].unencoded.clone() {
            self.close(&[i], Status::Unjudged, source);
            self.unencoded(k, id, i, &earliest);
            return;
        }
        let pack = pack_of(&w);
        let due = self.count(i, pack, d, &self.events(pack), k);
        let trigger = self.sorted[due.trigger].1;
        let how = deadline::explain(&due, &trigger.action);
        let trigger_at = self.at(due.trigger, |e| e.whole.as_ref());
        let date = effective(&self.obligations[i], &due);
        let late = matches!(&date, Ok(d) if event.date.as_str() > d.as_str());
        let unjudged = date.is_err() && due.unknown;
        self.close(
            &[i],
            if late {
                Status::Late
            } else if unjudged {
                Status::Unjudged
            } else {
                Status::Discharged
            },
            source,
        );
        match date {
            Err(why) if unjudged => self.unjudged(k, id, &due, why, how),
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
                self.out.push(characterized(e, d, id));
            }
            Ok(_) => {}
        }
    }

    /// Event `k` took deadline `id` whose instance's trigger left the
    /// designated due date unknown (L4010, a warning). Fail closed: the
    /// notice's date is the only measure, so the act is neither timely nor
    /// late until the matter records it.
    fn unjudged(&mut self, k: usize, id: &str, due: &Due, why: String, how: String) {
        let event = self.sorted[k].1;
        let trigger = self.sorted[due.trigger].1;
        let trigger_at = self.at(due.trigger, |e| e.whole.as_ref());
        let mut e = error(
            C::TimelinessUnknown,
            self.at(k, |e| e.date.as_ref()),
            format!(
                "`{}` on {} took deadline `{id}`, but `{}` left its due date (`{}`) unknown: whether it is timely or late is unknown",
                display(&event.action),
                event.date,
                display(&trigger.action),
                due.designated.as_deref().unwrap_or_default(),
            ),
        )
        .note_at(Some(trigger_at), why)
        .note(how)
        .note("record the due date the notice designates in place of `?` to judge it");
        e.severity = Severity::Warning;
        self.out.push(e);
    }

    /// No version of deadline `id` (logical `q`) governs event `k`: L3007,
    /// or L3010 (a warning) before the declaration's `unknown_before`,
    /// where its law is not encoded and the act is judged against no
    /// period.
    fn ungoverned(&mut self, k: usize, id: &str, q: &str) {
        let date = &self.sorted[k].1.date;
        let unknown = self
            .graph
            .world
            .versions(&crate::case::graph::Graph::key(q))
            .unknown_before()
            .filter(|b| date < b);
        let e = match unknown {
            Some(before) => {
                let mut e = error(
                    C::NotEncoded,
                    self.at(k, |e| e.date.as_ref()),
                    format!("deadline `{id}` is not encoded before {before}: its act on {date} is not judged"),
                );
                e.severity = Severity::Warning;
                e
            }
            None => error(
                C::NoVersionInForce,
                self.at(k, |e| e.date.as_ref()),
                format!("deadline `{id}` has no version governing {date}"),
            ),
        };
        self.out.push(e);
    }

    /// Event `k` takes deadline `id` whose trigger predates the law encoded
    /// for it (L4010, a warning): fail closed, the act is neither timely nor late.
    fn unencoded(&mut self, k: usize, id: &str, i: usize, earliest: &str) {
        let event = self.sorted[k].1;
        let o = &self.obligations[i];
        let trigger = self.sorted[o.at].1;
        let trigger_at = self.at(o.at, |e| e.whole.as_ref());
        let mut e = error(
            C::TimelinessUnknown,
            self.at(k, |e| e.date.as_ref()),
            format!(
                "`{}` on {} took deadline `{id}`, but `{}` on {} started it before its law is encoded: whether it is timely or late is unknown",
                display(&event.action), event.date, display(&trigger.action), trigger.date,
            ),
        )
        .note_at(Some(trigger_at), crate::temporal::not_encoded(earliest));
        e.severity = Severity::Warning;
        self.out.push(e);
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
