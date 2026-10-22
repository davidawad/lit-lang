//! Which version of a declaration is *active* on a date, once declared
//! supersession and revival are applied (SPEC §5.5 step 1).
//!
//! `A supersedes B` makes the versions of B already in force when A takes
//! effect inactive from A's start; a version of B enacted later is new law
//! and unaffected. Repeal never revives:
//! when A's window ends, B stays inactive unless A also says `revives B`
//! (B is active again after A's window), or a later declaration — a new
//! version of B or a third declaration — says `revives B`, which ends
//! every open suspension at its start (Scalia & Garner, *Reading Law*
//! §§ 13, 27-29, 55-57).
use super::interval::{cmp_end, Window};
use super::version::{Miss, Version};
use super::world::{Key, World};
use crate::hir::Relation;

/// A window during which `by` keeps (some versions of) a declaration out of
/// force.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suspension {
    /// The superseding declaration (index into [`World::decls`]).
    pub by: usize,
    /// Days the target is inactive.
    pub window: Window,
    /// Only the version starting on this date is affected (`supersedes x@D`).
    pub version: Option<String>,
    /// Whether the window ends because a `revives` was declared.
    pub revived: bool,
}

/// Why a declaration selected no active version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inactive {
    /// No version in force (or recorded) on the date.
    Miss(Miss),
    /// The version in force is superseded.
    Superseded(Suspension),
}

fn targets<'w>(
    world: &'w World<'_>,
    i: usize,
    want: fn(&Relation) -> Option<&String>,
) -> impl Iterator<Item = super::world::Target> + 'w {
    let pack = world.pack_of(i).to_owned();
    world.decls[i]
        .meta
        .relations
        .iter()
        .filter_map(want)
        .map(move |t| world.target(&pack, t))
}

fn supersedes(r: &Relation) -> Option<&String> {
    match r {
        Relation::Supersedes(t) => Some(t),
        _ => None,
    }
}

fn revives(r: &Relation) -> Option<&String> {
    match r {
        Relation::Revives(t) => Some(t),
        _ => None,
    }
}

/// Dates from which `key` is revived: the end of every superseding window
/// that declares `revives`, and the start of every other `revives` of it.
#[must_use]
pub fn revival_points(world: &World<'_>, key: &Key) -> Vec<String> {
    let mut out: Vec<String> = (0..world.decls.len())
        .filter(|&i| targets(world, i, revives).any(|t| t.key == *key))
        .filter_map(|i| {
            let w = Window::of(&world.decls[i].meta.effective);
            if targets(world, i, supersedes).any(|t| t.key == *key) {
                w.to
            } else {
                Some(w.from)
            }
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every suspension of `key`.
#[must_use]
pub fn suspensions(world: &World<'_>, key: &Key) -> Vec<Suspension> {
    let revivals = revival_points(world, key);
    let mut out = Vec::new();
    for i in 0..world.decls.len() {
        let own = world.key(i) == *key;
        let window = Window::of(&world.decls[i].meta.effective);
        for t in targets(world, i, supersedes).filter(|t| t.key == *key) {
            if own && t.version.is_none() {
                // `p@D supersedes p` documents succession; earlier versions
                // already end by D, so it suspends nothing.
                continue;
            }
            let revives_self = targets(world, i, revives).any(|r| r.key == *key);
            let (end, revived) = if revives_self {
                (window.to.clone(), true)
            } else {
                let next = revivals
                    .iter()
                    .find(|d| d.as_str() > window.from.as_str())
                    .cloned();
                (next.clone(), next.is_some())
            };
            let suspended = Window {
                from: window.from.clone(),
                to: end,
            };
            if !suspended.is_empty() {
                out.push(Suspension {
                    by: i,
                    window: suspended,
                    version: t.version,
                    revived,
                });
            }
        }
    }
    out
}

/// The version of `key` active on `date` among versions recorded by
/// `known_at`.
///
/// # Errors
/// [`Inactive`] when no version is in force or the one in force is
/// superseded.
pub fn active<'v>(
    world: &World<'_>,
    key: &Key,
    set: &'v super::version::VersionSet<usize>,
    date: &str,
    known_at: Option<&str>,
) -> Result<&'v Version<usize>, Inactive> {
    let v = set.as_of(date, known_at).map_err(Inactive::Miss)?;
    match suspensions(world, key).into_iter().find(|s| {
        s.window.contains(date)
            && v.in_force.from < s.window.from
            && s.version.as_ref().is_none_or(|d| *d == v.in_force.from)
    }) {
        Some(s) => Err(Inactive::Superseded(s)),
        None => Ok(v),
    }
}

/// Windows during which `key` is suspended as a whole.
#[must_use]
pub fn suspended_windows(world: &World<'_>, key: &Key) -> Vec<Window> {
    suspensions(world, key)
        .into_iter()
        .filter(|s| s.version.is_none())
        .map(|s| s.window)
        .collect()
}

/// Boundary dates of `windows`, sorted and unique.
#[must_use]
pub fn boundaries<'w>(windows: impl IntoIterator<Item = &'w Window>) -> Vec<String> {
    let mut out: Vec<String> = windows
        .into_iter()
        .flat_map(|w| std::iter::once(w.from.clone()).chain(w.to.clone()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The elementary window starting at `points[i]`.
#[must_use]
pub fn piece(points: &[String], i: usize) -> Window {
    Window {
        from: points[i].clone(),
        to: points.get(i + 1).cloned(),
    }
}

/// True when the open end `a` reaches at least `b`.
#[must_use]
pub fn reaches(a: Option<&str>, b: Option<&str>) -> bool {
    cmp_end(a, b).is_ge()
}
