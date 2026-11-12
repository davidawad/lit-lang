//! `basis` on a node or edge (design review 2026-10-23 §2.10): where its
//! numbers come from, one of `sourced("cite")`, `calibrated("dataset",
//! date)`, `calibrated(set-id)`, `expert("name")`, `estimate` or
//! `placeholder`. The cite of `sourced` resolves
//! like an `authority` (`cite-resolves`); a number with no `basis` is an
//! estimate.
use super::cx::{first_expr, Cx};
use crate::db::parse::tree::{Atom, Expr, ExprKind, Field};
use crate::db::span::Report;
use crate::hir::Basis;
use litigate_base::DiagnosticCode as C;

/// The forms a `basis` takes, for the error message.
const FORMS: &str =
    "`estimate`, `placeholder`, `expert(\"name\")`, `calibrated(\"dataset\", date)`, `calibrated(set-id)` or `sourced(\"cite\")`";

impl Cx<'_, '_> {
    /// The typed `basis` field, if written (L2001 when malformed).
    pub fn basis(&mut self, f: Option<&&Field>) -> Option<Basis> {
        let x = f.and_then(|f| first_expr(f))?;
        self.basis_expr(x)
    }

    /// Resolves either object-level or per-number basis through one path.
    fn basis_expr(&mut self, x: &Expr) -> Option<Basis> {
        let basis = shape(x);
        if basis.is_none() {
            self.report(
                Report::new(C::TypeMismatch, x.range, format!("a `basis` is {FORMS}"))
                    .note("it says where the numbers of this node or edge come from"),
            );
        }
        if let Some(Basis::CalibratedSet { set }) = &basis {
            let r = crate::db::parse::tree::RefPath {
                head: crate::db::parse::tree::Name {
                    text: set.clone(),
                    range: x.range,
                },
                instance: None,
                rest: Vec::new(),
                version: None,
                range: x.range,
            };
            let known = self.ck.scope.value(&r).is_ok_and(|(_, found)| {
                matches!(found.item.kind, crate::resolve::sig::ValueKind::Calibration)
            });
            if !known {
                self.report(Report::new(
                    C::NumericProvenanceIncomplete,
                    x.range,
                    format!("basis names no calibration set `{set}` in scope"),
                ));
            }
        }
        basis
    }

    /// Resolves the provenance suffix, checking its semantic completeness.
    pub fn number_provenance(
        &mut self,
        f: Option<&&Field>,
    ) -> Option<crate::hir::NumberProvenance> {
        let provenance = f?.values.iter().find_map(|(atom, _)| match atom {
            Atom::NumericProvenance(p) => Some(p.clone()),
            _ => None,
        })?;
        let basis = provenance
            .basis
            .as_ref()
            .and_then(|expr| self.basis_expr(expr));
        if matches!(basis.as_ref(), Some(Basis::Estimate | Basis::Placeholder))
            && provenance.vintage.is_none()
        {
            self.report(Report::new(
                C::NumericProvenanceIncomplete,
                provenance
                    .basis
                    .as_ref()
                    .map_or(f?.range, |expr| expr.range),
                "`estimate` and `placeholder` numeric bases should include `vintage <Date>`",
            ));
        }
        Some(crate::hir::NumberProvenance {
            basis,
            vintage: provenance.vintage.map(|date| date.text),
        })
    }
}

/// `x` as a basis, when it has one of the four forms.
fn shape(x: &Expr) -> Option<Basis> {
    let text = |e: &Expr| match &e.kind {
        ExprKind::Str(s) if !s.trim().is_empty() => Some(s.clone()),
        _ => None,
    };
    match &x.kind {
        ExprKind::Ref(r) if r.is_simple() && r.head.text == "estimate" => Some(Basis::Estimate),
        ExprKind::Ref(r) if r.is_simple() && r.head.text == "placeholder" => {
            Some(Basis::Placeholder)
        }
        ExprKind::Call(callee, args) if callee.is_simple() => {
            match (callee.head.text.as_str(), args.as_slice()) {
                ("expert", [name]) => text(name).map(|name| Basis::Expert { name }),
                ("calibrated", [dataset, date]) => match (text(dataset), &date.kind) {
                    (Some(dataset), ExprKind::Date(as_of)) => Some(Basis::Calibrated {
                        dataset,
                        as_of: as_of.clone(),
                    }),
                    _ => None,
                },
                ("calibrated", [set]) => match &set.kind {
                    ExprKind::Ref(r) if r.is_simple() => Some(Basis::CalibratedSet {
                        set: r.head.text.clone(),
                    }),
                    _ => None,
                },
                ("sourced", [cite]) => text(cite).map(|cite| Basis::Sourced { cite }),
                _ => None,
            }
        }
        _ => None,
    }
}
