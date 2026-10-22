//! Option-removing events (SPEC §5.6). A waiver or an order striking a
//! defense does not move the matter; it takes an option away for the rest
//! of it. Such an event is a call to an event type named `Struck`,
//! `Forfeited`, or `Withdrawn` that declares a parameter `option: String`;
//! the string names the removed edge (`edge`, or `pack::edge` outside the
//! primary package) and the event must cite its authority. Replay records
//! each removal in [`Case::removals`](crate::hir::Case) and refuses any
//! later event that would take the removed edge.
use super::Cx;
use crate::case::graph::{display, Graph};
use crate::case::spans::{error, Build};
use crate::hir::{Edge, Expr, ExprKind, Removal, Type, Value};
use crate::resolve::suggest::nearest;
use litigate_base::{Applicability, DiagnosticCode as C};

/// Event type names that remove an option when they declare
/// `option: String` (compared case-insensitively, like the excusals).
pub(in crate::case) const REMOVALS: [&str; 3] = ["struck", "forfeited", "withdrawn"];

/// The `option` argument of `action` when it is an option-removing event.
pub(in crate::case) fn option<'e>(graph: &Graph<'_>, action: &'e Expr) -> Option<&'e Expr> {
    let ExprKind::Call { callee, args } = &action.kind else {
        return None;
    };
    let (pack, name) = callee
        .split_once("::")
        .unwrap_or((graph.primary(), callee.as_str()));
    if !REMOVALS.contains(&name.to_ascii_lowercase().as_str()) {
        return None;
    }
    let ty = graph.event_type(pack, name)?;
    let i = ty
        .params
        .iter()
        .position(|(p, t)| p == "option" && *t == Type::String)?;
    args.get(i)
}

impl Cx<'_, '_> {
    /// Event `k` removes the edge its `option` argument names.
    pub(super) fn remove(&mut self, k: usize, arg: &Expr) {
        let (index, event) = self.sorted[k];
        let at = arg
            .span
            .clone()
            .unwrap_or_else(|| self.at(k, |e| e.action.as_ref()));
        let ExprKind::Literal {
            value: Value::String(text),
        } = &arg.kind
        else {
            return;
        };
        let Some(authority) = event.authority.clone() else {
            let d = error(
                C::InvalidCaseEvent,
                self.at(k, |e| e.action.as_ref()),
                format!(
                    "`{}` removes an option only when it cites its authority",
                    display(&event.action)
                ),
            )
            .note("add the order or rule after the party, e.g. `cite FRCP 12(h)(1)`");
            self.out.push(d);
            return;
        };
        let qualified = self.graph.qualify(self.graph.primary(), text);
        let (pack, local) = qualified.split_once("::").unwrap_or(("", text));
        let ids = self.graph.edge_ids(pack);
        if !ids.contains(local) {
            let mut d = error(
                C::UnresolvedName,
                at.clone(),
                format!(
                    "`{pack}` has no edge `{local}` for `{}` to remove",
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
            return;
        }
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
        self.removed.push(Removal {
            edge: qualified,
            date: event.date.clone(),
            event: index,
            authority,
        });
    }

    /// The removal of `edge` (declared in `pack`), if an earlier event removed it.
    pub(super) fn removal(&self, pack: &str, edge: &Edge) -> Option<&Removal> {
        let id = edge.id.as_deref()?;
        let qualified = format!("{pack}::{id}");
        self.removed.iter().find(|r| r.edge == qualified)
    }

    /// Event `k` would take an option an earlier event removed (L4001);
    /// replay stops (`None`).
    pub(super) fn takes_removed(
        &mut self,
        k: usize,
        posture: &str,
        (pack, edge): &(String, &Edge),
    ) -> Option<String> {
        let r = self.removal(pack, edge)?;
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
        None
    }
}
