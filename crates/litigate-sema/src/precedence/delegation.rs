//! `delegation-respected` (SPEC §5.5 step 3, L3004): a lower authority may
//! change a higher one's rule only where that rule delegates
//! (`delegable_to`, "unless the court orders otherwise"). Every relation
//! that displaces its target needs it: `override`, `notwithstanding`,
//! `supersedes`, and `revives`. An unlicensed relation is L3004 here and
//! has no effect on selection ([`licensed`]).
use super::lattice::{Lattice, Order};
use crate::hir::{Declaration, Extension, Relation};
use crate::temporal::interval::Window;
use crate::temporal::world::{error, span_of, Decl, DiagnosticExt, Key, Locate, World};
use litigate_base::{Applicability, Derivation, Diagnostic, DiagnosticCode as C, Span};

/// Clauses after whose value a `delegable_to` clause can be inserted.
const ANCHORS: [&str; 7] = [
    "effective",
    "known_at",
    "supersedes",
    "revives",
    "subject_to",
    "notwithstanding",
    "override",
];

fn end_of(s: &Span) -> Span {
    Span {
        file: s.file.clone(),
        line: s.end_line,
        col: s.end_col,
        end_line: s.end_line,
        end_col: s.end_col,
    }
}

/// Whether declaration `by` may displace version `target`: its authority
/// equals or outranks the target's, or the target delegates to it
/// (`delegable_to` naming `by`'s authority exactly).
/// A delegation never reaches past a lock: a lower authority may not
/// displace a deadline locked by `extend { never cite X }` with one
/// granting an extension the target does not ([`lock_widened`]).
#[must_use]
pub fn licensed(world: &World<'_>, by: usize, target: usize) -> bool {
    let mine = &world.decls[by].meta.authority;
    let theirs = &world.decls[target].meta;
    matches!(
        world.lattice().compare(mine, &theirs.authority),
        Order::Above | Order::Equal
    ) || (theirs
        .relations
        .iter()
        .any(|r| matches!(r, Relation::DelegableTo(x) if x == mine))
        && lock_widened(world, by, target).is_none())
}

/// The authority of deadline `target`'s lock (`extend { never cite X }`)
/// when deadline `by` grants an extension `target` does not state (FRAP
/// 26(b): a local rule or order cannot grant there).
#[must_use]
pub fn lock_widened<'w>(world: &'w World<'_>, by: usize, target: usize) -> Option<&'w str> {
    let (Declaration::Deadline(mine), Declaration::Deadline(theirs)) =
        (world.decls[by].decl, world.decls[target].decl)
    else {
        return None;
    };
    let lock = theirs.extend.iter().find_map(Extension::lock)?;
    mine.extend
        .iter()
        .any(|x| x.by().is_some() && !theirs.extend.contains(x))
        .then_some(lock)
}

/// The versions of `key` (only the one starting on `version`, when given)
/// a `supersedes` or `revives` declared by `by` acts on: those that took
/// effect before it did.
fn earlier(world: &World<'_>, by: usize, key: &Key, version: Option<&str>) -> Vec<usize> {
    let from = &world.decls[by].meta.effective.from;
    world
        .versions(key)
        .versions
        .iter()
        .filter(|v| v.in_force.from < *from && version.is_none_or(|d| v.in_force.from == d))
        .map(|v| v.decl)
        .collect()
}

/// Whether `by` may supersede or revive `key` (`@version` when given):
/// [`licensed`] over every version it acts on.
#[must_use]
pub fn licensed_over(world: &World<'_>, by: usize, key: &Key, version: Option<&str>) -> bool {
    earlier(world, by, key, version)
        .into_iter()
        .all(|v| licensed(world, by, v))
}

/// L3004 for every displacing relation of pack `pack` that its target does
/// not delegate, L3007 for overrides of a rule never in force alongside,
/// and L3004 for delegations that do not point downward.
#[must_use]
pub fn check(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    pack: usize,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (a, da) in world
        .decls
        .iter()
        .enumerate()
        .filter(|(_, d)| d.pack == pack)
    {
        let window = Window::of(&da.meta.effective);
        for r in &da.meta.relations {
            match r {
                Relation::Override(text) => {
                    out.extend(overriding(world, lattice, loc, a, text, &window));
                }
                Relation::Notwithstanding(text) => {
                    let target = world.target(world.pack_of(a), text);
                    let affected = overlapping(world, &target, &window);
                    out.extend(unlicensed(
                        world,
                        lattice,
                        loc,
                        a,
                        &target.key,
                        "notwithstanding",
                        &affected,
                    ));
                }
                Relation::Supersedes(text) | Relation::Revives(text) => {
                    let keyword = if matches!(r, Relation::Supersedes(_)) {
                        "supersedes"
                    } else {
                        "revives"
                    };
                    let target = world.target(world.pack_of(a), text);
                    let affected = earlier(world, a, &target.key, target.version.as_deref());
                    out.extend(unlicensed(
                        world,
                        lattice,
                        loc,
                        a,
                        &target.key,
                        keyword,
                        &affected,
                    ));
                }
                Relation::DelegableTo(to) => {
                    out.extend(misdelegated(lattice, loc, da, to));
                }
                // `subject_to` yields: it displaces nothing.
                Relation::SubjectTo(_) => {}
            }
        }
    }
    out
}

/// L3004 when `da` delegates to `to`, which is not below its authority.
fn misdelegated(
    lattice: &Lattice,
    loc: &dyn Locate,
    da: &Decl<'_>,
    to: &str,
) -> Option<Diagnostic> {
    let own = &da.meta.authority;
    let order = lattice.compare(to, own);
    if order == Order::Below {
        return None;
    }
    let why = match order {
        Order::Equal => format!(
            "`{to}` is `{}`'s own authority, which needs no delegation",
            da.meta.id
        ),
        Order::Above => format!("`{to}` outranks `{own}`"),
        _ => format!("`{to}` and `{own}` are not ranked by any `authority` chain"),
    };
    let at = span_of(loc, da, "delegable_to");
    let derivation = Derivation::new(
        "unlicensed",
        format!("unlicensed(`{}` delegable_to {to})", da.meta.id),
    )
    .at(Some(at.clone()))
    .premise(Derivation::new("rank", format!("¬below({to}, {own})")));
    Some(
        error(
            C::MissingDelegation,
            at,
            format!(
                "`{}` ({own}) delegates to `{to}`, which is not a lower authority",
                da.meta.id
            ),
        )
        .with_note(None, why)
        .derived(derivation),
    )
}

/// The versions of `target` (only its `@D` version, when given) in force
/// at some point of `window`.
fn overlapping(
    world: &World<'_>,
    target: &crate::temporal::world::Target,
    window: &Window,
) -> Vec<usize> {
    world
        .versions(&target.key)
        .versions
        .iter()
        .filter(|v| {
            target
                .version
                .as_ref()
                .is_none_or(|d| *d == v.in_force.from)
        })
        .filter(|v| v.in_force.overlaps(window))
        .map(|v| v.decl)
        .collect()
}

fn overriding(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    a: usize,
    text: &str,
    window: &Window,
) -> Vec<Diagnostic> {
    let da = &world.decls[a];
    let target = world.target(world.pack_of(a), text);
    let set = world.versions(&target.key);
    if set.versions.is_empty() {
        return Vec::new();
    }
    let name = target.key.display(world.pack_of(a));
    let affected = overlapping(world, &target, window);
    if affected.is_empty() {
        return vec![error(
            C::NoVersionInForce,
            span_of(loc, da, "override"),
            format!(
                "`{}` overrides `{text}`, which is not in force during {}",
                da.meta.id,
                window.display()
            ),
        )
        .with_note(
            None,
            format!(
                "`{name}` is in force on {}",
                set.versions
                    .iter()
                    .map(|v| v.in_force.display())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )];
    }
    unlicensed(world, lattice, loc, a, &target.key, "override", &affected)
        .into_iter()
        .collect()
}

/// L3004 when `a`'s `keyword` relation on `key` displaces versions in
/// `affected` that do not license it: a missing delegation, or a lock the
/// relation would widen.
fn unlicensed(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    a: usize,
    key: &Key,
    keyword: &str,
    affected: &[usize],
) -> Vec<Diagnostic> {
    let (locked, missing): (Vec<usize>, Vec<usize>) = affected
        .iter()
        .copied()
        .filter(|&b| !licensed(world, a, b))
        .partition(|&b| lock_widened(world, a, b).is_some());
    let name = key.display(world.pack_of(a));
    let span = span_of(loc, &world.decls[a], keyword);
    let da = &world.decls[a];
    let lock = locked.first().map(|&b| {
        let db = &world.decls[b];
        let authority = lock_widened(world, a, b).unwrap_or_default();
        error(
            C::MissingDelegation,
            span.clone(),
            format!(
                "`{}` ({}) {} `{name}` ({}) with an extension it does not grant, but `{name}` is locked: `extend {{ never cite {authority} }}`",
                da.meta.id,
                da.meta.authority,
                verb(keyword),
                db.meta.authority
            ),
        )
        .with_note(
            db.meta.span.clone(),
            format!(
                "`{}` delegates to {}, but a delegation does not reach past its lock: no local rule or order grants an extension there ({authority})",
                world.versioned(b, a),
                da.meta.authority
            ),
        )
    });
    lock.into_iter()
        .chain(missing_delegation(
            world, lattice, loc, a, keyword, &name, &missing, span,
        ))
        .collect()
}

/// L3004 naming the delegation `missing` versions of the target of `a`'s
/// `keyword` relation lack.
#[allow(clippy::too_many_arguments)]
fn missing_delegation(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    a: usize,
    keyword: &str,
    name: &str,
    missing: &[usize],
    span: Span,
) -> Option<Diagnostic> {
    let da = &world.decls[a];
    let mine = &da.meta.authority;
    let &first = missing.first()?;
    let db = &world.decls[first];
    let theirs = &db.meta.authority;
    let mut d = error(
        C::MissingDelegation,
        span,
        format!(
            "`{}` ({mine}) {} `{name}` ({theirs}), but `{name}` does not delegate to {mine}: missing `delegable_to [{mine}]`",
            da.meta.id,
            verb(keyword)
        ),
    );
    let rank = match lattice.compare(mine, theirs) {
        Order::Incomparable => {
            d = d.with_note(None, format!("{mine} and {theirs} are not ranked by any `authority` chain, so `{keyword}` needs an explicit delegation"));
            format!("incomparable({mine}, {theirs})")
        }
        _ => format!("below({mine}, {theirs})"),
    };
    let mut derivation = Derivation::new(
        "unlicensed",
        format!("unlicensed(`{}` {keyword} `{name}`)", da.meta.id),
    )
    .at(Some(d.span.clone()))
    .premise(
        Derivation::new("claim", format!("displaces(`{}`, `{name}`)", da.meta.id))
            .at(Some(d.span.clone())),
    )
    .premise(Derivation::new("rank", rank));
    for &b in missing {
        let db = &world.decls[b];
        derivation = derivation.premise(
            Derivation::new(
                "no-capability",
                format!("¬delegates(`{}`, {mine})", world.versioned(b, a)),
            )
            .at(db.meta.span.clone()),
        );
        d = d.with_note(
            db.meta.span.clone(),
            format!(
                "`{}` {} delegates to no {mine}",
                world.versioned(b, a),
                Window::of(&db.meta.effective).display()
            ),
        );
        let anchor = ANCHORS
            .iter()
            .find_map(|k| loc.clause(db, k).and_then(|(_, v)| v));
        if let Some(at) = anchor {
            d = d.with_fix(
                end_of(&at),
                format!(" delegable_to [{mine}]"),
                Applicability::MaybeIncorrect,
            );
        }
    }
    Some(d.derived(derivation).with_note(
        None,
        format!("only a rule's own text can delegate; `{keyword}` cannot grant itself the power"),
    ))
}

/// How the message reads `keyword`'s effect.
fn verb(keyword: &str) -> &str {
    match keyword {
        "override" => "overrides",
        "notwithstanding" => "is declared notwithstanding",
        other => other,
    }
}
