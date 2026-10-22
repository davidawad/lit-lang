//! `lit/core`, the standard library: the epic gate (`litigate check
//! --profile strict lit/core`), and its lowered clock IR against the
//! engine's hand-authored `packs/clock` declarations. Every in-force rule
//! must reproduce the shipped rule's outputs (due, base and anchor dates,
//! roll and service flags, refusals) on every trigger date of its window,
//! and its calendars the shipped holidays in every year from 1978. The one
//! deliberate difference is FRAP 26(a)(2)-(3), which `lit/core` takes from
//! the vendored rule text.
// Test code: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use litgraph::clock::{
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

fn shipped(id: &str) -> Clock<'static> {
    ClockLibrary::embedded().0.clock(id).unwrap()
}

fn ours(id: &str) -> Clock<'static> {
    lowered().clock(id).unwrap()
}

fn d(s: &str) -> Date {
    Date::parse_iso(s).unwrap()
}

/// Each rule and the first trigger date its in-force version and the
/// shipped one agree on: the current version's start, and for the ITC rule
/// (one version since 1976) the day Veterans Day returned to November 11.
const RULES: [(&str, &str); 4] = [
    ("frcp6", "2009-12-01"),
    ("rcfc6", "2010-01-11"),
    ("frap26", "2009-12-01"),
    ("itc210", "1978-01-01"),
];

/// The methods a rule's service step lists (both libraries list the same).
fn methods(id: &str) -> &'static [ServiceMethod] {
    use ServiceMethod as M;
    match id {
        "frap26" => &[M::Mail, M::CommercialCarrier, M::PrisonMail],
        "itc210" => &[M::Mail, M::MailForeign, M::Express, M::ExpressForeign],
        _ => &[M::Mail, M::Clerk, M::OtherConsented],
    }
}

/// Everything a computation reports except the trace text, which each
/// library words from its own citations.
fn outcome(r: litgraph::error::Result<ComputedDeadline>) -> Result<[String; 6], String> {
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

#[test]
fn strict_check_of_lit_core_passes_with_no_holes() {
    let run = litigate(&["check", "--profile", "strict", "lit/core"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    for f in result["obligations"].as_array().unwrap() {
        assert_eq!(f["passed"], true, "{f}");
    }
    let core = &result["packages"][0];
    assert_eq!(core["name"], "core");
    assert_eq!(core["holes"], 0);
    assert_eq!(core["tests"]["failed"], 0);
    assert!(core["tests"]["passed"].as_u64().unwrap() >= 100, "{core}");
    assert!(
        run.json["diagnostics"].as_array().unwrap().is_empty(),
        "{}",
        run.stdout
    );
}

#[test]
fn the_fragment_holds_all_four_rules_and_both_calendars() {
    let ids: Vec<&str> = lowered()
        .counting_rules
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(ids, ["frap26", "frcp6", "itc210", "rcfc6"]);
    let calendars: Vec<&str> = lowered().calendars.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(calendars, ["us-federal-holidays", "rcfc-legal-holidays"]);
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
fn frap26_takes_the_vendored_inaccessibility_and_hours_rules() {
    let dl = DayDeadline {
        trigger: d("2026-07-20"),
        days: 14,
        direction: Direction::Forward,
        unit: Unit::Calendar,
        service_method: None,
    };
    let closed = ClockConfig {
        clerk_inaccessible: true,
        ..ClockConfig::default()
    };
    for id in ["frcp6", "rcfc6"] {
        let (a, b) = both(id, &dl, &closed);
        assert_eq!(a, b, "{id}");
    }
    // FRAP 26(a)(3)(A): extended to the next accessible day; the shipped
    // rule ignores the closure.
    let (a, b) = both("frap26", &dl, &closed);
    assert_eq!(a.unwrap()[2], "2026-08-04");
    assert_eq!(b.unwrap()[2], "2026-08-03");
    // FRAP 26(a)(2): hours periods.
    let h = HourDeadline {
        trigger_epoch_seconds: d("2026-07-24").epoch_day() * 86_400 + 12 * 3_600,
        hours: 24,
    };
    let cfg = ClockConfig::default();
    assert!(compute_hour_deadline(&h, &ours("frap26"), &cfg).is_ok());
    assert!(compute_hour_deadline(&h, &shipped("frap26"), &cfg).is_err());
    for id in ["frcp6", "rcfc6"] {
        let a = compute_hour_deadline(&h, &ours(id), &cfg).unwrap();
        let b = compute_hour_deadline(&h, &shipped(id), &cfg).unwrap();
        assert_eq!(
            (a.due_datetime, a.last_day_rolled),
            (b.due_datetime, b.last_day_rolled),
            "{id}"
        );
    }
    assert!(compute_hour_deadline(&h, &ours("itc210"), &cfg).is_err());
}

#[test]
fn additional_holidays_and_inaccessibility_agree_elsewhere() {
    // FRCP 6(a)(6)(C) state holidays arrive as additional closed days; the
    // clerk's office extension (6(a)(3)) under the Rule 6 family.
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
        service in proptest::option::of(0usize..4),
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

/// The event pattern the FRCP 86(a)(2)(B) transition's `unless` arm tests.
fn order_pattern(pkg: &hir::Package) -> hir::Expr {
    let transition = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Transition(t) if t.meta.id == "frcp6-2009" => Some(t),
            _ => None,
        })
        .expect("the 2009 transition");
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

#[test]
fn pending_matters_keep_the_pre_2009_rule_only_on_an_order() {
    let pkg = core();
    let world = World::from_packages(&[pkg]);
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: "frcp6".into(),
    };
    let (event, order) = (served(pkg), order_pattern(pkg));
    // The version governing event `at` of a matter's history.
    let governs = |history: &[(&str, &hir::Expr)], at: usize| {
        let events: Vec<MatterEvent<'_>> = history
            .iter()
            .map(|(date, action)| MatterEvent { date, action })
            .collect();
        let g = governing(&world, &key, &events, at, None).unwrap();
        world.decls[g.version].meta.effective.from.clone()
    };
    // Commenced before December 1, 2009 and pending after it: the 2009 rule
    // governs, unless the court ordered the prior rules kept.
    let pending = [("2009-11-20", &event), ("2009-12-11", &event)];
    assert_eq!(governs(&pending, 1), "2009-12-01");
    let ordered = [
        ("2009-11-20", &event),
        ("2009-12-03", &order),
        ("2009-12-11", &event),
    ];
    assert_eq!(governs(&ordered, 2), "2001-12-01");
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
    assert_eq!(governs(&before, 1), "2001-12-01");
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
