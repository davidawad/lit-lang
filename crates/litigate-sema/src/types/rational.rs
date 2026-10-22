//! Exact rational arithmetic over `i64` numerators and `u64` denominators
//! (SPEC §6). Every operation is checked: overflow and division by zero are
//! errors, never wrap-around or infinities.
use crate::hir::Rational;

/// Why an exact operation failed (reported as L2006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithError {
    /// Numerator or denominator left the representable range.
    Overflow,
    /// Division or remainder by zero.
    DivisionByZero,
    /// `%` on a non-integer operand.
    NonInteger,
    /// A money or day result that is not a whole number of units.
    Inexact,
}

impl ArithError {
    /// Human-readable explanation.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Overflow => "exact arithmetic overflowed",
            Self::DivisionByZero => "division by zero",
            Self::NonInteger => "`%` needs integer operands",
            Self::Inexact => "the exact result is not a whole number of minor units or days",
        }
    }
}

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Builds the reduced rational `numerator / denominator` from wide parts.
///
/// # Errors
/// Division by zero, or a reduced value that does not fit.
pub fn make(numerator: i128, denominator: i128) -> Result<Rational, ArithError> {
    if denominator == 0 {
        return Err(ArithError::DivisionByZero);
    }
    let negative = (numerator < 0) != (denominator < 0);
    let (n, d) = (numerator.unsigned_abs(), denominator.unsigned_abs());
    let g = gcd(n, d).max(1);
    let (n, d) = (n / g, d / g);
    let n = i128::try_from(n).map_err(|_| ArithError::Overflow)?;
    let n = i64::try_from(if negative { -n } else { n }).map_err(|_| ArithError::Overflow)?;
    let d = u64::try_from(d).map_err(|_| ArithError::Overflow)?;
    Ok(Rational {
        numerator: n,
        denominator: d,
    })
}

fn parts(r: Rational) -> (i128, i128) {
    (i128::from(r.numerator), i128::from(r.denominator))
}

/// `a + b`.
///
/// # Errors
/// Overflow.
pub fn add(a: Rational, b: Rational) -> Result<Rational, ArithError> {
    let ((an, ad), (bn, bd)) = (parts(a), parts(b));
    let n = an
        .checked_mul(bd)
        .and_then(|x| bn.checked_mul(ad).and_then(|y| x.checked_add(y)));
    make(n.ok_or(ArithError::Overflow)?, ad * bd)
}

/// `a - b`.
///
/// # Errors
/// Overflow.
pub fn sub(a: Rational, b: Rational) -> Result<Rational, ArithError> {
    add(a, neg(b)?)
}

/// `-a`.
///
/// # Errors
/// Overflow (`i64::MIN`).
pub fn neg(a: Rational) -> Result<Rational, ArithError> {
    make(-i128::from(a.numerator), i128::from(a.denominator))
}

/// `a * b`.
///
/// # Errors
/// Overflow.
pub fn mul(a: Rational, b: Rational) -> Result<Rational, ArithError> {
    let ((an, ad), (bn, bd)) = (parts(a), parts(b));
    make(an * bn, ad * bd)
}

/// `a / b`.
///
/// # Errors
/// Division by zero or overflow.
pub fn div(a: Rational, b: Rational) -> Result<Rational, ArithError> {
    let ((an, ad), (bn, bd)) = (parts(a), parts(b));
    make(an * bd, ad * bn)
}

/// `a % b` on integers (sign follows the dividend).
///
/// # Errors
/// Non-integer operands or division by zero.
pub fn rem(a: Rational, b: Rational) -> Result<Rational, ArithError> {
    if a.denominator != 1 || b.denominator != 1 {
        return Err(ArithError::NonInteger);
    }
    a.numerator
        .checked_rem(b.numerator)
        .map(Rational::integer)
        .ok_or(if b.numerator == 0 {
            ArithError::DivisionByZero
        } else {
            ArithError::Overflow
        })
}

/// Total order of two rationals.
#[must_use]
pub fn cmp(a: Rational, b: Rational) -> std::cmp::Ordering {
    let ((an, ad), (bn, bd)) = (parts(a), parts(b));
    (an * bd).cmp(&(bn * ad))
}

/// True for values in the closed unit interval.
#[must_use]
pub fn is_probability(r: Rational) -> bool {
    r.numerator >= 0 && r.numerator.unsigned_abs() <= r.denominator
}

/// Exact sum of `values`.
///
/// # Errors
/// Overflow.
pub fn sum(values: impl IntoIterator<Item = Rational>) -> Result<Rational, ArithError> {
    values.into_iter().try_fold(Rational::integer(0), add)
}

/// Parses decimal text (`2_000`, `0.35`) into an exact rational.
///
/// # Errors
/// Overflow; the text must already be lexically valid.
pub fn parse_decimal(text: &str) -> Result<Rational, ArithError> {
    let digits: String = text.chars().filter(|c| *c != '_').collect();
    let (int, frac) = digits.split_once('.').unwrap_or((&digits, ""));
    let scale = u32::try_from(frac.len()).map_err(|_| ArithError::Overflow)?;
    let denominator = 10_i128.checked_pow(scale).ok_or(ArithError::Overflow)?;
    let numerator: i128 = format!("{int}{frac}")
        .parse()
        .map_err(|_| ArithError::Overflow)?;
    make(numerator, denominator)
}

/// Renders `r` as `n` or `n/d`.
#[must_use]
pub fn display(r: Rational) -> String {
    if r.denominator == 1 {
        r.numerator.to_string()
    } else {
        format!("{}/{}", r.numerator, r.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn r(n: i64, d: i64) -> Rational {
        make(i128::from(n), i128::from(d)).unwrap_or(Rational::integer(0))
    }

    #[test]
    fn reduces_and_normalizes_sign() {
        assert_eq!(
            r(2, -4),
            Rational {
                numerator: -1,
                denominator: 2
            }
        );
        assert_eq!(parse_decimal("0.35"), Ok(r(7, 20)));
        assert_eq!(parse_decimal("2_000"), Ok(Rational::integer(2000)));
        assert_eq!(div(r(1, 1), r(0, 1)), Err(ArithError::DivisionByZero));
        assert_eq!(rem(r(1, 2), r(1, 1)), Err(ArithError::NonInteger));
        assert_eq!(
            mul(Rational::integer(i64::MAX), r(2, 1)),
            Err(ArithError::Overflow)
        );
        assert_eq!(display(r(1, 3)), "1/3");
    }

    proptest! {
        #[test]
        fn prop_add_sub_round_trip(a in -1000_i64..1000, b in 1_i64..1000, c in -1000_i64..1000, d in 1_i64..1000) {
            let (x, y) = (r(a, b), r(c, d));
            prop_assert_eq!(add(x, y).and_then(|s| sub(s, y)), Ok(x));
            prop_assert_eq!(cmp(add(x, y)?, add(y, x)?), std::cmp::Ordering::Equal);
        }

        #[test]
        fn prop_results_are_reduced(a in -1000_i64..1000, b in 1_i64..1000, c in 1_i64..1000, d in 1_i64..1000) {
            let q = div(r(a, b), r(c, d))?;
            let g = gcd(u128::from(q.numerator.unsigned_abs()), u128::from(q.denominator));
            prop_assert!(g == 1 || q.numerator == 0);
            prop_assert!(q.denominator >= 1);
        }
    }

    impl From<ArithError> for TestCaseError {
        fn from(e: ArithError) -> Self {
            TestCaseError::fail(e.message())
        }
    }
}
