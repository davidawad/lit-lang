//! Fact nodes in replay (SPEC §5.1, §5.6). A `fact` chance node is
//! resolved by the matter: by the guard of the one edge whose `when` holds
//! on the history so far, else by the case's `facts` entry. A `facts`
//! entry the history contradicts is L4003.
//!
//! A guard reads `has_event(P)`, `date_of(P)` (the date of the latest
//! event matching `P`) and `deadline(d)` (the earliest due date of the
//! matter's instances of `d`). An atom over an event that has not happened
//! or a deadline with no instance is unknown, and so, unless the
//! connectives decide it anyway, is the guard: the fact is not yet
//! derivable.
use super::obligations::{effective, logical};
use super::Cx;
use crate::case::graph::pack_of;
use crate::case::spans::error;
use crate::case::{Effect, Step};
use crate::hir::{BinaryOp, Declaration, Expr, ExprKind, Node, Value};
use crate::temporal::condition::{eval, has_event, matches};
use litigate_base::DiagnosticCode as C;
use std::collections::BTreeSet;

impl<'g> Cx<'_, 'g> {
    /// Follows fact nodes before sorted event `k`, each at most once: the
    /// posture the event finds, with its node version.
    pub(super) fn follow_facts(
        &mut self,
        mut posture: String,
        k: usize,
    ) -> Option<(String, &'g Node)> {
        let mut followed = BTreeSet::new();
        loop {
            let node = self.node(&posture, k)?;
            let Some(edge) = self
                .fact_edge(&posture, node, k)
                .filter(|_| followed.insert(posture.clone()))
            else {
                return Some((posture, node));
            };
            let pack = pack_of(&posture).to_owned();
            let local = edge.rsplit("::").next().unwrap_or(&edge);
            let next = node
                .edges
                .iter()
                .find(|e| e.id.as_deref() == Some(local))
                .map(|e| self.graph.target(&pack, e))?;
            self.trace.push(Step {
                date: String::new(),
                event: "fact".to_owned(),
                effect: Effect::Fact,
                from: posture,
                to: next.clone(),
                edge: Some(edge),
            });
            posture = next;
        }
    }

    /// At `as_of`, the fact the guards of the replayed `posture` derive from
    /// the whole history. The posture stays; the fact goes to the matter's
    /// `facts`, which resolve it in the engine.
    pub(super) fn derive_at_as_of(&mut self, posture: &str) {
        let at = self.sorted.len();
        let events = self.events_to_as_of(pack_of(posture));
        let known_at = self.case.known_at.as_deref();
        if let Ok((_, Declaration::Node(node))) =
            self.graph.governing(posture, &events, at, known_at)
        {
            if let Some(edge) = self.fact_edge(posture, node, at) {
                self.derived.insert(posture.to_owned(), edge);
            }
        }
    }

    /// The qualified edge that resolves fact node `posture` (version `node`)
    /// before sorted event `k` (`as_of` when `k` is past the last event):
    /// the edge whose guard holds, which a `facts` entry must agree with
    /// (L4003), else the `facts` entry. `None` for any other node.
    fn fact_edge(&mut self, posture: &str, node: &Node, k: usize) -> Option<String> {
        if !node.tags.iter().any(|t| t == "fact") {
            return None;
        }
        let pack = pack_of(posture);
        let derived = node
            .edges
            .iter()
            .find(|e| {
                e.guard
                    .as_ref()
                    .is_some_and(|g| eval(g, &mut |a| self.atom(pack, k, a)) == Some(true))
            })
            .and_then(|e| e.id.as_ref())
            .map(|id| format!("{pack}::{id}"));
        let stated = self.case.facts.get(posture).cloned();
        match (derived, stated) {
            (Some(d), Some(s)) if d != s => {
                let at = self.spans.facts.get(posture).map(|(_, edge)| edge);
                let e = error(
                    C::InvalidCaseFact,
                    self.spans.at(at, self.id()),
                    format!("the matter's history resolves fact `{posture}` to `{d}`, not `{s}`"),
                )
                .note("a `when` guard that holds derives the fact; drop the `facts` entry or correct the history");
                self.out.push(e);
                Some(d)
            }
            (derived, stated) => derived.or(stated),
        }
    }

    /// One guard atom before sorted event `k`; `None` when unknown.
    fn atom(&self, pack: &str, k: usize, a: &Expr) -> Option<bool> {
        let ExprKind::Binary { op, lhs, rhs } = &a.kind else {
            return has_event(a).map(|p| (0..k).any(|i| matches(p, self.view(pack, i))));
        };
        let (x, y) = (self.date(pack, k, lhs)?, self.date(pack, k, rhs)?);
        Some(match op {
            BinaryOp::Lt => x < y,
            BinaryOp::Le => x <= y,
            BinaryOp::Gt => x > y,
            BinaryOp::Ge => x >= y,
            BinaryOp::Ne => x != y,
            _ => x == y,
        })
    }

    /// A date term of a guard before sorted event `k`; `None` when unknown.
    fn date(&self, pack: &str, k: usize, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Literal {
                value: Value::Date(d),
            } => Some(d.clone()),
            ExprKind::Call { callee, args } if callee == "date_of" => args.iter().find_map(|p| {
                (0..k)
                    .rev()
                    .find(|&i| matches(p, self.view(pack, i)))
                    .map(|i| self.sorted[i].1.date.clone())
            }),
            ExprKind::Call { args, .. } => args.iter().find_map(|a| match &a.kind {
                ExprKind::Ref { target } => self.due(pack, k, target),
                _ => None,
            }),
            _ => None,
        }
    }

    /// `deadline(target)` before sorted event `k`: the earliest current due
    /// date of the matter's instances of the deadline; `None` with none.
    fn due(&self, pack: &str, k: usize, target: &str) -> Option<String> {
        let q = logical(&self.graph.qualify(pack, target));
        let as_of = k == self.sorted.len();
        (0..self.obligations.len())
            .filter(|&i| logical(&self.obligations[i].deadline) == q)
            .filter_map(|i| {
                self.due_of(i, k, as_of)
                    .and_then(|due| effective(&self.obligations[i], &due).ok())
            })
            .min()
    }
}
