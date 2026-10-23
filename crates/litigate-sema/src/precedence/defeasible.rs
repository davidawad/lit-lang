//! Defaults and exceptions (SPEC §5.5, DESIGN §4.5; Pollock 1987).
//!
//! `default d when C => V` concludes V when C holds. A *rebutting*
//! exception concludes something else instead; an *undercutting* one only
//! disables its target. Exceptions may target exceptions. Exceptions on
//! one target must be provably disjoint (no assignment of their atoms makes
//! two conditions true together) or carry distinct priorities, larger
//! winning (`exceptions-ordered`, L3005).
use crate::hir::{Declaration, Exception, Expr};
use crate::temporal::condition::{eval, jointly_satisfiable};
use crate::temporal::interval::Window;
use crate::temporal::world::{error, span_of, Decl, DiagnosticExt, Key, Locate, World};
use litigate_base::{Applicability, Derivation, Diagnostic, DiagnosticCode as C};

/// How a default (or exception) came out, with the declarations that
/// decided it in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion<'a> {
    /// Its condition does not hold.
    NotApplicable,
    /// An undercutting exception disabled it; nothing is concluded.
    Undercut {
        /// Ids from the target down to the undercutting exception.
        by: Vec<String>,
    },
    /// A value is concluded: the target's own, or a rebutting exception's.
    Value {
        /// The concluded expression.
        value: &'a Expr,
        /// Ids from the target down to the declaration that concluded it.
        by: Vec<String>,
    },
}

enum Res<'a> {
    NotApplicable,
    Undercut(Vec<String>),
    Cut(Vec<String>),
    Value(&'a Expr, Vec<String>),
}

/// Exceptions (declaration indexes) targeting `key`.
#[must_use]
pub fn exceptions_on(world: &World<'_>, key: &Key) -> Vec<usize> {
    world.exceptions_on(key).iter().map(|&(i, _)| i).collect()
}

/// Evaluates default `key` on `date`, asking `atom` for each atom (for
/// example `has_event(...)` over a matter history).
///
/// # Errors
/// A message when an atom is unknown, nothing is active, or two applicable
/// exceptions are unordered (statically L3005).
pub fn conclude<'a>(
    world: &'a World<'a>,
    key: &Key,
    date: &str,
    atom: &mut dyn FnMut(&Expr) -> Option<bool>,
) -> Result<Conclusion<'a>, String> {
    let w = super::select(world, key, date, None)
        .map_err(|_| format!("`{}` is not in force on {date}", key.name))?;
    Ok(match node(world, w, date, atom, &mut Vec::new())? {
        Res::NotApplicable => Conclusion::NotApplicable,
        Res::Undercut(by) | Res::Cut(by) => Conclusion::Undercut { by },
        Res::Value(value, by) => Conclusion::Value { value, by },
    })
}

fn node<'a>(
    world: &'a World<'a>,
    i: usize,
    date: &str,
    atom: &mut dyn FnMut(&Expr) -> Option<bool>,
    path: &mut Vec<usize>,
) -> Result<Res<'a>, String> {
    let d = &world.decls[i];
    if path.contains(&i) {
        return Err(format!(
            "`{}` is part of an exception cycle (L3006)",
            d.meta.id
        ));
    }
    let (condition, own): (&Expr, Option<&'a Expr>) = match d.decl {
        Declaration::Default(x) => (&x.condition, Some(&x.conclusion)),
        Declaration::Exception(e) => (&e.condition, e.conclusion.as_ref()),
        _ => return Err(format!("`{}` is not a default or exception", d.meta.id)),
    };
    let holds = eval(condition, atom)
        .ok_or_else(|| format!("`{}`: a condition atom is unknown", d.meta.id))?;
    if !holds {
        return Ok(Res::NotApplicable);
    }
    let me = &d.meta.id;
    let prefix = |mut by: Vec<String>| {
        by.insert(0, me.clone());
        by
    };
    // Undefeated exceptions in force on `date`, as what they make of `i`.
    let mut live: Vec<(Option<u32>, usize, Res<'a>)> = Vec::new();
    for &(e, x) in world.exceptions_on(&world.key(i)) {
        if !Window::of(&world.decls[e].meta.effective).contains(date) {
            continue;
        }
        path.push(i);
        let r = node(world, e, date, atom, path);
        path.pop();
        let made = match r? {
            Res::Cut(by) => Res::Undercut(prefix(by)),
            Res::Value(v, by) => Res::Value(v, prefix(by)),
            Res::NotApplicable | Res::Undercut(_) => continue,
        };
        live.push((x.priority, e, made));
    }
    live.sort_by_key(|x| std::cmp::Reverse(x.0));
    if let [(p, x, _), (q, y, _), ..] = live.as_slice() {
        if p == q {
            return Err(format!(
                "`{}` and `{}` both apply to `{}` with no order",
                world.decls[*x].meta.id, world.decls[*y].meta.id, d.meta.id
            ));
        }
    }
    Ok(match (live.into_iter().next(), own) {
        (Some((_, _, made)), _) => made,
        (None, Some(v)) => Res::Value(v, vec![me.clone()]),
        (None, None) => Res::Cut(vec![me.clone()]),
    })
}

/// L3005 for unordered overlapping exceptions (one of them in pack
/// `pack`), and L3007 for exceptions never in force alongside their target.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let targets: Vec<Key> = world.exception_targets().cloned().collect();
    for t in &targets {
        let on = world.exceptions_on(t);
        let set = world.versions(t);
        for &(e, _) in on.iter().filter(|(e, _)| world.decls[*e].pack == pack) {
            let w = Window::of(&world.decls[e].meta.effective);
            if !set.versions.iter().any(|v| v.in_force.overlaps(&w)) {
                out.push(error(
                    C::NoVersionInForce,
                    span_of(loc, &world.decls[e], "when"),
                    format!(
                        "exception `{}` applies on {}, when `{}` is not in force",
                        world.decls[e].meta.id,
                        w.display(),
                        t.display(world.pack_of(e))
                    ),
                ));
            }
        }
        for (j, &b) in on.iter().enumerate() {
            for &a in &on[..j] {
                if world.decls[a.0].pack == pack || world.decls[b.0].pack == pack {
                    out.extend(pair(world, loc, t, a, b, on));
                }
            }
        }
    }
    out
}

/// The L3005 derivation: `da` and `db` both except `target` on `common`,
/// unordered by priority and not proved disjoint (`overlap` says why).
fn unordered(
    loc: &dyn Locate,
    (da, db): (&Decl<'_>, &Decl<'_>),
    target: &str,
    common: &Window,
    overlap: String,
) -> Derivation {
    let exception = |d: &Decl<'_>| {
        Derivation::new("claim", format!("exception(`{}`, `{target}`)", d.meta.id))
            .at(d.meta.span.clone())
    };
    Derivation::new(
        "conflict",
        format!(
            "conflict(`{target}`, `{}`, `{}`) on {}",
            da.meta.id,
            db.meta.id,
            common.display()
        ),
    )
    .at(Some(span_of(loc, db, "when")))
    .premise(exception(da))
    .premise(exception(db))
    .premise(Derivation::new(
        "no-priority",
        format!("¬ordered(`{}`, `{}`)", da.meta.id, db.meta.id),
    ))
    .premise(Derivation::new("not-disjoint", overlap))
}

fn pair(
    world: &World<'_>,
    loc: &dyn Locate,
    t: &Key,
    (a, ea): (usize, &Exception),
    (b, eb): (usize, &Exception),
    all: &[(usize, &Exception)],
) -> Option<Diagnostic> {
    let (wa, wb) = (
        Window::of(&world.decls[a].meta.effective),
        Window::of(&world.decls[b].meta.effective),
    );
    let common = wa.intersect(&wb)?;
    if matches!((ea.priority, eb.priority), (Some(p), Some(q)) if p != q) {
        return None;
    }
    let sat = jointly_satisfiable(&[&ea.condition, &eb.condition]);
    if sat == Some(false) {
        return None;
    }
    let (da, db) = (&world.decls[a], &world.decls[b]);
    let target = t.display(world.pack_of(b));
    let overlap = if sat.is_none() {
        "their conditions have too many atoms to prove disjoint"
    } else {
        "some history makes both conditions true"
    }
    .to_owned();
    let mut d = error(
        C::ExceptionConflict,
        span_of(loc, db, "when"),
        format!(
            "exceptions `{}` and `{}` to `{target}` can both apply on {} and are not ordered",
            db.meta.id,
            da.meta.id,
            common.display()
        ),
    )
    .with_note(
        da.meta.span.clone(),
        format!("`{}` is declared here", da.meta.id),
    )
    .with_note(None, overlap.clone())
    .derived(unordered(loc, (da, db), &target, &common, overlap))
    .with_note(
        None,
        "make the conditions exclusive, or give both distinct `priority` values (larger wins)",
    );
    let top = all
        .iter()
        .filter_map(|(_, e)| e.priority)
        .max()
        .unwrap_or(0);
    if ea.priority.is_none() {
        if let Some(at) = loc.end(da) {
            d = d.with_fix(
                at,
                format!(" priority {}", top + 1),
                Applicability::MaybeIncorrect,
            );
        }
    }
    let next = if ea.priority.is_none() {
        top + 2
    } else {
        top + 1
    };
    match (eb.priority, loc.end(db), loc.clause(db, "priority")) {
        (None, Some(at), _) => {
            d = d.with_fix(
                at,
                format!(" priority {next}"),
                Applicability::MaybeIncorrect,
            );
        }
        (Some(_), _, Some((_, Some(value)))) => {
            d = d.with_fix(value, next.to_string(), Applicability::MaybeIncorrect);
        }
        _ => {}
    }
    Some(d)
}
