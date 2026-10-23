//! Instances of a links package (SPEC §8.4): `instance base@origin`, its
//! probabilities, roles, and typed `patch`es.
use super::cx::{first_expr, first_ref, holes_of, nested, Cx};
use crate::db::parse::tree::{ExprKind, Field, Name};
use crate::db::span::Report;
use crate::hir::{ActorRole, EdgePatch, Instance, Type, Value};
use litigate_base::DiagnosticCode as C;
use std::collections::BTreeMap;

impl Cx<'_, '_> {
    fn instance_probabilities(
        &mut self,
        f: Option<&&Field>,
    ) -> BTreeMap<String, crate::hir::Rational> {
        let mut probabilities = BTreeMap::new();
        if let Some(ExprKind::Record(ps)) = f.and_then(|x| first_expr(x)).map(|e| &e.kind) {
            for (edge, e) in ps {
                match self.ck.check(e, &Type::Probability).value {
                    // The checker keeps every `Probability` value in [0, 1].
                    Some(Value::Probability(p)) => {
                        probabilities.insert(edge.text.clone(), p);
                    }
                    _ => self.report(Report::new(
                        C::InvalidProbability,
                        e.range,
                        "instance probabilities must be constants in [0, 1]",
                    )),
                }
            }
        }
        probabilities
    }

    fn instance_roles(&mut self, f: Option<&&Field>) -> BTreeMap<String, ActorRole> {
        let mut roles = BTreeMap::new();
        if let Some(ExprKind::Record(rs)) = f.and_then(|x| first_expr(x)).map(|e| &e.kind) {
            for (actor, e) in rs {
                let role = match &e.kind {
                    ExprKind::Ref(r) if r.head.text == "self" => Some(ActorRole::Protagonist),
                    ExprKind::Ref(r) if r.head.text == "opponent" => Some(ActorRole::Opponent),
                    ExprKind::Ref(r) if r.head.text == "nature" => Some(ActorRole::Nature),
                    _ => None,
                };
                match role {
                    Some(role) => {
                        roles.insert(actor.text.clone(), role);
                    }
                    None => self.report(Report::new(
                        C::TypeMismatch,
                        e.range,
                        "a role is `self`, `opponent`, or `nature`",
                    )),
                }
            }
        }
        roles
    }

    /// `instance base@origin { ... }`.
    pub fn instance(&mut self, base: &Name, origin: &Name, fields: &[Field]) -> Option<Instance> {
        let f = self.once_except(fields, "patch");
        let pack = first_ref(f.get("pack")).and_then(|r| {
            let id = r.display();
            if self.ck.scope.packs.iter().any(|s| s.pack_id == id) {
                return Some(id);
            }
            self.report(
                Report::new(C::UnresolvedName, r.range, format!("unknown pack `{id}`"))
                    .note("the pack must be a declared dependency"),
            );
            None
        });
        if !f.contains_key("pack") {
            self.report(Report::new(
                C::InvalidField,
                base.range,
                "an instance needs `pack <pack-id>`",
            ));
        }
        let probabilities = self.instance_probabilities(f.get("probabilities"));
        let roles = self.instance_roles(f.get("roles"));
        let mut patch_edges = BTreeMap::new();
        for p in fields.iter().filter(|x| x.key.text == "patch") {
            if let Some((edge, patch)) = self.patch(p) {
                if patch_edges.insert(edge.clone(), patch).is_some() {
                    self.report(Report::new(
                        C::InvalidField,
                        p.key.range,
                        format!("`{edge}` is patched more than once"),
                    ));
                }
            }
        }
        Some(Instance {
            name: format!("{}@{}", base.text, origin.text),
            pack: pack?,
            note: Self::string(f.get("note")),
            remove_edges: self.idents(f.get("remove_edges")),
            probabilities,
            roles,
            payoff_transform: Self::string(f.get("payoff_transform")),
            stake: f
                .get("stake")
                .and_then(|x| first_expr(x))
                .and_then(|e| self.money(e)),
            patch_edges,
        })
    }

    /// `patch edge { ... }`: the edge fields written, typed as on an edge
    /// declaration. `event` and `replaces` are not patchable: replay binds
    /// the base edge, and only a link replaces edges.
    fn patch(&mut self, p: &Field) -> Option<(String, EdgePatch)> {
        let ExprKind::Ref(edge) = &first_expr(p)?.kind else {
            return None;
        };
        let fields = self.once(nested(p));
        for key in ["event", "replaces"] {
            if let Some(f) = fields.get(key) {
                self.not_allowed(f, "in a `patch`: it writes the base edge's other fields");
            }
        }
        let patch = EdgePatch {
            span: self.ck.span(p.range),
            holes: holes_of(nested(p)),
            ..self.edge_fields(&fields)
        };
        Some((edge.display(), patch))
    }
}
