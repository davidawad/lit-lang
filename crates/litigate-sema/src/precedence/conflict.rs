//! Two-stage conflict resolution (SPEC §5.5, DESIGN §4.5).
//!
//! The *claimants* of a declaration T are T itself and every declaration
//! that says `override T`, `notwithstanding T`, or `subject_to T`. When two
//! claimants are active on the same day, the winner is chosen by
//!
//! 1. declared relations: `A notwithstanding B` and `A override B` (A
//!    wins), `A subject_to B` (B wins); then
//! 2. canons: higher authority, then the more specific declaration (inside
//!    a `rule` beats package level, then the narrower in-force window),
//!    then the later `from`.
//!
//! A claimant must beat every other active claimant. A pair neither stage
//! orders is L3003 with both spans; it is never resolved silently.
use super::lattice::{Lattice, Order};
use crate::hir::Relation;
use crate::temporal::activity::{active, boundaries, piece, suspensions};
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
    /// Declared inside a `rule`, the other at package level.
    InRule,
    /// Narrower in-force window.
    Narrower,
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

/// Stage 2 for the ordered pair.
fn canon(world: &World<'_>, lattice: &Lattice, a: usize, b: usize) -> Option<(bool, Reason)> {
    let (da, db) = (&world.decls[a], &world.decls[b]);
    match lattice.compare(&da.meta.authority, &db.meta.authority) {
        Order::Above => return Some((true, Reason::Authority)),
        Order::Below => return Some((false, Reason::Authority)),
        Order::Equal | Order::Incomparable => {}
    }
    if da.rule.is_some() != db.rule.is_some() {
        return Some((da.rule.is_some(), Reason::InRule));
    }
    let (wa, wb) = (
        Window::of(&da.meta.effective),
        Window::of(&db.meta.effective),
    );
    match width(&wa).cmp(&width(&wb)) {
        Ordering::Less => return Some((true, Reason::Narrower)),
        Ordering::Greater => return Some((false, Reason::Narrower)),
        Ordering::Equal => {}
    }
    match wa.from.cmp(&wb.from) {
        Ordering::Greater => Some((true, Reason::Later)),
        Ordering::Less => Some((false, Reason::Later)),
        Ordering::Equal => None,
    }
}

/// Window width for specificity: open-ended is widest.
fn width(w: &Window) -> (bool, i64) {
    w.days().map_or((true, 0), |d| (false, d))
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
    let mut keys: Vec<Key> = claimants(world, t)
        .into_iter()
        .map(|c| world.key(c))
        .collect();
    keys.sort();
    keys.dedup();
    let live = |k: &Key| {
        active(world, k, &world.versions(k), date, None)
            .ok()
            .map(|v| v.decl)
    };
    let base = live(t);
    keys.iter()
        .filter_map(&live)
        .filter(|&c| world.key(c) == *t || claim(world, c, t, base).is_some())
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
    let live = active_claimants(world, t, date);
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
    let authority = match lattice.compare(&da.meta.authority, &db.meta.authority) {
        Order::Equal => format!("both are {}", da.meta.authority),
        _ => format!(
            "{} and {} are not ranked by any `authority` chain",
            da.meta.authority, db.meta.authority
        ),
    };
    d = d
        .with_note(None, "no declared relation (`notwithstanding`, `override`, `subject_to`) orders them")
        .with_note(
            None,
            format!(
                "the canons tie: {authority}; equally specific ({}, same window length); both take effect on {}",
                if da.rule.is_some() { "both inside a `rule`" } else { "both at package level" },
                da.meta.effective.from
            ),
        )
        .with_note(None, format!("declare `{} notwithstanding {}` or `{} subject_to {}`", db.meta.id, world.name(a, b), db.meta.id, world.name(a, b)));
    d
}
