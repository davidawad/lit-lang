//! `transition` declarations: which version governs a proceeding pending
//! across an amendment (SPEC §5.4, DESIGN §4.4; FRCP 86(a) is written this
//! way in the stdlib, never in compiler code).
//!
//! By default every event is governed by the declaration the precedence
//! selection ([`crate::precedence::select`]) picks on its date: the version
//! in force, or the claimant that displaces it. When that version took
//! effect after the matter's first event (the matter
//! was pending when it did) and a transition amends that version, the
//! transition's arms decide instead: the first `unless` whose condition
//! holds on the history up to the event selects its version, else
//! `otherwise` does. A discretionary leg ("unless the court orders
//! otherwise", "just and practicable") is an `unless has_event(Order...)`
//! arm the case satisfies with an order event.
use super::activity::Inactive;
use super::condition::{eval, first_foreign, has_event, matches};
use super::world::{error, span_of, DiagnosticExt, Key, Locate, World};
use crate::hir::{Declaration, Expr, Transition, TransitionArm};
use crate::precedence::select::{select, Unselected};
use litigate_base::{Diagnostic, DiagnosticCode as C};

/// One dated event of a matter (a complete event constructor call).
#[derive(Debug, Clone, Copy)]
pub struct MatterEvent<'e> {
    /// Event date.
    pub date: &'e str,
    /// The event constructor call.
    pub action: &'e Expr,
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

/// The transition (declaration index) amending `key@from`, if one exists.
#[must_use]
pub fn transition_for(world: &World<'_>, key: &Key, from: &str) -> Option<usize> {
    world.transition_for(key, from)
}

/// The version governing `key` for `events[at]` in a matter whose events
/// are `events` (sorted by date): the selected declaration on the event's
/// date, through the transition amending it if the matter was pending.
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
    let event = events
        .get(at)
        .ok_or_else(|| Ungoverned::Transition(format!("no event #{at}")))?;
    let w = select(world, key, event.date, known_at).map_err(|e| match e {
        Unselected::Inactive(why) => Ungoverned::Inactive(why),
        Unselected::Unresolved(pairs) => Ungoverned::Unresolved(pairs),
    })?;
    let key = &world.key(w);
    let set = world.versions(key);
    let from = &world.decls[w].meta.effective.from;
    let pending = events
        .iter()
        .map(|e| e.date)
        .min()
        .is_some_and(|first| first < from.as_str());
    let Some((ti, t)) = pending.then(|| world.transition(key, from)).flatten() else {
        return Ok(Governing {
            version: w,
            via: None,
        });
    };
    let history: Vec<&Expr> = events
        .iter()
        .filter(|e| e.date <= event.date)
        .map(|e| e.action)
        .collect();
    for (n, arm) in t.arms.iter().enumerate() {
        let holds = match &arm.unless {
            None => true,
            Some(c) => eval(c, &mut |a| {
                has_event(a).map(|p| history.iter().any(|e| matches(p, e)))
            })
            .ok_or_else(|| {
                Ungoverned::Transition(format!(
                    "`{}`: condition is not over `has_event`",
                    t.meta.id
                ))
            })?,
        };
        if holds {
            let sel = world.target(world.pack_of(ti), &arm.selects);
            let chosen = sel
                .version
                .as_deref()
                .and_then(|d| set.starting(d))
                .filter(|_| sel.key == *key)
                .ok_or_else(|| {
                    Ungoverned::Transition(format!("arm selects unknown `{}`", arm.selects))
                })?;
            return Ok(Governing {
                version: chosen.decl,
                via: Some((ti, n)),
            });
        }
    }
    Err(Ungoverned::Transition(format!(
        "`{}` has no `otherwise` arm",
        t.meta.id
    )))
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
    // The world keeps the first transition per amended version, so any
    // other is a later duplicate.
    if let Some(first) = transition_for(world, &amended.key, &at).filter(|&f| f != i) {
        out.push(
            error(
                C::InvalidTransition,
                span_of(loc, d, "amends"),
                format!(
                    "`{name}@{at}` already has transition `{}`",
                    world.decls[first].meta.id
                ),
            )
            .with_note(
                world.decls[first].meta.span.clone(),
                "the first transition is declared here",
            ),
        );
    }
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
