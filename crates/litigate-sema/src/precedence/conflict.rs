//! Two-stage conflict resolution (SPEC §5.5, DESIGN §4.5).
//!
//! The *claimants* of a declaration T are T itself and every declaration
//! that says `override T`, `notwithstanding T`, or `subject_to T`. When two
//! claimants are active on the same day, the winner is chosen by
//!
//! 1. declared relations: `A notwithstanding B` and `A override B` (A
//!    wins), `A subject_to B` (B wins); then
//! 2. canons, only between comparable authorities: higher authority, then
//!    the declaration with the narrower subject matter (its trigger or
//!    condition entails the other's), then one inside a `rule` over package
//!    level, then the later `from`.
//!
//! A displacing claim (`override`, `notwithstanding`) counts only when it
//! is licensed: its authority equals or outranks the target's, or the
//! target delegates to it (SPEC §5.5 step 3; otherwise L3004). A claimant
//! must beat every other active claimant. A pair neither stage orders,
//! including every pair of incomparable authorities no relation orders, is
//! L3003 with both spans; it is never resolved silently.
use super::delegation::licensed;
use super::lattice::{Lattice, Order};
use crate::hir::{Declaration, DefaultRule, Exception, Expr, Relation};
use crate::temporal::activity::{boundaries, piece, suspensions};
use crate::temporal::condition::{entails, matches};
use crate::temporal::interval::Window;
use crate::temporal::world::{error, span_of, DiagnosticExt, Key, Locate, World};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Which stage decided a pair, for explanations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// A declared relation (`notwithstanding`, `override`, `subject_to`).
    Declared(&'static str),
    /// Higher authority.
    Authority,
    /// Narrower subject matter: its trigger or condition entails the
    /// other's, not conversely.
    Specific,
    /// Declared inside a `rule`, the other at package level.
    InRule,
    /// Later `from`.
    Later,
}

/// The claim `a` makes on `t` that applies to version `of` of `t` (a
/// `t@D` claim applies only to the version starting on D; `of = None`
/// accepts only unversioned claims).
fn claim(world: &World<'_>, a: usize, t: &Key, of: Option<usize>) -> Option<&'static str> {
    let pack = world.pack_of(a).to_owned();
    world.decls[a].meta.relations.iter().find_map(|r| {
        let (key, text) = match r {
            Relation::Override(x) => ("override", x),
            Relation::Notwithstanding(x) => ("notwithstanding", x),
            Relation::SubjectTo(x) => ("subject_to", x),
            _ => return None,
        };
        let target = world.target(&pack, text);
        let applies = target
            .version
            .as_ref()
            .is_none_or(|d| of.is_some_and(|o| world.decls[o].meta.effective.from == *d));
        (target.key == *t && applies).then_some(key)
    })
}

/// The first relation clause of `a` naming `t` (for spans).
fn clause_on(world: &World<'_>, a: usize, t: &Key) -> &'static str {
    let pack = world.pack_of(a).to_owned();
    world.decls[a]
        .meta
        .relations
        .iter()
        .find_map(|r| {
            let (key, text) = match r {
                Relation::Override(x) => ("override", x),
                Relation::Notwithstanding(x) => ("notwithstanding", x),
                Relation::SubjectTo(x) => ("subject_to", x),
                _ => return None,
            };
            (world.target(&pack, text).key == *t).then_some(key)
        })
        .unwrap_or("")
}

/// Stage 1 for the ordered pair: `Some(true)` when `a` is declared to win
/// over `b`, `Some(false)` when `b` is.
fn declared(world: &World<'_>, a: usize, b: usize) -> Option<(bool, &'static str)> {
    let forward = claim(world, a, &world.key(b), Some(b));
    let backward = claim(world, b, &world.key(a), Some(a));
    let a_wins = |k: &str| k != "subject_to";
    match (forward, backward) {
        (Some(k), None) => Some((a_wins(k), k)),
        (None, Some(k)) => Some((!a_wins(k), k)),
        (Some(k1), Some(k2)) if a_wins(k1) != a_wins(k2) => Some((a_wins(k1), k1)),
        _ => None,
    }
}

/// Stage 2 for the ordered pair. Incomparable authorities stop here: no
/// canon ranks law from two chains (`lit/core/authority.lit`).
fn canon(world: &World<'_>, lattice: &Lattice, a: usize, b: usize) -> Option<(bool, Reason)> {
    let (da, db) = (&world.decls[a], &world.decls[b]);
    match lattice.compare(&da.meta.authority, &db.meta.authority) {
        Order::Above => return Some((true, Reason::Authority)),
        Order::Below => return Some((false, Reason::Authority)),
        Order::Incomparable => return None,
        Order::Equal => {}
    }
    if let Some(w) = specific(da.decl, db.decl) {
        return Some((w, Reason::Specific));
    }
    if da.rule.is_some() != db.rule.is_some() {
        return Some((da.rule.is_some(), Reason::InRule));
    }
    let (wa, wb) = (
        Window::of(&da.meta.effective),
        Window::of(&db.meta.effective),
    );
    match wa.from.cmp(&wb.from) {
        Ordering::Greater => Some((true, Reason::Later)),
        Ordering::Less => Some((false, Reason::Later)),
        Ordering::Equal => None,
    }
}

/// What a declaration applies to: a deadline's trigger pattern, or a
/// default's or exception's condition.
enum Antecedent<'e> {
    Trigger(&'e Expr),
    Condition(&'e Expr),
}

fn antecedent(d: &Declaration) -> Option<Antecedent<'_>> {
    match d {
        Declaration::Deadline(x) => Some(Antecedent::Trigger(&x.trigger)),
        Declaration::Default(DefaultRule { condition, .. })
        | Declaration::Exception(Exception { condition, .. }) => {
            Some(Antecedent::Condition(condition))
        }
        _ => None,
    }
}

/// Subject-matter specificity (the specific provision governs the general): `Some(true)` when `a`'s
/// antecedent entails `b`'s and not conversely, `Some(false)` for the
/// converse, `None` when neither is narrower or they are not comparable.
/// A trigger pattern entails another that is its prefix (omitted trailing
/// arguments match anything).
fn specific(a: &Declaration, b: &Declaration) -> Option<bool> {
    let (ab, ba) = match (antecedent(a), antecedent(b)) {
        (Some(Antecedent::Trigger(x)), Some(Antecedent::Trigger(y))) => {
            (matches(y, x), matches(x, y))
        }
        (Some(Antecedent::Condition(x)), Some(Antecedent::Condition(y))) => {
            (entails(x, y) == Some(true), entails(y, x) == Some(true))
        }
        _ => return None,
    };
    (ab != ba).then_some(ab)
}

/// Whether `a` beats `b`, and by which stage (`None`: unordered).
#[must_use]
pub fn beats(world: &World<'_>, lattice: &Lattice, a: usize, b: usize) -> Option<(bool, Reason)> {
    declared(world, a, b)
        .map(|(w, k)| (w, Reason::Declared(k)))
        .or_else(|| canon(world, lattice, a, b))
}

/// Every claimant of `t` (declaration indexes): its versions, then claims.
#[must_use]
pub fn claimants(world: &World<'_>, t: &Key) -> Vec<usize> {
    let mut out: Vec<usize> = world.versions(t).versions.iter().map(|v| v.decl).collect();
    let mut by: Vec<usize> = world
        .incoming(t)
        .iter()
        .filter(|r| is_claim(r.relation))
        .map(|r| r.by)
        .filter(|&a| world.key(a) != *t)
        .collect();
    by.dedup();
    out.extend(by);
    out
}

fn is_claim(r: &Relation) -> bool {
    matches!(
        r,
        Relation::Override(_) | Relation::Notwithstanding(_) | Relation::SubjectTo(_)
    )
}

/// Claimants of `t` active on `date`.
#[must_use]
pub fn active_claimants(world: &World<'_>, t: &Key, date: &str) -> Vec<usize> {
    claimants_at(world, t, date, None)
}

/// Claimants of `t` active on `date` among declarations recorded by
/// `known_at`: `t`'s own active version, and the active version of every
/// other claimant whose claim applies to it and is licensed (a displacing
/// claim needs [`licensed`]; `subject_to` displaces nothing).
#[must_use]
pub fn claimants_at(world: &World<'_>, t: &Key, date: &str, known_at: Option<&str>) -> Vec<usize> {
    let mut keys: Vec<Key> = claimants(world, t)
        .into_iter()
        .map(|c| world.key(c))
        .collect();
    keys.sort();
    keys.dedup();
    let live = |k: &Key| crate::temporal::select(world, k, date, known_at).ok();
    let base = live(t);
    keys.iter()
        .filter_map(&live)
        .filter(|&c| {
            world.key(c) == *t
                || claim(world, c, t, base).is_some_and(|k| {
                    k == "subject_to" || base.is_none_or(|b| licensed(world, c, b))
                })
        })
        .collect()
}

/// The claimant of `t` that prevails on `date`.
///
/// # Errors
/// The unordered pairs when no claimant beats every other one (empty when
/// nothing is active).
pub fn winner(
    world: &World<'_>,
    lattice: &Lattice,
    t: &Key,
    date: &str,
) -> Result<usize, Vec<(usize, usize)>> {
    decide(world, lattice, &active_claimants(world, t, date))
}

/// The claimant among `live` that beats every other one.
///
/// # Errors
/// The unordered pairs (empty when `live` is).
pub(crate) fn decide(
    world: &World<'_>,
    lattice: &Lattice,
    live: &[usize],
) -> Result<usize, Vec<(usize, usize)>> {
    let wins = |a: usize| {
        live.iter()
            .all(|&b| a == b || beats(world, lattice, a, b).is_some_and(|(w, _)| w))
    };
    if let Some(&w) = live.iter().find(|&&a| wins(a)) {
        return Ok(w);
    }
    let mut pairs = Vec::new();
    for (j, &b) in live.iter().enumerate() {
        for &a in &live[..j] {
            if beats(world, lattice, a, b).is_none() {
                pairs.push((a, b));
            }
        }
    }
    if pairs.is_empty() && live.len() > 1 {
        pairs.extend(live.windows(2).map(|w| (w[0], w[1])));
    }
    Err(pairs)
}

/// Every day boundary where the set of active claimants of `t` may change.
fn points(world: &World<'_>, t: &Key) -> Vec<String> {
    let mut keys: Vec<Key> = claimants(world, t)
        .into_iter()
        .map(|c| world.key(c))
        .collect();
    keys.sort();
    keys.dedup();
    let mut windows: Vec<Window> = Vec::new();
    for k in &keys {
        windows.extend(
            world
                .versions(k)
                .versions
                .iter()
                .map(|v| v.in_force.clone()),
        );
        windows.extend(suspensions(world, k, None).into_iter().map(|s| s.window));
    }
    boundaries(&windows)
}

/// L3003 for every unordered pair of claimants, at least one of them in
/// pack `pack`, reported once with the first window it occurs in.
#[must_use]
pub fn check(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    pack: usize,
) -> Vec<Diagnostic> {
    let targets: Vec<Key> = world
        .targets()
        .filter(|(_, rs)| rs.iter().any(|r| is_claim(r.relation)))
        .map(|(k, _)| k.clone())
        .collect();
    let mut out = Vec::new();
    let mut reported: BTreeSet<(usize, usize)> = BTreeSet::new();
    for t in &targets {
        let pts = points(world, t);
        for i in 0..pts.len() {
            let window = piece(&pts, i);
            let Err(pairs) = winner(world, lattice, t, &window.from) else {
                continue;
            };
            for (a, b) in pairs {
                let owned = world.decls[a].pack == pack || world.decls[b].pack == pack;
                // Contradictory declared relations are a cycle (L3006), not
                // a canon tie.
                let (a, b) = (a.min(b), a.max(b));
                let contradictory = claim(world, a, &world.key(b), Some(b)).is_some()
                    && claim(world, b, &world.key(a), Some(a)).is_some();
                if owned && !contradictory && reported.insert((a, b)) {
                    out.push(unresolved(world, lattice, loc, t, a, b, &window));
                }
            }
        }
    }
    out
}

fn unresolved(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    t: &Key,
    a: usize,
    b: usize,
    window: &Window,
) -> Diagnostic {
    let (da, db) = (&world.decls[a], &world.decls[b]);
    let clause = |i: usize| clause_on(world, i, t);
    let target = t.display(world.pack_of(b));
    let (na, nb) = (world.versioned(a, b), world.versioned(b, b));
    let mut d = error(
        C::UnresolvedPrecedence,
        span_of(loc, db, clause(b)),
        format!(
            "`{nb}` and `{na}` both apply to `{target}` on {}, and neither prevails",
            window.display()
        ),
    )
    .with_note(
        Some(span_of(loc, da, clause(a))),
        format!("`{na}` is declared here"),
    );
    let canons = match lattice.compare(&da.meta.authority, &db.meta.authority) {
        Order::Equal => format!(
            "the canons tie: both are {}; neither applies to a narrower subject matter; {}; both take effect on {}",
            da.meta.authority,
            if da.rule.is_some() { "both inside a `rule`" } else { "both at package level" },
            da.meta.effective.from
        ),
        _ => format!(
            "{} and {} are not ranked by any `authority` chain, so no canon orders them",
            da.meta.authority, db.meta.authority
        ),
    };
    d = d
        .with_note(
            None,
            "no declared relation (`notwithstanding`, `override`, `subject_to`) orders them",
        )
        .with_note(None, canons)
        .with_note(
            None,
            format!(
                "declare `{} notwithstanding {}` or `{} subject_to {}`",
                db.meta.id,
                world.name(a, b),
                db.meta.id,
                world.name(a, b)
            ),
        );
    d
}
