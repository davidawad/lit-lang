//! Defaults, exceptions, tests, nominal types, authority chains, and links
//! and instances (SPEC §5.2, §5.5, §8.4).
use super::cx::{first_expr, Cx};
use crate::db::parse::tree::{
    EdgeItem, Expr, ExprKind, Field, Item, Json, Name, TestStmt, TypeRef,
};
use crate::db::span::Report;
use crate::hir::{
    ActorRole, Authority, BinaryOp, DefaultRule, EnumDecl, EventType, Exception, ExceptionKind,
    Expectation, Instance, Link, Test, Type, TypeAlias, Value,
};
use crate::resolve::sig::ValueKind;
use crate::types::{rational as q, ty};
use litigate_base::DiagnosticCode as C;
use std::collections::{BTreeMap, BTreeSet};

impl Cx<'_, '_> {
    /// `default d when C => V`.
    pub fn default_rule(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        when: &Expr,
        then: &Expr,
    ) -> DefaultRule {
        let meta = self.meta(name, None, item, rule);
        let condition = self.ck.check(when, &Type::Boolean).hir;
        let conclusion = self.ck.infer(then).hir;
        DefaultRule {
            meta,
            condition,
            conclusion,
        }
    }

    /// `exception e rebuts|undercuts d when C [=> V] [priority N]`.
    #[allow(clippy::too_many_arguments)]
    pub fn exception(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        kind: &Name,
        target: &crate::db::parse::tree::RefPath,
        when: &Expr,
        then: Option<&Expr>,
        priority: Option<&Expr>,
    ) -> Option<Exception> {
        let meta = self.meta(name, None, item, rule);
        let resolved = self.ck.scope.value(target).map_err(|r| self.report(r)).ok();
        let target_ty = match resolved.as_ref().map(|(_, f)| &f.item.kind) {
            Some(ValueKind::Default(t)) => t.clone(),
            Some(ValueKind::Exception) | None => None,
            Some(other) => {
                self.report(Report::new(
                    C::TypeMismatch,
                    target.range,
                    format!(
                        "`{}` is a {}, expected a default or exception",
                        target.display(),
                        other.noun()
                    ),
                ));
                None
            }
        };
        let kind = if kind.text == "undercuts" {
            ExceptionKind::Undercutting
        } else {
            ExceptionKind::Rebutting
        };
        match (kind, then) {
            (ExceptionKind::Undercutting, Some(v)) => self.report(Report::new(
                C::InvalidField,
                v.range,
                "an undercutting exception concludes nothing; remove `=> ...`",
            )),
            (ExceptionKind::Rebutting, None) => self.report(Report::new(
                C::InvalidField,
                name.range,
                "a rebutting exception needs `=> <value>`",
            )),
            _ => {}
        }
        let condition = self.ck.check(when, &Type::Boolean).hir;
        let conclusion =
            then.filter(|_| kind == ExceptionKind::Rebutting)
                .map(|v| match &target_ty {
                    Some(t) => self.ck.check(v, t).hir,
                    None => self.ck.infer(v).hir,
                });
        let priority = priority.and_then(|p| {
            match self.ck.check(p, &Type::Number).value {
                Some(Value::Number(v)) if v.denominator == 1 => u32::try_from(v.numerator).ok(),
                _ => None,
            }
            .or_else(|| {
                self.report(Report::new(
                    C::TypeMismatch,
                    p.range,
                    "`priority` is a non-negative integer",
                ));
                None
            })
        });
        Some(Exception {
            meta,
            target: resolved?.0,
            kind,
            condition,
            conclusion,
            priority,
        })
    }

    /// `test "name" { given E on D, expect A op B }`.
    pub fn test(
        &mut self,
        item: &Item,
        name: &Name,
        clauses: &[Field],
        stmts: &[TestStmt],
    ) -> Test {
        let mut givens = Vec::new();
        let mut expects = Vec::new();
        for s in stmts {
            match s {
                TestStmt::Given(e, date) => match &e.kind {
                    ExprKind::Call(callee, args) => {
                        let ch = self.ck.event_call(callee, args, false, e.range);
                        givens.push((ch.hir, date.text.clone()));
                    }
                    _ => self.report(Report::new(
                        C::TypeMismatch,
                        e.range,
                        "`given` takes a complete event such as `Served(complaint, Mail)`",
                    )),
                },
                TestStmt::Expect(e) => {
                    let ch = self.ck.check(e, &Type::Boolean);
                    match ch.hir.kind {
                        crate::hir::ExprKind::Binary {
                            op:
                                op @ (BinaryOp::Eq
                                | BinaryOp::Ne
                                | BinaryOp::Lt
                                | BinaryOp::Le
                                | BinaryOp::Gt
                                | BinaryOp::Ge),
                            lhs,
                            rhs,
                        } => {
                            expects.push(Expectation {
                                lhs: *lhs,
                                op,
                                rhs: *rhs,
                            });
                        }
                        _ if ch.ty.is_some() => self.report(Report::new(
                            C::TypeMismatch,
                            e.range,
                            "`expect` takes a comparison such as `deadline(d) == 2025-03-24`",
                        )),
                        _ => {}
                    }
                }
            }
        }
        Test {
            name: name.text.clone(),
            cite: Self::string(clauses.iter().find(|f| f.key.text == "cite").as_ref()),
            givens,
            expects,
            span: self.ck.span(item.range),
        }
    }

    /// `type T = U` (transparent alias); cycles are L3006.
    pub fn alias(
        &mut self,
        name: &Name,
        target: &TypeRef,
        aliases: &BTreeMap<String, &TypeRef>,
    ) -> Option<TypeAlias> {
        let mut stack = vec![name.text.clone()];
        let mut work = vec![target];
        while let Some(t) = work.pop() {
            if t.name.text == name.text {
                self.report(
                    Report::new(
                        C::PrecedenceCycle,
                        target.range,
                        format!("type alias `{}` refers to itself", name.text),
                    )
                    .note(format!(
                        "cycle: {} -> {}",
                        stack.join(" -> "),
                        name.text
                    )),
                );
                return None;
            }
            if let Some(next) = aliases
                .get(&t.name.text)
                .filter(|_| !stack.contains(&t.name.text))
            {
                stack.push(t.name.text.clone());
                work.push(next);
            }
            work.extend(t.args.iter());
        }
        let mut reports = Vec::new();
        let resolved = ty::resolve(self.ck.scope, target, &mut reports);
        for r in reports {
            self.report(r);
        }
        Some(TypeAlias {
            name: name.text.clone(),
            target: resolved?,
        })
    }

    /// `enum E { a, b }`.
    pub fn enumeration(&mut self, name: &Name, variants: &[Name]) -> EnumDecl {
        let mut seen = BTreeSet::new();
        for v in variants {
            if !seen.insert(v.text.as_str()) {
                self.report(Report::new(
                    C::DuplicateName,
                    v.range,
                    format!("variant `{}` is declared twice", v.text),
                ));
            }
        }
        if variants.is_empty() {
            self.report(Report::new(
                C::InvalidField,
                name.range,
                format!("enum `{}` needs at least one variant", name.text),
            ));
        }
        EnumDecl {
            name: name.text.clone(),
            variants: variants.iter().map(|v| v.text.clone()).collect(),
        }
    }

    /// `event E(p: T)`.
    pub fn event(&mut self, name: &Name, params: &[(Name, TypeRef)]) -> Option<EventType> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for (p, t) in params {
            if !seen.insert(p.text.as_str()) {
                self.report(Report::new(
                    C::DuplicateName,
                    p.range,
                    format!("parameter `{}` is declared twice", p.text),
                ));
            }
            let mut reports = Vec::new();
            let resolved = ty::resolve(self.ck.scope, t, &mut reports);
            for r in reports {
                self.report(r);
            }
            out.push((p.text.clone(), resolved?));
        }
        Some(EventType {
            name: name.text.clone(),
            params: out,
        })
    }

    /// `authority A > B > C`: one HIR entry per level that outranks others.
    #[must_use]
    pub fn authorities(chain: &[Name]) -> Vec<Authority> {
        (0..chain.len().saturating_sub(1))
            .map(|i| Authority {
                name: chain[i].text.clone(),
                outranks: chain[i + 1..].iter().map(|n| n.text.clone()).collect(),
            })
            .collect()
    }

    /// `link id "label": A -> B ...`.
    pub fn link(&mut self, e: &EdgeItem) -> Option<Link> {
        let from_ref = e.from.as_ref()?;
        let from = self.value_ref(from_ref, "node", |k| matches!(k, ValueKind::Node { .. }));
        let edge = self.edge(e, from.as_deref().unwrap_or_default(), None, false);
        let fields = self.once(&e.fields);
        let mut replaces = Vec::new();
        for r in Self::refs(fields.get("replaces")) {
            let known = self.ck.scope.packs.iter().any(|s| r.rest.len() == 1 && s.pack_id == r.head.text && s.values.values().any(|v| matches!(&v.kind, ValueKind::Node { edges, .. } if edges.contains(&r.rest[0].1.text))));
            if known {
                replaces.push(r.display());
            } else {
                self.report(
                    Report::new(
                        C::UnresolvedName,
                        r.range,
                        format!("no edge `{}` to replace", r.display()),
                    )
                    .note("replaced edges are written `pack::edge-id`"),
                );
            }
        }
        Some(Link {
            from: from?,
            edge,
            replaces,
        })
    }

    fn instance_probabilities(
        &mut self,
        f: Option<&&Field>,
    ) -> BTreeMap<String, crate::hir::Rational> {
        let mut probabilities = BTreeMap::new();
        if let Some(ExprKind::Record(ps)) = f.and_then(|x| first_expr(x)).map(|e| &e.kind) {
            for (edge, e) in ps {
                match self.ck.check(e, &Type::Probability).value {
                    Some(Value::Probability(p)) if q::is_probability(p) => {
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
        let pack = f
            .get("pack")
            .and_then(|x| first_expr(x))
            .and_then(|e| match &e.kind {
                ExprKind::Ref(r)
                    if self.ck.scope.packs.iter().any(|s| s.pack_id == r.display()) =>
                {
                    Some(r.display())
                }
                ExprKind::Ref(r) => {
                    self.report(
                        Report::new(
                            C::UnresolvedName,
                            r.range,
                            format!("unknown pack `{}`", r.display()),
                        )
                        .note("the pack must be a declared dependency"),
                    );
                    None
                }
                _ => None,
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
            if let (Some(ExprKind::Ref(edge)), Some((crate::db::parse::tree::Atom::Json(j), _))) =
                (first_expr(p).map(|e| &e.kind), p.values.get(1))
            {
                patch_edges.insert(edge.display(), json(j));
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
            patch_edges,
        })
    }
}

/// JSON record to `serde_json::Value` (`unset` is null; numbers keep their
/// decimal text, rounded to binary64 by serde).
#[must_use]
pub fn json(j: &Json) -> serde_json::Value {
    match j {
        Json::Number(n) => {
            serde_json::from_str(&n.replace('_', "")).unwrap_or(serde_json::Value::Null)
        }
        Json::Str(s) => serde_json::Value::String(s.clone()),
        Json::Bool(b) => serde_json::Value::Bool(*b),
        Json::Unset => serde_json::Value::Null,
        Json::Object(fields) => serde_json::Value::Object(
            fields
                .iter()
                .map(|(k, v)| (k.text.clone(), json(v)))
                .collect(),
        ),
        Json::Array(items) => serde_json::Value::Array(items.iter().map(json).collect()),
    }
}
