//! Proleptic Gregorian dates as ISO `YYYY-MM-DD` text (SPEC §2 `Date`).

/// Days in `month` of `year` (0 for an invalid month).
#[must_use]
pub const fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    }
}

/// Parses `YYYY-MM-DD` into `(year, month, day)` when it names a real date.
#[must_use]
pub fn parse(text: &str) -> Option<(u32, u32, u32)> {
    let bytes = text.as_bytes();
    let shape = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shape {
        return None;
    }
    // The shape check guarantees ASCII digits, so each field is exact.
    let field = |digits: &[u8]| digits.iter().fold(0, |n, d| n * 10 + u32::from(d - b'0'));
    let (year, month, day) = (
        field(&bytes[0..4]),
        field(&bytes[5..7]),
        field(&bytes[8..10]),
    );
    (day >= 1 && day <= days_in_month(year, month)).then_some((year, month, day))
}

/// True when `text` is a valid ISO date.
#[must_use]
pub fn is_valid(text: &str) -> bool {
    parse(text).is_some()
}

/// Days since 1970-01-01 of a valid `(year, month, day)` (proleptic
/// Gregorian; H. Hinnant's `days_from_civil`).
fn days_from_civil(y: u32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The date `days` after 1970-01-01, as `YYYY-MM-DD` (`civil_from_days`).
fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// The date `days` calendar days after valid ISO date `text`.
#[must_use]
pub fn add_days(text: &str, days: u64) -> Option<String> {
    let (y, m, d) = parse(text)?;
    let n = i64::try_from(days).ok()?;
    Some(civil_from_days(days_from_civil(y, m, d).checked_add(n)?))
}

/// The date `days` calendar days before valid ISO date `text`.
#[must_use]
pub fn sub_days(text: &str, days: u64) -> Option<String> {
    let (y, m, d) = parse(text)?;
    let n = i64::try_from(days).ok()?;
    Some(civil_from_days(days_from_civil(y, m, d).checked_sub(n)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_calendar_days() {
        assert_eq!(add_days("2024-02-28", 1).as_deref(), Some("2024-02-29"));
        assert_eq!(add_days("2023-02-28", 1).as_deref(), Some("2023-03-01"));
        assert_eq!(add_days("1999-12-31", 1).as_deref(), Some("2000-01-01"));
        assert_eq!(add_days("2004-02-24", 30).as_deref(), Some("2004-03-25"));
        assert_eq!(add_days("1970-01-01", 0).as_deref(), Some("1970-01-01"));
        assert_eq!(add_days("0001-01-01", 0).as_deref(), Some("0001-01-01"));
        assert_eq!(add_days("2024-13-01", 1), None);
        assert_eq!(add_days("2024-01-01", u64::MAX), None);
        assert_eq!(add_days("2024-01-01", i64::MAX as u64), None);
    }

    #[test]
    fn subtracts_calendar_days() {
        assert_eq!(sub_days("2026-01-13", 10).as_deref(), Some("2026-01-03"));
        assert_eq!(sub_days("2024-03-01", 1).as_deref(), Some("2024-02-29"));
        assert_eq!(sub_days("2000-01-01", 1).as_deref(), Some("1999-12-31"));
        assert_eq!(sub_days("2024-13-01", 1), None);
        assert_eq!(sub_days("2024-01-01", u64::MAX), None);
        assert_eq!(sub_days("0001-01-01", i64::MAX as u64), None);
    }
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_adding_days_round_trips(y in 1_u32..9000, m in 1_u32..=12, d in 1_u32..=28, n in 0_u64..100_000) {
            let text = format!("{y:04}-{m:02}-{d:02}");
            let later = add_days(&text, n).expect("in range");
            let (ly, lm, ld) = parse(&later).expect("a valid date");
            prop_assert_eq!(days_from_civil(ly, lm, ld) - days_from_civil(y, m, d), i64::try_from(n).expect("small"));
        }
    }

    #[test]
    fn validates_calendar_dates() {
        assert!(is_valid("2024-02-29"));
        assert!(!is_valid("2023-02-29"));
        assert!(!is_valid("1900-02-29"));
        assert!(is_valid("2000-02-29"));
        assert!(!is_valid("2025-13-01"));
        assert!(!is_valid("2025-1-01"));
        assert!(!is_valid("2025/01/01"));
        assert!(!is_valid("2025-01/01"));
        assert!(!is_valid("2025-0a-01"));
        assert!(!is_valid("2025-01-00"));
        assert_eq!(parse("2024-12-31"), Some((2024, 12, 31)));
    }

    proptest! {
        #[test]
        fn prop_every_formatted_day_in_range_parses(y in 1_u32..9999, m in 1_u32..=12, d in 1_u32..=31) {
            let text = format!("{y:04}-{m:02}-{d:02}");
            prop_assert_eq!(is_valid(&text), d <= days_in_month(y, m));
        }
    }
}
