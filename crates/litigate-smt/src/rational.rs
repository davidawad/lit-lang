//! Exact rationals for coefficients, constants and model values.
use std::fmt;

use crate::Error;

/// An exact rational number, always reduced with a positive denominator.
///
/// Both parts are `i64`; arithmetic is done in `i128` and an
/// [`Error::Overflow`] is returned when a reduced result leaves `i64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
    num: i64,
    den: i64,
}

impl Rational {
    /// Zero.
    pub const ZERO: Self = Self { num: 0, den: 1 };
    /// One.
    pub const ONE: Self = Self { num: 1, den: 1 };
    /// Minus one.
    pub const MINUS_ONE: Self = Self { num: -1, den: 1 };

    /// The integer `n`.
    #[must_use]
    pub const fn int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    /// `num / den`, reduced.
    ///
    /// # Errors
    ///
    /// [`Error::ZeroDenominator`] when `den` is zero, [`Error::Overflow`]
    /// when the reduced value does not fit (only `i64::MIN` parts can).
    pub fn new(num: i64, den: i64) -> Result<Self, Error> {
        if den == 0 {
            return Err(Error::ZeroDenominator);
        }
        Self::reduce(num.into(), den.into())
    }

    /// Reduces `num / den` (`den != 0`) into `i64` parts.
    fn reduce(num: i128, den: i128) -> Result<Self, Error> {
        let g = gcd(num.unsigned_abs(), den.unsigned_abs());
        // g divides both, so the quotients are exact; g >= 1 since den != 0.
        let (num, den) = (num / g, den / g);
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        i64::try_from(num)
            .and_then(|num| i64::try_from(den).map(|den| Self { num, den }))
            .map_err(|_| Error::Overflow)
    }

    /// The numerator (sign-carrying).
    #[must_use]
    pub const fn numer(self) -> i64 {
        self.num
    }

    /// The denominator (always positive).
    #[must_use]
    pub const fn denom(self) -> i64 {
        self.den
    }

    /// Whether the value is an integer.
    #[must_use]
    pub const fn is_integer(self) -> bool {
        self.den == 1
    }

    /// Exact sum.
    ///
    /// # Errors
    ///
    /// [`Error::Overflow`] when the result leaves `i64` parts.
    pub fn checked_add(self, other: Self) -> Result<Self, Error> {
        // |num * den| < 2^126 for each product, so the sum fits i128.
        let num = i128::from(self.num) * i128::from(other.den)
            + i128::from(other.num) * i128::from(self.den);
        Self::reduce(num, i128::from(self.den) * i128::from(other.den))
    }

    /// Exact product.
    ///
    /// # Errors
    ///
    /// [`Error::Overflow`] when the result leaves `i64` parts.
    pub fn checked_mul(self, other: Self) -> Result<Self, Error> {
        Self::reduce(
            i128::from(self.num) * i128::from(other.num),
            i128::from(self.den) * i128::from(other.den),
        )
    }

    /// SMT-LIB literal of this value in sort `Int` (only called on integers).
    pub(crate) fn smt_int(self) -> String {
        neg(self.num < 0, self.num.unsigned_abs().to_string())
    }

    /// SMT-LIB literal of this value in sort `Real`.
    pub(crate) fn smt_real(self) -> String {
        let mag = format!("{}.0", self.num.unsigned_abs());
        let body = if self.is_integer() {
            mag
        } else {
            format!("(/ {mag} {}.0)", self.den)
        };
        neg(self.num < 0, body)
    }
}

fn neg(negative: bool, body: String) -> String {
    if negative {
        format!("(- {body})")
    } else {
        body
    }
}

fn gcd(mut a: u128, mut b: u128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    // a <= 2^127 only when both inputs were i128::MIN in magnitude, which
    // products of i64 parts never reach; a == 0 only if both were zero,
    // which a non-zero denominator rules out.
    i128::try_from(a).unwrap_or(1)
}

impl From<i64> for Rational {
    fn from(n: i64) -> Self {
        Self::int(n)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reduces_and_normalises_sign() {
        let r = Rational::new(6, -4).unwrap();
        assert_eq!((r.numer(), r.denom()), (-3, 2));
        assert_eq!(Rational::new(0, -7).unwrap(), Rational::ZERO);
        assert!(!r.is_integer());
        assert!(Rational::from(5).is_integer());
    }

    #[test]
    fn new_rejects_zero_denominator() {
        assert_eq!(Rational::new(1, 0), Err(Error::ZeroDenominator));
    }

    #[test]
    fn new_overflows_on_negated_min() {
        // -i64::MIN is not an i64.
        assert_eq!(Rational::new(i64::MIN, -1), Err(Error::Overflow));
        // The denominator side overflows too.
        assert_eq!(Rational::new(1, i64::MIN), Err(Error::Overflow));
        assert_eq!(Rational::new(i64::MIN, i64::MIN).unwrap(), Rational::ONE);
    }

    #[test]
    fn arithmetic_is_exact() {
        let third = Rational::new(1, 3).unwrap();
        let sixth = Rational::new(1, 6).unwrap();
        assert_eq!(
            third.checked_add(sixth).unwrap(),
            Rational::new(1, 2).unwrap()
        );
        assert_eq!(
            third.checked_mul(Rational::MINUS_ONE).unwrap(),
            Rational::new(-1, 3).unwrap()
        );
        assert_eq!(
            Rational::int(i64::MAX).checked_add(Rational::ONE),
            Err(Error::Overflow)
        );
        assert_eq!(
            Rational::int(i64::MAX).checked_mul(Rational::int(2)),
            Err(Error::Overflow)
        );
    }

    #[test]
    fn display_and_smtlib() {
        let r = Rational::new(-3, 4).unwrap();
        assert_eq!(r.to_string(), "-3/4");
        assert_eq!(Rational::int(7).to_string(), "7");
        assert_eq!(r.smt_real(), "(- (/ 3.0 4.0))");
        assert_eq!(Rational::int(2).smt_real(), "2.0");
        assert_eq!(Rational::int(-2).smt_int(), "(- 2)");
        assert_eq!(Rational::int(i64::MIN).smt_int(), "(- 9223372036854775808)");
    }

    #[test]
    fn gcd_saturates_out_of_range() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(1 << 127, 0), 1);
    }
}
