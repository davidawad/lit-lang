//! Deadlines, counting rules, holiday calendars, and transitions lowered to
//! HIR (SPEC §5.3-§5.4). Structural checks only; overlap, coverage, and
//! counting semantics are verified downstream.
use super::cx::{first_expr, Cx};
use crate::db::parse::tree::{ArmItem, Expr, ExprKind, Field, HolidayItem, Item, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    Anchor, Calendar, Counting, DayUnit, Deadline, Direction, Holiday, HolidayDate, Interval,
    Observance, RollOn, ShortPeriod, Transition, TransitionArm, Type, Value,
};
use crate::resolve::sig::ValueKind;
use crate::resolve::suggest;
use crate::types::date;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

impl Cx<'_, '_> {
    fn bad_counting(&mut self, name: &Name, expected: &[&str]) {
        let mut report = Report::new(
            C::InvalidCounting,
            name.range,
            format!("unknown counting keyword `{}`", name.text),
        )
        .note(format!("expected one of {}", expected.join(", ")));
        if let Some(best) = suggest::nearest(&name.text, expected.iter().copied()) {
            report = report.fix(name.range, best, Applicability::MaybeIncorrect);
        }
        self.report(report);
    }

    fn keyword<T: Copy>(&mut self, e: Option<&Expr>, table: &[(&str, T)]) -> Option<T> {
        let ExprKind::Ref(r) = &e?.kind else {
            return None;
        };
        let hit = table
            .iter()
            .find(|(k, _)| r.is_simple() && *k == r.head.text)
            .map(|(_, v)| *v);
        if hit.is_none() {
            self.bad_counting(&r.head, &table.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        }
        hit
    }

    fn days(&mut self, e: &Expr) -> Option<(u64, DayUnit)> {
        match self.ck.infer(e).value {
            Some(Value::Days { amount, unit }) => match u64::try_from(amount) {
                Ok(n) => Some((n, unit)),
                Err(_) => {
                    self.report(Report::new(
                        C::ArithmeticError,
                        e.range,
                        "a period cannot be negative",
                    ));
                    None
                }
            },
            Some(Value::Hours(_)) => {
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
        allow_hours: impl Fn(&str) -> bool,
    ) -> Option<Deadline> {
        let meta = self.meta(name, None, item, rule);
        let counting = self.value_ref(counted_by, "counting rule", |k| {
            matches!(k, ValueKind::Counting)
        });
        let period = match self.ck.infer(amount).value {
            Some(Value::Days { amount: n, unit }) => u64::try_from(n).ok().map(|n| (n, unit)),
            Some(Value::Hours(_)) => {
                let allowed = counting.as_deref().is_some_and(&allow_hours);
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
        let trigger = self.pattern(trigger);
        let o = self.once(opts);
        let (amount, unit) = period?;
        Some(Deadline {
            meta,
            amount,
            unit,
            direction,
            trigger: trigger?,
            counting: counting?,
            extendable: o
                .get("extendable")
                .and_then(|f| first_expr(f))
                .and_then(|x| match x.kind {
                    ExprKind::Bool(b) => Some(b),
                    _ => None,
                }),
            extension_authority: Self::string(o.get("extension_authority")),
            note: Self::string(o.get("note")),
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
                let e = Expr {
                    kind: ExprKind::Ref(r.clone()),
                    range: r.range,
                };
                self.keyword(
                    Some(&e),
                    &[
                        ("weekend", RollOn::Weekend),
                        ("legal_holiday", RollOn::LegalHoliday),
                        ("inaccessible", RollOn::Inaccessible),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let mut service_extensions = BTreeMap::new();
        if let Some(ExprKind::Record(methods)) = f
            .get("service_extension")
            .and_then(|x| first_expr(x))
            .map(|e| &e.kind)
        {
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
        let calendar = match f
            .get("holidays")
            .and_then(|x| first_expr(x))
            .map(|e| &e.kind)
        {
            Some(ExprKind::Ref(r)) => {
                self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar))
            }
            _ => {
                self.report(Report::new(
                    C::InvalidCounting,
                    name.range,
                    format!("counting rule `{}` needs `holidays <calendar>`", name.text),
                ));
                None
            }
        };
        let anchor = match f.get("anchor") {
            Some(a) => self.keyword(
                first_expr(a),
                &[
                    ("trigger_day", Anchor::TriggerDay),
                    ("next_business_day", Anchor::NextBusinessDay),
                ],
            ),
            None => Some(Anchor::TriggerDay),
        };
        let short_period = match f
            .get("short_period")
            .and_then(|x| first_expr(x))
            .map(|e| (&e.kind, e.range))
        {
            Some((ExprKind::Record(parts), range)) => {
                let below = parts
                    .iter()
                    .find(|(k, _)| k.text == "below")
                    .and_then(|(_, e)| self.days(e));
                let count = parts
                    .iter()
                    .find(|(k, _)| k.text == "count")
                    .and_then(|(_, e)| {
                        self.keyword(
                            Some(e),
                            &[
                                ("calendar_days", DayUnit::Calendar),
                                ("court_days", DayUnit::Court),
                                ("business_days", DayUnit::Business),
                            ],
                        )
                    });
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
        };
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
        })
    }

    fn small(&mut self, e: &Expr, lo: i64, hi: i64, what: &str) -> Option<i64> {
        match self.ck.check(e, &Type::Number).value {
            Some(Value::Number(v)) if v.denominator == 1 && (lo..=hi).contains(&v.numerator) => {
                Some(v.numerator)
            }
            _ => {
                self.report(Report::new(
                    C::InvalidCounting,
                    e.range,
                    format!("{what} must be an integer from {lo} to {hi}"),
                ));
                None
            }
        }
    }

    fn holiday_date(&mut self, rule: &Expr) -> Option<HolidayDate> {
        let ExprKind::Call(callee, args) = &rule.kind else {
            self.report(Report::new(
                C::InvalidCounting,
                rule.range,
                "expected `fixed`, `nth`, `last`, `every`, or `once`",
            ));
            return None;
        };
        let weekday = |cx: &mut Self, e: &Expr| -> Option<String> {
            match &e.kind {
                ExprKind::Ref(r) if r.is_simple() && WEEKDAYS.contains(&r.head.text.as_str()) => {
                    Some(r.head.text.clone())
                }
                ExprKind::Ref(r) => {
                    cx.bad_counting(&r.head, &WEEKDAYS);
                    None
                }
                _ => None,
            }
        };
        let shape: &[&str] = match callee.head.text.as_str() {
            "fixed" => &["month", "day"],
            "nth" => &["n", "weekday", "month"],
            "last" => &["weekday", "month"],
            "every" => &["years", "from_year", "month", "day"],
            "once" => &["date"],
            _ => {
                self.bad_counting(&callee.head, &["fixed", "nth", "last", "every", "once"]);
                return None;
            }
        };
        if args.len() != shape.len() {
            self.report(Report::new(
                C::InvalidCounting,
                rule.range,
                format!("`{}` takes ({})", callee.head.text, shape.join(", ")),
            ));
            return None;
        }
        let to8 = |v: i64| u8::try_from(v).ok();
        let date = match callee.head.text.as_str() {
            "fixed" => {
                let (m, d) = (
                    self.small(&args[0], 1, 12, "month")?,
                    self.small(&args[1], 1, 31, "day")?,
                );
                if i64::from(date::days_in_month(2000, u32::try_from(m).ok()?)) < d {
                    self.report(Report::new(
                        C::InvalidCounting,
                        rule.range,
                        format!("month {m} has no day {d}"),
                    ));
                    return None;
                }
                HolidayDate::Fixed {
                    month: to8(m)?,
                    day: to8(d)?,
                }
            }
            "nth" => HolidayDate::Nth {
                n: to8(self.small(&args[0], 1, 4, "n")?)?,
                weekday: weekday(self, &args[1])?,
                month: to8(self.small(&args[2], 1, 12, "month")?)?,
            },
            "last" => HolidayDate::Last {
                weekday: weekday(self, &args[0])?,
                month: to8(self.small(&args[1], 1, 12, "month")?)?,
            },
            "every" => HolidayDate::Every {
                years: u16::try_from(self.small(&args[0], 1, 100, "years")?).ok()?,
                from_year: u16::try_from(self.small(&args[1], 1, 9999, "from_year")?).ok()?,
                month: to8(self.small(&args[2], 1, 12, "month")?)?,
                day: to8(self.small(&args[3], 1, 31, "day")?)?,
            },
            _ => match &args[0].kind {
                ExprKind::Date(d) => HolidayDate::Once { date: d.clone() },
                _ => {
                    self.report(Report::new(
                        C::InvalidCounting,
                        args[0].range,
                        "`once` takes a date",
                    ));
                    return None;
                }
            },
        };
        Some(date)
    }

    /// `calendar c { id: rule effective [a, b) observed kind }`.
    pub fn calendar(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        holidays: &[HolidayItem],
    ) -> Option<Calendar> {
        let meta = self.meta(name, None, item, rule);
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        let mut ok = true;
        for h in holidays {
            if !seen.insert(h.name.text.clone()) {
                self.report(Report::new(
                    C::DuplicateName,
                    h.name.range,
                    format!("calendar `{}` already defines `{}`", name.text, h.name.text),
                ));
            }
            let date = self.holiday_date(&h.rule);
            let observed = h.observed.as_ref().and_then(|o| {
                let e = Expr {
                    kind: ExprKind::Ref(RefPath {
                        head: o.clone(),
                        instance: None,
                        rest: vec![],
                        version: None,
                        range: o.range,
                    }),
                    range: o.range,
                };
                self.keyword(
                    Some(&e),
                    &[
                        ("nearest_weekday", Observance::NearestWeekday),
                        ("next_day", Observance::NextDay),
                    ],
                )
            });
            ok &= date.is_some() && (h.observed.is_none() || observed.is_some());
            let (from, to) = &h.effective;
            if let Some(date) = date {
                out.push(Holiday {
                    id: h.name.text.clone(),
                    date,
                    in_force: Interval {
                        from: from.text.clone(),
                        to: to.as_ref().map(|t| t.text.clone()),
                        known_at: meta.effective.known_at.clone(),
                    },
                    observed,
                });
            }
        }
        ok.then_some(Calendar {
            meta,
            holidays: out,
        })
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
