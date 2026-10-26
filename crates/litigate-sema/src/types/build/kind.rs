//! The instrument class as a capability index (SPEC §5.7, design review
//! 2026-10-23 §2.5): each declaration's effects must be in its package's
//! class row (L2010). Set inclusion over a closed finite set: class (a).
use super::cx::Cx;
use super::item_name;
use crate::db::parse::tree::Item;
use crate::db::span::{Report, TextRange};
use crate::hir::{Declaration, Effect, Kind, Metadata, NodeKind, Relation};
use crate::resolve::sig::{clause, clauses_of};
use litigate_base::DiagnosticCode as C;

/// One effect of a declaration: what, where it is written, and for a
/// prior the actor whose draw it sets.
type Use<'d> = (Effect, TextRange, Option<&'d str>);

/// A reference to another package's declaration (`pack::id`).
fn foreign(target: &str) -> bool {
    target.contains("::")
}

/// The effects `decl`, built from `item`, has.
fn effects<'d>(item: &Item, decl: &'d Declaration) -> Vec<Use<'d>> {
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

/// L2010 for each effect of `decl` its package's class does not permit,
/// once per effect (and, for a prior under guidance, per actor).
pub fn check(cx: &mut Cx<'_, '_>, item: &Item, decl: &Declaration) {
    let Some(kind) = cx.manifest.map(|m| m.kind.clone()).filter(|k| !k.is_base()) else {
        return;
    };
    let row = kind.row();
    let mut seen: Vec<(Effect, Option<&str>)> = Vec::new();
    for (effect, at, actor) in effects(item, decl) {
        let permitted = row.has(effect);
        let key = (effect, actor.filter(|_| permitted));
        if (permitted && actor.is_none_or(|a| kind.may_set_prior(a))) || seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let report = if let (Kind::Guidance { binds }, Some(actor)) = (&kind, key.1) {
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
            .note(format!(
                "its class permits {}",
                row.effects()
                    .map(Effect::name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .note(format!(
                "`{}` is permitted to: {}",
                effect.name(),
                Kind::permitting(effect).join(", ")
            ))
        };
        cx.report(report);
    }
}
