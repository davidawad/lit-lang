//! Counting rules and calendars to the engine clock library (SPEC §8.2):
//! each rule shape the engine runs, as a snapshot, and every lowering
//! error of a counting rule or its calendar, as a diagnostic snapshot.
//! Each case mutates the checked HIR of `lit/examples`.
mod support;

use litigate_lower::{render, Lowerer};
use litigate_sema::hir::{Anchor, CiteRule, DayUnit, HolidayDate, Observance, RollOn, ShortPeriod};
use support::{
    calendar_mut, clock_of, counting_mut, example, lowered, lowering_errors, render_diagnostics,
};

/// The rendered counting rule of `pkg`'s fragment.
fn rule(pkg: &litigate_sema::hir::Package) -> String {
    render(&clock_of(&lowered(pkg)).counting_rules[0]).unwrap_or_default()
}

#[test]
fn holidays_must_name_a_calendar() {
    let mut p = example();
    counting_mut(&mut p).calendar = "respond".into();
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn holidays_must_resolve() {
    let mut p = example();
    counting_mut(&mut p).calendar = "nowhere".into();
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn a_calendar_version_is_named_by_its_start() {
    let mut p = example();
    let from = calendar_mut(&mut p).meta.effective.from.clone();
    counting_mut(&mut p).calendar = format!("federal-holidays@{from}");
    assert_eq!(clock_of(&lowered(&p)).calendars[0].id, "federal-holidays");

    counting_mut(&mut p).calendar = "federal-holidays@1900-01-01".into();
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn an_invalid_engine_calendar_is_rejected() {
    let mut p = example();
    calendar_mut(&mut p).holidays[0].date = HolidayDate::Fixed { month: 13, day: 1 };
    insta::assert_snapshot!(lowering_errors(&p));
}

/// Without `cite_prefixes`, a `[[cite_syntax.rules]]` prefix bound to the
/// source supplies the rule's `sourceUrl`; an unbound rule is skipped.
#[test]
fn a_cite_syntax_rule_binds_the_source_url() {
    let mut p = example();
    p.manifest.sources[0].cite_prefixes.clear();
    p.manifest.cite_syntax.rules = vec![
        CiteRule {
            prefixes: vec!["FRCP".into()],
            heading: "Rule {}".into(),
            source: None,
        },
        CiteRule {
            prefixes: vec!["FRCP".into()],
            heading: "Rule {}".into(),
            source: Some("frcp".into()),
        },
    ];
    let lowered = lowered(&p);
    assert_eq!(
        clock_of(&lowered).counting_rules[0].source_url,
        "https://www.law.cornell.edu/rules/frcp"
    );
}

#[test]
fn a_counting_rule_needs_a_cite() {
    let mut p = example();
    counting_mut(&mut p).meta.cite = None;
    insta::assert_snapshot!(lowering_errors(&p));
}

/// A span-less counting rule still resolves its calendar; an uncited
/// calendar has no source; a labelled one is titled by its label.
#[test]
fn span_less_rules_and_uncited_labelled_calendars() {
    let mut p = example();
    counting_mut(&mut p).meta.span = None;
    let cal = calendar_mut(&mut p);
    cal.meta.cite = None;
    cal.meta.label = "Federal holidays".into();
    let lowered = lowered(&p);
    let calendar = &clock_of(&lowered).calendars[0];
    assert_eq!(
        (calendar.title.as_str(), calendar.cite.as_deref()),
        ("Federal holidays", None)
    );
    assert_eq!(calendar.source_url, None);
}

#[test]
fn a_short_period_counts_court_days() {
    let mut p = example();
    counting_mut(&mut p).short_period = Some(ShortPeriod {
        threshold: 11,
        unit: DayUnit::Court,
    });
    insta::assert_snapshot!(rule(&p));
}

#[test]
fn a_short_period_counts_calendar_days() {
    let mut p = example();
    counting_mut(&mut p).short_period = Some(ShortPeriod {
        threshold: 7,
        unit: DayUnit::Calendar,
    });
    insta::assert_snapshot!(rule(&p));
}

#[test]
fn a_short_period_threshold_must_fit() {
    let mut p = example();
    counting_mut(&mut p).short_period = Some(ShortPeriod {
        threshold: u64::MAX,
        unit: DayUnit::Court,
    });
    insta::assert_snapshot!(lowering_errors(&p));
}

/// `next_business_day` without `exclude_trigger_day` is the engine's
/// adjacent-business-day anchor, with a one-day minimum period.
#[test]
fn a_next_business_day_anchor() {
    let mut p = example();
    let c = counting_mut(&mut p);
    c.anchor = Anchor::NextBusinessDay;
    c.exclude_trigger_day = false;
    insta::assert_snapshot!(rule(&p));
}

/// No roll, no service extension, no inaccessibility, no hours: anchor,
/// count and base only.
#[test]
fn a_bare_rule() {
    let mut p = example();
    let c = counting_mut(&mut p);
    c.roll_on.clear();
    c.service_extensions.clear();
    c.hours = false;
    insta::assert_snapshot!(rule(&p));
}

#[test]
fn a_camel_case_service_method_is_kebab_case() {
    let mut p = example();
    let c = counting_mut(&mut p);
    c.service_extensions.clear();
    c.service_extensions
        .insert("OtherConsented".into(), (3, DayUnit::Calendar));
    c.roll_on = vec![RollOn::Weekend, RollOn::LegalHoliday];
    insta::assert_snapshot!(rule(&p));
}

#[test]
fn service_days_must_fit() {
    let mut p = example();
    counting_mut(&mut p)
        .service_extensions
        .insert("Mail".into(), (u64::MAX, DayUnit::Calendar));
    insta::assert_snapshot!(lowering_errors(&p));
}

/// Every holiday date rule: `last`, `every` (effective from its first
/// year), `once` (one day, widened by observance, clipped by `to`), and an
/// untitled calendar titled by its id.
#[test]
fn holiday_date_rules() {
    let mut p = example();
    let cal = calendar_mut(&mut p);
    cal.meta.label.clear();
    let template = cal.holidays[0].clone();
    let holiday = |id: &str, date: HolidayDate, to: Option<&str>, observed| {
        let mut h = template.clone();
        h.id = id.into();
        h.date = date;
        h.in_force.to = to.map(str::to_owned);
        h.observed = observed;
        h
    };
    cal.holidays = vec![
        holiday(
            "memorial-day",
            HolidayDate::Last {
                weekday: "monday".into(),
                month: 5,
            },
            Some("2100-01-01"),
            None,
        ),
        holiday(
            "inauguration-day",
            HolidayDate::Every {
                years: 4,
                from_year: 1937,
                month: 1,
                day: 20,
            },
            None,
            None,
        ),
        holiday(
            "day-of-mourning",
            HolidayDate::Once {
                date: "2018-12-05".into(),
            },
            None,
            None,
        ),
        holiday(
            "observed-once",
            HolidayDate::Once {
                date: "2019-12-24".into(),
            },
            Some("2019-12-25"),
            Some(Observance::NearestWeekday),
        ),
    ];
    let lowered = lowered(&p);
    insta::assert_snapshot!(render(&clock_of(&lowered).calendars[0]).unwrap_or_default());
}

#[test]
fn holiday_dates_must_be_dates() {
    let mut p = example();
    let cal = calendar_mut(&mut p);
    cal.holidays[0].in_force.from = "the founding".into();
    cal.holidays[1].in_force.to = Some("2020-13-01".into());
    cal.holidays[2].date = HolidayDate::Once {
        date: "someday".into(),
    };
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn holiday_weekdays_must_be_weekdays() {
    let mut p = example();
    let cal = calendar_mut(&mut p);
    cal.holidays[0].date = HolidayDate::Nth {
        n: 1,
        weekday: "funday".into(),
        month: 1,
    };
    cal.holidays[1].date = HolidayDate::Last {
        weekday: "moonday".into(),
        month: 5,
    };
    insta::assert_snapshot!(lowering_errors(&p));
}

/// `clock_version` lowers one counting version at its own `from`: `None`
/// for a version the package does not declare, its diagnostics for one
/// that fails, and a version recorded after the package's corpus date
/// still lowers (selection, not lowering, honours `known_at`).
#[test]
fn clock_version_lowers_one_version() {
    let mut p = example();
    let c = counting_mut(&mut p).clone();
    let one = Lowerer::new(&[&p]).clock_version(&p, &c);
    let rules = one.map(|c| c.map(|c| c.counting_rules.len()));
    assert_eq!(rules, Ok(Some(1)));
    let mut other = c.clone();
    other.meta.id = "rcfc6".into();
    assert_eq!(Lowerer::new(&[&p]).clock_version(&p, &other), Ok(None));

    counting_mut(&mut p).meta.effective.known_at = "2099-01-01".into();
    let late = counting_mut(&mut p).clone();
    let rules = Lowerer::new(&[&p]).clock_version(&p, &late);
    assert_eq!(
        rules.map(|c| c.map(|c| c.counting_rules.len())),
        Ok(Some(1))
    );

    counting_mut(&mut p).include_last_day = false;
    let failing = counting_mut(&mut p).clone();
    let found = Lowerer::new(&[&p])
        .clock_version(&p, &failing)
        .err()
        .unwrap_or_default();
    insta::assert_snapshot!(render_diagnostics(&found));
}
