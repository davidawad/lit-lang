//! Refinements of aliases and events (SPEC §3.5): typed in the qualifier
//! grammar, then checked satisfiable by the SMT tier: a refinement no
//! value meets makes its type empty, which is a declaration error (L2014),
//! and a question the solver does not decide is a hole (L2015).
use super::cx::Cx;
use crate::db::parse::tree::{Field, Name, TypeRef};
use crate::db::span::{Report, TextRange};
use crate::hir::{self, Refinement, Type};
use crate::types::refinement::{self as refine, smt, Located};
use litigate_base::DiagnosticCode as C;

impl Cx<'_, '_> {
    /// The refinements of alias `name = target`, resolved to `base`.
    pub(super) fn alias_refinements(
        &mut self,
        name: &Name,
        target: &TypeRef,
        base: &Type,
    ) -> Vec<Refinement> {
        let Some(found) = refine::alias(&mut self.ck, target, base) else {
            return Vec::new();
        };
        // Each refinement names the value by its own binder; the question
        // names it once.
        let one: Vec<Located> = found
            .iter()
            .map(|(r, at)| {
                let mut r = r.clone();
                r.predicate = refine::rename(&r.predicate, &r.binder, "it");
                (r, *at)
            })
            .collect();
        let vars = [("it".to_owned(), base.clone())];
        self.satisfiable(&format!("type `{}`", name.text), &vars, &one, &found);
        found.into_iter().map(|(r, _)| r).collect()
    }

    /// The refinements of event `name(params)`, resolved to `typed`.
    pub(super) fn event_refinements(
        &mut self,
        name: &Name,
        params: &[(Name, TypeRef)],
        typed: &[(String, Type)],
        effect: &[Field],
    ) -> Vec<Refinement> {
        let extends = effect.iter().any(|f| f.key.text == "extends");
        let Some(found) = refine::event(&mut self.ck, params, typed, extends) else {
            return Vec::new();
        };
        let mut vars = typed.to_vec();
        if extends {
            vars.extend(
                refine::bound_dates()
                    .into_iter()
                    .filter(|(d, _)| typed.iter().all(|(p, _)| p != d)),
            );
        }
        self.satisfiable(&format!("`{}(..)`", name.text), &vars, &found, &found);
        found.into_iter().map(|(r, _)| r).collect()
    }

    /// L2014 when no value of `vars` satisfies every refinement of
    /// `subject` (`asked`, as the solver is asked; `shown`, as reported),
    /// naming an unsat core; L2015 when the solver does not decide.
    fn satisfiable(
        &mut self,
        subject: &str,
        vars: &[(String, Type)],
        asked: &[Located],
        shown: &[Located],
    ) {
        if asked.is_empty() {
            return;
        }
        // Each conjunct is its own assertion, so an unsat core names only
        // the conjuncts that conflict.
        let (asked, shown) = (split(asked), split(shown));
        let scope = self.ck.scope;
        let variants: Vec<Option<&[String]>> = vars
            .iter()
            .map(|(_, t)| match t {
                Type::Named(en) => scope.enum_variants(en),
                _ => None,
            })
            .collect();
        let smt_vars: Vec<smt::Var<'_>> = vars
            .iter()
            .zip(&variants)
            .map(|((name, ty), variants)| smt::Var {
                name,
                ty,
                variants: *variants,
            })
            .collect();
        let preds: Vec<_> = asked.iter().map(|(r, _)| r.predicate.clone()).collect();
        match smt::decide(&smt_vars, &preds) {
            smt::Decision::Sat(_) => {}
            smt::Decision::Unsat(core) => self.vacuous(subject, &shown, &core),
            smt::Decision::Unknown(why) => {
                let at = shown.first().map_or(TextRange::default(), |(_, r)| *r);
                self.report(
                    Report::new(
                        C::RefinementUndecided,
                        at,
                        format!(
                            "the SMT tier did not decide whether any {subject} satisfies its refinements: {why}"
                        ),
                    )
                    .note("an undecided refinement is a hole, never a pass: simplify the predicates (fewer disjunctions, smaller constants)"),
                );
            }
        }
    }

    /// L2014 at the last conjunct of `core`, the others as located notes.
    /// The solver names at least one assertion of an unsatisfiable
    /// question: each conjunct's domain is satisfiable alone.
    fn vacuous(&mut self, subject: &str, shown: &[Located], core: &[usize]) {
        let members: Vec<&Located> = core.iter().filter_map(|&i| shown.get(i)).collect();
        let text = |r: &Refinement| {
            let via = r
                .via
                .as_ref()
                .map(|a| format!(" (from type `{a}`)"))
                .unwrap_or_default();
            format!("`{}`{via}", refine::render(&r.predicate))
        };
        let (rest, last) = members.split_at(members.len().saturating_sub(1));
        let (shown_last, at) = (
            last.iter().map(|(r, _)| text(r)).collect::<String>(),
            last.iter().map(|(_, at)| *at).next().unwrap_or_default(),
        );
        let message = if rest.is_empty() {
            format!("no {subject} satisfies its refinement {shown_last}: the type is empty")
        } else {
            format!(
                "no {subject} satisfies its refinements: {shown_last} contradicts {}",
                rest.iter()
                    .map(|(r, _)| text(r))
                    .collect::<Vec<_>>()
                    .join(" and ")
            )
        };
        let mut report = Report::new(C::VacuousRefinement, at, message);
        for (r, range) in rest {
            report = report.note_at(*range, format!("contradicted: {}", text(r)));
        }
        self.report(report.note(
            "a value no program can write is a declaration error: relax or remove one of these refinements",
        ));
    }
}

/// Each refinement's top-level conjuncts, as refinements at its range.
fn split(located: &[Located]) -> Vec<Located> {
    let mut out = Vec::new();
    for (r, at) in located {
        let mut parts = Vec::new();
        conjuncts(&r.predicate, &mut parts);
        out.extend(parts.into_iter().map(|p| {
            let mut part = r.clone();
            part.predicate = p.clone();
            (part, *at)
        }));
    }
    out
}

fn conjuncts<'e>(e: &'e hir::Expr, out: &mut Vec<&'e hir::Expr>) {
    match &e.kind {
        hir::ExprKind::Binary {
            op: hir::BinaryOp::And,
            lhs,
            rhs,
        } => {
            conjuncts(lhs, out);
            conjuncts(rhs, out);
        }
        _ => out.push(e),
    }
}
