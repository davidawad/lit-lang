//! `delegation-respected` (SPEC §5.5 step 3, L3004): a lower authority may
//! change a higher one's rule only where that rule delegates
//! (`delegable_to`, "unless the court orders otherwise").
use super::lattice::{Lattice, Order};
use crate::hir::Relation;
use crate::temporal::interval::Window;
use crate::temporal::world::{error, span_of, DiagnosticExt, Locate, World};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, Span};

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

/// L3004 for every `override` of pack `pack` that its target does not
/// delegate, L3007 for overrides of a rule never in force alongside, and
/// L3004 for delegations that do not point downward.
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
                    out.extend(overriding(world, lattice, loc, a, text, &window))
                }
                Relation::DelegableTo(to) => {
                    let own = &da.meta.authority;
                    let order = lattice.compare(to, own);
                    if order != Order::Below {
                        let why = match order {
                            Order::Equal => format!(
                                "`{to}` is `{}`'s own authority, which needs no delegation",
                                da.meta.id
                            ),
                            Order::Above => format!("`{to}` outranks `{own}`"),
                            _ => format!(
                                "`{to}` and `{own}` are not ranked by any `authority` chain"
                            ),
                        };
                        out.push(
                            error(
                                C::MissingDelegation,
                                span_of(loc, da, "delegable_to"),
                                format!("`{}` ({own}) delegates to `{to}`, which is not a lower authority", da.meta.id),
                            )
                            .with_note(None, why),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    out
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
    let affected: Vec<usize> = set
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
        .collect();
    let span = span_of(loc, da, "override");
    if affected.is_empty() {
        return vec![error(
            C::NoVersionInForce,
            span,
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
    let mine = &da.meta.authority;
    let missing: Vec<usize> = affected
        .into_iter()
        .filter(|&b| {
            let db = &world.decls[b];
            let order = lattice.compare(mine, &db.meta.authority);
            let delegated = db
                .meta
                .relations
                .iter()
                .any(|r| matches!(r, Relation::DelegableTo(x) if x == mine));
            !matches!(order, Order::Above | Order::Equal) && !delegated
        })
        .collect();
    let Some(&first) = missing.first() else {
        return Vec::new();
    };
    let db = &world.decls[first];
    let theirs = &db.meta.authority;
    let mut d = error(
        C::MissingDelegation,
        span,
        format!(
            "`{}` ({mine}) overrides `{name}` ({theirs}), but `{name}` does not delegate to {mine}: missing `delegable_to [{mine}]`",
            da.meta.id
        ),
    );
    if lattice.compare(mine, theirs) == Order::Incomparable {
        d = d.with_note(None, format!("{mine} and {theirs} are not ranked by any `authority` chain, so the override needs an explicit delegation"));
    }
    for &b in &missing {
        let db = &world.decls[b];
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
    vec![d.with_note(
        None,
        "only a rule's own text can delegate; an override cannot grant itself the power",
    )]
}
