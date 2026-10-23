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
use crate::case::graph::{display, pack_of, Graph};
use crate::case::spans::{error, Build};
use crate::case::{Effect, Obligation, Status};
use crate::hir::{
    Capability, Deadline, Expr, ExprKind, Extension, ExtensionWindow, TriggerPolicy, Value,
};
use crate::temporal::condition::matches;
use crate::temporal::MatterEvent;
use litigate_base::{Diagnostic, DiagnosticCode as C, Severity};

/// How an `extends`/`waives` event transforms an obligation.
#[derive(Debug, Clone, Copy)]
pub(super) enum Transform<'e> {
    /// `extends d to p`, with the event's argument for `p`.
    Extend(Option<&'e Expr>),
    /// `waives d`.
    Waive,
}

impl Transform<'_> {
    /// The capability it needs, as a verb.
    fn verb(self) -> &'static str {
        match self {
            Self::Extend(_) => "extend",
            Self::Waive => "waive",
        }
    }

    /// The verb's participle.
    fn done(self) -> &'static str {
        match self {
            Self::Extend(_) => "extended",
            Self::Waive => "waived",
        }
    }
}

/// What licenses a transform, read off one `extend` grant or the `waive`
/// capability of a deadline: who holds it, whether it reaches an instance
/// before and after its due date, and its numeric ceiling, if any.
#[derive(Debug, Clone, Copy)]
struct Grant<'g> {
    by: &'g str,
    before: bool,
    after: bool,
    /// Calendar days past the counted due date; `None` under the court's
    /// discretion (and for a waiver, which has no extent).
    ceiling: Option<u64>,
}

impl<'g> Grant<'g> {
    /// An `extend` grant; `None` for `extend { never }`.
    fn extend(x: &'g Extension) -> Option<Self> {
        match x {
            Extension::Bounded {
                by,
                max_days,
                after_expiry,
            } => Some(Self {
                by,
                before: true,
                after: *after_expiry,
                ceiling: Some(*max_days),
            }),
            Extension::Discretionary { by, window, .. } => {
                let after = *window == ExtensionWindow::AfterExpiryOnMotion;
                Some(Self {
                    by,
                    before: !after,
                    after,
                    ceiling: None,
                })
            }
            Extension::NonExtendable => None,
        }
    }

    /// The `waive` capability.
    fn waive(c: &'g Capability) -> Self {
        Self {
            by: &c.by,
            before: true,
            after: c.after_expiry,
            ceiling: None,
        }
    }

    /// Whether it may be exercised on an instance that has (`expired`) or
    /// has not passed its due date.
    fn reaches(self, expired: bool) -> bool {
        if expired {
            self.after
        } else {
            self.before
        }
    }
}

/// `pack::name` of qualified `q`, an instance pack read as its base.
pub(super) fn logical(q: &str) -> String {
    let k = Graph::key(q);
    format!("{}::{}", k.pack, k.name)
}

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
        let found: Vec<(String, String, &'g Deadline)> = self
            .graph
            .deadlines()
            .into_iter()
            .filter_map(|q| {
                let (w, d) = self.governing(&q, k, false)?;
                Some((q, w, d))
            })
            .collect();
        for (q, w, d) in &found {
            let view = self.view(pack_of(w), k);
            let candidates = match &view.kind {
                ExprKind::Call { args, .. } if matches(&d.trigger, view) => {
                    deadline::variant_args(args)
                }
                _ => continue,
            };
            fired = true;
            let displacing = self.graph.claims(q, date, known_at).iter().any(|x| {
                found
                    .iter()
                    .any(|(other, ow, _)| other != q && other == x && ow == w)
            });
            if displacing {
                continue;
            }
            self.create(q, pack_of(w), d, k, &candidates);
        }
        fired
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
    fn create(&mut self, q: &str, pack: &str, d: &'g Deadline, k: usize, candidates: &[String]) {
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
                status: Status::Open,
                closed_by: None,
                at: k,
                candidates: candidates.to_vec(),
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
    /// duty, L4009) or lapses (a power).
    pub(super) fn at_as_of(&mut self) {
        for i in 0..self.obligations.len() {
            if self.obligations[i].status != Status::Open {
                continue;
            }
            let Some(due) = self.due_of(i, self.sorted.len(), true) else {
                continue;
            };
            let Ok(date) = effective(&self.obligations[i], &due) else {
                continue;
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
            .note("add the order or rule after the party, e.g. `cite FRCP 6(b)(1)(A)`");
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
        let new = match how {
            Transform::Extend(Some(Expr {
                kind:
                    ExprKind::Literal {
                        value: Value::Date(new),
                    },
                ..
            })) => Some(new.as_str()),
            Transform::Extend(_) => return None,
            Transform::Waive => None,
        };
        // Each grant licenses the transform when every open instance is
        // within its window and bound; otherwise the most specific refusal
        // (past the window, out of bound) is reported.
        let refusals: Option<Vec<(bool, Diagnostic)>> = grants
            .iter()
            .map(|&grant| {
                open.iter()
                    .zip(&dues)
                    .find_map(|(&i, due)| self.problem(k, i, due, grant, new))
            })
            .collect();
        if let Some((_, e)) = refusals.and_then(|r| r.into_iter().max_by_key(|(bound, _)| *bound)) {
            self.out.push(e);
            return None;
        }
        let Some(new) = new else {
            self.close(&open, Status::Waived, source);
            return Some(Effect::Waived);
        };
        for i in open {
            self.obligations[i].extended_to = Some(new.to_owned());
        }
        Some(Effect::Extended)
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
        let grants: Vec<Grant<'g>> = match how {
            Transform::Extend(_) => d.extend.iter().filter_map(Grant::extend).collect(),
            Transform::Waive => d.waive.iter().map(Grant::waive).collect(),
        };
        if grants.is_empty() {
            let never = d.extend.contains(&Extension::NonExtendable);
            let e = if never && matches!(how, Transform::Extend(_)) {
                error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    format!("`{action}` cannot extend `{q}`: the deadline may not be extended (`extend {{ never }}`)"),
                )
                .note("no order, stipulation or consent extends a period its rule forbids extending (FRCP 6(b)(2))")
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

    /// Why event `k` may not transform instance `i` (counted `due`) under
    /// `grant`: extend it to `new`, or waive it when `new` is `None`. The
    /// flag is set when the refusal is the bound, not the window.
    fn problem(
        &self,
        k: usize,
        i: usize,
        due: &Due,
        grant: Grant<'_>,
        new: Option<&str>,
    ) -> Option<(bool, Diagnostic)> {
        let event = self.sorted[k].1;
        let o = &self.obligations[i];
        let how = deadline::explain(due, &self.sorted[o.at].1.action);
        let reject = |bound: bool, message: String| {
            Some((
                bound,
                error(
                    C::InvalidCaseEvent,
                    self.at(k, |e| e.action.as_ref()),
                    message,
                )
                .note(how.clone()),
            ))
        };
        let (Ok(current), Ok(counted)) = (effective(o, due), floored(o, due)) else {
            let why = due.date.clone().err().unwrap_or_default();
            return reject(
                true,
                format!(
                    "cannot count `{}` to judge `{}`",
                    o.deadline,
                    display(&event.action)
                ),
            )
            .map(|(b, e)| (b, e.note(why)));
        };
        let expired = event.date > current;
        if !grant.reaches(expired) {
            let message = if expired {
                format!(
                    "`{}` expired on {current}, before `{}` on {}; the capability does not reach an expired period (no `after_expiry`)",
                    o.deadline,
                    display(&event.action),
                    event.date
                )
            } else {
                format!(
                    "`{}` is due {current}, not expired when `{}` comes on {}; the grant reaches only an expired period, on motion",
                    o.deadline,
                    display(&event.action),
                    event.date
                )
            };
            return reject(false, message);
        }
        let new = new?;
        if new <= current.as_str() {
            return reject(
                true,
                format!(
                    "an extension to {new} does not extend `{}`, already due {current}",
                    o.deadline
                ),
            );
        }
        let max = grant.ceiling?;
        let bound = crate::types::date::add_days(&counted, max).unwrap_or_default();
        if new > bound.as_str() {
            return reject(
                true,
                format!(
                    "an extension to {new} exceeds the bound of `{}`: due {counted} + {max} days = {bound}",
                    o.deadline
                ),
            );
        }
        None
    }
}
