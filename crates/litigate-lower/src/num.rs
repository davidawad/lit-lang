//! Exact numbers to binary64 (SPEC §8.5): an exact rational becomes the
//! nearest `f64`, ties to even, and money becomes major units.
use litigate_sema::hir::{Money, Rational};

/// Minor units per major unit of a supported currency (SPEC §2: `usd` is
/// the closed set in language version 1).
#[must_use]
pub fn minor_per_major(currency: &str) -> Option<u64> {
    (currency == "usd").then_some(100)
}

/// `r` rounded to the nearest binary64, ties to even. `None` when the
/// denominator is zero (never true of a reduced HIR rational).
#[must_use]
pub fn rational(r: Rational) -> Option<f64> {
    ratio(r.numerator, r.denominator)
}

/// `money` in major units (`minor / 100` for usd), correctly rounded.
/// `None` for a currency with no lowering.
#[must_use]
pub fn money(m: &Money) -> Option<f64> {
    ratio(m.minor_units, minor_per_major(&m.currency)?)
}

/// `n / d` rounded to the nearest binary64, ties to even.
///
/// When both operands are exact in binary64 one IEEE division is already
/// correctly rounded. Otherwise the quotient is scaled to a 54-bit integer
/// `q` (`2^53 <= q < 2^54`) with remainder `r` in `u128`, rounded half to
/// even, and scaled back exactly: every intermediate fits because
/// `|n| < 2^63` and `d < 2^64`, and the result's exponent stays far from
/// the subnormal and overflow ranges.
#[must_use]
pub fn ratio(n: i64, d: u64) -> Option<f64> {
    const EXACT: u64 = 1 << 53;
    if d == 0 {
        return None;
    }
    let magnitude = n.unsigned_abs();
    if magnitude < EXACT && d < EXACT {
        #[allow(clippy::cast_precision_loss)]
        return Some(n as f64 / d as f64);
    }
    if magnitude == 0 {
        return Some(0.0);
    }
    let bits = |v: u128| 128 - v.leading_zeros();
    // Choose `shift` so that (magnitude << shift) / d lands in [2^53, 2^54).
    let mut shift = 54 - i32::try_from(bits(u128::from(magnitude))).ok()?
        + i32::try_from(bits(u128::from(d))).ok()?;
    let quotient = |shift: i32| -> (u128, u128, u128) {
        let (num, den) = if shift >= 0 {
            (u128::from(magnitude) << shift, u128::from(d))
        } else {
            (u128::from(magnitude), u128::from(d) << -shift)
        };
        (num / den, num % den, den)
    };
    let (mut q, mut r, mut den) = quotient(shift);
    while q >= 1 << 54 {
        shift -= 1;
        (q, r, den) = quotient(shift);
    }
    while q < 1 << 53 {
        shift += 1;
        (q, r, den) = quotient(shift);
    }
    let twice = r * 2;
    if twice > den || (twice == den && q & 1 == 1) {
        q += 1;
    }
    // `q < 2^55` is exact in binary64, and so is the power-of-two scaling.
    #[allow(clippy::cast_precision_loss)]
    let value = q as f64 * 2f64.powi(-shift);
    Some(if n < 0 { -value } else { value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn small_ratios_are_one_division() {
        assert_eq!(ratio(23, 25), Some(0.92));
        assert_eq!(ratio(-1, 3), Some(-1.0 / 3.0));
        assert_eq!(ratio(0, 7), Some(0.0));
        assert_eq!(ratio(1, 0), None);
    }

    #[test]
    fn money_is_major_units() {
        let m = Money {
            minor_units: 60_500,
            currency: "usd".into(),
        };
        assert_eq!(money(&m), Some(605.0));
        let eur = Money {
            minor_units: 1,
            currency: "eur".into(),
        };
        assert_eq!(money(&eur), None);
    }

    #[test]
    fn large_operands_round_half_to_even() {
        // 2^53 + 1 is not representable: it is a tie between 2^53 and
        // 2^53 + 2, and rounds to the even mantissa 2^53.
        let n = (1i64 << 53) + 1;
        assert_eq!(ratio(n, 1), Some(9_007_199_254_740_992.0));
        // 2^53 + 3 ties between 2^53 + 2 and 2^53 + 4: even is + 4.
        assert_eq!(ratio((1i64 << 53) + 3, 1), Some(9_007_199_254_740_996.0));
        assert_eq!(ratio(i64::MAX, 1), Some(9_223_372_036_854_775_808.0));
        assert_eq!(ratio(1, u64::MAX), Some(1.0 / 18_446_744_073_709_551_615.0));
    }

    proptest! {
        #[test]
        fn agrees_with_ieee_division_when_operands_are_exact(
            n in -(1i64 << 52)..(1i64 << 52),
            d in 1u64..(1u64 << 52),
        ) {
            #[allow(clippy::cast_precision_loss)]
            let expected = n as f64 / d as f64;
            prop_assert_eq!(ratio(n, d), Some(expected));
        }

        #[test]
        fn scaling_both_operands_is_invisible(n in -(1i64 << 40)..(1i64 << 40), d in 1u64..(1u64 << 20), k in 1u64..(1u64 << 22)) {
            // n*k / d*k needs the wide path once d*k >= 2^53; the value is
            // the same rational, so the rounding must be too.
            let big = n * i64::try_from(k).unwrap_or(1);
            prop_assert_eq!(ratio(big, d * k), ratio(n, d));
        }

        #[test]
        fn integers_round_like_the_cast(n in any::<i64>()) {
            #[allow(clippy::cast_precision_loss)]
            let expected = n as f64;
            prop_assert_eq!(ratio(n, 1), Some(expected));
        }
    }
}
