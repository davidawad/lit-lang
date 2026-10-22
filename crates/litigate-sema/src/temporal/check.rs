//! `effective-disjoint` (L3001), `effective-covered` (L3002), and version
//! succession (L3006) for the declarations of one package.
//!
//! A declaration's lifetime runs from its earliest version to the end of
//! its package. A day of it no version covers is a gap, legal only when
//! declared: covered by a declaration that `supersedes` it, ended by a
//! version that `revives` it, or — for the trailing gap — left by a version
//! that is itself a declared amendment (it `supersedes` or `revives`
//! something), such as a sunset provision.
use super::activity::{revival_points, suspended_windows};
use super::interval::{gaps, Window};
use super::version::VersionSet;
use super::world::{error, span_of, DiagnosticExt, Key, Locate, World};
use crate::hir::Relation;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C};

/// Temporal diagnostics for every declaration owned by pack `pack`.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (key, set) in world.sets() {
        let Some(&first) = set.versions.first().map(|v| &v.decl) else {
            continue;
        };
        if world.decls[first].pack != pack {
            continue;
        }
        out.extend(overlaps(world, loc, &set));
        out.extend(coverage(world, loc, key, &set));
        out.extend(succession(world, loc, key, &set));
    }
    out
}

/// Replacement interval text.
fn interval_text(w: &Window) -> String {
    w.display()
}

fn overlaps(world: &World<'_>, loc: &dyn Locate, set: &VersionSet<usize>) -> Vec<Diagnostic> {
    set.overlaps()
        .into_iter()
        .map(|o| {
            let (a, b) = (&set.versions[o.first], &set.versions[o.second]);
            let (da, db) = (&world.decls[a.decl], &world.decls[b.decl]);
            let name = world.name(b.decl, b.decl);
            let mut d = error(
                C::EffectiveOverlap,
                span_of(loc, db, "effective"),
                format!(
                    "versions of `{name}` overlap on {}: {} and {}",
                    o.common.display(),
                    a.in_force.display(),
                    b.in_force.display()
                ),
            )
            .with_note(
                Some(span_of(loc, da, "effective")),
                "the overlapping version is declared here",
            )
            .with_note(
                None,
                "versions of one declaration must be pairwise disjoint so `as_of` selects exactly one",
            );
            if a.in_force.from < b.in_force.from {
                if let Some((_, Some(value))) = loc.clause(da, "effective") {
                    let closed = Window::new(a.in_force.from.clone(), Some(b.in_force.from.clone()));
                    d = d.with_fix(value, interval_text(&closed), Applicability::MaybeIncorrect);
                }
            }
            d
        })
        .collect()
}

fn coverage(
    world: &World<'_>,
    loc: &dyn Locate,
    key: &Key,
    set: &VersionSet<usize>,
) -> Vec<Diagnostic> {
    let Some(&first) = set.versions.first().map(|v| &v.decl) else {
        return Vec::new();
    };
    let lifetime = &world.packs[world.decls[first].pack].lifetime;
    let suspended = suspended_windows(world, key);
    let revivals = revival_points(world, key);
    let name = world.name(first, first);
    set.gaps(lifetime)
        .into_iter()
        .filter(|g| {
            let revived = g.to.as_ref().is_some_and(|t| revivals.contains(t));
            let temporary = g.to == lifetime.to
                && set.versions.iter().any(|v| {
                    v.in_force.to.as_ref() == Some(&g.from)
                        && world.decls[v.decl]
                            .meta
                            .relations
                            .iter()
                            .any(|r| matches!(r, Relation::Supersedes(_) | Relation::Revives(_)))
                });
            !revived && !temporary && !gaps(g, &suspended).is_empty()
        })
        .map(|g| {
            let before = set.versions.iter().find(|v| v.in_force.to.as_ref() == Some(&g.from));
            let after = set.versions.iter().find(|v| Some(&v.in_force.from) == g.to.as_ref());
            let at = before.or(after).map_or(first, |v| v.decl);
            let mut d = error(
                C::EffectiveGap,
                span_of(loc, &world.decls[at], "effective"),
                format!("`{name}` has no version in force on {}", g.display()),
            )
            .with_note(
                None,
                format!(
                    "a gap is legal only when declared: {}a declaration that `supersedes {name}` over the gap",
                    g.to.as_deref().map_or_else(String::new, |t| format!("a version starting {t} that `revives {name}`, or "))
                ),
            );
            if let Some(prev) = before {
                if let Some((_, Some(value))) = loc.clause(&world.decls[prev.decl], "effective") {
                    let to = if after.is_some() { g.to.clone() } else { lifetime.to.clone() };
                    let closed = Window { from: prev.in_force.from.clone(), to };
                    d = d.with_fix(value, interval_text(&closed), Applicability::MaybeIncorrect);
                }
            }
            if let Some(next) = after {
                d = d.with_note(world.decls[next.decl].meta.span.clone(), "the next version starts here");
            }
            d
        })
        .collect()
}

/// `p@D supersedes p@E` must name an earlier version.
fn succession(
    world: &World<'_>,
    loc: &dyn Locate,
    key: &Key,
    set: &VersionSet<usize>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for v in &set.versions {
        let d = &world.decls[v.decl];
        for r in &d.meta.relations {
            let (Relation::Supersedes(t) | Relation::Revives(t)) = r else {
                continue;
            };
            let target = world.target(world.pack_of(v.decl), t);
            match target.version {
                Some(at) if target.key == *key && at >= v.in_force.from => out.push(
                    error(
                        C::PrecedenceCycle,
                        span_of(loc, d, relation_key(r)),
                        format!(
                            "`{}` {} `{t}`, which is not an earlier version",
                            world.versioned(v.decl, v.decl),
                            relation_key(r)
                        ),
                    )
                    .with_note(None, "a version may only supersede or revive versions that took effect before it"),
                ),
                _ => {}
            }
        }
    }
    out
}

/// The clause keyword of a relation.
#[must_use]
pub fn relation_key(r: &Relation) -> &'static str {
    match r {
        Relation::SubjectTo(_) => "subject_to",
        Relation::Notwithstanding(_) => "notwithstanding",
        Relation::Supersedes(_) => "supersedes",
        Relation::Revives(_) => "revives",
        Relation::DelegableTo(_) => "delegable_to",
        Relation::Override(_) => "override",
    }
}
