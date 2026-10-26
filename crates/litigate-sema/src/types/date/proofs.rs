//! Kani proofs (`just verify-kani`) of the date arithmetic deadline counting
//! rests on (design review §3.7 item 6). Bounded to years 0001-9999, the
//! range `parse` accepts.
use super::{civil, days_from_civil, days_in_month, parse};

/// 0001-01-01 and 9999-12-31 as day numbers since 1970-01-01.
const FIRST: i64 = -719_162;
const LAST: i64 = 2_932_896;

/// A month is 28 to 31 days, and February has 29 exactly in leap years.
#[kani::proof]
fn month_lengths_are_gregorian() {
    let (year, month): (u32, u32) = (kani::any(), kani::any());
    let n = days_in_month(year, month);
    if (1..=12).contains(&month) {
        assert!((28..=31).contains(&n));
    } else {
        assert_eq!(n, 0);
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    assert_eq!(days_in_month(year, 2) == 29, leap);
}

/// Every day number in range names a real date, and converting it back
/// yields the same day number: `civil` and `days_from_civil` are inverse.
#[kani::proof]
fn civil_round_trips() {
    let z: i64 = kani::any();
    kani::assume((FIRST..=LAST).contains(&z));
    let (y, m, d) = civil(z);
    assert!((1..=9999).contains(&y));
    assert!((1..=12).contains(&m));
    let (y, m, d) = (y as u32, m as u32, d as u32);
    assert!(d >= 1 && d <= days_in_month(y, m));
    assert_eq!(days_from_civil(y, m, d), z);
}

/// Day numbers count consecutive days: the calendar successor of a valid
/// date is exactly one day number later.
#[kani::proof]
fn successor_is_one_day_later() {
    let (y, m, d): (u32, u32, u32) = (kani::any(), kani::any(), kani::any());
    kani::assume((1..9999).contains(&y) && (1..=12).contains(&m));
    kani::assume(d >= 1 && d <= days_in_month(y, m));
    let next = if d < days_in_month(y, m) {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    };
    assert_eq!(
        days_from_civil(next.0, next.1, next.2),
        days_from_civil(y, m, d) + 1
    );
}

/// `parse` accepts only real dates, whatever ten bytes it is given.
#[kani::proof]
#[kani::unwind(11)]
fn parse_accepts_only_real_dates() {
    let bytes: [u8; 10] = kani::any();
    let Ok(text) = core::str::from_utf8(&bytes) else {
        return;
    };
    if let Some((y, m, d)) = parse(text) {
        assert!(y <= 9999);
        assert!((1..=12).contains(&m));
        assert!(d >= 1 && d <= days_in_month(y, m));
    }
}
