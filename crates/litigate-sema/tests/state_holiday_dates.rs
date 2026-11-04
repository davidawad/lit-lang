//! State day-after-Thanksgiving calendars use mechanically checked rules.
use std::fs;
use std::path::Path;

fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// Gregorian weekday, with Sunday = 0 (Zeller's congruence).
fn weekday(year: i32, month: i32, day: i32) -> i32 {
    let (year, month) = if month < 3 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let century = year / 100;
    let within = year % 100;
    (day + (13 * (month + 1)) / 5 + within + within / 4 + century / 4 + 5 * century) % 7
}

fn expected_day_after_thanksgiving(year: i32) -> String {
    // Zeller uses Saturday = 0. Find the first Thursday, then advance 21 days
    // to the fourth Thursday and one more day to the statutory holiday.
    let first = weekday(year, 11, 1);
    let first_thursday = 1 + (5 - first).rem_euclid(7);
    format!("{year:04}-11-{:02}", first_thursday + 22)
}

#[test]
fn state_thanksgiving_calendars_use_fourth_thursday_plus_one() {
    let states = repo().join("lit/core/states");
    let mut found = std::collections::BTreeSet::new();
    for entry in fs::read_dir(states).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "lit") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains("day-after-thanksgiving") {
                assert!(
                    !trimmed.contains("nth(4, friday, 11)"),
                    "{}: {trimmed}",
                    path.display()
                );
            }
            if trimmed.starts_with("day-after-thanksgiving:") {
                assert!(
                    trimmed.contains("offset(4, thursday, 11, 1)"),
                    "{}: unexpected Thanksgiving rule: {trimmed}",
                    path.display()
                );
                found.insert(path.file_stem().unwrap().to_string_lossy().to_string());
            }
            let Some(rest) = trimmed.strip_prefix("day-after-thanksgiving-") else {
                continue;
            };
            let Some((year, date)) = rest.split_once(": once(") else {
                continue;
            };
            let year: i32 = year
                .parse()
                .unwrap_or_else(|_| panic!("invalid year in {}: {trimmed}", path.display()));
            let date = date.split(')').next().unwrap();
            assert_eq!(
                date,
                expected_day_after_thanksgiving(year),
                "{}: {trimmed}",
                path.display()
            );
            found.insert(path.file_stem().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        found.contains("delaware"),
        "Delaware calendar was not checked"
    );
    assert!(
        found.contains("oklahoma"),
        "Oklahoma calendar was not checked"
    );
}
