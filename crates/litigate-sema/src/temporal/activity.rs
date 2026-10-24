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
//!
//! Both relations displace law, so a declaration whose authority neither
//! equals nor outranks the versions it acts on, and that they do not
//! delegate to, has no effect here (it is L3004, SPEC §5.5 step 3).
use super::interval::Window;
use super::version::{Miss, Version};
use super::world::{Incoming, Key, World};
use crate::hir::Relation;
use crate::precedence::delegation::licensed_over;

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

/// True when declaration `by` has a relation of kind `want` on `key`.
fn says(world: &World<'_>, key: &Key, by: usize, want: fn(&Relation) -> bool) -> bool {
    world
        .incoming(key)
        .iter()
        .any(|r| r.by == by && want(r.relation))
}

/// True when the corpus recorded declaration `i` by `known_at` (`None`:
/// the latest corpus).
fn recorded(world: &World<'_>, i: usize, known_at: Option<&str>) -> bool {
    known_at.is_none_or(|k| world.decls[i].meta.effective.known_at.as_str() <= k)
}

fn is_supersedes(r: &Relation) -> bool {
    matches!(r, Relation::Supersedes(_))
}

fn is_revives(r: &Relation) -> bool {
    matches!(r, Relation::Revives(_))
}

/// Whether the relation `r` on `key` is recorded by `known_at` and
/// licensed over the versions it acts on.
fn effective(world: &World<'_>, key: &Key, r: &Incoming<'_>, known_at: Option<&str>) -> bool {
    recorded(world, r.by, known_at) && licensed_over(world, r.by, key, r.version.as_deref())
}

/// Dates from which `key` is revived: the end of every superseding window
/// that declares `revives`, and the start of every other `revives` of it,
/// among licensed declarations recorded by `known_at`.
#[must_use]
pub fn revival_points(world: &World<'_>, key: &Key, known_at: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = world
        .incoming(key)
        .iter()
        .filter(|r| is_revives(r.relation) && effective(world, key, r, known_at))
        .filter_map(|r| {
            let w = Window::of(&world.decls[r.by].meta.effective);
            if says(world, key, r.by, is_supersedes) {
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

/// Every suspension of `key` declared by licensed declarations recorded
/// by `known_at`.
#[must_use]
pub fn suspensions(world: &World<'_>, key: &Key, known_at: Option<&str>) -> Vec<Suspension> {
    let incoming = world.incoming(key);
    if !incoming.iter().any(|r| is_supersedes(r.relation)) {
        return Vec::new();
    }
    let revivals = revival_points(world, key, known_at);
    let mut out = Vec::new();
    for r in incoming
        .iter()
        .filter(|r| is_supersedes(r.relation) && effective(world, key, r, known_at))
    {
        let i = r.by;
        if world.key(i) == *key && r.version.is_none() {
            // `p@D supersedes p` documents succession; earlier versions
            // already end by D, so it suspends nothing.
            continue;
        }
        let window = Window::of(&world.decls[i].meta.effective);
        let (end, revived) = if says(world, key, i, is_revives) {
            (window.to.clone(), true)
        } else {
            let next = revivals
                .iter()
                .find(|d| d.as_str() > window.from.as_str())
                .cloned();
            (next.clone(), next.is_some())
        };
        // Never empty: in-force windows are non-empty (L2007), and a
        // revival point ends a suspension only after it starts.
        out.push(Suspension {
            by: i,
            window: Window {
                from: window.from,
                to: end,
            },
            version: r.version.clone(),
            revived,
        });
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
    let v = set
        .as_of(date, known_at)
        .map_err(|miss| Inactive::Miss(encoded(world, set, date, miss)))?;
    match suspensions(world, key, known_at).into_iter().find(|s| {
        s.window.contains(date)
            && v.in_force.from < s.window.from
            && s.version.as_ref().is_none_or(|d| *d == v.in_force.from)
    }) {
        Some(s) => Err(Inactive::Superseded(s)),
        None => Ok(v),
    }
}

/// `miss` as the query on `date` should report it. A date before every
/// recorded version is a hole in the corpus (`unknown_before`, SPEC §5.4)
/// only when it also precedes the package's `in_force`, the start of the
/// corpus the package encodes; inside that corpus, a declaration whose
/// first version starts later was not yet law, a finding (L3007 as no
/// version in force).
fn encoded(
    world: &World<'_>,
    set: &super::version::VersionSet<usize>,
    date: &str,
    miss: Miss,
) -> Miss {
    let covered = set
        .versions
        .first()
        .is_some_and(|v| world.packs[world.decls[v.decl].pack].lifetime.from.as_str() <= date);
    match miss {
        Miss::Before { .. } if covered => Miss::NotInForce { hidden: Vec::new() },
        other => other,
    }
}

/// Windows during which `key` is suspended as a whole.
#[must_use]
pub fn suspended_windows(world: &World<'_>, key: &Key) -> Vec<Window> {
    suspensions(world, key, None)
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
