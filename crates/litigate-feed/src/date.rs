//! Civil dates (`YYYY-MM-DD`) as days since 1970-01-01, proleptic
//! Gregorian (H. Hinnant's algorithms).

use std::time::{SystemTime, UNIX_EPOCH};

/// Today, UTC.
#[must_use]
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format_days(i64::try_from(secs / 86_400).unwrap_or_default())
}

/// `date` moved by `days`; `None` for a malformed date.
#[must_use]
pub fn add_days(date: &str, days: i64) -> Option<String> {
    Some(format_days(parse_days(date)? + days))
}

/// Days since 1970-01-01 of `YYYY-MM-DD`.
#[must_use]
pub fn parse_days(date: &str) -> Option<i64> {
    let mut parts = date.splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || date.len() != 10 {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn format_days(days: i64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_across_months_years_and_leap_days() {
        assert_eq!(parse_days("1970-01-01"), Some(0));
        assert_eq!(add_days("2026-10-27", -1).as_deref(), Some("2026-10-26"));
        assert_eq!(add_days("2026-03-01", -1).as_deref(), Some("2026-02-28"));
        assert_eq!(add_days("2024-03-01", -1).as_deref(), Some("2024-02-29"));
        assert_eq!(add_days("2026-12-31", 1).as_deref(), Some("2027-01-01"));
        assert_eq!(today().len(), 10);
        for bad in [
            "2026-13-01",
            "2026-10-00",
            "26-1-1",
            "x",
            "2026-10",
            "2026-x-01",
        ] {
            assert_eq!(parse_days(bad), None, "{bad}");
        }
    }
}
