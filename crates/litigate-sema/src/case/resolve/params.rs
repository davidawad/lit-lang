//! `analysis.params`, typed by the `Params` record (SPEC §8.3; design
//! review 2026-10-23 §2.1, §2.11): the engine's own parameters each have a
//! type, so a billing rate is money per hour, written as one
//! (`650.usd / 1.hours`), not a bare number. A parameter the engine does
//! not define (one the matter's own `metrics` read) is a `Number` or a
//! `Rate` of either unit.
use super::R;
use crate::db::parse::tree::{Expr, ExprKind, Name};
use crate::db::span::{Report, TextRange};
use crate::hir::{BinaryOp, Param, Rate, RateUnit, Rational};
use crate::types::rational as q;
use litigate_base::{Applicability, DiagnosticCode as C};

/// The type a parameter must have.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Expected {
    /// A dimensionless `Number`.
    Number,
    /// A `Rate<usd, per>`.
    Rate(RateUnit),
}

/// The engine's built-in parameters (litgraph `metrics::PARAMS`) and
/// their types.
const PARAMS: [(&str, Expected); 5] = [
    ("rate", Expected::Rate(RateUnit::Hour)),
    ("opp_rate", Expected::Rate(RateUnit::Hour)),
    ("carry_per_day", Expected::Rate(RateUnit::CalendarDay)),
    ("stakes", Expected::Number),
    ("loss_aversion", Expected::Number),
];

/// `Rate<cur, per>` as written in a type.
fn rate_type(currency: &str, per: RateUnit) -> String {
    format!("Rate<{currency}, {}>", per_type(per))
}

const fn per_type(per: RateUnit) -> &'static str {
    match per {
        RateUnit::Hour => "Hours",
        RateUnit::CalendarDay => "CalendarDays",
    }
}

/// The duration suffix one `per` is written with.
const fn per_suffix(per: RateUnit) -> &'static str {
    match per {
        RateUnit::Hour => "hours",
        RateUnit::CalendarDay => "days",
    }
}

impl R<'_, '_> {
    /// One `params` entry, typed by its name.
    pub(super) fn param(&mut self, name: &Name, value: &Expr) -> Option<Param> {
        let path = format!("params.{}", name.text);
        let expected = PARAMS
            .iter()
            .find(|(n, _)| *n == name.text)
            .map(|(_, e)| *e);
        if let ExprKind::Binary(BinaryOp::Div, money, per) = &value.kind {
            let rate = self.rate(&path, money, per)?;
            let wrong = match expected {
                Some(Expected::Number) => Some("Number".to_owned()),
                Some(Expected::Rate(unit)) if unit != rate.per => Some(rate_type("usd", unit)),
                _ => None,
            };
            if let Some(want) = wrong {
                self.invalid(
                    value.range,
                    format!(
                        "`{path}` is a `{want}`, found `{}`",
                        rate_type(&rate.currency, rate.per)
                    ),
                );
                return None;
            }
            return Some(Param::Rate(rate));
        }
        let Some(Expected::Rate(unit)) = expected else {
            return self.number(&path, value).map(Param::Number);
        };
        let want = rate_type("usd", unit);
        let mut report = Report::new(
            C::InvalidAnalysis,
            value.range,
            format!(
                "`{path}` is a `{want}`, written as money per {}: `N.usd / 1.{}`",
                per_suffix(unit).trim_end_matches('s'),
                per_suffix(unit)
            ),
        );
        if let ExprKind::Number(text) = &value.kind {
            let spelled = format!("{text}.usd");
            if super::money(&spelled).is_some() {
                report = report
                    .note("a bare number has no unit; the engine reads it as USD per unit of time")
                    .fix(
                        value.range,
                        format!("{spelled} / 1.{}", per_suffix(unit)),
                        Applicability::MachineApplicable,
                    );
            }
        }
        self.out.push(report);
        None
    }

    /// `money / duration` as an exact rate.
    fn rate(&mut self, path: &str, money: &Expr, per: &Expr) -> Option<Rate> {
        let ExprKind::Money(text) = &money.kind else {
            self.invalid(
                money.range,
                format!("`{path}`: a rate is a money literal over a period, `650.usd / 1.hours`"),
            );
            return None;
        };
        let Some(amount) = super::money(text) else {
            self.out.push(Report::new(
                C::ArithmeticError,
                money.range,
                format!("`{text}` is not a whole number of cents"),
            ));
            return None;
        };
        let ExprKind::Duration(text) = &per.kind else {
            return self.not_per(path, per.range);
        };
        let (count, suffix) = text.rsplit_once('.').unwrap_or_default();
        let unit = match suffix {
            "hours" => RateUnit::Hour,
            "days" => RateUnit::CalendarDay,
            _ => return self.not_per(path, per.range),
        };
        match q::parse_decimal(count).and_then(|n| q::div(Rational::integer(amount.minor_units), n))
        {
            Ok(minor_units) => Some(Rate {
                minor_units,
                currency: amount.currency,
                per: unit,
            }),
            Err(e) => {
                self.out.push(Report::new(
                    C::ArithmeticError,
                    per.range,
                    format!("`{path}`: {}", e.message()),
                ));
                None
            }
        }
    }

    fn not_per(&mut self, path: &str, at: TextRange) -> Option<Rate> {
        self.invalid(
            at,
            format!("`{path}`: a rate is per `N.hours` or per calendar `N.days`"),
        );
        None
    }
}
