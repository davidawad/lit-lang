//! Edges and links' edge fields (SPEC §5.1, §6 field types).
use super::cx::{first_expr, first_ref, flag, nested, Cx};
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
        let actor = first_ref(clauses.get("by"))
            .map(|r| self.actor(r))
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
            authority: self.cite_text(fields.get("authority")),
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
        let (callee, args) = self.call(
            x,
            C::TypeMismatch,
            "expected an event pattern such as `Served(complaint)`",
        )?;
        let ch = self.ck.event_call(callee, args, true, x.range);
        ch.ty.map(|_| ch.hir)
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
        if self_edge {
            self.report(Report::new(
                C::InvalidProbability,
                p.range,
                "a self edge must not carry `p`",
            ));
            return None;
        }
        if let ExprKind::Hole(reason) = &p.kind {
            self.ck.holes.push((p.range, reason.clone()));
            return Some(Authored::Hole(reason.clone()));
        }
        let num = self.ck.infer(p);
        match num.value {
            // The checker keeps every `Probability` value in [0, 1].
            Some(Value::Probability(v)) => Some(Authored::Known(v)),
            Some(Value::Number(v)) if q::is_probability(v) => Some(Authored::Known(v)),
            Some(Value::Number(v)) => {
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
            DeadlineWindow::Declared {
                deadline: self.value_ref(first_ref(within)?, "deadline", |k| {
                    matches!(k, ValueKind::Deadline)
                })?,
            }
        } else {
            let x = first_expr(f)?;
            // An inline deadline is an unsigned duration literal (SPEC
            // §3.1), so its amount is never negative.
            if let Some(Value::Days { amount, unit }) = self.ck.infer(x).value {
                DeadlineWindow::Inline {
                    length: amount.unsigned_abs(),
                    unit,
                }
            } else {
                self.report(Report::new(
                    C::UnitMismatch,
                    x.range,
                    "an edge `deadline` is a period such as `30.days`, `5.court_days` or `3.months`",
                ));
                return None;
            }
        };
        let opts = nested(f);
        let opts = self.once(opts);
        Some(EdgeDeadline {
            window,
            extendable: flag(opts.get("extendable")),
            extension_authority: Self::string(opts.get("extension_authority")),
            note: Self::string(opts.get("note")),
        })
    }

    fn duration(&mut self, f: &Field) -> Option<TriangularDuration> {
        let days = Type::Days(DayUnit::Calendar);
        // Estimates may use decimal calendar days (`13.5.days`, SPEC §6);
        // every other spelling goes through the integral-days checker.
        let get = |cx: &mut Self, x: &Expr| match &x.kind {
            ExprKind::Duration(text) if decimal_days(text).is_some() => {
                q::parse_decimal(decimal_days(text).unwrap_or_default())
                    .map_err(|_| {
                        cx.report(Report::new(
                            C::ArithmeticError,
                            x.range,
                            format!("`{text}` is not an exact day amount"),
                        ));
                    })
                    .ok()
            }
            ExprKind::Duration(text) if calendar_span(text).is_some() => {
                let (unit, numerator, denominator) =
                    calendar_span(text).unwrap_or((DayUnit::Calendar, 1, 1));
                let Some(Value::Days { amount, .. }) = cx.constant(x, &Type::Days(unit)) else {
                    return None;
                };
                q::make(i128::from(amount) * numerator, denominator)
                    .map_err(|_| {
                        cx.report(Report::new(
                            C::ArithmeticError,
                            x.range,
                            format!("`{text}` is out of range"),
                        ));
                    })
                    .ok()
            }
            _ => match cx.constant(x, &days) {
                Some(Value::Days { amount, .. }) => Some(Rational::integer(amount)),
                _ => None,
            },
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

/// The unit of a week, month or year estimate (`6.years`) and its mean
/// length in days as a fraction: an estimate has no trigger date to count
/// from, so a month is the mean Gregorian month (146097/4800 days) and a
/// year the mean Gregorian year (365.2425 days).
fn calendar_span(text: &str) -> Option<(DayUnit, i128, i128)> {
    match text.rsplit_once('.').map_or("", |(_, unit)| unit) {
        "weeks" => Some((DayUnit::Week, 7, 1)),
        "months" => Some((DayUnit::Month, 146_097, 4800)),
        "years" => Some((DayUnit::Year, 146_097, 400)),
        _ => None,
    }
}

/// The amount of a decimal calendar-day duration (`13.5` of `13.5.days`).
fn decimal_days(text: &str) -> Option<&str> {
    text.strip_suffix(".days")
        .filter(|amount| amount.contains('.'))
}
