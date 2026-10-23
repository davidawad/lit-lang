//! Deadlines, counting rules, holiday calendars, and transitions lowered to
//! HIR (SPEC §5.3-§5.4). Structural checks only; overlap, coverage, and
//! counting semantics are verified downstream.
use super::cx::{first_expr, first_ref, flag, Cx};
use crate::db::parse::tree::{ArmItem, Expr, ExprKind, Field, Item, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    Anchor, Counting, DayUnit, Deadline, Designated, Direction, EndOfMonth, RollOn, ShortPeriod,
    Transition, TransitionArm, Type, Value,
};
use crate::resolve::sig::{TypeSig, ValueKind};
use crate::resolve::suggest;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

impl Cx<'_, '_> {
    /// Reports `r` as none of the `expected` keywords.
    pub(super) fn bad_counting(&mut self, r: &RefPath, expected: &[&str]) {
        let shown = r.display();
        let mut report = Report::new(
            C::InvalidCounting,
            r.range,
            format!("unknown counting keyword `{shown}`"),
        )
        .note(format!("expected one of {}", expected.join(", ")));
        if let Some(best) = suggest::nearest(&shown, expected.iter().copied()) {
            report = report.fix(r.range, best, Applicability::MaybeIncorrect);
        }
        self.report(report);
    }

    pub(super) fn keyword<T: Copy>(&mut self, r: &RefPath, table: &[(&str, T)]) -> Option<T> {
        let hit = table
            .iter()
            .find(|(k, _)| r.is_simple() && *k == r.head.text)
            .map(|(_, v)| *v);
        if hit.is_none() {
            self.bad_counting(r, &table.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        }
        hit
    }

    pub(super) fn days(&mut self, e: &Expr) -> Option<(u64, DayUnit)> {
        match self.ck.infer(e).value {
            // Day amounts here are unsigned duration literals (SPEC §3.1),
            // so the conversion cannot fail.
            Some(Value::Days {
                amount,
                unit: unit @ (DayUnit::Calendar | DayUnit::Court | DayUnit::Business),
            }) => u64::try_from(amount).ok().map(|n| (n, unit)),
            Some(Value::Hours(_) | Value::Days { .. }) => {
                self.report(Report::new(
                    C::InvalidCounting,
                    e.range,
                    "this period must be counted in days",
                ));
                None
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    e.range,
                    "expected a day count such as `3.days`",
                ));
                None
            }
        }
    }

    /// `deadline d: N.unit from|after|before T counted_by c`.
    #[allow(clippy::too_many_arguments)]
    pub fn deadline(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amount: &Expr,
        direction: &Name,
        trigger: &Expr,
        counted_by: &RefPath,
        opts: &[Field],
        local_counting: impl Fn(&str, &str) -> Option<bool>,
    ) -> Option<Deadline> {
        let meta = self.meta(name, None, item, rule);
        let counting = self.value_ref(counted_by, "counting rule", |k| {
            matches!(k, ValueKind::Counting)
        });
        let period = match self.ck.infer(amount).value {
            Some(Value::Days { amount: n, unit }) => {
                let counted = counting
                    .as_deref()
                    .map(|c| local_counting(c, "end_of_month"));
                if unit.is_monthly() && counted == Some(Some(false)) {
                    self.report(Report::new(
                        C::InvalidCounting,
                        amount.range,
                        "a month or year deadline needs a counting rule with `end_of_month`",
                    ));
                }
                u64::try_from(n).ok().map(|n| (n, unit))
            }
            Some(Value::Hours(_)) => {
                let allowed = counting
                    .as_deref()
                    .is_some_and(|c| local_counting(c, "allow_hours") == Some(true));
                let (code, message) = if allowed {
                    (
                        C::LoweringUnsupported,
                        "hour deadlines have no HIR/litgraph representation in language version 1",
                    )
                } else {
                    (
                        C::InvalidCounting,
                        "an hours deadline needs a counting rule with `allow_hours`",
                    )
                };
                self.report(Report::new(code, amount.range, message));
                None
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    amount.range,
                    "a deadline period is a duration such as `21.days`",
                ));
                None
            }
        };
        let direction = match direction.text.as_str() {
            "before" => Direction::Backward,
            _ => Direction::Forward,
        };
        let o = self.once_except(opts, "extend");
        let designated = first_ref(o.get("designated"))
            .and_then(|r| self.designated(name, trigger, r, (direction, period.map(|p| p.1))));
        let trigger = self.pattern(trigger);
        let obligation = self.obligation(name, opts, &o, rule);
        let (amount, unit) = period?;
        Some(Deadline {
            meta,
            amount,
            unit,
            direction,
            trigger: trigger?,
            counting: counting?,
            extendable: flag(o.get("extendable")),
            extension_authority: Self::string(o.get("extension_authority")),
            note: Self::string(o.get("note")),
            bearer: obligation.bearer,
            policy: obligation.policy,
            linear: obligation.linear,
            extend: obligation.extend,
            waive: obligation.waive,
            designated,
        })
    }

    /// `designated p` of deadline `name`: `p` is a `Date` parameter of the
    /// event `trigger` matches, and the deadline counts forward in calendar
    /// days (its least time, SPEC §5.3).
    fn designated(
        &mut self,
        name: &Name,
        trigger: &Expr,
        r: &RefPath,
        (direction, unit): (Direction, Option<DayUnit>),
    ) -> Option<Designated> {
        if unit.is_some_and(|u| u != DayUnit::Calendar) {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "the least time a designated due date of `{}` allows is counted in calendar days",
                    name.text
                ),
            ));
            return None;
        }
        if direction == Direction::Backward {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "deadline `{}` counts backward: only a forward period's due date is designated",
                    name.text
                ),
            ));
            return None;
        }
        let ExprKind::Call(callee, _) = &trigger.kind else {
            return None;
        };
        // The trigger pattern itself reports a callee that is no event.
        let (event, found) = self.ck.scope.user_type(&callee.head, &[]).ok()?;
        let TypeSig::Event(params) = found.item else {
            return None;
        };
        let Some(param) = params
            .iter()
            .position(|(p, _)| r.is_simple() && *p == r.head.text)
        else {
            self.report(Report::new(
                C::UnresolvedName,
                r.range,
                format!(
                    "the trigger `{event}` of deadline `{}` has no parameter `{}`",
                    name.text,
                    r.display()
                ),
            ));
            return None;
        };
        if params[param].1 != Some(Type::Date) {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!(
                    "parameter `{}` of `{event}` designates a due date, so it must have type `Date`",
                    params[param].0
                ),
            ));
            return None;
        }
        Some(Designated {
            param,
            name: params[param].0.clone(),
        })
    }

    /// `counting c { ... }`.
    pub fn counting(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        items: &[Field],
    ) -> Option<Counting> {
        let meta = self.meta(name, None, item, rule);
        let f = self.once(items);
        let roll_on = Self::refs(f.get("roll_forward_on"))
            .into_iter()
            .filter_map(|r| {
                self.keyword(
                    r,
                    &[
                        ("weekend", RollOn::Weekend),
                        ("legal_holiday", RollOn::LegalHoliday),
                        ("inaccessible", RollOn::Inaccessible),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let service_extensions = self.service_extensions(f.get("service_extension"));
        let calendar = if let Some(r) = first_ref(f.get("holidays")) {
            self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar))
        } else {
            self.report(Report::new(
                C::InvalidCounting,
                name.range,
                format!("counting rule `{}` needs `holidays <calendar>`", name.text),
            ));
            None
        };
        let anchor = if f.contains_key("anchor") {
            first_ref(f.get("anchor")).and_then(|r| {
                self.keyword(
                    r,
                    &[
                        ("trigger_day", Anchor::TriggerDay),
                        ("next_business_day", Anchor::NextBusinessDay),
                    ],
                )
            })
        } else {
            Some(Anchor::TriggerDay)
        };
        let short_period = self.short_period(f.get("short_period"));
        let end_of_month = first_ref(f.get("end_of_month")).and_then(|r| {
            self.keyword(
                r,
                &[("clamp", EndOfMonth::Clamp), ("roll", EndOfMonth::Roll)],
            )
        });
        Some(Counting {
            meta,
            exclude_trigger_day: f.contains_key("exclude_trigger_day"),
            include_last_day: f.contains_key("include_last_day"),
            roll_on,
            service_extensions,
            calendar: calendar?,
            anchor: anchor?,
            short_period,
            hours: f.contains_key("allow_hours"),
            end_of_month,
        })
    }

    fn service_extensions(&mut self, f: Option<&&Field>) -> BTreeMap<String, (u64, DayUnit)> {
        let mut service_extensions = BTreeMap::new();
        if let Some(ExprKind::Record(methods)) = f.and_then(|x| first_expr(x)).map(|e| &e.kind) {
            for (method, e) in methods {
                if let Some(d) = self.days(e) {
                    if service_extensions.insert(method.text.clone(), d).is_some() {
                        self.report(Report::new(
                            C::InvalidCounting,
                            method.range,
                            format!("service method `{}` is listed twice", method.text),
                        ));
                    }
                }
            }
        }
        service_extensions
    }

    fn short_period(&mut self, f: Option<&&Field>) -> Option<ShortPeriod> {
        match f.and_then(|x| first_expr(x)).map(|e| (&e.kind, e.range)) {
            Some((ExprKind::Record(parts), range)) => {
                let below = parts
                    .iter()
                    .find(|(k, _)| k.text == "below")
                    .and_then(|(_, e)| self.days(e));
                let count = match parts.iter().find(|(k, _)| k.text == "count") {
                    Some((
                        _,
                        Expr {
                            kind: ExprKind::Ref(r),
                            ..
                        },
                    )) => self.keyword(
                        r,
                        &[
                            ("calendar_days", DayUnit::Calendar),
                            ("court_days", DayUnit::Court),
                            ("business_days", DayUnit::Business),
                        ],
                    ),
                    _ => None,
                };
                if parts.len() != 2 || below.is_none() || count.is_none() {
                    self.report(Report::new(
                        C::InvalidCounting,
                        range,
                        "`short_period` is `{ below: N.days, count: <unit> }`",
                    ));
                }
                below
                    .zip(count)
                    .map(|((threshold, _), unit)| ShortPeriod { threshold, unit })
            }
            _ => None,
        }
    }

    /// `transition t amends d@D { unless C -> v, otherwise -> w }`.
    pub fn transition(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amends: &RefPath,
        arms: &[ArmItem],
    ) -> Option<Transition> {
        let meta = self.meta(name, None, item, rule);
        let versioned = |cx: &mut Self, r: &RefPath| {
            if r.version.is_none() {
                cx.report(Report::new(
                    C::InvalidTransition,
                    r.range,
                    format!(
                        "`{}` must name a version (`{}@YYYY-MM-DD`)",
                        r.display(),
                        r.display()
                    ),
                ));
                return None;
            }
            cx.value_ref(r, "declaration", |_| true)
        };
        let amendment = versioned(self, amends);
        let mut out = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            let last = i + 1 == arms.len();
            if arm.unless.is_none() != last {
                let message = if last {
                    "the last arm must be `otherwise -> <version>`"
                } else {
                    "`otherwise` must be the last arm"
                };
                self.report(Report::new(C::InvalidTransition, arm.range, message));
            }
            let unless = arm
                .unless
                .as_ref()
                .map(|c| self.ck.check(c, &Type::Boolean).hir);
            if let Some(selects) = versioned(self, &arm.target) {
                out.push(TransitionArm { unless, selects });
            }
        }
        if arms.is_empty() {
            self.report(Report::new(
                C::InvalidTransition,
                name.range,
                "a transition needs at least an `otherwise` arm",
            ));
        }
        Some(Transition {
            meta,
            amendment: amendment?,
            arms: out,
        })
    }
}
