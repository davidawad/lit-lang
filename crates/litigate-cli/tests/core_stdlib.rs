//! `lit/core`, the standard library: the epic gate (`litigate check
//! --profile strict lit/core`), and its lowered clock IR, which is the
//! engine's shared clock library (`crates/litigate-engine/clock/`, kept equal to
//! the lowering here), against the hand-authored `packs/clock` declarations
//! it replaced (litgraph-final's, in `tests/litgraph-final/clock/`). Every
//! in-force rule must reproduce the shipped rule's outputs (due, base and
//! anchor dates, roll and service flags, refusals) on every trigger date of
//! its window, for every request the `.lit` surface can express, and its
//! calendars the shipped holidays in every year from 1978. Trace texts are
//! generated from each rule's `cite` (SPEC §8.2), so they are not compared.
// Test code: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use litigate_engine::clock::{
    compute_due_date, compute_hour_deadline, Clock, ClockConfig, ClockLibrary, ComputedDeadline,
    Date, DayDeadline, Direction, HourDeadline, ServiceMethod, Unit,
};
use litigate_lower::{Lowerer, Options};
use litigate_sema::hir::{self, Declaration, ExprKind};
use litigate_sema::temporal::{governing, Key, MatterEvent, World};
use proptest::prelude::*;
use std::sync::OnceLock;
use support::{litigate, repo};

/// The checked HIR of `lit/core`.
fn core() -> &'static hir::Package {
    static CORE: OnceLock<hir::Package> = OnceLock::new();
    CORE.get_or_init(|| {
        let root = repo();
        let (db, ws) = litigate_sema::load(&root, &[root.join("lit/core")], None).unwrap();
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .find_map(|c| c.package.clone().filter(|p| p.manifest.name == "core"))
            .expect("lit/core has HIR")
    })
}

/// `lit/core`'s clock fragment as `litigate build` lowers it.
fn lowered() -> &'static ClockLibrary {
    static LIB: OnceLock<ClockLibrary> = OnceLock::new();
    LIB.get_or_init(|| {
        Lowerer::new(&[core()])
            .package(core(), &Options::default())
            .unwrap_or_else(|d| panic!("{d:#?}"))
            .clock
            .expect("lit/core lowers a clock fragment")
            .0
    })
}

/// litgraph-final's hand-written clock library (`packs/clock/*.json` at tag
/// `litgraph-final`), which `lit/core` replaced.
fn hand() -> &'static ClockLibrary {
    static LIB: OnceLock<ClockLibrary> = OnceLock::new();
    LIB.get_or_init(|| {
        let dir = repo().join("crates/litigate-cli/tests/litgraph-final/clock");
        let files = std::fs::read_dir(&dir).unwrap().map(|e| {
            let path = e.unwrap().path();
            let name = format!("clock/{}", path.file_name().unwrap().to_string_lossy());
            (name, std::fs::read_to_string(&path).unwrap())
        });
        let (lib, errors) = ClockLibrary::merge(files);
        assert!(errors.is_empty(), "{errors:?}");
        lib
    })
}

fn shipped(id: &str) -> Clock<'static> {
    hand().clock(id).unwrap()
}

fn ours(id: &str) -> Clock<'static> {
    lowered().clock(id).unwrap()
}

fn d(s: &str) -> Date {
    Date::parse_iso(s).unwrap()
}

/// Each rule and the first trigger date its in-force version and the
/// shipped one agree on: the current version's start, and for the ITC rule
/// the day Veterans Day returned to November 11. The lowering emits only the
/// version in force at the build date (`itc210@2013-05-20`, whose service
/// days the shipped rule matches) and so does the shipped rule, so this
/// compares that version over every date.
const RULES: [(&str, &str); 6] = [
    ("frcp6", "2016-12-01"),
    ("rcfc6", "2017-08-01"),
    ("frap26", "2016-12-01"),
    ("itc210", "1978-01-01"),
    ("frcrimp45", "2023-12-01"),
    ("sup-ct-30", "2023-01-01"),
];

/// The methods a rule's service step is total over: every one, recognized
/// or not (a refusal must agree too).
fn methods(_id: &str) -> &'static [ServiceMethod] {
    &ServiceMethod::ALL
}

/// Everything a computation reports except the trace text, which each
/// library words from its own citations.
fn outcome(r: litigate_engine::error::Result<ComputedDeadline>) -> Result<[String; 6], String> {
    r.map(|c| {
        [
            c.anchor_date,
            c.base_date,
            c.due_date,
            c.last_day_rolled.to_string(),
            c.service_days_added.to_string(),
            c.clerk_inaccessibility_applied.to_string(),
        ]
    })
    .map_err(|e| e.to_string())
}

fn both(
    id: &str,
    dl: &DayDeadline,
    cfg: &ClockConfig,
) -> (Result<[String; 6], String>, Result<[String; 6], String>) {
    (
        outcome(compute_due_date(dl, &ours(id), cfg)),
        outcome(compute_due_date(dl, &shipped(id), cfg)),
    )
}

fn agree(id: &str, dl: &DayDeadline) -> Result<(), String> {
    let (a, b) = both(id, dl, &ClockConfig::default());
    match (&a, &b) {
        (Ok(x), Ok(y)) if x == y => Ok(()),
        (Err(_), Err(_)) => Ok(()),
        _ => Err(format!("{id} {dl:?}: lit/core {a:?} vs packs/clock {b:?}")),
    }
}

/// Requests the `.lit` surface can produce for `id`: no backward or court
/// periods under the ITC rule (its next-business-day anchor makes a
/// backward deadline L3009, and Part 210 counts no court days).
fn expressible(id: &str, direction: Direction, unit: Unit) -> bool {
    id != "itc210" || (direction == Direction::Forward && unit == Unit::Calendar)
}

/// The federal clock rules are strict-clean; state holiday texts without
/// pinned amendment histories remain explicit strict holes.
#[test]
fn strict_check_of_lit_core_reports_only_unverified_state_sources() {
    let dev = litigate(&["check", "lit/core"]);
    assert_eq!(dev.code, 0, "{}", dev.stdout);
    let run = litigate(&["check", "--profile", "strict", "lit/core"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let diagnostics = run.json["diagnostics"].as_array().unwrap();
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().all(|d| d["code"] == "L5002"));
    let result = &run.json["result"];
    for f in result["obligations"].as_array().unwrap() {
        // A rule package has no events, so only case replay is skipped.
        let skipped = f["name"] == "case-replay" && f["status"] == "skipped";
        let source_holes = f["name"] == "holes" && f["status"] == "failed";
        assert!(f["status"] == "passed" || skipped || source_holes, "{f}");
    }
    let core = &result["packages"][0];
    assert_eq!(core["name"], "core");
    assert_eq!(core["tests"]["failed"], 0);
    assert!(core["tests"]["passed"].as_u64().unwrap() >= 100, "{core}");
    assert_eq!(
        core["holes"].as_u64(),
        Some(u64::try_from(diagnostics.len()).unwrap())
    );
}

/// How a version's cite of a text vendored after it took effect is proved
/// that version's text (SPEC §4.3).
enum Proof {
    /// By the section's history in `sources/rule-amendment-history.txt`.
    History,
    /// By citing an expression of this source binding, dated on or before
    /// the version.
    Dated(&'static str),
}

/// The counting-rule versions whose cites were holes until each was proved
/// (lg-9iz0): the current ones by their amendment histories, the ones
/// before their last amendment by the edition that printed their text.
const PROVED: [(&str, Proof); 9] = [
    ("frcp6@2016-12-01", Proof::Dated("frcp-2017")),
    ("frcp6@2023-12-01", Proof::History),
    ("frap26@2016-12-01", Proof::Dated("frap-2017")),
    ("frap26@2023-12-01", Proof::History),
    ("rcfc6@2024-07-29", Proof::History),
    ("frcrimp45@2023-12-01", Proof::History),
    ("itc210@2013-05-20", Proof::History),
    ("sup-ct-30@2023-01-01", Proof::Dated("scotus-2023")),
    ("sup-ct-30@2026-03-16", Proof::Dated("scotus")),
];

#[test]
fn each_version_cites_a_text_proved_in_force_when_it_took_effect() {
    for (id, proof) in PROVED {
        let run = litigate(&["where", id, "--path", "lit/core"]);
        assert_eq!(run.code, 0, "{id}: {}", run.stdout);
        let from = id.split_once('@').unwrap().1;
        let result = &run.json["result"];
        let version = &result["versions"][0];
        assert_eq!(version["in_force"]["from"], from, "{id}: {version}");
        match proof {
            // A version in force today: its lowered cite carries the proof.
            Proof::History => {
                let sections = result["source"]["sections"].as_array().unwrap();
                assert!(!sections.is_empty(), "{id}: {result}");
                for s in sections {
                    let h = &s["history"];
                    assert_eq!(h["path"], "sources/rule-amendment-history.txt", "{id}: {s}");
                    assert_eq!(h["effective"], from, "{id}: {s}");
                    assert!(h["version"].as_str().unwrap() <= from, "{id}: {s}");
                }
            }
            Proof::Dated(binding) => {
                let b = core()
                    .manifest
                    .sources
                    .iter()
                    .find(|b| b.id == binding)
                    .unwrap_or_else(|| panic!("{binding}"));
                let cite = version["cite"].as_str().unwrap();
                assert!(
                    b.cite_prefixes
                        .iter()
                        .any(|p| cite.starts_with(&format!("{p} "))),
                    "{id}: {cite} through {binding}"
                );
                assert!(b.as_of.as_deref().unwrap() <= from, "{id}: {binding}");
            }
        }
    }
}

/// The FRCP 81(c)(2)(C) and 14(a)(1) periods are the 2009 amendments' 7
/// and 14 days, in force from December 1, 2009, not from the package's
/// 1963 `in_force`; their histories prove neither rule amended since
/// (lg-9iz0).
#[test]
fn the_removal_answer_and_impleader_periods_are_the_2009_texts() {
    for id in ["removal-answer", "third-party-complaint"] {
        let x = core()
            .modules
            .iter()
            .flat_map(|m| &m.declarations)
            .find_map(|d| match d {
                Declaration::Deadline(x) if x.meta.id == id => Some(x),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id}"));
        assert_eq!(x.meta.effective.from, "2009-12-01", "{id}");
        assert_eq!(x.meta.effective.to, None, "{id}");
        assert_eq!(x.meta.effective.known_at, "2009-03-26", "{id}");
    }
}

/// The cites marked `historical`: a later edition, or a committee note,
/// reporting the text of a version or calendar on purpose (lg-9iz0).
#[test]
fn later_texts_cited_for_earlier_versions_are_marked_historical() {
    let historical = |kind: &str, id: &str, from: Option<&str>| {
        core()
            .modules
            .iter()
            .flat_map(|m| &m.declarations)
            .filter_map(|d| match (kind, d) {
                ("counting", Declaration::Counting(x)) => Some(&x.meta),
                ("calendar", Declaration::Calendar(x)) => Some(&x.meta),
                _ => None,
            })
            .find(|m| m.id == id && from.is_none_or(|f| m.effective.from == f))
            .unwrap_or_else(|| panic!("{kind} {id}@{from:?}"))
            .historical
    };
    // The 2008 edition prints FRCP 6 as restyled in 2007, FRAP 26 as
    // amended in 2005; RCFC 6's 2017 Amendment note reports the 2017 text.
    assert!(historical("counting", "frcp6", Some("2005-12-01")));
    assert!(historical("counting", "frap26", Some("2002-12-01")));
    assert!(historical("counting", "rcfc6", Some("2017-08-01")));
    // Today's texts, with the notes that date each holiday's window.
    assert!(historical("calendar", "us-federal-holidays", None));
    assert!(historical("calendar", "rcfc-legal-holidays", None));
}

/// A fixture period is no rule in force: it cites nothing, and the tests
/// using it cite the rule they check (lg-9iz0).
#[test]
fn fixture_periods_cite_no_rule() {
    let rules = [
        "frcp6-",
        "frap26-",
        "rcfc6-",
        "itc210-",
        "frcrimp45-",
        "sup-ct-30-",
    ];
    let fixtures: Vec<&hir::Deadline> = core()
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(|d| match d {
            Declaration::Deadline(x) if rules.iter().any(|r| x.meta.id.starts_with(r)) => Some(x),
            _ => None,
        })
        .filter(|x| x.meta.id != "frcp6-1-year")
        .collect();
    assert!(fixtures.len() >= 40, "{}", fixtures.len());
    for x in fixtures {
        assert_eq!(x.meta.cite, None, "{}", x.meta.id);
    }
}

/// The engine embeds `lit/core` lowered as its shared clock library, the
/// one generated file committed (the engine cannot run the compiler that
/// depends on it); `just lit-sync` rewrites it.
#[test]
fn the_engine_clock_library_is_lit_core_lowered() {
    let committed =
        std::fs::read_to_string(repo().join("crates/litigate-engine/clock/litigate-core.json"))
            .unwrap();
    let fresh = litigate_lower::render(lowered()).unwrap();
    assert!(
        committed == fresh,
        "crates/litigate-engine/clock/litigate-core.json is stale: run `just lit-sync`"
    );
    let (embedded, errors) = ClockLibrary::embedded();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(embedded.counting_rules, lowered().counting_rules);
    assert_eq!(embedded.calendars, lowered().calendars);
}

#[test]
fn the_fragment_holds_every_rule_and_both_calendars() {
    let ids: Vec<&str> = lowered()
        .counting_rules
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    for base in [
        "frap26",
        "frbp9006",
        "frcp6",
        "frcrimp45",
        "itc210",
        "rcfc6",
        "sup-ct-30",
    ] {
        assert!(ids.contains(&base), "missing {base}");
    }
    let states = repo().join("lit/core/states");
    for entry in std::fs::read_dir(states).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("lit") {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        for line in text.lines().filter(|line| line.starts_with("counting ")) {
            let id = line.split_whitespace().nth(1).unwrap();
            assert!(ids.contains(&id), "missing state counting rule {id}");
        }
    }
    let calendars: Vec<&str> = lowered().calendars.iter().map(|c| c.id.as_str()).collect();
    for base in [
        "us-federal-holidays",
        "district-state-holidays",
        "rcfc-legal-holidays",
    ] {
        assert!(calendars.contains(&base), "missing {base}");
    }
    let states = repo().join("lit/core/states");
    for entry in std::fs::read_dir(states).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("lit") {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        for line in text.lines().filter(|line| line.starts_with("calendar ")) {
            let id = line.split_whitespace().nth(1).unwrap();
            assert!(calendars.contains(&id), "missing state calendar {id}");
        }
    }
}

#[test]
fn every_trigger_date_agrees_with_the_shipped_rules() {
    const LENGTHS: [i64; 12] = [0, 1, 2, 3, 6, 7, 10, 11, 14, 21, 30, 63];
    for (id, from) in RULES {
        let mut trigger = d(from);
        let mut i = 0usize;
        while trigger < d("2031-01-01") {
            let days = LENGTHS[i % LENGTHS.len()];
            let service = methods(id)[i % methods(id).len()];
            let other = [
                (Direction::Forward, Unit::Calendar, Some(service)),
                (Direction::Backward, Unit::Calendar, None),
                (Direction::Forward, Unit::Court, None),
                (Direction::Backward, Unit::Court, Some(service)),
            ][i % 4];
            for (direction, unit, service_method) in
                [(Direction::Forward, Unit::Calendar, None), other]
            {
                if expressible(id, direction, unit) {
                    let dl = DayDeadline {
                        trigger,
                        days,
                        direction,
                        unit,
                        service_method,
                    };
                    agree(id, &dl).unwrap();
                }
            }
            trigger = trigger.add_days(1);
            i += 1;
        }
    }
}

#[test]
fn calendars_agree_with_the_shipped_holidays_from_1978() {
    for (id, _) in RULES {
        for year in 1978..=2100 {
            assert_eq!(
                ours(id).holidays_in(year),
                shipped(id).holidays_in(year),
                "{id} {year}"
            );
        }
    }
}

#[test]
fn before_1978_the_calendar_follows_the_statute() {
    // Pub. L. 90-363 moved Veterans Day to the fourth Monday in October for
    // 1971-1977 (Pub. L. 94-97 moved it back); the shipped calendar applies
    // today's dates to every year.
    let (a, b) = (ours("frcp6"), shipped("frcp6"));
    assert!(a.is_legal_holiday(d("1975-10-27"), &[]));
    assert!(!a.is_legal_holiday(d("1975-11-11"), &[]));
    assert!(!b.is_legal_holiday(d("1975-10-27"), &[]));
    assert!(b.is_legal_holiday(d("1975-11-11"), &[]));
}

#[test]
fn inaccessibility_and_hours_agree_with_the_shipped_rules() {
    let closed = ClockConfig {
        clerk_inaccessible: true,
        ..ClockConfig::default()
    };
    // FRCP 6(a)(3)(A) / FRAP 26(a)(3)(A): extended to the next accessible
    // day (Mon Aug 3 -> Tue Aug 4).
    let dl = DayDeadline {
        trigger: d("2026-07-20"),
        days: 14,
        direction: Direction::Forward,
        unit: Unit::Calendar,
        service_method: None,
    };
    for id in ["frcp6", "rcfc6", "frap26"] {
        let (a, b) = both(id, &dl, &closed);
        assert_eq!(a, b, "{id}");
        assert_eq!(a.unwrap()[2], "2026-08-04", "{id}");
    }
    // FRCP 6(a)(2) / FRAP 26(a)(2): 24 hours from Friday noon end Saturday,
    // continue to Monday noon; 26(a)(3)(B) inaccessibility during the last
    // hour, to Tuesday noon.
    let h = HourDeadline {
        trigger_epoch_seconds: d("2026-07-24").epoch_day() * 86_400 + 12 * 3_600,
        hours: 24,
    };
    let open = ClockConfig::default();
    for id in ["frcp6", "rcfc6", "frap26"] {
        for (cfg, due) in [
            (&open, "2026-07-27T12:00:00"),
            (&closed, "2026-07-28T12:00:00"),
        ] {
            let a = compute_hour_deadline(&h, &ours(id), cfg).unwrap();
            assert_eq!(a.due_datetime, due, "{id}");
            // The shipped FRCP/RCFC hours provisions have no 6(a)(3)(B).
            if id == "frap26" {
                let b = compute_hour_deadline(&h, &shipped(id), cfg).unwrap();
                assert_eq!(b.due_datetime, due, "{id}");
                assert_eq!(b.clerk_inaccessibility_applied, cfg.clerk_inaccessible);
            }
        }
    }
    assert!(compute_hour_deadline(&h, &ours("itc210"), &open).is_err());
}

#[test]
fn additional_holidays_and_inaccessibility_agree_elsewhere() {
    // Closed days the caller adds (in either direction); the clerk's office
    // extension (6(a)(3)) under the Rule 6 family.
    let dl = DayDeadline {
        trigger: d("2026-07-27"),
        days: 14,
        direction: Direction::Forward,
        unit: Unit::Calendar,
        service_method: None,
    };
    let state = ClockConfig {
        additional_holidays: vec![d("2026-08-10")],
        clerk_inaccessible: false,
    };
    for (id, _) in RULES {
        let (a, b) = both(id, &dl, &state);
        assert_eq!(a, b, "{id}");
    }
    assert_eq!(both("frcp6", &dl, &state).0.unwrap()[2], "2026-08-11");
    let closed = ClockConfig {
        clerk_inaccessible: true,
        ..ClockConfig::default()
    };
    for id in ["frcp6", "rcfc6", "itc210"] {
        let (a, b) = both(id, &dl, &closed);
        assert_eq!(a, b, "{id}");
    }
}

fn rule_index() -> impl Strategy<Value = usize> {
    0..RULES.len()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Any period, direction, unit and service method agrees with the
    /// shipped rule, over the window both versions are in force.
    #[test]
    fn any_request_agrees_with_the_shipped_rule(
        r in rule_index(),
        offset in 0i64..30_000,
        days in 0i64..400,
        backward in any::<bool>(),
        court in any::<bool>(),
        service in proptest::option::of(0usize..10),
    ) {
        let (id, from) = RULES[r];
        let direction = if backward { Direction::Backward } else { Direction::Forward };
        let unit = if court { Unit::Court } else { Unit::Calendar };
        prop_assume!(expressible(id, direction, unit));
        let dl = DayDeadline {
            trigger: d(from).add_days(offset),
            days,
            direction,
            unit,
            service_method: service.map(|i| methods(id)[i % methods(id).len()]),
        };
        prop_assert!(agree(id, &dl).is_ok(), "{}", agree(id, &dl).unwrap_err());
    }

    /// A due date is never closed, never before a forward trigger, and
    /// never after a backward one (the Rule 6 family).
    #[test]
    fn due_dates_are_open_days_on_the_right_side_of_the_trigger(
        r in rule_index(),
        offset in 0i64..30_000,
        days in 1i64..200,
        backward in any::<bool>(),
    ) {
        let (id, from) = RULES[r];
        let direction = if backward { Direction::Backward } else { Direction::Forward };
        prop_assume!(expressible(id, direction, Unit::Calendar));
        let trigger = d(from).add_days(offset);
        let dl = DayDeadline { trigger, days, direction, unit: Unit::Calendar, service_method: None };
        let due = d(&compute_due_date(&dl, &ours(id), &ClockConfig::default()).unwrap().due_date);
        prop_assert!(!ours(id).is_closed(due, &[]));
        let beyond = if backward { due < trigger } else { due > trigger };
        prop_assert!(beyond);
    }
}

/// The event pattern FRCP 86(a)(2)(B) transition `id`'s `unless` arm
/// tests.
fn order_pattern(pkg: &hir::Package, id: &str) -> hir::Expr {
    let transition = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Transition(t) if t.meta.id == id => Some(t),
            _ => None,
        })
        .expect("the transition");
    match &transition.arms[0]
        .unless
        .as_ref()
        .expect("an unless arm")
        .kind
    {
        ExprKind::Call { args, .. } => args[0].clone(),
        other => panic!("unexpected condition {other:?}"),
    }
}

/// A `Served` event as a test block gives it (any one serves as an event).
fn served(pkg: &hir::Package) -> hir::Expr {
    pkg.modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Test(t) => t.givens.first().map(|(e, _)| e.clone()),
            _ => None,
        })
        .expect("a given")
}

/// The `effective.from` of the FRCP 6 version governing event `at` of a
/// matter's `history` (dated events), under `lit/core`'s transitions.
fn governs(history: &[(&str, &hir::Expr)], at: usize) -> String {
    let pkg = core();
    let world = World::from_packages(&[pkg]);
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: "frcp6".into(),
    };
    let events: Vec<MatterEvent<'_>> = history
        .iter()
        .map(|(date, action)| MatterEvent::on(date, action))
        .collect();
    let g = governing(&world, &key, &events, at, None).unwrap();
    world.decls[g.version].meta.effective.from.clone()
}

#[test]
fn pending_matters_keep_the_pre_2009_rule_only_on_an_order() {
    let pkg = core();
    let (event, order) = (served(pkg), order_pattern(pkg, "frcp6-2009"));
    // Commenced before December 1, 2009 and pending after it: the 2009 rule
    // governs, unless the court ordered the prior rules kept.
    let pending = [("2009-11-20", &event), ("2009-12-11", &event)];
    assert_eq!(governs(&pending, 1), "2009-12-01");
    let ordered = [
        ("2009-11-20", &event),
        ("2009-12-03", &order),
        ("2009-12-11", &event),
    ];
    assert_eq!(governs(&ordered, 2), "2005-12-01");
    // An order after the event it would govern comes too late for it.
    let late = [
        ("2009-11-20", &event),
        ("2009-12-11", &event),
        ("2009-12-15", &order),
    ];
    assert_eq!(governs(&late, 1), "2009-12-01");
    // Commenced after, or over before, the amendments: the version in force.
    let after = [("2009-12-04", &event), ("2009-12-28", &event)];
    assert_eq!(governs(&after, 1), "2009-12-01");
    let before = [("2009-10-01", &event), ("2009-10-21", &event)];
    assert_eq!(governs(&before, 1), "2005-12-01");
}

#[test]
fn pending_matters_keep_electronic_service_days_only_on_an_order() {
    let pkg = core();
    let (event, order) = (served(pkg), order_pattern(pkg, "frcp6-2016"));
    let prior = order_pattern(pkg, "frcp6-2009");
    // Commenced before December 1, 2016 and pending after it: the 2016
    // rule governs, unless the court kept the 2009 text.
    let pending = [("2016-11-21", &event), ("2016-12-12", &event)];
    assert_eq!(governs(&pending, 1), "2016-12-01");
    let ordered = [
        ("2016-11-21", &event),
        ("2016-12-02", &order),
        ("2016-12-12", &event),
    ];
    assert_eq!(governs(&ordered, 2), "2009-12-01");
    // The 2009 transition's order keeps the pre-2009 rules, not the 2009
    // text through the 2016 amendment.
    let other = [
        ("2016-11-21", &event),
        ("2016-12-02", &prior),
        ("2016-12-12", &event),
    ];
    assert_eq!(governs(&other, 2), "2016-12-01");
    let after = [("2016-12-05", &event), ("2016-12-28", &event)];
    assert_eq!(governs(&after, 1), "2016-12-01");
}

#[test]
fn a_backward_period_under_the_itc_rule_is_rejected() {
    let dir = support::scratch("core-itc-backward");
    std::fs::create_dir_all(dir.join("pkg")).unwrap();
    std::fs::write(
        dir.join("pkg/Lit.toml"),
        "[package]\nname = \"itc-backward\"\nversion = \"0.1.0\"\nlanguage = 1\n\
         pack_id = \"itc-backward\"\nforum = \"itc\"\nauthority = \"Regulation\"\n\
         title = \"t\"\nin_force = \"2020-01-01\"\nknown_at = \"2020-01-01\"\n\n\
         [package.roles]\nCommission = \"nature\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("pkg/a.lit"),
        "module itc.backward\n\nimport core.itc210.{ itc210 }\nimport core.time.{ Scheduled }\n\n\
         start done\n\nterminal done: \"Done\"\n\n\
         deadline before-hearing: 3.days before Scheduled() counted_by itc210\n",
    )
    .unwrap();
    let pkg = dir.join("pkg");
    let run = litigate(&["check", "lit/core", pkg.to_str().unwrap()]);
    let codes: Vec<&str> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["code"].as_str())
        .collect();
    assert!(codes.contains(&"L3009"), "{}", run.stdout);
}
