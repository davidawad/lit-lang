//! Defaults, exceptions, tests, nominal types, authority chains, and links
//! and instances (SPEC §5.2, §5.5, §8.4).
use super::cx::{first_expr, Cx};
use super::edge::Draw;
use crate::db::parse::tree::{EdgeItem, Expr, ExprKind, Field, Item, Name, TestStmt, TypeRef};
use crate::db::span::Report;
use crate::hir::{
    Authority, BinaryOp, DefaultRule, EnumDecl, EventType, Exception, ExceptionKind, Expectation,
    Link, Test, Type, TypeAlias, Value,
};
use crate::resolve::sig::ValueKind;
use crate::types::ty;
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
        let condition = self.ck.condition(when).hir;
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
        let target_ty = match resolved.as_ref().map(|(_, f)| (f.sig, &f.item.kind)) {
            Some((sig, ValueKind::Default(t) | ValueKind::Exception(t))) => {
                t.as_ref().map(|t| self.ck.scope.localize(sig, t))
            }
            None => None,
            Some((_, other)) => {
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
        let condition = self.ck.condition(when).hir;
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
                TestStmt::Given(e, date) => {
                    if let Some((callee, args)) = self.call(
                        e,
                        C::TypeMismatch,
                        "`given` takes a complete event such as `Served(complaint, Mail)`",
                    ) {
                        let ch = self.ck.event_call(callee, args, false, e.range);
                        givens.push((ch.hir, date.text.clone()));
                    }
                }
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

    /// `event E(p: T) [effect] [paper k]`.
    pub fn event(
        &mut self,
        name: &Name,
        params: &[(Name, TypeRef)],
        effect: &[Field],
    ) -> Option<EventType> {
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
        let paper = effect
            .iter()
            .find(|f| f.key.text == "paper")
            .and_then(|f| super::cx::first_ref(Some(&f)))
            .and_then(|r| self.value_ref(r, "filing limit", |k| matches!(k, ValueKind::Limit)));
        let effect = self.effect(name, &out, effect);
        Some(EventType {
            name: name.text.clone(),
            params: out,
            effect,
            paper,
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
        // A link's edge always has a `from` (the grammar requires it).
        let from = e
            .from
            .as_ref()
            .and_then(|r| self.value_ref(r, "node", |k| matches!(k, ValueKind::Node { .. })));
        self.link_from = e.from.as_ref().map(|r| r.head.text.clone());
        // A link leaving a chance node or a terminal (whose onward route is
        // drawn) may carry `p`; one leaving a choice may not (SPEC §5.1).
        let drawn = e.from.as_ref().is_some_and(|r| {
            matches!(
                self.ck.scope.value(r).map(|(_, found)| &found.item.kind),
                Ok(ValueKind::Node { chance, terminal, .. }) if *chance || *terminal
            )
        });
        let draw = if drawn {
            Draw::Allowed
        } else {
            Draw::Forbidden
        };
        let edge = self.edge(e, from.as_deref().unwrap_or_default(), None, draw, false);
        self.link_from = None;
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
        replaces.extend(self.derived_replaces(fields.get("replaces")));
        Some(Link {
            from: from?,
            edge,
            replaces,
        })
    }

    /// String items of `replaces`: anonymous edges named by their derived
    /// id `pack::from->to#n`, kept verbatim (SPEC §8.4). The pack and its
    /// `from` node must exist.
    fn derived_replaces(&mut self, f: Option<&&Field>) -> Vec<String> {
        let Some(ExprKind::List(items)) = f.and_then(|x| first_expr(x)).map(|e| &e.kind) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for item in items {
            let ExprKind::Str(text) = &item.kind else {
                continue;
            };
            let known = text
                .split_once("::")
                .and_then(|(pack, rest)| Some((pack, rest.split_once("->")?.0)))
                .is_some_and(|(pack, from)| {
                    self.ck.scope.packs.iter().any(|s| {
                        s.pack_id == pack
                            && s.values
                                .get(from)
                                .is_some_and(|v| matches!(&v.kind, ValueKind::Node { .. }))
                    })
                });
            if known {
                out.push(text.clone());
            } else {
                self.report(
                    Report::new(
                        C::UnresolvedName,
                        item.range,
                        format!("no edge `{text}` to replace"),
                    )
                    .note("an anonymous edge is named by its derived id `pack::from->to#n`"),
                );
            }
        }
        out
    }
}
