// SPDX-License-Identifier: GPL-3.0-or-later
//! The shipped clock library (`packs/clock/*.json`) run through the generic
//! interpreter: the known-answer cases that used to live beside the
//! hard-coded `general`/`itc`/`holidays` modules, backward counting under
//! each rule, every holiday's effective-date boundary, and a trace snapshot
//! per rule so a change to any authored step text is visible in review.

#![allow(clippy::unwrap_used, clippy::expect_used)]
// `expect!` snapshots are written as `r#"..."#` by `UPDATE_EXPECT=1`.
#![allow(clippy::needless_raw_string_hashes)]

use expect_test::expect;
use litgraph::clock::{
    compute_due_date, compute_hour_deadline, Clock, ClockConfig, ClockLibrary, ComputedDeadline,
    Date, DayDeadline, Direction, HourDeadline, ServiceMethod, Unit,
};
use proptest::prelude::*;

fn rule(id: &str) -> Clock<'static> {
    ClockLibrary::embedded().0.clock(id).unwrap()
}

fn d(s: &str) -> Date {
    Date::parse_iso(s).unwrap()
}

fn run(
    id: &str,
    trigger: &str,
    days: i64,
    direction: Direction,
    unit: Unit,
    service_method: Option<ServiceMethod>,
) -> litgraph::error::Result<ComputedDeadline> {
    compute_due_date(
        &DayDeadline {
            trigger: d(trigger),
            days,
            direction,
            unit,
            service_method,
        },
        &rule(id),
        &ClockConfig::default(),
    )
}

fn fwd(id: &str, trigger: &str, days: i64) -> ComputedDeadline {
    run(id, trigger, days, Direction::Forward, Unit::Calendar, None).unwrap()
}

fn bwd(id: &str, trigger: &str, days: i64) -> ComputedDeadline {
    run(id, trigger, days, Direction::Backward, Unit::Calendar, None).unwrap()
}

// ---------------------------------------------------------------------------
// Known answers (formerly clock::general / clock::itc unit tests)
// ---------------------------------------------------------------------------

#[test]
fn excludes_trigger_day_and_rolls_past_weekends_and_holidays() {
    assert_eq!(fwd("frcp6", "2026-01-06", 1).due_date, "2026-01-07");
    assert_eq!(fwd("frcp6", "2026-07-18", 7).due_date, "2026-07-27");
    assert_eq!(fwd("frcp6", "2026-01-08", 11).due_date, "2026-01-20"); // MLK
}

#[test]
fn service_extension_follows_the_initial_saturday_roll() {
    // Jan 31 is the unextended last day (Saturday): Rule 6(a) first rolls it
    // to Monday Feb 2, then the three added days end Thursday Feb 5.
    for id in ["frcp6", "rcfc6", "frap26"] {
        let method = if id == "frap26" {
            ServiceMethod::CommercialCarrier
        } else {
            ServiceMethod::Mail
        };
        let result = run(
            id,
            "2026-01-05",
            26,
            Direction::Forward,
            Unit::Calendar,
            Some(method),
        )
        .unwrap();
        assert_eq!(result.base_date, "2026-02-02", "{id}");
        assert_eq!(result.due_date, "2026-02-05", "{id}");
        assert!(result.last_day_rolled);
        assert!(result.service_days_added);
    }
}

#[test]
fn frap_rejects_a_service_method_it_does_not_list() {
    let err = run(
        "frap26",
        "2026-07-20",
        14,
        Direction::Forward,
        Unit::Calendar,
        Some(ServiceMethod::Clerk),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid: service method Clerk is not valid under Fed. R. App. P. 26; valid methods: mail, commercial-carrier, prison-mail"
    );
}

#[test]
fn rcfc_matches_frcp_on_ordinary_mail_service() {
    let mail = |id| {
        run(
            id,
            "2026-07-20",
            14,
            Direction::Forward,
            Unit::Calendar,
            Some(ServiceMethod::Mail),
        )
        .unwrap()
        .due_date
    };
    assert_eq!(mail("frcp6"), "2026-08-06");
    assert_eq!(mail("frcp6"), mail("rcfc6"));
}

#[test]
fn hours_are_rejected_where_the_rule_has_no_hours_provision() {
    let h = HourDeadline {
        trigger_epoch_seconds: 0,
        hours: 24,
    };
    for id in ["frap26", "itc210"] {
        assert!(compute_hour_deadline(&h, &rule(id), &ClockConfig::default()).is_err());
    }
    assert!(compute_hour_deadline(&h, &rule("frcp6"), &ClockConfig::default()).is_ok());
}

#[test]
fn itc_short_period_counts_business_days_from_a_business_day_anchor() {
    // Friday trigger: anchor Monday Jan 5; Mon/Tue/Wed -> Jan 7. The same
    // FRCP period runs Sat/Sun/Mon -> Monday Jan 5.
    let itc = fwd("itc210", "2026-01-02", 3);
    assert_eq!(itc.anchor_date, "2026-01-05");
    assert_eq!(itc.due_date, "2026-01-07");
    assert_eq!(fwd("frcp6", "2026-01-02", 3).due_date, "2026-01-05");
}

#[test]
fn itc_rejects_a_zero_day_period_with_its_own_message() {
    let err = run(
        "itc210",
        "2026-01-01",
        0,
        Direction::Forward,
        Unit::Calendar,
        None,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid: 19 CFR 201.14(a) periods must be at least 1 day"
    );
}

#[test]
fn itc_mail_service_extends_and_frcp_only_methods_are_rejected() {
    let base = fwd("itc210", "2026-06-01", 20);
    let mail = run(
        "itc210",
        "2026-06-01",
        20,
        Direction::Forward,
        Unit::Calendar,
        Some(ServiceMethod::Mail),
    )
    .unwrap();
    assert!(mail.due_date > base.due_date);
    assert!(mail.service_days_added);
    assert!(run(
        "itc210",
        "2026-06-01",
        20,
        Direction::Forward,
        Unit::Calendar,
        Some(ServiceMethod::Clerk)
    )
    .is_err());
}

#[test]
fn negative_periods_are_rejected() {
    assert!(run(
        "frcp6",
        "2026-01-01",
        -1,
        Direction::Forward,
        Unit::Calendar,
        None
    )
    .is_err());
}

// ---------------------------------------------------------------------------
// Backward counting
// ---------------------------------------------------------------------------

#[test]
fn backward_calendar_period_rolls_earlier_under_the_rule_6_family() {
    // 2026-07-27 (Mon) - 2 = Sat 07-25 -> rolls back to Fri 07-24.
    for id in ["frcp6", "rcfc6", "frap26"] {
        let r = bwd(id, "2026-07-27", 2);
        assert_eq!(r.due_date, "2026-07-24", "{id}");
        assert!(r.last_day_rolled);
        assert!(r.steps.iter().any(|s| s.contains("(a)(5)")), "{id}");
    }
}

#[test]
fn backward_itc_period_anchors_on_the_prior_business_day_and_rolls_later() {
    // Anchor: business day before Mon 2026-08-03 = Fri 07-31; day 10
    // counting back is 07-22 (Wed), a business day.
    let r = bwd("itc210", "2026-08-03", 10);
    assert_eq!(r.anchor_date, "2026-07-31");
    assert_eq!(r.due_date, "2026-07-22");
    // A 7-day backward count from Tue 2026-07-28: anchor Mon 07-27, day 7
    // is Tue 07-21.
    assert_eq!(bwd("itc210", "2026-07-28", 7).due_date, "2026-07-21");
    // Landing on a Sunday rolls *later*, unlike FRCP 6(a)(5): anchor Fri
    // 07-31, day 13 back is Sun 07-19 -> Mon 07-20.
    let r = bwd("itc210", "2026-08-03", 13);
    assert_eq!(r.due_date, "2026-07-20");
    assert!(r.last_day_rolled);
}

#[test]
fn backward_periods_ignore_service_extensions_with_a_trace_step() {
    for (id, method) in [
        ("frcp6", ServiceMethod::Mail),
        ("itc210", ServiceMethod::Mail),
    ] {
        let r = run(
            id,
            "2026-07-27",
            10,
            Direction::Backward,
            Unit::Calendar,
            Some(method),
        )
        .unwrap();
        assert!(!r.service_days_added);
        assert!(r.steps.iter().any(|s| s.contains("ignored")), "{id}");
    }
}

#[test]
fn backward_court_days_from_a_closed_day_never_land_after_the_trigger() {
    // A zero-length "before" period from a Sunday rolls back to Friday.
    let r = run(
        "frcp6",
        "2026-07-26",
        0,
        Direction::Backward,
        Unit::Court,
        None,
    )
    .unwrap();
    assert_eq!(r.due_date, "2026-07-24");
    let r = run(
        "frcp6",
        "2026-07-27",
        3,
        Direction::Backward,
        Unit::Court,
        None,
    )
    .unwrap();
    assert_eq!(r.due_date, "2026-07-22");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every backward period, in either unit and under every shipped rule,
    /// ends on an open day; the Rule 6 family never ends after the trigger.
    #[test]
    fn backward_periods_end_on_an_open_day(
        y in 1980i32..2060, m in 1u32..=12, day in 1u32..=28, n in 1i64..120,
        court in any::<bool>(), id in prop_oneof![Just("frcp6"), Just("rcfc6"), Just("frap26"), Just("itc210")],
    ) {
        let trigger = Date::from_ymd(y, m, day).unwrap();
        let unit = if court { Unit::Court } else { Unit::Calendar };
        let r = run(id, &trigger.to_iso(), n, Direction::Backward, unit, None).unwrap();
        let due = d(&r.due_date);
        prop_assert!(!rule(id).is_closed(due, &[]));
        if id != "itc210" {
            prop_assert!(due < trigger);
        }
    }

    /// The observed federal calendar never falls on a weekend and has
    /// eleven holidays in every year since Juneteenth took effect.
    #[test]
    fn federal_holidays_are_observed_on_weekdays(y in 2022i32..2200) {
        let hs = rule("frcp6").holidays_in(y);
        prop_assert_eq!(hs.len(), 11);
        prop_assert!(hs.iter().all(|h| !h.is_weekend()));
    }
}

// ---------------------------------------------------------------------------
// Holiday effective-date boundaries (formerly clock::holidays unit tests)
// ---------------------------------------------------------------------------

#[test]
fn juneteenth_is_a_holiday_from_its_june_17_2021_effective_date() {
    let frcp = rule("frcp6");
    // 2021-06-19 is a Saturday, observed Friday 06-18 (after 06-17).
    assert!(frcp.is_legal_holiday(d("2021-06-18"), &[]));
    assert!(!frcp.is_legal_holiday(d("2021-06-19"), &[]));
    // 2020-06-19 (a Friday) and 2011-06-20 (a would-be observed Monday)
    // predate it.
    assert!(!frcp.is_legal_holiday(d("2020-06-19"), &[]));
    assert!(!rule("itc210").is_legal_holiday(d("2011-06-20"), &[]));
    assert_eq!(frcp.holidays_in(2020).len(), 10);
    assert_eq!(frcp.holidays_in(2021).len(), 11);
    // A deadline landing on the observed day rolls only once it exists.
    assert_eq!(fwd("frcp6", "2021-06-08", 10).due_date, "2021-06-21");
    assert_eq!(fwd("frcp6", "2020-06-09", 10).due_date, "2020-06-19");
}

#[test]
fn mlk_day_is_a_holiday_from_1986() {
    let frcp = rule("frcp6");
    assert!(!frcp.is_legal_holiday(d("1985-01-21"), &[]));
    assert!(frcp.is_legal_holiday(d("1986-01-20"), &[]));
    assert_eq!(frcp.holidays_in(1985).len(), 9);
    assert_eq!(frcp.holidays_in(1986).len(), 10);
}

#[test]
fn inauguration_day_is_an_rcfc_holiday_every_fourth_year_after_1965() {
    let (rcfc, frcp, frap) = (rule("rcfc6"), rule("frcp6"), rule("frap26"));
    // 2029-01-20 is a Saturday, observed Friday 01-19 (2025 would coincide
    // with MLK Day and prove nothing).
    let jan19 = d("2029-01-19");
    assert!(rcfc.is_legal_holiday(jan19, &[]));
    assert!(!frcp.is_legal_holiday(jan19, &[]));
    assert!(!frap.is_legal_holiday(jan19, &[]));
    // First year after the statute: 1969 (a Monday); 1965 predates it.
    assert!(rcfc.is_legal_holiday(d("1969-01-20"), &[]));
    assert!(!rcfc.is_legal_holiday(d("1965-01-20"), &[]));
    // Not a cycle year.
    assert!(!rcfc.is_legal_holiday(d("2026-01-20"), &[]));
    assert!(!rcfc.is_legal_holiday(d("2027-01-20"), &[]));
}

#[test]
fn weekend_observance_spills_across_the_year_boundary() {
    // 2022-01-01 is a Saturday: observed Friday 2021-12-31.
    assert!(rule("frcp6").is_legal_holiday(d("2021-12-31"), &[]));
    // 2021-12-25 is a Saturday: observed Friday 12-24.
    assert!(rule("frcp6").is_legal_holiday(d("2021-12-24"), &[]));
    assert!(!rule("frcp6").is_legal_holiday(d("2021-12-25"), &[]));
}

// ---------------------------------------------------------------------------
// Trace snapshots: one representative computation per shipped rule.
// ---------------------------------------------------------------------------

fn trace(r: &ComputedDeadline) -> String {
    format!(
        "{} anchor={} base={} due={} rolled={} service={} clerk={}\n{}",
        r.rule_set,
        r.anchor_date,
        r.base_date,
        r.due_date,
        r.last_day_rolled,
        r.service_days_added,
        r.clerk_inaccessibility_applied,
        r.steps.join("\n")
    )
}

#[test]
fn trace_snapshots() {
    let clerk = ClockConfig {
        additional_holidays: vec![],
        clerk_inaccessible: true,
    };
    let dl = |days, service_method| DayDeadline {
        trigger: d("2026-01-05"),
        days,
        direction: Direction::Forward,
        unit: Unit::Calendar,
        service_method,
    };
    let frcp = compute_due_date(&dl(30, Some(ServiceMethod::Mail)), &rule("frcp6"), &clerk);
    expect![[r#"
        frcp6 anchor=2026-01-05 base=2026-02-04 due=2026-02-10 rolled=true service=true clerk=true
        FRCP 6(a)(1)(A): exclude the trigger date 2026-01-05
        FRCP 6(a)(1)(B): count 30 calendar day(s) after the trigger, including weekends/holidays -> 2026-02-04
        FRCP 6(d)/(c): service by Mail adds 3 day(s) (after the period expires under the counting rule) -> 2026-02-07
        FRCP 6(a)(1)(C): last day fell on a Saturday, Sunday, or legal holiday; roll forward -> 2026-02-09
        FRCP 6(a)(3): clerk's office inaccessible on the last day; extend -> 2026-02-10"#]]
    .assert_eq(&trace(&frcp.unwrap()));

    let frap = compute_due_date(&dl(30, None), &rule("frap26"), &clerk);
    expect![[r#"
        frap26 anchor=2026-01-05 base=2026-02-04 due=2026-02-04 rolled=false service=false clerk=false
        Fed. R. App. P. 26(a)(1)(A): exclude the trigger date 2026-01-05
        Fed. R. App. P. 26(a)(1)(B): count 30 calendar day(s) after the trigger, including weekends/holidays -> 2026-02-04
        clerk_inaccessible requested but Fed. R. App. P. 26 has no applicable clerk-inaccessibility extension for this direction; ignored"#]]
    .assert_eq(&trace(&frap.unwrap()));

    let itc = compute_due_date(
        &dl(12, Some(ServiceMethod::MailForeign)),
        &rule("itc210"),
        &clerk,
    );
    expect![[r#"
        itc210 anchor=2026-01-06 base=2026-01-20 due=2026-01-30 rolled=true service=true clerk=false
        19 CFR 201.14(a): anchor at the first business day after 2026-01-05 -> 2026-01-06
        19 CFR 201.14(a) general rule (>= 7 days): count 12 calendar day(s) from the anchor -> 2026-01-17
        19 CFR 201.14(a): last day fell on a Saturday, Sunday, or Federal legal holiday; extends until the end of the next business day -> 2026-01-20
        19 CFR 201.16(d)/(e): service by MailForeign adds 10 calendar day(s) -> 2026-01-30"#]]
    .assert_eq(&trace(&itc.unwrap()));

    let hours = compute_hour_deadline(
        &HourDeadline {
            trigger_epoch_seconds: d("2026-01-16").epoch_day() * 86400 + 9 * 3600,
            hours: 72,
        },
        &rule("rcfc6"),
        &ClockConfig::default(),
    )
    .unwrap();
    expect![[r#"
        2026-01-20T09:00:00 rolled=true
        RCFC 6(a)(2)(A): begin counting immediately (the trigger hour counts); +72 hour(s)
        RCFC 6(a)(2)(C): period would end on a Saturday, Sunday, or legal holiday; continue to the same time on 2026-01-20"#]]
    .assert_eq(&format!(
        "{} rolled={}\n{}",
        hours.due_datetime,
        hours.last_day_rolled,
        hours.steps.join("\n")
    ));
}
