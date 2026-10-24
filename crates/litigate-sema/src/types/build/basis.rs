//! `basis` on a node or edge (design review 2026-10-23 §2.10): where its
//! numbers come from, one of `sourced("cite")`, `calibrated("dataset",
//! date)`, `expert("name")` or `estimate`. The cite of `sourced` resolves
//! like an `authority` (`cite-resolves`); a number with no `basis` is an
//! estimate.
use super::cx::{first_expr, Cx};
use crate::db::parse::tree::{Expr, ExprKind, Field};
use crate::db::span::Report;
use crate::hir::Basis;
use litigate_base::DiagnosticCode as C;

/// The forms a `basis` takes, for the error message.
const FORMS: &str =
    "`estimate`, `expert(\"name\")`, `calibrated(\"dataset\", date)` or `sourced(\"cite\")`";

impl Cx<'_, '_> {
    /// The typed `basis` field, if written (L2001 when malformed).
    pub fn basis(&mut self, f: Option<&&Field>) -> Option<Basis> {
        let x = f.and_then(|f| first_expr(f))?;
        let basis = shape(x);
        if basis.is_none() {
            self.report(
                Report::new(C::TypeMismatch, x.range, format!("a `basis` is {FORMS}"))
                    .note("it says where the numbers of this node or edge come from"),
            );
        }
        basis
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
                ("sourced", [cite]) => text(cite).map(|cite| Basis::Sourced { cite }),
                _ => None,
            }
        }
        _ => None,
    }
}
