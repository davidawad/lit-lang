//! Holiday calendars (SPEC §5.3): date rules, weekdays, observance.
use super::cx::Cx;
use crate::db::parse::tree::{Expr, ExprKind, HolidayItem, Item, Name, RefPath};
use crate::db::span::{Report, TextRange};
use crate::hir::{Calendar, Holiday, HolidayDate, Interval, Observance, Type, Value};
use crate::types::date;
use litigate_base::DiagnosticCode as C;

const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

const RULES: [&str; 5] = ["fixed", "nth", "last", "every", "once"];

impl Cx<'_, '_> {
    /// An integer constant in `lo..=hi`, as `T` (which holds that range).
    fn small<T: TryFrom<i64>>(&mut self, e: &Expr, lo: i64, hi: i64, what: &str) -> Option<T> {
        let n = match self.ck.check(e, &Type::Number).value {
            Some(Value::Number(v)) if v.denominator == 1 && (lo..=hi).contains(&v.numerator) => {
                T::try_from(v.numerator).ok()
            }
            _ => None,
        };
        if n.is_none() {
            self.report(Report::new(
                C::InvalidCounting,
                e.range,
                format!("{what} must be an integer from {lo} to {hi}"),
            ));
        }
        n
    }

    fn weekday(&mut self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Ref(r) if r.is_simple() && WEEKDAYS.contains(&r.head.text.as_str()) => {
                Some(r.head.text.clone())
            }
            ExprKind::Ref(r) => {
                self.bad_counting(r, &WEEKDAYS);
                None
            }
            _ => {
                self.report(Report::new(
                    C::InvalidCounting,
                    e.range,
                    "expected a weekday such as `monday`",
                ));
                None
            }
        }
    }

    /// The date rule `callee(args)` of a holiday spanning `range`.
    fn holiday_date(
        &mut self,
        callee: &RefPath,
        args: &[Expr],
        range: TextRange,
    ) -> Option<HolidayDate> {
        let shape: &[&str] = match callee.head.text.as_str() {
            "fixed" => &["month", "day"],
            "nth" => &["n", "weekday", "month"],
            "last" => &["weekday", "month"],
            "every" => &["years", "from_year", "month", "day"],
            "once" => &["date"],
            _ => {
                self.bad_counting(callee, &RULES);
                return None;
            }
        };
        if args.len() != shape.len() {
            self.report(Report::new(
                C::InvalidCounting,
                range,
                format!("`{}` takes ({})", callee.head.text, shape.join(", ")),
            ));
            return None;
        }
        let date = match callee.head.text.as_str() {
            "fixed" => {
                let month: u8 = self.small(&args[0], 1, 12, "month")?;
                let day: u8 = self.small(&args[1], 1, 31, "day")?;
                if date::days_in_month(2000, u32::from(month)) < u32::from(day) {
                    self.report(Report::new(
                        C::InvalidCounting,
                        range,
                        format!("month {month} has no day {day}"),
                    ));
                    return None;
                }
                HolidayDate::Fixed { month, day }
            }
            "nth" => HolidayDate::Nth {
                n: self.small(&args[0], 1, 4, "n")?,
                weekday: self.weekday(&args[1])?,
                month: self.small(&args[2], 1, 12, "month")?,
            },
            "last" => HolidayDate::Last {
                weekday: self.weekday(&args[0])?,
                month: self.small(&args[1], 1, 12, "month")?,
            },
            "every" => HolidayDate::Every {
                years: self.small(&args[0], 1, 100, "years")?,
                from_year: self.small(&args[1], 1, 9999, "from_year")?,
                month: self.small(&args[2], 1, 12, "month")?,
                day: self.small(&args[3], 1, 31, "day")?,
            },
            _ => {
                if let ExprKind::Date(d) = &args[0].kind {
                    HolidayDate::Once { date: d.clone() }
                } else {
                    self.report(Report::new(
                        C::InvalidCounting,
                        args[0].range,
                        "`once` takes a date",
                    ));
                    return None;
                }
            }
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
            let date = self
                .call(
                    &h.rule,
                    C::InvalidCounting,
                    "expected `fixed`, `nth`, `last`, `every`, or `once`",
                )
                .and_then(|(callee, args)| self.holiday_date(callee, args, h.rule.range));
            let observed = h.observed.as_ref().and_then(|o| {
                let r = RefPath {
                    head: o.clone(),
                    instance: None,
                    rest: vec![],
                    version: None,
                    range: o.range,
                };
                self.keyword(
                    &r,
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
}
