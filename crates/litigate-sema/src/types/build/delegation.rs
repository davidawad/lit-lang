//! Typed delegation capabilities and characterization (SPEC §5.3, §5.5;
//! design review §2.4): `delegable { to L, op O, max N.unit, discretion
//! S, on M(), cite X }` clauses, the legacy `delegable_to [L]` they
//! replace, a deadline's `characterization`, and the capabilities a
//! jurisdictional limit forbids.
use super::cx::{first_expr, first_ref, nested, Cx};
use crate::db::parse::tree::{ExprKind, Field, RefPath};
use crate::db::span::Report;
use crate::hir::{CauseStandard, CharacterKind, Characterization, DelegatedOp, Delegation};
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

/// The names of a `Ref` or `[Ref, ...]` field value.
fn names<'f>(f: Option<&&'f Field>) -> Vec<&'f RefPath> {
    match f.and_then(|f| first_expr(f)).map(|e| &e.kind) {
        Some(ExprKind::Ref(r)) => vec![r],
        _ => Cx::refs(f),
    }
}

impl Cx<'_, '_> {
    /// `delegable_to [L, ...]`: replaced by typed capabilities (L2007, with
    /// a fix spelling the replacement); it delegates nothing.
    pub(super) fn legacy_delegation(&mut self, f: &Field) {
        let levels: Vec<String> = Self::refs(Some(&f)).iter().map(|r| r.display()).collect();
        let to = match levels.as_slice() {
            [one] => one.clone(),
            _ => format!("[{}]", levels.join(", ")),
        };
        self.report(
            Report::new(
                C::InvalidField,
                f.key.range,
                "`delegable_to` is replaced by a typed capability naming what the lower authority may do: `delegable { to <Level>, op <op> [, max N.days] [, discretion <Standard>] [, on <Motion>()] [, cite ...] }`",
            )
            .note(format!(
                "operations: {}",
                DelegatedOp::ALL.map(DelegatedOp::name).join(", ")
            ))
            .fix(
                f.range,
                format!("delegable {{ to {to}, op set }}"),
                Applicability::MaybeIncorrect,
            ),
        );
    }

    /// One `delegable { ... }` clause: a capability per principal it
    /// names. `to` and `op` are required, each principal must be an
    /// authority level, each operation one of [`DelegatedOp::ALL`], `max`
    /// bounds an extension, so it needs `op extend` or `op set`, and
    /// `discretion` names a cause standard (L2007); `on` is an event
    /// pattern (L1002, L2001).
    pub(super) fn delegation(&mut self, f: &Field) -> Vec<Delegation> {
        let fields = self.once(nested(f));
        let mut ok = true;
        for key in ["to", "op"] {
            if !fields.contains_key(key) {
                self.report(Report::new(
                    C::InvalidField,
                    f.key.range,
                    format!("a `delegable` capability names `{key}`: `delegable {{ to <Level>, op <op> }}`"),
                ));
                ok = false;
            }
        }
        let mut ops = Vec::new();
        for r in names(fields.get("op")) {
            let text = r.display();
            match DelegatedOp::ALL.into_iter().find(|o| o.name() == text) {
                Some(op) if !ops.contains(&op) => ops.push(op),
                Some(_) => {}
                None => {
                    self.report(Report::new(
                        C::InvalidField,
                        r.range,
                        format!(
                            "`{text}` is not a delegable operation: expected {}",
                            DelegatedOp::ALL
                                .map(|o| format!("`{}`", o.name()))
                                .join(", ")
                        ),
                    ));
                    ok = false;
                }
            }
        }
        let max = match fields.get("max").and_then(|m| first_expr(m)) {
            Some(e) => {
                let bounded = ops
                    .iter()
                    .any(|o| matches!(o, DelegatedOp::Extend | DelegatedOp::Set));
                if !bounded {
                    self.report(Report::new(
                        C::InvalidField,
                        e.range,
                        "`max` bounds an extension: it needs `op extend` or `op set`",
                    ));
                    ok = false;
                }
                self.days(e)
            }
            None => None,
        };
        let (standard, on) = self.condition_and_procedure(&fields).unwrap_or_else(|| {
            ok = false;
            (None, None)
        });
        let cite = self.cite_text(fields.get("cite"));
        let mut out = Vec::new();
        for r in names(fields.get("to")) {
            let known = self
                .ck
                .scope
                .authority(&r.head)
                .map_err(|e| self.ck.report(e))
                .is_ok();
            if known && ok {
                out.push(Delegation {
                    to: r.head.text.clone(),
                    ops: ops.clone(),
                    max,
                    standard,
                    on: on.clone(),
                    cite: cite.clone(),
                });
            }
        }
        out
    }

    /// The condition `discretion S` (a cause standard, L2007 otherwise) and
    /// the procedure `on M()` (an event pattern) of a `delegable` clause;
    /// `None` when either is ill-typed.
    fn condition_and_procedure(
        &mut self,
        fields: &BTreeMap<&str, &Field>,
    ) -> Option<(Option<CauseStandard>, Option<crate::hir::Expr>)> {
        let standard = match fields.get("discretion") {
            Some(d) => Some(self.standard(first_ref(Some(d))?)?),
            None => None,
        };
        let on = match fields.get("on") {
            Some(m) => Some(self.pattern(first_expr(m)?)?),
            None => None,
        };
        Some((standard, on))
    }

    /// `characterization jurisdictional | claim_processing [cite X]`.
    pub(super) fn characterization(&mut self, f: &Field) -> Option<Characterization> {
        let r = first_ref(Some(&f))?;
        let text = r.display();
        let kind = [
            CharacterKind::Jurisdictional,
            CharacterKind::ClaimProcessing,
        ]
        .into_iter()
        .find(|k| k.name() == text);
        if kind.is_none() {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!("`{text}` is not a characterization: expected `jurisdictional` or `claim_processing`"),
            ));
        }
        // The value after the level is the optional `cite`.
        let rest = Field {
            key: f.key.clone(),
            values: f.values.iter().skip(1).cloned().collect(),
            range: f.range,
        };
        let cite = self.cite_text(Some(&&rest));
        Some(Characterization { kind: kind?, cite })
    }

    /// L2009 for each capability a jurisdictional deadline declares that
    /// its character forbids: a `waive`, a discretionary (unbounded)
    /// `extend`, or a `delegable` (*Bowles v. Russell*, 551 U.S. 205
    /// (2007): no equitable exception, waiver or forfeiture; only the
    /// bounded extension the statute itself provides).
    pub(super) fn jurisdictional(&mut self, clauses: &[Field], opts: &[Field]) {
        let unbounded = |f: &&Field| {
            f.key.text == "extend"
                && !nested(f)
                    .iter()
                    .any(|n| matches!(n.key.text.as_str(), "max" | "to" | "never"))
        };
        for f in opts.iter().filter(unbounded) {
            self.forbidden(
                f,
                "extension without a bound (only the bounded extension a statute provides)",
            );
        }
        for f in opts.iter().filter(|f| f.key.text == "waive") {
            self.forbidden(f, "waiver");
        }
        for f in clauses.iter().filter(|f| f.key.text == "delegable") {
            self.forbidden(f, "delegation to a lower authority");
        }
    }

    fn forbidden(&mut self, f: &Field, what: &str) {
        self.report(
            Report::new(
                C::InvalidCapability,
                f.key.range,
                format!("a jurisdictional deadline admits no {what}"),
            )
            .note("a jurisdictional time limit admits no waiver, forfeiture or equitable exception (Bowles v. Russell, 551 U.S. 205 (2007))")
            .fix(f.range, "", Applicability::MaybeIncorrect),
        );
    }
}

impl Cx<'_, '_> {
    /// `grant g on [d, ...] effective [from, to) cite X { unlock, extend
    /// {...}, waive {...}, delegable {...} }`: each target is a deadline,
    /// the window is bounded (an emergency ends: L2007 for an open one),
    /// and the grant gives at least one capability (L2007).
    pub(super) fn grant(
        &mut self,
        item: &crate::db::parse::tree::Item,
        rule: Option<&crate::db::parse::tree::Item>,
        name: &crate::db::parse::tree::Name,
        clauses: &[Field],
        fields: &[Field],
    ) -> Option<crate::hir::Grant> {
        let meta = self.meta(name, None, item, rule);
        let on = clauses.iter().find(|f| f.key.text == "on");
        let targets: Vec<String> = Self::refs(on.as_ref())
            .into_iter()
            .filter_map(|r| {
                self.value_ref(r, "deadline", |k| {
                    matches!(k, crate::resolve::sig::ValueKind::Deadline)
                })
            })
            .collect();
        if meta.effective.to.is_none() {
            self.report(Report::new(
                C::InvalidField,
                name.range,
                format!(
                    "grant `{}` has no end: an emergency grant is in force for a bounded window, `effective [from, to)`",
                    name.text
                ),
            ));
        }
        let single: Vec<Field> = fields
            .iter()
            .filter(|f| !matches!(f.key.text.as_str(), "extend" | "delegable"))
            .cloned()
            .collect();
        let o = self.once(&single);
        let unlock = o.contains_key("unlock");
        let extend: Vec<_> = fields
            .iter()
            .filter(|f| f.key.text == "extend")
            .filter_map(|f| self.extension(f))
            .collect();
        let waive = o.get("waive").and_then(|f| self.waiver(f));
        let delegations: Vec<Delegation> = fields
            .iter()
            .filter(|f| f.key.text == "delegable")
            .flat_map(|f| self.delegation(f))
            .collect();
        let empty = !unlock && extend.is_empty() && waive.is_none() && delegations.is_empty();
        if empty {
            self.report(Report::new(
                C::InvalidField,
                name.range,
                format!(
                    "grant `{}` gives nothing: it states `unlock`, an `extend` or `waive` grant, or a `delegable` capability",
                    name.text
                ),
            ));
        }
        (!targets.is_empty() && !empty).then_some(crate::hir::Grant {
            meta,
            targets,
            unlock,
            extend,
            waive,
            delegations,
        })
    }
}
