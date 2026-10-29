//! Exact domain values and types.
use serde::{Deserialize, Serialize};

/// Domain type of a resolved expression (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "argument", rename_all = "snake_case")]
pub enum Type {
    /// Gregorian calendar date.
    Date,
    /// Period with a fixed unit (days of a kind, weeks, months, years).
    Days(DayUnit),
    /// Attorney or elapsed hours.
    Hours,
    /// Integral minor-unit money in a named currency.
    Money(String),
    /// Exact rational probability in `[0,1]`.
    Probability,
    /// Exact distribution over a closed enum.
    Distribution(String),
    /// `Dist<E, C>`: a `Dist<E>` for each variant of the conditioning
    /// enum `C` (outcome enum, then condition enum), total over `C`.
    Conditional(String, String),
    /// `Credal`: a closed interval of probabilities `[lower, upper]`
    /// (epistemic uncertainty about a point probability).
    Credal,
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
    /// Reference to a declared edge (`EdgeRef`): written as the edge's id
    /// (`edge`, or `pack::edge` outside the primary package), never as a
    /// string.
    Edge,
    /// Homogeneous finite list.
    List(Box<Type>),
    /// Record with fields sorted by name.
    Record(Vec<(String, Type)>),
    /// User-declared nominal type or enum.
    Named(String),
    /// The single value `unit`.
    Unit,
}

/// Unit carried by a typed period. The units do not convert into one
/// another (design review 2026-10-23 §2.1): a week is seven calendar days
/// only once a counting rule counts it, and months and years are calendar
/// months counted under the rule's end-of-month policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayUnit {
    /// Every calendar day.
    Calendar,
    /// Court days after exclusions.
    Court,
    /// Business days after exclusions.
    Business,
    /// Weeks of seven calendar days.
    Week,
    /// Calendar months.
    Month,
    /// Calendar years.
    Year,
    /// Hours, counted from the trigger instant under the counting rule's
    /// hours provision (FRCP 6(a)(2), Fed. R. Crim. P. 45(a)(2)). Only a deadline's
    /// period carries it: an `N.hours` literal is `Type::Hours`.
    Hour,
}

impl DayUnit {
    /// `n` of this unit in prose: `21 calendar days`, `1 month`.
    #[must_use]
    pub fn prose(self, n: u64) -> String {
        let noun = match self {
            Self::Calendar => "calendar day",
            Self::Court => "court day",
            Self::Business => "business day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
            Self::Hour => "hour",
        };
        format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
    }

    /// Whether the unit is a calendar month or year, counted under an
    /// end-of-month policy.
    #[must_use]
    pub const fn is_monthly(self) -> bool {
        matches!(self, Self::Month | Self::Year)
    }
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
    /// Typed period.
    Days {
        /// Signed number of units.
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
    /// A conditional distribution: each condition variant (in variant
    /// order) with its exhaustive distribution.
    Conditional(Vec<(String, Vec<(String, Rational)>)>),
    /// A credal interval `[lower, upper]` within `[0, 1]`.
    Credal {
        /// Least probability.
        lower: Rational,
        /// Greatest probability.
        upper: Rational,
    },
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
    /// A resolved edge: `edge` when declared in the primary package (or the
    /// view's package), else `pack::edge`.
    Edge(String),
    /// Enum variant `Enum::Variant` (fully qualified).
    Variant(String),
    /// The unit value.
    Unit,
}
