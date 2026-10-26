//! Obligation instances (SPEC §5.6; design review §2.7). A deadline's
//! trigger creates an instance per bearing party, under the deadline's
//! trigger policy. An instance is consumed exactly once: discharged by a
//! timely act through a `within` edge (`within.rs`), breached by a late
//! one, waived, superseded by a later trigger, or absorbed by a
//! `later_of` deadline. A duty's instance borne by a party and still open
//! past its due date at `as_of` is breached (L4009). Any other instance
//! past due lapses: a power's, or a period no party bears, since a breach
//! is a liability of its bearer. An extension or a waiver
//! is a typed transform of one instance, licensed by one of the deadline's
//! `extend` grants or its `waive` capability: its principal, its window,
//! and its bound (a numeric cap, the court's discretion, or `never`).
use super::Cx;
use crate::case::deadline::{self, Due};
use crate::case::grant::{self, Grant, Instance};
use crate::case::graph::{display, pack_of, Graph};
use crate::case::spans::error;
use crate::case::{Effect, Obligation, Status};
use crate::hir::{Deadline, Declaration, Expr, ExprKind, TriggerPolicy, Undesignated, Value};
use crate::temporal::condition::matches;
use crate::temporal::MatterEvent;
use crate::types::refinement;
use litigate_base::{Diagnostic, DiagnosticCode as C, Severity};

/// How an `extends`/`waives` event transforms an obligation.
#[derive(Debug, Clone, Copy)]
pub(super) enum Transform<'e> {
    /// `extends d to p`, with the event's argument for `p`, its type and
    /// arguments.
    Extend(Option<&'e Expr>, Refined<'e>),
    /// `waives d`.
    Waive,
}

/// An `extends` event's type and arguments: its refinements naming a
/// bound date are judged per instance (SPEC §3.5).
#[derive(Debug, Clone, Copy)]
pub(super) struct Refined<'e> {
    /// The event's type.
    pub ty: &'e crate::hir::EventType,
    /// The call's arguments.
    pub args: &'e [Expr],
}

impl<'e> Transform<'e> {
    /// An extension's type and arguments.
    const fn refined(self) -> Option<Refined<'e>> {
        match self {
            Self::Extend(_, by) => Some(by),
            Self::Waive => None,
        }
    }

    /// The capability it needs, as a verb.
    fn verb(self) -> &'static str {
        match self {
            Self::Extend(..) => "extend",
            Self::Waive => "waive",
        }
    }

    /// The verb's participle.
    fn done(self) -> &'static str {
        match self {
            Self::Extend(..) => "extended",
            Self::Waive => "waived",
        }
    }
}

/// `pack::name` of qualified `q`, an instance pack read as its base.
pub(super) fn logical(q: &str) -> String {
    let k = Graph::key(q);
    format!("{}::{}", k.pack, k.name)
}

/// How many chained periods a trigger is followed through.
const CHAIN_DEPTH: usize = 16;

/// The due date of `o` given its counted `due`: an extension replaces it;
/// otherwise a `later_of` floor can only make it later.
pub(super) fn effective(o: &Obligation, due: &Due) -> Result<String, String> {
    match &o.extended_to {
        Some(x) => Ok(x.clone()),
        None => floored(o, due),
    }
}

/// The counted due date of `o`, raised to its `later_of` floor.
fn floored(o: &Obligation, due: &Due) -> Result<String, String> {
    match (&due.date, &o.floor) {
        (Ok(c), Some(f)) if f > c => Ok(f.clone()),
        (date, _) => date.clone(),
    }
}

impl<'g> Cx<'_, 'g> {
    /// Fires every in-force deadline trigger that event `k` matches;
    /// whether any matched. A claimant displacing a deadline that also
    /// fires opens no instance of its own: the displaced deadline's
    /// instance is counted under it (SPEC §5.5).
    pub(super) fn triggers(&mut self, k: usize) -> bool {
        let known_at = self.case.known_at.as_deref();
        let date = self.sorted[k].1.date.as_str();
        let mut fired = false;
        let mut unencoded = Vec::new();
        let found: Vec<(String, String, &'g Deadline)> = self
            .graph
            .deadlines()
            .into_iter()
            .filter_map(|q| {
                let found = self.governing(&q, k, false).map(|(w, d)| (q.clone(), w, d));
                if found.is_none() {
                    unencoded.push(q);
                }
                found
            })
            .collect();
        for q in &unencoded {
            fired |= self.not_encoded(q, k);
        }
        for (q, w, d) in &found {
            let Some((root, pattern)) = self.root(w, d, k, CHAIN_DEPTH) else {
                continue;
            };
            // The pattern is written in its own deadline's package, a
            // dependency's for a chain on its period.
            let view = self.view(pack_of(&root), k);
            let candidates = match &view.kind {
                ExprKind::Call { args, .. } if matches(pattern, view) => {
                    deadline::variant_args(args)
                }
                _ => continue,
            };
            // A notice designating less time than its rule allows is
            // rejected (37 C.F.R. § 2.105(a)); its instance has no due date.
            if let Some(Err(Undesignated::BelowFloor(why))) = d.designation(date, view) {
                let e = error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    why,
                )
                .note("a designated due date must allow at least the deadline's own period; correct the date as the notice gives it");
                self.out.push(e);
            } else if let Some(why) = d.interpreted_shortfall(date, view) {
                // Where the rule names no start for its least time (37
                // C.F.R. § 2.106(b)(3)(iii)), the date the notice gives
                // stands: only an interpretation finds it short.
                let mut e = error(
                    C::DesignationShortOfInterpretation,
                    self.at(k, |e| e.action.as_ref()),
                    why,
                )
                .note("check the date as the notice gives it; if it is right, the interpretation, not the notice, is in question");
                e.severity = Severity::Warning;
                self.out.push(e);
            }
            fired = true;
            let displacing = self.graph.claims(q, date, known_at).iter().any(|x| {
                found
                    .iter()
                    .any(|(other, ow, _)| other != q && other == x && ow == w)
            });
            if displacing {
                continue;
            }
            self.create(q, pack_of(w), d, k, &candidates, None);
        }
        fired | self.unencoded_triggers(k, &found)
    }

    /// Opens an instance, due date unknown, of every deadline whose
    /// trigger event `k` matches but whose law is not encoded on its date
    /// (`unknown_before`, SPEC §5.4), with a warning; whether any did.
    /// Fail closed: a period the corpus does not know is neither absent
    /// nor counted under a later version.
    fn unencoded_triggers(&mut self, k: usize, found: &[(String, String, &'g Deadline)]) -> bool {
        let known_at = self.case.known_at.as_deref();
        let date = self.sorted[k].1.date.clone();
        let mut fired = false;
        for q in self.graph.deadlines() {
            if found.iter().any(|(f, _, _)| *f == q) {
                continue;
            }
            let Some((earliest, Declaration::Deadline(d))) =
                self.graph.unencoded(&q, &date, known_at)
            else {
                continue;
            };
            let view = self.view(pack_of(&q), k);
            let candidates = match &view.kind {
                ExprKind::Call { args, .. } if d.chain.is_none() && matches(&d.trigger, view) => {
                    deadline::variant_args(args)
                }
                _ => continue,
            };
            let mut e = error(
                C::NoVersionInForce,
                self.at(k, |e| e.action.as_ref()),
                format!(
                    "`{}` on {date} starts deadline `{q}`, which is not encoded before {earliest}: its due date is unknown",
                    display(view)
                ),
            )
            .note(crate::temporal::not_encoded(&earliest));
            e.severity = Severity::Warning;
            self.out.push(e);
            self.create(&q, pack_of(&q), d, k, &candidates, Some(&earliest));
            fired = true;
        }
        fired
    }

    /// The event pattern that starts deadline `d` (logical `q`) at event
    /// `k`, with the qualified deadline whose package it is written in:
    /// its own trigger, or for a chained period the one starting the
    /// period it follows (at most `depth` links deep; a longer chain is a
    /// cycle, which no event starts).
    fn root(&self, q: &str, d: &'g Deadline, k: usize, depth: usize) -> Option<(String, &'g Expr)> {
        let Some(chain) = &d.chain else {
            return Some((q.to_owned(), &d.trigger));
        };
        let base = self.graph.qualify(pack_of(q), &chain.deadline);
        let (w, base) = self.governing(&base, k, false)?;
        self.root(&w, base, k, depth.checked_sub(1)?)
    }

    /// L3010 when event `k` matches the trigger of a version of deadline
    /// `q`, none of which governs its date because the law of `q` is not
    /// encoded before the declaration's `unknown_before`: the matter has a
    /// period there the corpus cannot count, a hole rather than no period.
    /// Whether it reported one.
    fn not_encoded(&mut self, q: &str, k: usize) -> bool {
        let event = self.sorted[k].1;
        let set = self.graph.world.versions(&Graph::key(q));
        let Some(before) = set.unknown_before().filter(|b| event.date < *b) else {
            return false;
        };
        let pack = pack_of(q);
        let view = self.view(pack, k);
        let triggered = set.versions.iter().any(|v| {
            matches!(self.graph.world.decls[v.decl].decl, Declaration::Deadline(d) if matches(&d.trigger, view))
        });
        if triggered {
            let mut e = error(
                C::NotEncoded,
                self.at(k, |e| e.date.as_ref()),
                format!(
                    "`{}` on {} triggers `{}`, whose law is not encoded before {before}: no due date is counted",
                    display(&event.action),
                    event.date,
                    logical(q)
                ),
            )
            .note(format!("`unknown_before {before}` marks the earlier law as a hole in the corpus; encode the version in force on {} to count it", event.date));
            e.severity = Severity::Warning;
            self.out.push(e);
        }
        triggered
    }

    /// The deadline declaration governing logical deadline `q` at event
    /// `at` (`to_as_of`: of the events followed by the `as_of` sentinel),
    /// with the qualified id of its own logical declaration: `q`, or the
    /// claimant that displaces it, judged in its own package's view under
    /// its own transitions (SPEC §5.5).
    pub(super) fn governing(
        &self,
        q: &str,
        at: usize,
        to_as_of: bool,
    ) -> Option<(String, &'g Deadline)> {
        let known_at = self.case.known_at.as_deref();
        let governing = |pack: &str, id: &str| {
            let events = if to_as_of {
                self.events_to_as_of(pack)
            } else {
                self.events(pack)
            };
            deadline::governing(self.graph, pack, id, &events, at, known_at)
        };
        let (w, d) = governing(pack_of(q), q)?;
        if pack_of(&w) == pack_of(q) {
            Some((w, d))
        } else {
            governing(pack_of(&w), &w)
        }
    }

    /// Instances of deadline `q` (in `pack`, version `d`) for event `k`.
    fn create(
        &mut self,
        q: &str,
        pack: &str,
        d: &'g Deadline,
        k: usize,
        candidates: &[String],
        unencoded: Option<&str>,
    ) {
        let source = self.sorted[k].0;
        for bearer in self.bearers(d, k) {
            let open = self.open(q, |b| b == bearer.as_deref());
            match &d.policy {
                TriggerPolicy::First if !open.is_empty() => continue,
                TriggerPolicy::Latest | TriggerPolicy::LaterOf(_) => {
                    self.close(&open, Status::Superseded, source);
                }
                TriggerPolicy::First | TriggerPolicy::Each => {}
            }
            let floor = match &d.policy {
                TriggerPolicy::LaterOf(other) => {
                    let other = logical(&self.graph.qualify(pack, other));
                    self.absorb(&other, bearer.as_deref(), k)
                }
                _ => None,
            };
            self.obligations.push(Obligation {
                deadline: q.to_owned(),
                trigger: source,
                date: self.sorted[k].1.date.clone(),
                bearer,
                linear: d.linear,
                floor,
                extended_to: None,
                extended_under: None,
                request_made: None,
                pending: None,
                unencoded: unencoded.map(str::to_owned),
                status: Status::Open,
                closed_by: None,
                at: k,
                candidates: candidates.to_vec(),
                answered: None,
            });
        }
    }

    /// The parties bearing a new instance of `d` triggered by event `k`:
    /// the event's own party when it plays `d`'s bearer, else every party
    /// that does; one unbound instance when `d` names no bearer or no
    /// party plays it.
    fn bearers(&self, d: &Deadline, k: usize) -> Vec<Option<String>> {
        let Some(actor) = &d.bearer else {
            return vec![None];
        };
        let party = &self.sorted[k].1.party;
        let players: Vec<&str> = self
            .case
            .parties
            .iter()
            .filter(|p| p.actor == *actor)
            .map(|p| p.id.as_str())
            .collect();
        if players.contains(&party.as_str()) {
            return vec![Some(party.clone())];
        }
        if players.is_empty() {
            return vec![None];
        }
        players.into_iter().map(|p| Some(p.to_owned())).collect()
    }

    /// Indices of the open instances of logical deadline `q` whose bearer
    /// satisfies `bearer`, oldest first.
    pub(super) fn open(&self, q: &str, bearer: impl Fn(Option<&str>) -> bool) -> Vec<usize> {
        (0..self.obligations.len())
            .filter(|&i| {
                let o = &self.obligations[i];
                o.status == Status::Open && o.deadline == q && bearer(o.bearer.as_deref())
            })
            .collect()
    }

    /// Ends instances `which` with `status`, by the event at source `index`.
    pub(super) fn close(&mut self, which: &[usize], status: Status, index: usize) {
        for &i in which {
            self.obligations[i].status = status;
            self.obligations[i].closed_by = Some(index);
        }
    }

    /// The open instance of `other` borne by `bearer` (or unbound), which a
    /// `later_of` instance created by event `k` absorbs: its due date.
    fn absorb(&mut self, other: &str, bearer: Option<&str>, k: usize) -> Option<String> {
        let i = *self.open(other, |b| b.is_none() || b == bearer).first()?;
        let floor = self
            .due_of(i, k, false)
            .and_then(|due| effective(&self.obligations[i], &due).ok());
        let source = self.sorted[k].0;
        self.close(&[i], Status::Merged, source);
        floor
    }

    /// The counted due date of instance `i`, judged at event `at`
    /// (`to_as_of`: of the events followed by the `as_of` sentinel) under
    /// the deadline declaration governing it; `None` when none does.
    pub(super) fn due_of(&self, i: usize, at: usize, to_as_of: bool) -> Option<Due> {
        if self.obligations[i].unencoded.is_some() {
            return None;
        }
        let (w, d) = self.governing(&self.obligations[i].deadline, at, to_as_of)?;
        let pack = pack_of(&w);
        let events = if to_as_of {
            self.events_to_as_of(pack)
        } else {
            self.events(pack)
        };
        Some(self.count(i, pack, d, &events, at))
    }

    /// The counted due date of instance `i` under deadline version `d`
    /// (declared in `pack`), judged at `events[at]`.
    pub(super) fn count(
        &self,
        i: usize,
        pack: &str,
        d: &Deadline,
        events: &[MatterEvent<'_>],
        at: usize,
    ) -> Due {
        let o = &self.obligations[i];
        deadline::due(
            self.graph,
            self.clock,
            pack,
            d,
            events,
            (o.at, &o.candidates),
            at,
            self.case.known_at.as_deref(),
        )
    }

    /// At `as_of`: every open instance past its due date is breached (a
    /// duty, L4009) or lapses (a power); one whose trigger left its
    /// designated due date unknown is unknown.
    pub(super) fn at_as_of(&mut self) {
        for i in 0..self.obligations.len() {
            if self.obligations[i].status != Status::Open {
                continue;
            }
            let Some(due) = self.due_of(i, self.sorted.len(), true) else {
                continue;
            };
            let date = match effective(&self.obligations[i], &due) {
                Ok(date) => date,
                // The trigger left the designated date unknown (`?`): the
                // instance is neither open nor lapsed, but unknown.
                Err(_) if due.unknown => {
                    self.obligations[i].status = Status::Unknown;
                    continue;
                }
                Err(_) => continue,
            };
            if self.case.as_of <= date {
                continue;
            }
            let how = deadline::explain(&due, &self.sorted[due.trigger].1.action);
            let span = self.at(self.obligations[i].at, |e| e.whole.as_ref());
            let o = &mut self.obligations[i];
            let Some(bearer) = o.bearer.as_ref().filter(|_| o.linear) else {
                o.status = Status::Lapsed;
                continue;
            };
            o.status = Status::Breached;
            let message = format!(
                "obligation `{}` borne by `{bearer}`, created by `{}` on {}, was neither discharged nor transformed by its due date {date}: at as_of {} it is breached",
                o.deadline,
                display(&self.sorted[o.at].1.action),
                o.date,
                self.case.as_of,
            );
            let mut d = error(C::ObligationBreached, span, message)
            .note(how)
            .note("a duty is consumed exactly once: by a timely act through a `within` edge, or by a breach");
            d.severity = Severity::Warning;
            self.out.push(d);
        }
    }

    /// Event `k`, whose type (declared in `pack`) extends or waives
    /// deadline `target`, transforms every open instance of it (an order
    /// extending "the time to answer" reaches each defendant's), all or
    /// none; the effect when it does.
    pub(super) fn transform(
        &mut self,
        k: usize,
        pack: &str,
        target: &str,
        how: Transform<'_>,
    ) -> Option<Effect> {
        let (source, event) = self.sorted[k];
        let verb = how.verb();
        let action = display(&event.action);
        if event.authority.is_none() {
            let d = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.action.as_ref()),
                format!("`{action}` transforms an obligation only when it cites its authority"),
            )
            .note("add the order or rule after the party, e.g. `cite \"FRCP 6(b)(1)(A)\"`");
            self.out.push(d);
            return None;
        }
        let q = logical(&self.graph.qualify(pack, target));
        let open = self.open(&q, |_| true);
        if open.is_empty() {
            let d = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.action.as_ref()),
                format!("`{action}` has no open obligation of `{q}` to {verb}"),
            )
            .note("an obligation opens when its deadline's trigger happens, and closes when it is discharged, breached, waived, or superseded");
            self.out.push(d);
            return None;
        }
        let (w, d, grants) = self.licence(k, &q, how)?;
        let pack = pack_of(&w);
        let events = self.events(pack);
        let dues: Vec<Due> = open
            .iter()
            .map(|&i| self.count(i, pack, d, &events, k))
            .collect();
        drop(events);
        let new = self.new_due(k, &q, how).ok()?.zip(how.refined());
        // Each grant licenses the transform when every open instance is
        // within its window and bound; the first that does is recorded,
        // with the request each instance's grant read. Otherwise the most
        // specific refusal (out of bound, then no motion, then past the
        // window) is reported.
        let mut refusals = Vec::new();
        let mut licensed = None;
        for &grant in &grants {
            let judged: Result<Vec<Option<String>>, _> = open
                .iter()
                .zip(&dues)
                .map(|(&i, due)| self.problem(pack, k, i, due, grant, new))
                .collect();
            match judged {
                Ok(requests) => {
                    licensed = Some((grant, requests));
                    break;
                }
                Err(refusal) => refusals.push(refusal),
            }
        }
        let Some((grant, requests)) = licensed else {
            let refusal = refusals.into_iter().max_by_key(|(rank, _)| *rank);
            self.out.extend(refusal.map(|(_, e)| e));
            return None;
        };
        let Some((new, _)) = new else {
            self.close(&open, Status::Waived, source);
            return Some(Effect::Waived);
        };
        for (i, request) in open.into_iter().zip(requests) {
            let o = &mut self.obligations[i];
            o.extended_to = Some(new.to_owned());
            o.extended_under = grant.standard;
            o.request_made = request;
            o.answered = Some(k);
        }
        if let Some(why) = grant.interpretation {
            // The rule reaches the period only as the author reads it (FRCP
            // 6(b) through 37 C.F.R. § 2.116(a), "wherever applicable and
            // appropriate"): the extension stands, with a warning.
            let mut e = error(
                C::ExtensionByInterpretation,
                self.at(k, |e| e.action.as_ref()),
                format!(
                    "`{action}` on {} extends `{q}` to {new} under a grant that rests on an interpretation: {why}",
                    event.date
                ),
            )
            .note("the order stands; whether the rule reaches the period at all is the interpretation's, not the order's");
            e.severity = Severity::Warning;
            self.out.push(e);
        }
        Some(Effect::Extended)
    }

    /// The date event `k` moves `q`'s instances to, `None` for a waiver;
    /// `Err` after reporting an extension to no known date.
    fn new_due<'e>(
        &mut self,
        k: usize,
        q: &str,
        how: Transform<'e>,
    ) -> Result<Option<&'e str>, ()> {
        match how {
            Transform::Extend(
                Some(Expr {
                    kind:
                        ExprKind::Literal {
                            value: Value::Date(new),
                        },
                    ..
                }),
                _,
            ) => Ok(Some(new.as_str())),
            Transform::Extend(..) => {
                let action = display(&self.sorted[k].1.action);
                let d = error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    format!("`{action}` extends `{q}` to no known date, which moves nothing"),
                )
                .note("an extension's new due date is the order's; record it, or omit the event until it is known");
                self.out.push(d);
                Err(())
            }
            Transform::Waive => Ok(None),
        }
    }

    /// The deadline declaration governing `q` at event `k` (with its own
    /// logical id, as [`Self::governing`]) and the grants for `how` that
    /// event `k`'s party holds.
    fn licence(
        &mut self,
        k: usize,
        q: &str,
        how: Transform<'_>,
    ) -> Option<(String, &'g Deadline, Vec<Grant<'g>>)> {
        let event = self.sorted[k].1;
        let action = display(&event.action);
        let verb = how.verb();
        let Some((w, d)) = self.governing(q, k, false) else {
            let e = error(
                C::NoVersionInForce,
                self.at(k, |e| e.date.as_ref()),
                format!("deadline `{q}` has no version governing {}", event.date),
            );
            self.out.push(e);
            return None;
        };
        let grants = self.grants(&w, d, &event.date, how);
        if grants.is_empty() {
            let lock = d.extend.iter().find(|x| x.lock().is_some());
            let e = if let (Some(lock), Transform::Extend(..)) = (lock, how) {
                error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    format!("`{action}` cannot extend `{q}`: the deadline may not be extended (`{}`)", lock.lock_source().unwrap_or_default()),
                )
                .note(format!("no order, stipulation or consent extends a period its rule forbids extending ({})", lock.lock().unwrap_or_default()))
            } else {
                error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    format!("`{action}` cannot {verb} `{q}`: the deadline grants no `{verb}` capability"),
                )
                .note(format!(
                    "a limit that grants no `{verb}` capability cannot be {}: no order has power over it (Bowles v. Russell, 551 U.S. 205 (2007))",
                    how.done()
                ))
            };
            self.out.push(e);
            return None;
        }
        let actor = self.actor(event);
        let held: Vec<Grant<'g>> = grants
            .iter()
            .copied()
            .filter(|g| Some(g.by) == actor)
            .collect();
        if held.is_empty() {
            let mut by: Vec<&str> = grants.iter().map(|g| g.by).collect();
            by.sort_unstable();
            by.dedup();
            let e = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.party.as_ref()),
                format!(
                    "only {} may {verb} `{q}`, but `{}` is {}",
                    by.join(" or "),
                    event.party,
                    actor.map_or_else(
                        || "not a party acting by the event".to_owned(),
                        str::to_owned
                    )
                ),
            );
            self.out.push(e);
            return None;
        }
        Some((w, d, held))
    }

    /// The grants for `how` that deadline `d` (logical `w`) declares, then
    /// those of the emergency grants in force on `date` reaching it; an
    /// emergency extension reaches a deadline locked against its grant's
    /// authority only when a grant also lifts the lock (`unlock`).
    pub(super) fn grants(
        &self,
        w: &str,
        d: &'g Deadline,
        date: &str,
        how: Transform<'_>,
    ) -> Vec<Grant<'g>> {
        match how {
            Transform::Extend(..) => self.extend_grants(w, d, date),
            Transform::Waive => {
                let emergency =
                    self.graph
                        .emergency(w, &d.meta.authority, date, self.case.known_at.as_deref());
                d.waive
                    .iter()
                    .chain(emergency.iter().filter_map(|g| g.waive.as_ref()))
                    .map(Grant::waive)
                    .collect()
            }
        }
    }

    /// The `extend` grants of deadline `d` (logical `w`) on `date`: its own,
    /// then the emergency grants in force reaching it (see [`Self::grants`]).
    /// Needs no extending event, so a pending request can read them.
    pub(super) fn extend_grants(&self, w: &str, d: &'g Deadline, date: &str) -> Vec<Grant<'g>> {
        let emergency =
            self.graph
                .emergency(w, &d.meta.authority, date, self.case.known_at.as_deref());
        let unlocked = emergency.iter().any(|g| g.unlock);
        let reaches = |g: &&&crate::hir::Grant| {
            unlocked || !d.extend.iter().any(|x| x.locks(&g.meta.authority))
        };
        d.extend
            .iter()
            .chain(emergency.iter().filter(reaches).flat_map(|g| &g.extend))
            .filter_map(Grant::extend)
            .collect()
    }

    /// Whether event `k` may transform instance `i` (counted `due`) under
    /// `grant`: extend it to `new`, or waive it when `new` is `None`
    /// ([`grant::judge`]). `Ok` carries the date of the motion the grant
    /// read, if it reads one; `Err` the refusal, ranked by how specific it
    /// is: the window (0), the motion (1), the bound (2), the event type's
    /// refinements (3).
    fn problem(
        &self,
        pack: &str,
        k: usize,
        i: usize,
        due: &Due,
        grant: Grant<'_>,
        new: Option<(&str, Refined<'_>)>,
    ) -> Result<Option<String>, (u8, Diagnostic)> {
        let event = self.sorted[k].1;
        let o = &self.obligations[i];
        let how = deadline::explain(due, &self.sorted[o.at].1.action);
        let reject = |rank: u8, message: String| {
            (
                rank,
                error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    message,
                )
                .note(how.clone()),
            )
        };
        let (Ok(current), Ok(counted)) = (effective(o, due), floored(o, due)) else {
            let why = due.date.clone().err().unwrap_or_default();
            let (rank, e) = reject(
                2,
                format!(
                    "cannot count `{}` to judge `{}`",
                    o.deadline,
                    display(&event.action)
                ),
            );
            return Err((rank, e.note(why)));
        };
        let history: Vec<grant::Moment<'_>> = (0..self.sorted.len())
            .map(|j| {
                let e = self.sorted[j].1;
                (e.date.as_str(), &e.action, self.view(pack, j))
            })
            .collect();
        let instance = Instance {
            deadline: &o.deadline,
            from: &self.sorted[o.at].1.date,
            prescribed: &counted,
            current: &current,
            spent: o.request_made.as_deref(),
        };
        let judged = grant::judge(&history, k, instance, grant, new.map(|(n, _)| n));
        // A refinement is judged after the window, the motion and the
        // extension's direction, before the bound (rank 3: the most
        // specific refusal).
        if let (Ok(_) | Err((2, _)), Some((new, by))) = (&judged, new) {
            if new > current.as_str() {
                if let Some(d) = self.unrefined(k, by, &counted, &current) {
                    return Err((3, d.note(how)));
                }
            }
        }
        judged.map_err(|(rank, why)| reject(rank, why))
    }

    /// Tier 1 at replay (SPEC §3.5): the first refinement of event `k`'s
    /// type naming a bound date that its arguments and the instance's
    /// dates (`counted` prescribed, `current` as last extended; the
    /// order's date `grant_order_entered`) do not satisfy, as L2013.
    fn unrefined(
        &self,
        k: usize,
        by: Refined<'_>,
        counted: &str,
        current: &str,
    ) -> Option<Diagnostic> {
        let event = self.sorted[k].1;
        let dates = [
            ("prescribed_due", counted),
            ("current_due", current),
            ("grant_order_entered", event.date.as_str()),
        ];
        let env = |n: &str| {
            let arg = by.ty.params.iter().position(|(p, _)| p == n);
            arg.and_then(|i| by.args.get(i))
                .and_then(|e| refinement::eval::term(e, &|_| None))
                .or_else(|| {
                    let date = dates.iter().find(|(d, _)| *d == n);
                    date.and_then(|(_, v)| refinement::eval::of(&Value::Date((*v).to_owned())))
                })
        };
        let r = by.ty.refinements.iter().find(|r| {
            refinement::names_bound_date(&r.predicate)
                && refinement::eval(&r.predicate, &env) == Some(false)
        })?;
        let name = &by.ty.name;
        Some(
            error(
                C::RefinementViolated,
                self.at(k, |e| e.action.as_ref()),
                refinement::message(name, r, &env),
            )
            .note(refinement::origin(name, r)),
        )
    }
}
