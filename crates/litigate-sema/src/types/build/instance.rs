//! Instances of a links package (SPEC §8.4): `instance base@origin`, its
//! probabilities, roles, and typed `patch`es.
use super::cx::{first_expr, first_ref, holes_of, nested, Cx};
use crate::db::parse::tree::{Expr, ExprKind, Field, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{self, ActorRole, BinaryOp, EdgePatch, Instance, Type, UnaryOp, Value};
use litigate_base::{Applicability, DiagnosticCode as C};
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
            let (edge, patch) = self.patch(p);
            if patch_edges.insert(edge.clone(), patch).is_some() {
                self.report(Report::new(
                    C::InvalidField,
                    p.key.range,
                    format!("`{edge}` is patched more than once"),
                ));
            }
        }
        let stake = f.get("stake");
        let payoff_transform = self.payoff_transform(f.get("payoff_transform"), stake);
        Some(Instance {
            name: format!("{}@{}", base.text, origin.text),
            pack: pack?,
            note: Self::string(f.get("note")),
            remove_edges: self.idents(f.get("remove_edges")),
            probabilities,
            roles,
            payoff_transform,
            stake: stake
                .and_then(|x| first_expr(x))
                .and_then(|e| self.money(e)),
            patch_edges,
        })
    }

    /// `payoff_transform (e)` (SPEC §8.4): a `Money<usd>` expression over
    /// `payoff` and, when the instance declares `stake`, `stake` (both
    /// `Money<usd>`), built from money and number literals, `+ - * /` and
    /// unary `-`. A declared `stake` the transform does not read is L2007.
    fn payoff_transform(
        &mut self,
        f: Option<&&Field>,
        stake: Option<&&Field>,
    ) -> Option<hir::Expr> {
        let x = f.and_then(|f| first_expr(f));
        let reads_stake = x.is_some_and(|x| reads(x, "stake"));
        if let (Some(s), false) = (stake, reads_stake) {
            self.report(Report::new(
                C::InvalidField,
                s.key.range,
                "the `payoff_transform` does not read this `stake`",
            ));
        }
        let x = x?;
        if !self.transform_shape(x, stake.is_some()) {
            return None;
        }
        let usd = Type::Money("usd".to_owned());
        let mut locals = vec![("payoff".to_owned(), usd.clone())];
        locals.extend(stake.map(|_| ("stake".to_owned(), usd.clone())));
        let outer = std::mem::replace(&mut self.ck.locals, locals);
        let checked = self.ck.check(x, &usd);
        self.ck.locals = outer;
        checked.ty.map(|_| checked.hir)
    }

    /// Whether `x` uses only what a `payoff_transform` may (reporting the
    /// first thing it may not).
    fn transform_shape(&mut self, x: &Expr, stake: bool) -> bool {
        let why = match &x.kind {
            ExprKind::Number(_) | ExprKind::Money(_) => return true,
            ExprKind::Paren(inner) | ExprKind::Unary(UnaryOp::Neg, inner) => {
                return self.transform_shape(inner, stake);
            }
            ExprKind::Binary(
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div,
                l,
                r,
            ) => {
                return self.transform_shape(l, stake) && self.transform_shape(r, stake);
            }
            ExprKind::Ref(r) if r.is_simple() && r.head.text == "payoff" => return true,
            ExprKind::Ref(r) if r.is_simple() && r.head.text == "stake" && stake => return true,
            ExprKind::Ref(r) if r.is_simple() && r.head.text == "stake" => Report::new(
                C::UnresolvedName,
                x.range,
                "`stake` is bound only when the instance declares a `stake`",
            )
            .note("write `stake ?(\"why\")` when only the matter knows it"),
            _ => Report::new(
                C::InvalidOperation,
                x.range,
                "a `payoff_transform` reads `payoff` and a declared `stake`, with money and \
                 number literals, `+ - * /` and unary `-`",
            ),
        };
        self.report(why);
        false
    }

    /// `patch edge { ... }`: the edge fields written, typed as on an edge
    /// declaration. `event`, `replaces` and `when` are not patchable: replay
    /// binds the base edge and derives facts from the base guards, and only
    /// a link replaces edges. A guard changes only by an explicit operation
    /// with its authority (`when and`, `when or`, `supersedes when`; SPEC
    /// §3.6), reserved today (L0010), never by an implicit overwrite.
    fn patch(&mut self, p: &Field) -> (String, EdgePatch) {
        // The parser writes a patch only after its edge's identifier.
        let edge = first_ref(Some(&p))
            .map(RefPath::display)
            .unwrap_or_default();
        let fields = self.once(nested(p));
        for key in ["event", "replaces"] {
            if let Some(f) = fields.get(key) {
                self.not_allowed(f, "in a `patch`: it writes the base edge's other fields");
            }
        }
        if let Some(f) = fields.get("basis") {
            self.not_allowed(
                f,
                "in a `patch`: the base edge's numbers keep their own basis",
            );
        }
        if let Some(f) = fields.get("when") {
            self.report(
                Report::new(
                    C::InvalidField,
                    f.key.range,
                    "`when` is not allowed in a `patch`: it would overwrite the base edge's guard",
                )
                .fix(f.range, "", Applicability::MaybeIncorrect)
                .note(
                    "a guard changes only by an explicit operation citing its authority: \
                     `when and p` narrows it, `when or p` widens it, `supersedes when p` \
                     replaces it (SPEC §3.6; reserved, L0010)",
                ),
            );
        }
        let patch = EdgePatch {
            span: self.ck.span(p.range),
            holes: holes_of(nested(p)),
            ..self.edge_fields(&fields)
        };
        (edge, patch)
    }
}

/// Whether `x` names `name` anywhere.
fn reads(x: &Expr, name: &str) -> bool {
    match &x.kind {
        ExprKind::Ref(r) => r.is_simple() && r.head.text == name,
        ExprKind::Paren(inner) | ExprKind::Unary(_, inner) => reads(inner, name),
        ExprKind::Binary(_, l, r) => reads(l, name) || reads(r, name),
        _ => false,
    }
}
