//! The engine clock behind case replay (SPEC §8.2): the embedded library
//! overlaid with lowered fragments, every version of a package's counting
//! rules, and due dates computed by litgraph's own interpreter.
mod support;

use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{self, Anchor, DayUnit, Direction, RollOn, ServiceTime};
use support::{counting_mut, example, lowered, render_diagnostics};

/// A forward 21-calendar-day request under `counting`.
fn request(counting: &str, version: Option<&str>) -> DueRequest {
    DueRequest {
        counting: counting.into(),
        version: version.map(str::to_owned),
        trigger: "2025-03-03".into(),
        amount: 21,
        unit: DayUnit::Calendar,
        direction: Direction::Forward,
        service: Vec::new(),
    }
}

fn from_of(pkg: &mut litigate_sema::hir::Package) -> String {
    counting_mut(pkg).meta.effective.from.clone()
}

#[test]
fn every_counting_version_is_lowered_at_its_start() {
    let mut p = example();
    let from = from_of(&mut p);
    // This package's version adds five days for mail; the library's adds
    // three.
    counting_mut(&mut p)
        .service_extensions
        .insert("Mail".into(), ServiceTime::Adds(5, DayUnit::Calendar));
    // A links package is skipped: its counting rule, which does not lower,
    // is never asked for.
    let mut links = example();
    links.manifest.links = true;
    links.manifest.pack_id = "links".into();
    counting_mut(&mut links).roll_on = vec![RollOn::Weekend];
    let packages = [&p, &links];
    let clock = EngineClock::lowered(&Lowerer::new(&packages), &packages)
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    let mailed = |version: Option<&str>| {
        let mut r = request("frcp6", version);
        r.service = vec!["mail".into()];
        clock.due(&r)
    };
    // 2025-03-29 is a Saturday: the last day rolls to Monday.
    assert_eq!(mailed(Some(&from)).as_deref(), Ok("2025-03-31"));
    assert_eq!(mailed(None).as_deref(), Ok("2025-03-27"));
    // A version no package declares falls back to the library rule.
    assert_eq!(mailed(Some("1900-01-01")).as_deref(), Ok("2025-03-27"));
}

#[test]
fn a_counting_version_that_does_not_lower_is_an_error() {
    let mut p = example();
    counting_mut(&mut p).roll_on = vec![RollOn::Weekend];
    let found = EngineClock::lowered(&Lowerer::new(&[&p]), &[&p])
        .err()
        .unwrap_or_default();
    insta::assert_snapshot!(render_diagnostics(&found));
}

/// A version recorded after the package's corpus date still has its
/// fragment: whether a request names it is the selection's `known_at`
/// answer (`temporal::select`), never the clock's.
#[test]
fn a_version_recorded_late_still_lowers() {
    let mut p = example();
    counting_mut(&mut p).meta.effective.known_at = "2099-01-01".into();
    let from = from_of(&mut p);
    let clock = EngineClock::lowered(&Lowerer::new(&[&p]), &[&p])
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    assert_ne!(clock, EngineClock::new([]));
    assert_eq!(
        clock.due(&request("frcp6", Some(&from))).as_deref(),
        Ok("2025-03-24")
    );
}

/// A lowered fragment replaces the library's rule and calendar of the
/// same id; `of` takes the fragments of lowered packages.
#[test]
fn fragments_overlay_the_library() {
    let mut p = example();
    let c = counting_mut(&mut p);
    c.anchor = Anchor::NextBusinessDay;
    c.exclude_trigger_day = false;
    let ours = lowered(&p);
    let clock = EngineClock::of(std::slice::from_ref(&ours));
    let mut zero = request("frcp6", None);
    zero.amount = 0;
    // The overlaid rule has a one-day minimum; the library's has none.
    insta::assert_snapshot!(clock.due(&zero).err().unwrap_or_default());
    assert!(EngineClock::new([]).due(&zero).is_ok());
    assert_ne!(clock, EngineClock::new([]));
}

#[test]
fn the_trigger_s_service_method_extends_the_period_or_is_refused() {
    let clock = EngineClock::new([]);
    let mut r = request("frcp6", None);
    r.service = vec!["complaint".into(), "mail".into()];
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-27"));
    r.service = vec!["personal".into(), "mail".into()];
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-24"));
    r.service = vec!["carrier-pigeon".into()];
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-24"));
    // FRCP 6(d) names no prison mailing system: the period is not counted.
    r.service = vec!["prison-mail".into()];
    assert!(clock
        .due(&r)
        .is_err_and(|e| e.contains("PrisonMail is not valid")));
}

#[test]
fn units_and_directions() {
    let clock = EngineClock::new([]);
    let mut r = request("frcp6", None);
    r.direction = Direction::Backward;
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-02-10"));
    r.direction = Direction::Forward;
    r.amount = 5;
    r.unit = DayUnit::Court;
    let court = clock.due(&r);
    assert_eq!(court.as_deref(), Ok("2025-03-10"));
    // LW3: business days are refused here as in edge lowering and the CLI,
    // never counted as court days.
    r.unit = DayUnit::Business;
    assert!(clock
        .due(&r)
        .is_err_and(|e| e.contains("business days has no engine lowering")));
}

#[test]
fn requests_the_engine_cannot_count() {
    let clock = EngineClock::new([]);
    let unknown = clock.due(&request("no-such-rule", None));
    let mut bad_date = request("frcp6", None);
    bad_date.trigger = "March 3".into();
    let mut too_long = request("frcp6", None);
    too_long.amount = u64::MAX;
    // The unknown rule's message ends with the library's rule list, which
    // grows with `packs/clock`; its head is the finding.
    let unknown = unknown.map_err(|e| e.split(';').next().unwrap_or_default().to_owned());
    let text = [unknown, clock.due(&bad_date)]
        .map(|r| r.err().unwrap_or_default())
        .join("\n");
    insta::assert_snapshot!(text);
    // The standard library words this one differently across toolchains.
    assert!(clock.due(&too_long).is_err());
}

/// Two versions of a rule in one window select neither at its start, so
/// neither contributes a fragment.
#[test]
fn an_ambiguous_version_contributes_no_fragment() {
    let mut p = example();
    let twin = hir::Declaration::Counting(counting_mut(&mut p).clone());
    p.modules[0].declarations.push(twin);
    let clock = EngineClock::lowered(&Lowerer::new(&[&p]), &[&p])
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    assert_eq!(clock, EngineClock::new([]));
}

/// Weeks are seven calendar days; months and years count under the
/// package rule's `end_of_month` policy, and a rule without one refuses.
#[test]
fn weeks_months_and_years_are_counted() {
    let mut p = example();
    let from = from_of(&mut p);
    counting_mut(&mut p).end_of_month = Some(litigate_sema::hir::EndOfMonth::Clamp);
    let clock = EngineClock::lowered(&Lowerer::new(&[&p]), &[&p])
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    let due = |amount, unit, trigger: &str| {
        let mut r = request("frcp6", Some(&from));
        (r.amount, r.unit, r.trigger) = (amount, unit, trigger.to_owned());
        clock.due(&r)
    };
    assert_eq!(
        due(3, DayUnit::Week, "2025-03-03").as_deref(),
        Ok("2025-03-24")
    );
    // November 30 plus three months: February has no 30th.
    assert_eq!(
        due(3, DayUnit::Month, "2024-11-30").as_deref(),
        Ok("2025-02-28")
    );
    assert_eq!(
        due(1, DayUnit::Year, "2025-03-04").as_deref(),
        Ok("2026-03-04")
    );
    // The library's own `frcp6` carries a clamp provision too.
    let mut r = request("frcp6", None);
    (r.amount, r.unit) = (1, DayUnit::Month);
    assert_eq!(EngineClock::new([]).due(&r).as_deref(), Ok("2025-04-03"));
}

/// A version whose calendar closes every day would roll forever: replay is
/// refused with the reason (`cycle::endless_roll`) instead of counting.
#[test]
fn a_version_that_never_rolls_is_refused_not_counted() {
    let mut p = example();
    let from = from_of(&mut p);
    let calendar = p
        .modules
        .iter_mut()
        .flat_map(|m| &mut m.declarations)
        .find_map(|d| match d {
            hir::Declaration::Calendar(c) => Some(c),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the example has a calendar"));
    let template = calendar.holidays[0].clone();
    calendar.holidays = (1..=12u8)
        .flat_map(|month| {
            let last = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][usize::from(month) - 1];
            (1..=last).map(move |day| (month, day))
        })
        .map(|(month, day)| hir::Holiday {
            id: format!("d{month}-{day}"),
            date: hir::HolidayDate::Fixed { month, day },
            observed: None,
            ..template.clone()
        })
        .collect();
    let clock = EngineClock::lowered(&Lowerer::new(&[&p]), &[&p])
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    let refused = clock.due(&request("frcp6", Some(&from)));
    assert!(
        refused
            .as_ref()
            .is_err_and(|e| e.starts_with("counting rule `frcp6` never rolls to an open day from ")),
        "{refused:?}"
    );
    // The library's own rule, which closes on federal holidays, still counts.
    assert_eq!(
        clock.due(&request("frcp6", None)).as_deref(),
        Ok("2025-03-24")
    );
}

/// The nominal end is the last day as counted, before the roll: 23 days
/// from Thursday 2025-03-06 is Saturday 2025-03-29, rolled to Monday
/// 2025-03-31.
#[test]
fn the_nominal_end_is_before_the_roll() {
    let clock = EngineClock::new([]);
    let mut r = request("frcp6", None);
    r.trigger = "2025-03-06".into();
    r.amount = 23;
    assert_eq!(clock.nominal(&r).as_deref(), Ok("2025-03-29"));
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-31"));
}
