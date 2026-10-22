//! Edges and links' edge fields (SPEC §5.1, §6 field types).
use super::cx::{first_expr, nested, Cx};
use crate::db::parse::tree::{EdgeItem, Expr, ExprKind, Field};
use crate::db::span::Report;
use crate::hir::{
    Authored, DayUnit, DeadlineWindow, Edge, EdgeDeadline, Rational, TriangularDuration, Type,
    Value,
};
use crate::resolve::sig::ValueKind;
use crate::types::rational as q;
use litigate_base::{Applicability, DiagnosticCode as C};

impl Cx<'_, '_> {
    /// Builds one edge from node `from` (or a link when `from` is qualified).
    pub fn edge(
        &mut self,
        e: &EdgeItem,
        from: &str,
        node_actor: Option<&str>,
        chance: bool,
    ) -> Edge {
        let clauses = self.once(&e.clauses);
        let fields = self.once(&e.fields);
        let to = self
            .value_ref(&e.to, "node", |k| matches!(k, ValueKind::Node { .. }))
            .unwrap_or_else(|| e.to.display());
        let actor = clauses
            .get("by")
            .and_then(|f| first_expr(f))
            .and_then(|x| match &x.kind {
                ExprKind::Ref(r) => Some(r),
                _ => None,
            })
            .map(|r| self.actor(r));
        let actor = actor
            .or_else(|| node_actor.map(str::to_owned))
            .unwrap_or_else(|| "either".to_owned());
        let probability = self.probability(
            e,
            clauses.get("p").and_then(|f| first_expr(f)),
            to == from,
            chance,
        );
        if let Some(f) = fields.get("replaces").filter(|_| e.from.is_none()) {
            self.not_allowed(f, "outside a links package");
        }
        if let (Some(_), Some(f)) = (fields.get("within"), fields.get("deadline")) {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "an edge has either `within` or `deadline`, not both",
            ));
        }
        Edge {
            id: e.name.as_ref().map(|n| n.text.clone()),
            to,
            label: e.label.text.clone(),
            actor,
            authority: Self::string(fields.get("authority")),
            probability,
            hours: fields
                .get("effort")
                .and_then(|f| first_expr(f))
                .and_then(|x| match self.constant(x, &Type::Hours) {
                    Some(Value::Hours(h)) => Some(h),
                    _ => None,
                }),
            cost: fields
                .get("cost")
                .and_then(|f| first_expr(f))
                .and_then(|x| self.money(x)),
            event: fields
                .get("event")
                .and_then(|f| first_expr(f))
                .and_then(|x| self.pattern(x)),
            deadline: self.edge_deadline(fields.get("within"), fields.get("deadline")),
            duration: fields.get("duration").and_then(|f| self.duration(f)),
            valence: self.valence(fields.get("valence")),
            court_listener_url: Self::string(fields.get("court_listener_url")),
            action_id: Self::string(fields.get("action_id")),
            note: Self::string(fields.get("note")),
            tags: self.idents(fields.get("tags")),
            attrs: self.attrs(fields.get("attrs")),
            sets: self.idents(fields.get("sets")),
            clears: self.idents(fields.get("clears")),
            requires: self.idents(fields.get("requires")),
            forbids: self.idents(fields.get("forbids")),
            span: self.ck.span(e.range),
        }
    }

    /// An event pattern (trailing arguments may be omitted).
    pub fn pattern(&mut self, x: &Expr) -> Option<crate::hir::Expr> {
        if let ExprKind::Call(callee, args) = &x.kind {
            let ch = self.ck.event_call(callee, args, true, x.range);
            ch.ty.map(|_| ch.hir)
        } else {
            self.report(Report::new(
                C::TypeMismatch,
                x.range,
                "expected an event pattern such as `Served(complaint)`",
            ));
            None
        }
    }

    fn probability(
        &mut self,
        e: &EdgeItem,
        p: Option<&Expr>,
        self_edge: bool,
        chance: bool,
    ) -> Option<Authored<Rational>> {
        let Some(p) = p else {
            if chance && !self_edge {
                self.report(
                    Report::new(
                        C::InvalidProbability,
                        e.name_range.cover(e.label.range),
                        "every edge of a chance node needs `p`",
                    )
                    .fix(
                        e.to.range.at_end(),
                        " p ?",
                        Applicability::HasPlaceholders,
                    ),
                );
            }
            return None;
        };
        if self_edge && chance {
            self.report(Report::new(
                C::InvalidProbability,
                p.range,
                "a self edge of a `chance` node must not carry `p`",
            ));
            return None;
        }
        if let ExprKind::Hole(reason) = &p.kind {
            self.ck.holes.push((p.range, reason.clone()));
            return Some(Authored::Hole(reason.clone()));
        }
        let num = self.ck.infer(p);
        match (&p.kind, num.value) {
            (_, Some(Value::Number(v) | Value::Probability(v))) if q::is_probability(v) => {
                Some(Authored::Known(v))
            }
            (_, Some(Value::Number(v) | Value::Probability(v))) => {
                self.report(Report::new(
                    C::InvalidProbability,
                    p.range,
                    format!("probability {} is outside [0, 1]", q::display(v)),
                ));
                None
            }
            _ => {
                if num.ty.is_some() {
                    self.report(Report::new(
                        C::InvalidProbability,
                        p.range,
                        "`p` must be a constant probability",
                    ));
                }
                None
            }
        }
    }

    fn edge_deadline(
        &mut self,
        within: Option<&&Field>,
        inline: Option<&&Field>,
    ) -> Option<EdgeDeadline> {
        let f = within.or(inline)?;
        let window = if within.is_some() {
            let ExprKind::Ref(r) = &first_expr(f)?.kind else {
                return None;
            };
            DeadlineWindow::Declared {
                deadline: self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))?,
            }
        } else {
            let x = first_expr(f)?;
            if let Some(Value::Days {
                amount,
                unit: unit @ (DayUnit::Calendar | DayUnit::Court | DayUnit::Business),
            }) = self.ck.infer(x).value
            {
                if let Ok(length) = u64::try_from(amount) {
                    DeadlineWindow::Inline { length, unit }
                } else {
                    self.report(Report::new(
                        C::ArithmeticError,
                        x.range,
                        "a deadline cannot be negative",
                    ));
                    return None;
                }
            } else {
                self.report(Report::new(
                    C::UnitMismatch,
                    x.range,
                    "an edge `deadline` is a day count such as `30.days` or `5.court_days`",
                ));
                return None;
            }
        };
        let opts = nested(f);
        let opts = self.once(opts);
        Some(EdgeDeadline {
            window,
            extendable: opts
                .get("extendable")
                .and_then(|o| first_expr(o))
                .and_then(|x| match x.kind {
                    ExprKind::Bool(b) => Some(b),
                    _ => None,
                }),
            extension_authority: Self::string(opts.get("extension_authority")),
            note: Self::string(opts.get("note")),
        })
    }

    fn duration(&mut self, f: &Field) -> Option<TriangularDuration> {
        let days = Type::Days(DayUnit::Calendar);
        // Estimates may use decimal calendar days (`13.5.days`, SPEC §6);
        // every other spelling goes through the integral-days checker.
        let get = |cx: &mut Self, x: &Expr| {
            if let ExprKind::Duration(text) = &x.kind {
                let decimal = text
                    .strip_suffix(".days")
                    .filter(|amount| amount.contains('.'));
                if let Some(amount) = decimal {
                    return q::parse_decimal(amount)
                        .map_err(|_| {
                            cx.report(Report::new(
                                C::ArithmeticError,
                                x.range,
                                format!("`{text}` is not an exact day amount"),
                            ));
                        })
                        .ok();
                }
            }
            match cx.constant(x, &days) {
                Some(Value::Days { amount, .. }) => Some(Rational::integer(amount)),
                _ => None,
            }
        };
        if let Some(x) = first_expr(f) {
            return get(self, x).map(|mode| TriangularDuration {
                min: None,
                mode,
                max: None,
            });
        }
        // `min` and `max` are optional (SPEC §3.1 `duration`); `mode` is not.
        let parts = self.once(nested(f));
        let part = |cx: &mut Self, key: &str| match parts.get(key).and_then(|p| first_expr(p)) {
            Some(x) => get(cx, x).map(Some).ok_or(()),
            None => Ok(None),
        };
        let (Ok(min), Ok(mode), Ok(max)) =
            (part(self, "min"), part(self, "mode"), part(self, "max"))
        else {
            return None;
        };
        let Some(mode) = mode else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "triangular `duration` needs `mode`",
            ));
            return None;
        };
        let below = |a: Option<Rational>, b: Option<Rational>| match (a, b) {
            (Some(a), Some(b)) => q::cmp(a, b).is_gt(),
            _ => false,
        };
        if below(min, Some(mode)) || below(Some(mode), max) {
            self.report(Report::new(
                C::InvalidField,
                f.range,
                "`duration` needs min <= mode <= max",
            ));
        }
        Some(TriangularDuration { min, mode, max })
    }
}
