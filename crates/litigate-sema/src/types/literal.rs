//! Typed literals: money in integral minor units and day/hour durations
//! (SPEC §2 typed literals, §6).
use super::expr::{Checked, Checker};
use super::rational::{self, ArithError};
use crate::db::span::{Report, TextRange};
use crate::hir::{DayUnit, Money, Rational, Type, Value};
use litigate_base::DiagnosticCode as C;

impl Checker<'_, '_> {
    pub(crate) fn money(&mut self, text: &str, r: TextRange) -> Checked {
        let (amount, currency) = text.rsplit_once('.').unwrap_or((text, "usd"));
        let decimals = amount.split_once('.').map_or(0, |(_, f)| f.len());
        if decimals > 2 {
            self.report(Report::new(
                C::ArithmeticError,
                r,
                format!("`{currency}` has 2 minor-unit digits; `{amount}` has {decimals}"),
            ));
            return self.error(r);
        }
        let minor =
            rational::parse_decimal(amount).and_then(|v| rational::mul(v, Rational::integer(100)));
        match minor {
            Ok(v) if v.denominator == 1 => {
                let money = Money {
                    minor_units: v.numerator,
                    currency: currency.to_owned(),
                };
                self.literal(Value::Money(money), Type::Money(currency.to_owned()), r)
            }
            Ok(_) | Err(_) => self.arith(r, ArithError::Overflow),
        }
    }

    pub(crate) fn duration(&mut self, text: &str, r: TextRange) -> Checked {
        let (amount, unit) = text.rsplit_once('.').unwrap_or((text, "days"));
        let value = match rational::parse_decimal(amount) {
            Ok(v) => v,
            Err(err) => return self.arith(r, err),
        };
        let unit = match unit {
            "hours" => return self.literal(Value::Hours(value), Type::Hours, r),
            "court_days" => DayUnit::Court,
            "business_days" => DayUnit::Business,
            _ => DayUnit::Calendar,
        };
        if value.denominator != 1 {
            self.report(Report::new(
                C::InvalidNumber,
                r,
                "day amounts must be integers",
            ));
            return self.error(r);
        }
        self.literal(
            Value::Days {
                amount: value.numerator,
                unit,
            },
            Type::Days(unit),
            r,
        )
    }
}
