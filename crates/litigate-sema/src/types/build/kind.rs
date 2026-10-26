//! The instrument class as a capability index (SPEC §5.7, design review
//! 2026-10-23 §2.5): each declaration's effects must be in its package's
//! class row (L2010). Set inclusion over a closed finite set: class (a).
use super::cx::Cx;
use super::item_name;
use crate::db::parse::tree::Item;
use crate::db::span::{Report, TextRange};
use crate::hir::{Declaration, Effect, Kind, Metadata, NodeKind, Relation, Row};
use crate::resolve::sig::{clause, clauses_of};
use crate::types::date;
use litigate_base::DiagnosticCode as C;

/// One effect of a declaration: what, where it is written, and for a
/// prior the actor whose draw it sets.
type Use<'d> = (Effect, TextRange, Option<&'d str>);

/// A reference to another package's declaration (`pack::id`).
fn foreign(target: &str) -> bool {
    target.contains("::")
}

/// The effects `decl`, built from `item`, has in a package of class
/// `kind`: a precedent's displacing relation is its waiver (a prospective
/// override under the waiving power's delegation, docs/litigate/CHANGE.md
/// §2.8), and the period that relation states is the waiver's, not a
/// prescription of its own.
fn effects<'d>(item: &Item, decl: &'d Declaration, kind: Option<&Kind>) -> Vec<Use<'d>> {
    let mut out = declared(item, decl);
    if matches!(kind, Some(Kind::Precedent { .. }))
        && out.iter().any(|(e, _, _)| *e == Effect::Override)
    {
        out.retain(|(e, _, _)| *e != Effect::Prescribe);
        for u in &mut out {
            if u.0 == Effect::Override {
                u.0 = Effect::Waive;
            }
        }
    }
    out
}

/// The effects `decl`, built from `item`, has by its form alone.
#[allow(clippy::too_many_lines)] // one arm per declaration form
fn declared<'d>(item: &Item, decl: &'d Declaration) -> Vec<Use<'d>> {
    let name = item_name(item).map_or(item.range, |n| n.range);
    let mut out: Vec<Use<'d>> = Vec::new();
    let meta: Option<&Metadata> = match decl {
        Declaration::Node(n) => {
            out.push((Effect::Declare, name, None));
            if n.tags.iter().any(|t| t == "fact") {
                out.push((Effect::AddFact, name, None));
            }
            if matches!(n.kind, NodeKind::Chance { .. }) {
                out.extend(
                    n.edges
                        .iter()
                        .filter(|e| e.probability.is_some())
                        .map(|e| (Effect::SetPrior, name, Some(e.actor.as_str()))),
                );
            }
            Some(&n.meta)
        }
        Declaration::Deadline(d) => {
            out.push((Effect::Prescribe, name, None));
            if !d.extend.is_empty() || d.waive.is_some() {
                out.push((Effect::GrantCap, name, None));
            }
            Some(&d.meta)
        }
        Declaration::Counting(c) => {
            out.push((Effect::Prescribe, name, None));
            Some(&c.meta)
        }
        Declaration::Calendar(c) => {
            out.push((Effect::Prescribe, name, None));
            Some(&c.meta)
        }
        Declaration::Limit(l) => {
            out.push((Effect::Prescribe, name, None));
            Some(&l.meta)
        }
        Declaration::Exception(e) => {
            let effect = if foreign(&e.target) {
                Effect::Interpret
            } else {
                Effect::Declare
            };
            out.push((effect, name, None));
            Some(&e.meta)
        }
        Declaration::Grant(g) => {
            out.push((Effect::GrantCap, name, None));
            Some(&g.meta)
        }
        Declaration::Recharacterize(r) => {
            out.push((Effect::Recharacterize, name, None));
            Some(&r.meta)
        }
        Declaration::Invalidate(i) => {
            out.push((Effect::Invalidate, name, None));
            Some(&i.meta)
        }
        Declaration::Authority(_) => {
            out.push((Effect::Rank, name, None));
            None
        }
        Declaration::Rule(r) => {
            out.push((Effect::Declare, name, None));
            Some(&r.meta)
        }
        Declaration::Default(d) => {
            out.push((Effect::Declare, name, None));
            Some(&d.meta)
        }
        Declaration::Transition(t) => {
            out.push((Effect::Declare, name, None));
            Some(&t.meta)
        }
        // A test claims nothing.
        Declaration::Test(_) => None,
        Declaration::Link(_)
        | Declaration::Instance(_)
        | Declaration::TypeAlias(_)
        | Declaration::Enum(_)
        | Declaration::Event(_) => {
            out.push((Effect::Declare, name, None));
            None
        }
    };
    if let Some(m) = meta {
        out.extend(relational(item, m, name));
    }
    out
}

/// The effects of `m`'s relations and delegations, at their clauses of
/// `item` (else at `name`).
fn relational<'d>(item: &Item, m: &Metadata, name: TextRange) -> Vec<Use<'d>> {
    let at = |key: &str| clause(clauses_of(item), key).map_or(name, |f| f.key.range);
    let mut out: Vec<Use<'d>> = m
        .relations
        .iter()
        .filter_map(|r| {
            let (effect, key, target) = match r {
                Relation::Override(t) => (Effect::Override, "override", t),
                Relation::Notwithstanding(t) => (Effect::Override, "notwithstanding", t),
                Relation::Supersedes(t) => (Effect::Version, "supersedes", t),
                Relation::Revives(t) => (Effect::Version, "revives", t),
                // Yielding displaces nothing.
                Relation::SubjectTo(_) => return None,
            };
            foreign(target).then(|| (effect, at(key), None))
        })
        .collect();
    if !m.delegations.is_empty() {
        out.push((Effect::GrantCap, at("delegable"), None));
    }
    out
}

/// The effects `decl`, built from `item`, has in a package of class
/// `kind`, as a row.
pub fn row_of(item: &Item, decl: &Declaration, kind: Option<&Kind>) -> Row {
    effects(item, decl, kind)
        .into_iter()
        .fold(Row::default(), |row, (e, _, _)| row.with(e))
}

/// L2010 for each effect of `decl` its package's class does not permit
/// where its forum family has it, once per effect (and, for a prior under
/// guidance, per actor); L2011 for a grant beyond its cell's window.
pub fn check(cx: &mut Cx<'_, '_>, item: &Item, decl: &Declaration) {
    let Some(ix) = cx.class.filter(|ix| !ix.kind.is_base()) else {
        return;
    };
    let kind = &ix.kind;
    let row = ix.row;
    let mut seen: Vec<(Effect, Option<&str>)> = Vec::new();
    for (effect, at, actor) in effects(item, decl, Some(kind)) {
        let permitted = row.has(effect);
        let key = (effect, actor.filter(|_| permitted));
        if (permitted && actor.is_none_or(|a| kind.may_set_prior(a))) || seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let report = if let (Kind::Guidance { binds }, Some(actor)) = (kind, key.1) {
            Report::new(
                C::ForbiddenEffect,
                at,
                format!(
                    "guidance cannot set the prior of a draw by `{actor}`, whom it does not bind"
                ),
            )
            .note(format!(
                "it binds {} (`binds` in `[package.instrument]`)",
                binds
                    .iter()
                    .map(|b| format!("`{b}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else if let (true, Some((family, title)), Some(cell)) =
            (kind.row().has(effect), &ix.family, &ix.cell)
        {
            Report::new(
                C::ForbiddenEffect,
                at,
                format!(
                    "a package of class `{}` in the `{family}` forum family cannot {}",
                    kind.label(),
                    effect.what()
                ),
            )
            .note(format!(
                "{title} narrows `{}` to {} ({})",
                kind.name(),
                names(row),
                cell.cite
            ))
        } else {
            Report::new(
                C::ForbiddenEffect,
                at,
                format!(
                    "a package of class `{}` cannot {}",
                    kind.label(),
                    effect.what()
                ),
            )
            .note(format!("its class permits {}", names(row)))
            .note(format!(
                "`{}` is permitted to: {}",
                effect.name(),
                Kind::permitting(effect).join(", ")
            ))
        };
        cx.report(report);
    }
    window(cx, item, decl);
}

/// The effects of `row` by name.
fn names(row: Row) -> String {
    row.effects()
        .map(Effect::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// L2011 for a grant of a class whose cell bounds its window (an emergency
/// declaration's, FRCP 87(b)(1)(C)) open-ended or longer than the bound.
fn window(cx: &mut Cx<'_, '_>, item: &Item, decl: &Declaration) {
    let (Declaration::Grant(g), Some(ix)) = (decl, cx.class) else {
        return;
    };
    let Some((days, cell)) = ix.cell.as_ref().and_then(|c| c.window_days.map(|d| (d, c))) else {
        return;
    };
    let w = &g.meta.effective;
    let end = date::add_days(&w.from, u64::from(days));
    if w.to
        .as_ref()
        .is_some_and(|to| end.is_some_and(|end| *to <= end))
    {
        return;
    }
    let at = item_name(item).map_or(item.range, |n| n.range);
    let report = Report::new(
        C::ClassOutsideFamily,
        at,
        format!(
            "an `{}` grants for at most {days} days, but `{}` is in force [{}, {})",
            ix.kind.name(),
            g.meta.id,
            w.from,
            w.to.as_deref().unwrap_or("..")
        ),
    )
    .note(format!("the bound: {}", cell.cite));
    cx.report(report);
}
