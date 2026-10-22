//! Exact domain values and types.
use serde::{Deserialize, Serialize};

/// Domain type of a resolved expression (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "argument", rename_all = "snake_case")]
pub enum Type {
    /// Gregorian calendar date.
    Date,
    /// Day count with a fixed counting unit.
    Days(DayUnit),
    /// Attorney or elapsed hours.
    Hours,
    /// Integral minor-unit money in a named currency.
    Money(String),
    /// Exact rational probability in `[0,1]`.
    Probability,
    /// Exact distribution over a closed enum.
    Distribution(String),
    /// Citation resolved to an L0 span.
    Cite,
    /// Half-open date interval.
    DateInterval,
    /// Declared procedural actor role.
    Role,
    /// Boolean value.
    Boolean,
    /// Exact dimensionless rational (integers included).
    Number,
    /// String value.
    String,
    /// Homogeneous finite list.
    List(Box<Type>),
    /// Record with fields sorted by name.
    Record(Vec<(String, Type)>),
    /// User-declared nominal type or enum.
    Named(String),
    /// The single value `unit`.
    Unit,
}

/// Calendar unit carried by a typed day count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayUnit {
    /// Every calendar day.
    Calendar,
    /// Court days after exclusions.
    Court,
    /// Business days after exclusions.
    Business,
}

/// Exact rational number; always reduced with a positive denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rational {
    /// Signed numerator.
    pub numerator: i64,
    /// Positive denominator.
    pub denominator: u64,
}

impl Rational {
    /// The integer `value` as a rational.
    #[must_use]
    pub const fn integer(value: i64) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }
}

/// Integral minor-unit money (`2_000_000.usd` is 200,000,000 cents).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    /// Amount in the currency's minor unit.
    pub minor_units: i64,
    /// Lowercase ISO 4217 code as written in the literal suffix.
    pub currency: String,
}

/// A value that is either authored exactly or an explicit hole (`?`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum Authored<T> {
    /// Exactly authored value.
    Known(T),
    /// Explicit hole with its optional `?("reason")`.
    Hole(Option<String>),
}

/// Closed literal value in typed HIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Value {
    /// ISO-8601 Gregorian date.
    Date(String),
    /// Typed day count.
    Days {
        /// Signed number of days.
        amount: i64,
        /// Counting unit for the amount.
        unit: DayUnit,
    },
    /// Exact hour count.
    Hours(Rational),
    /// Integral minor-unit money.
    Money(Money),
    /// Exact probability.
    Probability(Rational),
    /// Exhaustive enum-to-probability pairs in variant order.
    Distribution(Vec<(String, Rational)>),
    /// Resolved citation reference.
    Cite(String),
    /// Explicit unresolved value and optional reason.
    Hole(Option<String>),
    /// Boolean literal.
    Boolean(bool),
    /// Exact dimensionless number.
    Number(Rational),
    /// String literal.
    String(String),
    /// Enum variant `Enum::Variant` (fully qualified).
    Variant(String),
    /// The unit value.
    Unit,
}
