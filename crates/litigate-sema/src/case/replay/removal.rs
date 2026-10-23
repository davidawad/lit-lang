//! Option-removing events (SPEC §5.6). A waiver or an order striking a
//! defense does not move the matter; it takes an option away for the rest
//! of it. Such an event is a call to an event type named `Struck`,
//! `Forfeited`, or `Withdrawn` that declares a parameter `option: String`;
//! the string names the removed edge (`edge`, or `pack::edge` outside the
//! primary package) and the event must cite its authority. An election
//! (`Elected`, same parameter) names the option chosen instead: it removes
//! every alternative at each fork offering that edge. Replay records each
//! removal in [`Case::removals`](crate::hir::Case) and refuses any later
//! event that would take a removed edge.
use super::Cx;
use crate::case::graph::{display, Graph};
use crate::case::spans::{error, Build};
use crate::hir::{Edge, Expr, ExprKind, Removal, Type, Value};
use crate::resolve::suggest::nearest;
use litigate_base::{Applicability, DiagnosticCode as C};

/// Event type names that remove an option when they declare
/// `option: String` (compared case-insensitively, like the excusals).
pub(in crate::case) const REMOVALS: [&str; 3] = ["struck", "forfeited", "withdrawn"];

/// The event type name that elects an option, removing its alternatives.
pub(in crate::case) const ELECTION: &str = "elected";

/// What an option event does with the edge its `option` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::case) enum OptionEffect {
    /// The edge leaves the matter.
    Remove,
    /// The edge is chosen; its alternatives leave the matter.
    Elect,
}

/// The effect and `option` argument of `action` when it is an
/// option-removing or electing event.
pub(in crate::case) fn option<'e>(
    graph: &Graph<'_>,
    action: &'e Expr,
) -> Option<(OptionEffect, &'e Expr)> {
    let ExprKind::Call { callee, args } = &action.kind else {
        return None;
    };
    let (pack, name) = callee
        .split_once("::")
        .unwrap_or((graph.primary(), callee.as_str()));
    let lower = name.to_ascii_lowercase();
    let effect = if REMOVALS.contains(&lower.as_str()) {
        OptionEffect::Remove
    } else if lower == ELECTION {
        OptionEffect::Elect
    } else {
        return None;
    };
    let ty = graph.event_type(pack, name)?;
    let i = ty
        .params
        .iter()
        .position(|(p, t)| p == "option" && *t == Type::String)?;
    args.get(i).map(|a| (effect, a))
}

impl Cx<'_, '_> {
    /// Event `k` removes the edge its `option` argument names.
    pub(super) fn remove(&mut self, k: usize, arg: &Expr) {
        let Some((qualified, at, authority)) = self.named(k, arg, "remove") else {
            return;
        };
        if let Some(prior) = self.removed.iter().find(|r| r.edge == qualified) {
            let first = self
                .spans
                .event(prior.event, |e| e.whole.as_ref(), self.id());
            let d = error(
                C::InvalidCaseEvent,
                at,
                format!("`{qualified}` was already removed on {}", prior.date),
            )
            .note_at(Some(first), "removed here");
            self.out.push(d);
            return;
        }
        let (index, event) = self.sorted[k];
        self.removed.push(Removal {
            edge: qualified,
            date: event.date.clone(),
            event: index,
            authority,
        });
    }

    /// Event `k` elects the edge its `option` argument names: every other
    /// edge at each fork offering it is removed (an alternative removed
    /// earlier stays removed once).
    pub(super) fn elect(&mut self, k: usize, arg: &Expr) {
        let Some((qualified, at, authority)) = self.named(k, arg, "elect") else {
            return;
        };
        let (index, event) = self.sorted[k];
        if let Some(prior) = self.removed.iter().find(|r| r.edge == qualified) {
            let first = self
                .spans
                .event(prior.event, |e| e.whole.as_ref(), self.id());
            let d = error(
                C::InvalidCaseEvent,
                at,
                format!(
                    "`{qualified}` was removed on {}, so it cannot be elected",
                    prior.date
                ),
            )
            .note_at(Some(first), "removed here");
            self.out.push(d);
            return;
        }
        let (pack, local) = qualified.split_once("::").unwrap_or(("", &qualified));
        for alternative in self.graph.alternatives(pack, local) {
            let Some(id) = &alternative.id else {
                let d = error(
                    C::InvalidCaseEvent,
                    at.clone(),
                    format!(
                        "`{}` cannot remove the alternative `{}` to `{local}`: it has no id",
                        display(&event.action),
                        alternative.label
                    ),
                )
                .note("name the alternative edge so an election can remove it");
                self.out.push(d);
                continue;
            };
            let edge = format!("{pack}::{id}");
            if self.removed.iter().any(|r| r.edge == edge) {
                continue;
            }
            self.removed.push(Removal {
                edge,
                date: event.date.clone(),
                event: index,
                authority: authority.clone(),
            });
        }
    }

    /// The qualified edge, its span and the cited authority of option
    /// event `k`'s literal argument `arg`; `None` (diagnosed) when the event
    /// is uncited or names no edge. `verb` says what the event does.
    fn named(
        &mut self,
        k: usize,
        arg: &Expr,
        verb: &str,
    ) -> Option<(String, litigate_base::Span, String)> {
        let event = self.sorted[k].1;
        let at = arg
            .span
            .clone()
            .unwrap_or_else(|| self.at(k, |e| e.action.as_ref()));
        let ExprKind::Literal {
            value: Value::String(text),
        } = &arg.kind
        else {
            return None;
        };
        let Some(authority) = event.authority.clone() else {
            let d = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.action.as_ref()),
                format!(
                    "`{}` {verb}s an option only when it cites its authority",
                    display(&event.action)
                ),
            )
            .note("add the order or rule after the party, e.g. `cite FRCP 12(h)(1)`");
            self.out.push(d);
            return None;
        };
        let qualified = self.graph.qualify(self.graph.primary(), text);
        let (pack, local) = qualified.split_once("::").unwrap_or(("", text));
        let ids = self.graph.edge_ids(pack);
        if !ids.contains(local) {
            let mut d = error(
                C::UnresolvedName,
                at.clone(),
                format!(
                    "`{pack}` has no edge `{local}` for `{}` to {verb}",
                    display(&event.action)
                ),
            )
            .note("`option` names an edge by its id: `edge`, or `pack::edge` in another package");
            if let Some(best) = nearest(local, ids.iter().copied()) {
                let spelled = self
                    .graph
                    .respell(self.graph.primary(), &format!("{pack}::{best}"));
                d = d.fix(at, format!("{spelled:?}"), Applicability::MaybeIncorrect);
            }
            self.out.push(d);
            return None;
        }
        Some((qualified, at, authority))
    }

    /// The removal of `edge` (declared in `pack`), if an earlier event removed it.
    pub(super) fn removal(&self, pack: &str, edge: &Edge) -> Option<&Removal> {
        let id = edge.id.as_deref()?;
        let qualified = format!("{pack}::{id}");
        self.removed.iter().find(|r| r.edge == qualified)
    }

    /// Event `k` would take only options earlier events removed, the first
    /// by `r` (L4001); replay stops.
    pub(super) fn takes_removed(&mut self, k: usize, posture: &str, r: &Removal) {
        let event = self.sorted[k].1;
        let removed_at = self.spans.event(r.event, |e| e.whole.as_ref(), self.id());
        let d = error(
            C::InvalidCaseEvent,
            self.at(k, |e| e.action.as_ref()),
            format!(
                "`{}` at stage `{posture}` takes `{}`, which was removed on {}",
                display(&event.action),
                r.edge,
                r.date
            ),
        )
        .note_at(
            Some(removed_at),
            format!("removed here, cite {}", r.authority),
        )
        .note("replay stops at the first illegal event");
        self.out.push(d);
    }
}
