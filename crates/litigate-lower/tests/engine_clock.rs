//! The engine clock behind case replay (SPEC §8.2): the embedded library
//! overlaid with lowered fragments, every version of a package's counting
//! rules, and due dates computed by litgraph's own interpreter.
mod support;

use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{Anchor, DayUnit, Direction, RollOn};
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
        .insert("Mail".into(), (5, DayUnit::Calendar));
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
fn service_methods_the_rule_recognizes_extend_the_period() {
    let clock = EngineClock::new([]);
    let mut r = request("frcp6", None);
    r.service = vec!["personal".into(), "mail".into()];
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-27"));
    r.service = vec!["personal".into(), "carrier-pigeon".into()];
    assert_eq!(clock.due(&r).as_deref(), Ok("2025-03-24"));
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
    r.unit = DayUnit::Business;
    assert_eq!(clock.due(&r), court);
    assert_eq!(court.as_deref(), Ok("2025-03-10"));
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
