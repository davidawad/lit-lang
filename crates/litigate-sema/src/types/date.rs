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
    let year = text[0..4].parse().ok()?;
    let month = text[5..7].parse().ok()?;
    let day = text[8..10].parse().ok()?;
    (day >= 1 && day <= days_in_month(year, month)).then_some((year, month, day))
}

/// True when `text` is a valid ISO date.
#[must_use]
pub fn is_valid(text: &str) -> bool {
    parse(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn validates_calendar_dates() {
        assert!(is_valid("2024-02-29"));
        assert!(!is_valid("2023-02-29"));
        assert!(!is_valid("1900-02-29"));
        assert!(is_valid("2000-02-29"));
        assert!(!is_valid("2025-13-01"));
        assert!(!is_valid("2025-1-01"));
    }

    proptest! {
        #[test]
        fn prop_every_formatted_day_in_range_parses(y in 1_u32..9999, m in 1_u32..=12, d in 1_u32..=31) {
            let text = format!("{y:04}-{m:02}-{d:02}");
            prop_assert_eq!(is_valid(&text), d <= days_in_month(y, m));
        }
    }
}
