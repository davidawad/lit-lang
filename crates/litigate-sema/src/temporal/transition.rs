//! `transition` declarations: which version governs a proceeding pending
//! across an amendment (SPEC §5.4, DESIGN §4.4; FRCP 86(a) is written this
//! way in the stdlib, never in compiler code).
//!
//! By default every event is governed by the declaration the precedence
//! selection ([`crate::precedence::select`]) picks on its date: the version
//! in force, or the claimant that displaces it. When that version took
//! effect after the matter's first event (the matter
//! was pending when it did) and a transition amends that version, the
//! transition's arms decide instead: the `unless` whose condition holds on
//! the history up to the event selects its version, else `otherwise` does.
//! Arms are disjoint (L3008), so their order is never consulted, and the
//! history is the events that precede the evaluated one in docket order
//! (SPEC §5.6), never a later event of the same day. A discretionary leg
//! ("unless the court orders otherwise", "just and practicable") is an
//! `unless has_event(Order...)` arm the case satisfies with an order event.
use super::activity::Inactive;
use super::condition::{eval, first_foreign, has_event, jointly_satisfiable, matches};
use super::world::{error, span_of, DiagnosticExt, Key, Locate, World};
use crate::hir::{Declaration, Expr, Transition, TransitionArm};
use crate::precedence::select::{select, Unselected};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use std::collections::BTreeSet;

/// Where an event falls among the events of its date (SPEC §5.6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Seq {
    /// No docket sequence: ordered against other dates only.
    #[default]
    Unsequenced,
    /// `docket N`: after each same-day event with a smaller docket number.
    Docket(u64),
    /// The end of the day, after every event on it (the `as_of` point).
    EndOfDay,
}

/// One dated event of a matter (a complete event constructor call).
#[derive(Debug, Clone, Copy)]
pub struct MatterEvent<'e> {
    /// Event date.
    pub date: &'e str,
    /// Its place among the events of its date.
    pub seq: Seq,
    /// The event constructor call.
    pub action: &'e Expr,
}

impl<'e> MatterEvent<'e> {
    /// An unsequenced event on `date`.
    #[must_use]
    pub fn on(date: &'e str, action: &'e Expr) -> Self {
        Self {
            date,
            seq: Seq::Unsequenced,
            action,
        }
    }

    /// Whether `self` happened before `other`: on an earlier date, or on
    /// the same date earlier in docket order. Same-day events that are not
    /// both docketed are unordered, so neither precedes the other.
    #[must_use]
    pub fn precedes(&self, other: &Self) -> bool {
        self.date < other.date
            || (self.date == other.date
                && match (self.seq, other.seq) {
                    (Seq::Docket(a), Seq::Docket(b)) => a < b,
                    (Seq::Docket(_) | Seq::Unsequenced, Seq::EndOfDay) => true,
                    _ => false,
                })
    }
}

/// The version that governs an event, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Governing {
    /// The governing version (index into [`World::decls`]).
    pub version: usize,
    /// The transition that decided, and the index of its chosen arm.
    pub via: Option<(usize, usize)>,
}

/// Why no version governs an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ungoverned {
    /// No active version on the event's date (L3007).
    Inactive(Inactive),
    /// Claimants no relation or canon orders on the event's date (L3003).
    Unresolved(Vec<(usize, usize)>),
    /// A transition arm could not be evaluated or names no version (L3008).
    Transition(String),
}

/// The one transition (declaration index) amending `key@from`, if exactly
/// one does.
#[must_use]
pub fn transition_for(world: &World<'_>, key: &Key, from: &str) -> Option<usize> {
    world.transition_for(key, from)
}

/// The version governing `key` for `events[at]` in a matter whose events
/// are `events`: the selected declaration on the event's date, through the
/// transition amending it if the matter was pending. The transition reads
/// the history of `events[at]`: itself and every event that
/// [precedes](MatterEvent::precedes) it, whatever the slice order. A
/// declaration superseded by another logical declaration (`supersedes`
/// across declarations: an amendment layer's new version of a base
/// declaration, docs/litigate/CHANGE.md §2.2) is governed by the one
/// superseding it, as by a displacing claimant, to the end of the chain
/// (SPEC §5.4).
///
/// # Errors
/// [`Ungoverned`] when nothing is selected or a transition fails.
pub fn governing(
    world: &World<'_>,
    key: &Key,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Governing, Ungoverned> {
    governing_from(world, key, events, at, known_at, &mut BTreeSet::new())
}

/// [`governing`], following supersession from `key` through declarations
/// not yet `seen` (a version superseding an earlier one of its own
/// declaration leads back to it); where the superseding declaration
/// governs nothing on the date, `key`'s own reason stands.
fn governing_from(
    world: &World<'_>,
    key: &Key,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
    seen: &mut BTreeSet<Key>,
) -> Result<Governing, Ungoverned> {
    let found = governing_one(world, key, events, at, known_at);
    match &found {
        Err(Ungoverned::Inactive(Inactive::Superseded(s))) if seen.insert(key.clone()) => {
            governing_from(world, &world.key(s.by), events, at, known_at, seen).or(found)
        }
        _ => found,
    }
}

/// [`governing`] for `key` alone: its own versions and claimants.
fn governing_one(
    world: &World<'_>,
    key: &Key,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Governing, Ungoverned> {
    let event = events
        .get(at)
        .ok_or_else(|| Ungoverned::Transition(format!("no event #{at}")))?;
    let w = select(world, key, event.date, known_at).map_err(|e| match e {
        Unselected::Inactive(why) => Ungoverned::Inactive(why),
        Unselected::Unresolved(pairs) => Ungoverned::Unresolved(pairs),
    })?;
    let key = &world.key(w);
    let from = &world.decls[w].meta.effective.from;
    let pending = events
        .iter()
        .map(|e| e.date)
        .min()
        .is_some_and(|first| first < from.as_str());
    let unamended = Governing {
        version: w,
        via: None,
    };
    let (ti, t) = match (pending, world.transitions(key, from)) {
        (false, _) | (true, []) => return Ok(unamended),
        (true, [one]) => *one,
        (true, [(a, _), (b, _), ..]) => {
            return Err(Ungoverned::Transition(format!(
                "`{}` and `{}` both amend `{}@{from}`",
                world.decls[*a].meta.id,
                world.decls[*b].meta.id,
                key.display(&key.pack)
            )))
        }
    };
    let history: Vec<&Expr> = events
        .iter()
        .enumerate()
        .filter(|&(j, e)| j == at || e.precedes(event))
        .map(|(_, e)| e.action)
        .collect();
    let (n, version) = decide(world, (ti, t), key, &history)?;
    Ok(Governing {
        version,
        via: Some((ti, n)),
    })
}

/// The arm of transition `ti` that holds on `history`, and the version it
/// selects: the `unless` arm that holds (arms holding together must agree),
/// else `otherwise`.
fn decide(
    world: &World<'_>,
    (ti, t): (usize, &Transition),
    key: &Key,
    history: &[&Expr],
) -> Result<(usize, usize), Ungoverned> {
    let mut chosen: Option<(usize, usize)> = None;
    for (n, arm) in t.arms.iter().enumerate() {
        let Some(c) = &arm.unless else {
            continue;
        };
        let holds = eval(c, &mut |a| {
            has_event(a).map(|p| history.iter().any(|e| matches(p, e)))
        })
        .ok_or_else(|| {
            Ungoverned::Transition(format!(
                "`{}`: condition is not over `has_event`",
                t.meta.id
            ))
        })?;
        if !holds {
            continue;
        }
        let version = selected(world, ti, key, arm)?;
        match chosen {
            Some((_, v)) if v != version => {
                return Err(Ungoverned::Transition(format!(
                    "`{}`: two arms that select different versions both hold",
                    t.meta.id
                )))
            }
            _ => chosen = Some((n, version)),
        }
    }
    if let Some(c) = chosen {
        return Ok(c);
    }
    let (n, arm) = t
        .arms
        .iter()
        .enumerate()
        .find(|(_, a)| a.unless.is_none())
        .ok_or_else(|| Ungoverned::Transition(format!("`{}` has no `otherwise` arm", t.meta.id)))?;
    Ok((n, selected(world, ti, key, arm)?))
}

/// The version (declaration index) `arm` of transition `ti` selects.
fn selected(
    world: &World<'_>,
    ti: usize,
    key: &Key,
    arm: &TransitionArm,
) -> Result<usize, Ungoverned> {
    let sel = world.target(world.pack_of(ti), &arm.selects);
    sel.version
        .as_deref()
        .filter(|_| sel.key == *key)
        .and_then(|d| world.versions(key).starting(d).map(|v| v.decl))
        .ok_or_else(|| Ungoverned::Transition(format!("arm selects unknown `{}`", arm.selects)))
}

/// Static transition checks (L3008) for the transitions of pack `pack`.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (i, d) in world.decls.iter().enumerate() {
        let (Declaration::Transition(t), true) = (d.decl, d.pack == pack) else {
            continue;
        };
        out.extend(one(world, loc, i, t));
    }
    out
}

fn one(world: &World<'_>, loc: &dyn Locate, i: usize, t: &Transition) -> Vec<Diagnostic> {
    let d = &world.decls[i];
    let mut out = Vec::new();
    let amended = world.target(world.pack_of(i), &t.amendment);
    let Some(at) = amended.version.clone() else {
        return out;
    };
    let set = world.versions(&amended.key);
    let name = amended.key.display(world.pack_of(i));
    let Some(index) = set.versions.iter().position(|v| v.in_force.from == at) else {
        return out;
    };
    let previous = set
        .predecessor(index)
        .map(|p| set.versions[p].in_force.from.clone());
    if previous.is_none() {
        out.push(
            error(
                C::InvalidTransition,
                span_of(loc, d, "amends"),
                format!("`{name}@{at}` replaces no version: no version of `{name}` ends on {at}, so no proceeding is pending across it"),
            )
            .with_note(None, "a transition amends a version that directly follows an earlier one"),
        );
    }
    // The world holds every transition per amended version; load order
    // picks none, so each one meets every other.
    for &(other, _) in world
        .transitions(&amended.key, &at)
        .iter()
        .filter(|(o, _)| *o != i)
    {
        out.push(
            error(
                C::InvalidTransition,
                span_of(loc, d, "amends"),
                format!(
                    "`{name}@{at}` also has transition `{}`",
                    world.decls[other].meta.id
                ),
            )
            .with_note(
                world.decls[other].meta.span.clone(),
                "the other transition is declared here",
            )
            .with_note(None, "one transition per amended version: merge their arms"),
        );
    }
    out.extend(overlaps(world, loc, i, t));
    for arm in &t.arms {
        out.extend(arm_checks(
            world,
            loc,
            i,
            t,
            arm,
            &amended.key,
            &at,
            previous.as_deref(),
        ));
    }
    out
}

/// L3008 for two `unless` arms selecting different versions whose
/// conditions some history makes true together: arms are disjoint, so
/// the one that holds never depends on the order they are written in.
fn overlaps(world: &World<'_>, loc: &dyn Locate, i: usize, t: &Transition) -> Vec<Diagnostic> {
    let d = &world.decls[i];
    let guarded: Vec<(&TransitionArm, &Expr)> = t
        .arms
        .iter()
        .filter_map(|a| Some((a, a.unless.as_ref()?)))
        .collect();
    let mut out = Vec::new();
    for (j, &(b, cb)) in guarded.iter().enumerate() {
        for &(a, ca) in &guarded[..j] {
            let sat = jointly_satisfiable(&[ca, cb]);
            let target = |arm: &TransitionArm| world.target(world.pack_of(i), &arm.selects);
            if target(a) == target(b) || sat == Some(false) {
                continue;
            }
            let why = if sat.is_none() {
                "their conditions have too many atoms to prove disjoint"
            } else {
                "some history makes both conditions true"
            };
            out.push(
                error(
                    C::InvalidTransition,
                    cb.span.clone().unwrap_or_else(|| span_of(loc, d, "amends")),
                    format!(
                        "arms `{}` and `{}` of `{}` can both hold",
                        a.selects, b.selects, t.meta.id
                    ),
                )
                .with_note(ca.span.clone(), "the other arm's condition")
                .with_note(None, why)
                .with_note(
                    None,
                    "make the conditions exclusive (`a && !b`): arms are disjoint, never first-match",
                ),
            );
        }
    }
    out
}

/// One arm: its condition reads only the history, and it selects the
/// amended version or the one it replaces.
#[allow(clippy::too_many_arguments)]
fn arm_checks(
    world: &World<'_>,
    loc: &dyn Locate,
    i: usize,
    t: &Transition,
    arm: &TransitionArm,
    amended: &Key,
    at: &str,
    previous: Option<&str>,
) -> Vec<Diagnostic> {
    let d = &world.decls[i];
    let name = amended.display(world.pack_of(i));
    let mut out = Vec::new();
    let span = arm
        .unless
        .as_ref()
        .and_then(|c| c.span.clone())
        .unwrap_or_else(|| span_of(loc, d, "amends"));
    if let Some(bad) = arm.unless.as_ref().and_then(first_foreign) {
        out.push(
            error(
                C::InvalidTransition,
                bad.span.clone().unwrap_or_else(|| span.clone()),
                "a transition condition may test only `has_event(...)`, `true`/`false`, and `!`, `&&`, `||`",
            )
            .with_note(None, "a transition chooses between versions from the matter's history alone"),
        );
    }
    let sel = world.target(world.pack_of(i), &arm.selects);
    let ok_version = sel
        .version
        .as_ref()
        .is_some_and(|v| v == at || previous == Some(v.as_str()));
    if sel.key != *amended {
        out.push(error(
            C::InvalidTransition,
            span.clone(),
            format!(
                "arm selects `{}`, but `{}` amends `{name}`",
                arm.selects, t.meta.id
            ),
        ));
    } else if !ok_version {
        let mut e = error(
            C::InvalidTransition,
            span.clone(),
            format!(
                "arm selects `{}`, which is neither the amended version `{name}@{at}` nor the version it replaces",
                arm.selects
            ),
        );
        if let Some(p) = previous {
            e = e.with_note(None, format!("the replaced version is `{name}@{p}`"));
        }
        out.push(e);
    }
    out
}
