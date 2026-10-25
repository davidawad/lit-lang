//! Instances of a links package (SPEC §8.4): `instance base@origin`, its
//! probabilities, roles, and typed `patch`es.
use super::cx::{first_expr, first_ref, holes_of, nested, Cx};
use crate::db::parse::tree::{Atom, Expr, ExprKind, Field, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    self, ActorRole, BinaryOp, EdgePatch, GuardOp, GuardOpKind, Instance, Interval, Type, UnaryOp,
    Value,
};
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
    /// declaration, and the typed operations on its guard. `event`,
    /// declaration. `event`, `replaces`, `when`, `spawn` and `joins` are not
    /// patchable: replay binds the base edge, its tracks, and the facts its
    /// guards derive, and only a link replaces edges. A guard changes only
    /// by an explicit operation with its authority (`when and`, `when or`,
    /// `supersedes when`; SPEC §3.6), reserved today (L0010), never by an
    /// implicit overwrite.
    fn patch(&mut self, p: &Field) -> (String, EdgePatch) {
        // The parser writes a patch only after its edge's identifier.
        let edge = first_ref(Some(&p))
            .map(RefPath::display)
            .unwrap_or_default();
        let kind = |f: &Field| {
            GUARD_OPS
                .iter()
                .find(|(k, _)| *k == f.key.text)
                .map(|(_, kind)| *kind)
        };
        let written: Vec<Field> = nested(p)
            .iter()
            .filter(|f| kind(f).is_none())
            .cloned()
            .collect();
        let ops: Vec<(GuardOpKind, &Field)> = nested(p)
            .iter()
            .filter_map(|f| Some((kind(f)?, f)))
            .collect();
        let fields = self.once(&written);
        for key in ["event", "replaces", "spawn", "joins"] {
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
                     replaces it (SPEC §3.6)",
                ),
            );
        }
        let patch = EdgePatch {
            span: self.ck.span(p.range),
            holes: holes_of(&written),
            guard_ops: ops
                .into_iter()
                .filter_map(|(kind, f)| self.guard_op(kind, f))
                .collect(),
            ..self.edge_fields(&fields)
        };
        (edge, patch)
    }

    fn guard_effective(&mut self, clauses: &BTreeMap<&str, &Field>) -> Option<Interval> {
        let Some((Atom::Interval(from, to), _)) =
            clauses.get("effective").and_then(|e| e.values.first())
        else {
            return None;
        };
        if let Some(to) = to.as_ref().filter(|to| to.text <= from.text) {
            self.report(Report::new(
                C::InvalidField,
                from.range.cover(to.range),
                format!("empty interval [{}, {})", from.text, to.text),
            ));
        }
        let known_at = match clauses.get("known_at").and_then(|k| first_expr(k)) {
            Some(Expr {
                kind: ExprKind::Date(d),
                ..
            }) => Some(d.clone()),
            _ => self.manifest.map(|m| m.in_force.known_at.clone()),
        };
        Some(Interval {
            from: from.text.clone(),
            to: to.as_ref().map(|t| t.text.clone()),
            known_at: known_at.unwrap_or_default(),
            retracted: None,
            unknown_before: None,
            commenced: None,
        })
    }

    /// One typed guard operation (SPEC §3.6): its condition, typed as a
    /// fact edge's `when`, and its authority, which is required whole: the
    /// instrument's level (`by Level`), `effective` interval and `cite`. An
    /// operation missing any of them is L2007 and dropped (fail-closed),
    /// never defaulted.
    fn guard_op(&mut self, kind: GuardOpKind, f: &Field) -> Option<GuardOp> {
        let condition = self.guard(f, true, true);
        let clauses = self.once(nested(f));
        let by = first_ref(clauses.get("by")).and_then(|r| {
            self.ck
                .scope
                .authority(&r.head)
                .map_err(|e| self.ck.report(e))
                .ok()
                .map(|()| r.head.text.clone())
        });
        let cite = match clauses.get("cite").and_then(|c| first_expr(c)) {
            Some(Expr {
                kind: ExprKind::Hole(reason),
                range,
            }) => {
                self.ck.holes.push((*range, reason.clone()));
                Some(hir::Authored::Hole(reason.clone()))
            }
            _ => Self::string(clauses.get("cite")).map(hir::Authored::Known),
        };
        let effective = self.guard_effective(&clauses);
        let missing: Vec<&str> = [
            ("`by Level`", clauses.contains_key("by")),
            ("`effective [from, to)`", clauses.contains_key("effective")),
            ("`cite`", clauses.contains_key("cite")),
        ]
        .into_iter()
        .filter_map(|(what, written)| (!written).then_some(what))
        .collect();
        if !missing.is_empty() {
            self.report(
                Report::new(
                    C::InvalidField,
                    f.key.range,
                    format!(
                        "`{}` needs its authority: {} missing",
                        f.key.text,
                        missing.join(", ")
                    ),
                )
                .note(
                    "a guard operation is a modification by a later instrument: it names the \
                     instrument's authority level, the date it takes effect and the instrument \
                     (SPEC §3.6); without them it does not apply",
                ),
            );
        }
        Some(GuardOp {
            kind,
            condition: condition?,
            by: by?,
            cite: cite?,
            effective: effective?,
            span: self.ck.span(f.range),
        })
    }
}

/// The guard operations' keys (the parse tree spells each by its words)
/// and kinds.
const GUARD_OPS: [(&str, GuardOpKind); 3] = [
    ("when and", GuardOpKind::Narrow),
    ("when or", GuardOpKind::Widen),
    ("supersedes when", GuardOpKind::Replace),
];

/// Whether `x` names `name` anywhere.
fn reads(x: &Expr, name: &str) -> bool {
    match &x.kind {
        ExprKind::Ref(r) => r.is_simple() && r.head.text == name,
        ExprKind::Paren(inner) | ExprKind::Unary(_, inner) => reads(inner, name),
        ExprKind::Binary(_, l, r) => reads(l, name) || reads(r, name),
        _ => false,
    }
}
