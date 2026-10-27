//! The date the `litigate` binary judges deadlines at (lg-rpmf): a matter
//! whose `as_of` is older has its deadlines past due by then lapse, so a
//! stale `as_of` cannot show a missed deadline as open.

/// The variable that pins today (a `YYYY-MM-DD` date), for reproducible
/// runs; the system date when unset.
pub const VAR: &str = "LITIGATE_TODAY";

/// Today: [`VAR`] when set (`check` and `timeline` reject one that is not
/// a date), else the system date in UTC; `None` on a target with no clock.
#[must_use]
pub fn today() -> Option<String> {
    std::env::var(VAR).ok().or_else(system)
}

/// The system date in UTC.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn system() -> Option<String> {
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    let days = i64::try_from(since.as_secs() / 86_400).ok()?;
    Some(litigate_engine::clock::Date::from_epoch_day(days).to_iso())
}

/// No clock on this target.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn system() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_engine::clock::Date;

    #[test]
    fn the_system_date_is_a_date() {
        let today = system().unwrap_or_default();
        assert!(Date::parse_iso(&today).is_ok(), "{today}");
        assert!(today.as_str() > "2026-01-01", "{today}");
    }
}
