//! The deadline domain's validation (lg-fq2.5; docs/litigate/VALIDATION.md):
//! the worked examples of the 2009 Committee Notes to FRCP 6 and FRAP 26
//! and the timeliness rulings of the lg-fq2.3 corpus, each counted under
//! the rules in force on its own dates.
//!
//! `docs/litigate/validation` holds the golden tests a `.lit` test can
//! state. This harness runs them, ties every Note example and every ruling
//! of `sources/change-corpus/rulings/MANIFEST.json` to its test or to a
//! documented disagreement, counts the examples no `.lit` test can state
//! (periods in hours, a clerk's office that is inaccessible) on the clock
//! IR the package and `lit/core` lower to, pins each disagreement, and
//! checks the agreement rates VALIDATION.md reports against the counts.
// Test code: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use litigate_engine::clock::{
    compute_due_date, compute_hour_deadline, Clock, ClockConfig, ClockLibrary, ComputedDeadline,
    Date, DayDeadline, Direction, HourDeadline, Unit,
};
use litigate_lower::{Lowerer, Options};
use litigate_sema::hir;
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::OnceLock;
use support::{litigate, repo};

const PACKAGE: &str = "docs/litigate/validation";

/// The worked examples of the 2009 Committee Notes, numbered as in
/// VALIDATION.md §2: `C` the civil Note to FRCP 6, `A` the appellate Note
/// to FRAP 26.
const NOTE_EXAMPLES: [&str; 25] = [
    "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "A1", "A2", "A3", "A4",
    "A5", "A6", "A7", "A8", "A9", "A10", "A11", "A12", "A13", "A14",
];

/// Note examples counted here, on the clock IR, because a `.lit` test
/// states neither a time of day nor an inaccessible clerk's office
/// (VALIDATION.md §5, D2 and D5).
const NOTE_ON_THE_ENGINE: [&str; 8] = ["C4", "C5", "C8", "C11", "A4", "A5", "A8", "A11"];

/// Note examples no part of the model can state: when the last day ends
/// for a filing by an inmate or through a carrier's drop box (FRAP
/// 26(a)(4)(C)), a time of day the filer's own circumstances fix (D5).
const NOTE_NOT_STATED: [&str; 2] = ["A13", "A14"];

/// Rulings whose outcome is counted here: the closed courthouse of
/// Circuitronix is an inaccessible clerk's office (D2).
const RULINGS_ON_THE_ENGINE: [&str; 1] = ["circuitronix-11cir-2021"];

/// Rulings the model refuses to count: their one-year periods run under
/// FRCP 6 as amended in 1985, which lit/core gives no month or year
/// provision (D1).
const RULINGS_REFUSED: [&str; 6] = [
    "flanagan-5cir-1998",
    "hernandez-4cir-2000",
    "patterson-9cir-2001",
    "moore-8cir-1999",
    "mickens-2cir-1998",
    "burns-3cir-1998",
];

fn d(s: &str) -> Date {
    Date::parse_iso(s).unwrap()
}

/// The checked HIR of the validation package and its dependencies.
fn packages() -> &'static [hir::Package] {
    static PACKAGES: OnceLock<Vec<hir::Package>> = OnceLock::new();
    PACKAGES.get_or_init(|| {
        let root = repo();
        let (db, ws) = litigate_sema::load(&root, &[root.join(PACKAGE)], None).unwrap();
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .filter_map(|c| c.package.clone())
            .collect()
    })
}

fn package(name: &str) -> &'static hir::Package {
    packages()
        .iter()
        .find(|p| p.manifest.name == name)
        .unwrap_or_else(|| panic!("{name} has HIR"))
}

/// Package `name`'s clock fragment lowered at `as_of`: the versions of its
/// counting rules in force that day.
fn lowered(name: &str, as_of: &str) -> ClockLibrary {
    let all: Vec<&hir::Package> = packages().iter().collect();
    let options = Options {
        as_of: Some(as_of.to_owned()),
        known_at: None,
    };
    Lowerer::new(&all)
        .package(package(name), &options)
        .unwrap_or_else(|e| panic!("{name} at {as_of}: {e:#?}"))
        .clock
        .expect("a clock fragment")
        .0
}

fn rule<'l>(lib: &'l ClockLibrary, id: &str) -> Clock<'l> {
    lib.clock(id).unwrap()
}

fn days(trigger: &str, n: i64, direction: Direction) -> DayDeadline {
    DayDeadline {
        trigger: d(trigger),
        days: n,
        direction,
        unit: Unit::Calendar,
        service_method: None,
    }
}

fn due(dl: &DayDeadline, clock: &Clock, cfg: &ClockConfig) -> String {
    compute_due_date(dl, clock, cfg).unwrap().due_date
}

/// The clerk's office inaccessible on the period's last day (FRCP
/// 6(a)(3)(A), FRAP 26(a)(3)(A)).
fn inaccessible() -> ClockConfig {
    ClockConfig {
        clerk_inaccessible: true,
        ..ClockConfig::default()
    }
}

/// Seconds since the epoch of `date` at `hh:mm` UTC.
fn instant(date: &str, hh: i64, mm: i64) -> i64 {
    d(date).epoch_day() * 86_400 + hh * 3_600 + mm * 60
}

/// `litigate test` on the validation package: the names of its tests.
fn golden_tests() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let run = litigate(&["test", PACKAGE]);
        assert_eq!(run.code, 0, "{}", run.stdout);
        let tests = run.json["result"]["tests"].as_array().unwrap();
        let failed: Vec<&Value> = tests.iter().filter(|t| t["passed"] != true).collect();
        assert!(failed.is_empty(), "{failed:#?}");
        tests
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_owned())
            .collect()
    })
}

/// Whether a golden test is named for `id`: `id: ...`, `id (...): ...` or
/// `id, ...`.
fn named_for(name: &str, id: &str) -> bool {
    name.split([':', ' ', ',']).next() == Some(id)
}

fn covered(id: &str) -> bool {
    golden_tests().iter().any(|n| named_for(n, id))
}

/// The validation rulings of the lg-fq2.3 manifest, by id.
fn rulings() -> Vec<String> {
    let path = repo().join("sources/change-corpus/rulings/MANIFEST.json");
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    manifest["rulings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["counts_as_validation_ruling"] == true)
        .map(|r| r["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn every_note_example_has_its_golden_test_or_its_reason() {
    for id in NOTE_EXAMPLES {
        let engine = NOTE_ON_THE_ENGINE.contains(&id);
        let unstated = NOTE_NOT_STATED.contains(&id);
        assert!(
            covered(id) ^ (engine || unstated),
            "{id}: a golden test, or counted on the engine, or not stated; exactly one"
        );
    }
}

#[test]
fn every_ruling_has_its_golden_test_or_its_reason() {
    let all = rulings();
    assert_eq!(
        all.len(),
        35,
        "the lg-fq2.3 corpus counts 35 validation rulings"
    );
    for id in &all {
        let refused = RULINGS_REFUSED.contains(&id.as_str());
        assert!(
            covered(id) != refused,
            "{id}: a golden test, or refused (D1); exactly one"
        );
    }
    for id in RULINGS_REFUSED.iter().chain(&RULINGS_ON_THE_ENGINE) {
        assert!(all.iter().any(|r| r == id), "{id} is a validation ruling");
    }
    // Every golden test names a Note example, a ruling, the split, or a
    // pinned disagreement.
    for name in golden_tests() {
        let known = NOTE_EXAMPLES.iter().any(|id| named_for(name, id))
            || all.iter().any(|id| named_for(name, id))
            || named_for(name, "frap-4a6-notice")
            || name.starts_with("DISAGREES ");
        assert!(known, "{name}");
    }
}

/// The 2009 Notes' examples a `.lit` test cannot state, on the 2009 rules
/// (lowered at a date in their window) and on the dates the `.lit` golden
/// tests use (VALIDATION.md §2).
#[test]
fn the_hour_and_clerk_examples_hold_on_the_2009_rules() {
    let core = lowered("core", "2012-06-01");
    let state = lowered("change-validation", "2012-06-01");
    let open = ClockConfig::default();
    for (example, id, start) in [("C4", "frcp6", (10, 23)), ("A4", "frap26", (10, 0))] {
        // "A 72-hour period that commences at 10:23 a.m. [10:00 a.m. in the
        // appellate Note] on Friday, November 2, 2007, will run until 9:23
        // a.m. [9:00 a.m.] on Monday, November 5": every hour counts, so
        // the end is 72 elapsed hours later, an hour earlier on the clock
        // once daylight saving time ends. On November 2, 2012 (a Friday,
        // daylight time ending Sunday, November 4, as in 2007), 10:23 a.m.
        // EDT is 14:23 UTC and 9:23 a.m. EST on Monday is 14:23 UTC.
        let (hh, mm) = start;
        let h = HourDeadline {
            trigger_epoch_seconds: instant("2012-11-02", hh + 4, mm),
            hours: 72,
        };
        let got = compute_hour_deadline(&h, &rule(&core, id), &open).unwrap();
        assert_eq!(
            got.due_datetime,
            format!("2012-11-05T{:02}:{mm:02}:00", hh + 4),
            "{example}"
        );
        assert!(!got.last_day_rolled, "{example}");
        // "If ... the time period expires at a specific time (say, 2:17
        // p.m.) on a Saturday, Sunday, or legal holiday, then the deadline
        // is extended to the same time (2:17 p.m.) on the next day that is
        // not": 48 hours from 2:17 p.m. on Saturday, September 1, 2012 end
        // on Labor Day, and run to 2:17 p.m. on Tuesday.
        let example = if example == "C4" { "C5" } else { "A5" };
        let h = HourDeadline {
            trigger_epoch_seconds: instant("2012-09-01", 14, 17),
            hours: 48,
        };
        let got = compute_hour_deadline(&h, &rule(&core, id), &open).unwrap();
        assert_eq!(got.due_datetime, "2012-09-04T14:17:00", "{example}");
        assert!(got.last_day_rolled, "{example}");
    }
    // C8, A8: "If the clerk's office is inaccessible on August 31, then
    // subdivision (a)(3) extends the filing deadline forward to the next
    // accessible day that is not a Saturday, Sunday, or legal holiday":
    // Tuesday, September 4.
    for (example, id, dl) in [
        ("C8", "frcp6", days("2012-09-22", 21, Direction::Backward)),
        ("A8", "frap26", days("2012-09-11", 10, Direction::Backward)),
    ] {
        assert_eq!(due(&dl, &rule(&core, id), &open), "2012-08-31", "{example}");
        assert_eq!(
            due(&dl, &rule(&core, id), &inaccessible()),
            "2012-09-04",
            "{example}"
        );
    }
    // C11, A11: a 14th day before an event on Patriot's Day is due that
    // day; "if the clerk's office is inaccessible on Monday, April 21, then
    // subdivision (a)(3) extends the April 21 filing deadline forward to
    // the next accessible day": Tuesday, April 22.
    for (example, id) in [
        ("C11", "frcp6-patriots-day"),
        ("A11", "frap26-patriots-day"),
    ] {
        let dl = days("2014-05-05", 14, Direction::Backward);
        assert_eq!(
            due(&dl, &rule(&state, id), &open),
            "2014-04-21",
            "{example}"
        );
        assert_eq!(
            due(&dl, &rule(&state, id), &inaccessible()),
            "2014-04-22",
            "{example}"
        );
    }
}

/// Circuitronix: "the Chief Judge closed the courthouse on July 5", so the
/// clerk's office was inaccessible on the last day after the July 4 roll,
/// and the motion filed Monday, July 8, 2019 was timely (FRCP 6(a)(3)(A)).
#[test]
fn circuitronix_the_closed_courthouse_makes_the_monday_motion_timely() {
    let core = lowered("core", "2019-06-06");
    let dl = days("2019-06-06", 28, Direction::Forward);
    let frcp6 = rule(&core, "frcp6");
    assert_eq!(due(&dl, &frcp6, &ClockConfig::default()), "2019-07-05");
    let got = compute_due_date(&dl, &frcp6, &inaccessible()).unwrap();
    assert_eq!(got.due_date, "2019-07-08");
    assert!(got.clerk_inaccessibility_applied);
    assert!(d(&got.due_date) >= d("2019-07-08"), "the court: timely");
}

/// `litigate worlds` on Bowles's appeal (`cases/bowles.case`) at
/// `date`, its `as_of` and `known_at`: Thompson v. INS's `in_force`, and
/// each filing's worlds by outcome.
fn bowles_at(date: &str) -> (bool, Value) {
    let run = litigate(&[
        "worlds",
        "docs/litigate/validation/cases/bowles.case",
        "--world",
        "docs/litigate/validation/cases/bowles.world",
        "--path",
        "docs/litigate/examples/change",
        "--path",
        PACKAGE,
        "--as-of",
        date,
        "--known-at",
        date,
    ]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let [world] = run.json["result"]["worlds"].as_array().unwrap().as_slice() else {
        panic!("one world: {}", run.stdout);
    };
    let thompson = world["layers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["name"] == "change-thompson-v-ins")
        .unwrap();
    assert_eq!(
        world["deadlines"][0]["due"], "2004-02-24",
        "FRAP 4(a)(6)'s 14 days from February 10, 2004"
    );
    (
        thompson["in_force"].as_bool().unwrap(),
        world["filings"].clone(),
    )
}

/// Replay before an overruling (VALIDATION.md §4): in the world holding
/// Thompson v. INS and Bowles v. Russell, Bowles's notice of appeal of
/// February 26, 2004 replayed at a date before June 14, 2007 has Thompson
/// in force and is late: "The Sixth Circuit held that the notice was
/// untimely" (Bowles, 551 U.S. at 205, syllabus); from the overruling's date
/// Thompson is withdrawn and the notice is late, as the Supreme Court
/// held.
#[test]
fn bowles_replayed_before_the_overruling_is_late_as_the_sixth_circuit_held() {
    let notice_late = serde_json::json!({"notice": "late"});
    assert_eq!(bowles_at("2007-06-13"), (true, notice_late.clone()));
    assert_eq!(bowles_at("2007-06-14"), (false, notice_late));
}

/// One year from `trigger` under `clock`.
fn year(trigger: &str, clock: &Clock) -> litigate_engine::error::Result<ComputedDeadline> {
    let dl = DayDeadline {
        unit: Unit::Year,
        ..days(trigger, 1, Direction::Forward)
    };
    compute_due_date(&dl, clock, &ClockConfig::default())
}

/// D1 (a modelling error, pinned): the AEDPA rulings count a one-year
/// period from 1996 and 1997 dates under FRCP 6 as amended in 1985, which
/// lit/core gives no month or year provision, so the model refuses them.
/// Counted by the anniversary method these rulings apply (and lit/core's
/// 2001 version uses), every one agrees with its court: this test fails
/// once the 1985 version counts years, and VALIDATION.md must then move
/// them to the agreements.
#[test]
fn d1_the_aedpa_rulings_are_refused_under_the_1985_rule() {
    let core = lowered("core", "1997-01-01");
    let rule1985 = rule(&core, "frcp6");
    for trigger in ["1996-04-24", "1997-06-19"] {
        let err = year(trigger, &rule1985).unwrap_err().to_string();
        assert!(err.contains("no calendar-month provision"), "{err}");
    }
    let core2001 = lowered("core", "2002-01-01");
    let anniversary = rule(&core2001, "frcp6");
    let grace = year("1996-04-24", &anniversary).unwrap().due_date;
    assert_eq!(grace, "1997-04-24");
    // (ruling, last day, filed, timely): Hernandez's state motion of April
    // 23, 1997 tolled the period, leaving one day once certiorari was
    // denied on September 8 (1997-04-24 plus the 138 days tolled).
    let hernandez = d(&grace).add_days(138);
    for (id, last, filed, timely) in [
        ("flanagan-5cir-1998", d(&grace), "1997-04-24", true),
        ("moore-8cir-1999", d(&grace), "1997-04-24", true),
        ("mickens-2cir-1998", d(&grace), "1997-04-23", true),
        ("burns-3cir-1998", d(&grace), "1997-04-22", true),
        ("hernandez-4cir-2000", hernandez, "1997-09-09", true),
        (
            "patterson-9cir-2001",
            d(&year("1997-06-19", &anniversary).unwrap().due_date),
            "1998-06-19",
            true,
        ),
    ] {
        assert!(RULINGS_REFUSED.contains(&id));
        assert_eq!(last >= d(filed), timely, "{id}: last day {last}");
    }
}

/// VALIDATION.md states the rates these tests give.
#[test]
fn the_report_states_the_agreement_rates() {
    let note_agree = NOTE_EXAMPLES
        .iter()
        .filter(|id| covered(id) || NOTE_ON_THE_ENGINE.contains(id))
        .count();
    let all = rulings();
    let ruling_agree = all.iter().filter(|id| covered(id)).count();
    let split_tests = golden_tests()
        .iter()
        .filter(|n| named_for(n, "frap-4a6-notice"))
        .count();
    let pinned: BTreeSet<&str> = golden_tests()
        .iter()
        .filter_map(|n| n.strip_prefix("DISAGREES "))
        .filter_map(|n| n.split(':').next())
        .collect();
    let report = std::fs::read_to_string(repo().join("docs/litigate/VALIDATION.md")).unwrap();
    for line in [
        format!(
            "- 2009 Committee Note examples: {note_agree} of {} agree, 0 disagree, {} cannot be stated.",
            NOTE_EXAMPLES.len(),
            NOTE_NOT_STATED.len()
        ),
        format!(
            "- Timeliness rulings: {ruling_agree} of {} agree with the court, 0 disagree, {} are refused (D1).",
            all.len(),
            RULINGS_REFUSED.len()
        ),
        format!(
            "- Circuit split `frap-4a6-notice`: {split_tests} golden tests, the outcome differing by circuit on both split cases' facts."
        ),
        // bowles_replayed_before_the_overruling_is_late_as_the_sixth_circuit_held
        "- Replay before an overruling (Bowles of Thompson): 1 of 1 reproduces the outcome decided at the time.".to_owned(),
    ] {
        assert!(report.contains(&line), "VALIDATION.md lacks: {line}");
    }
    assert_eq!(pinned.into_iter().collect::<Vec<_>>(), ["D4"]);
}
