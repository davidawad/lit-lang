//! Golden lowering of the SPEC §12 worked example (`lit/examples`): the
//! pack, its clock fragment and source maps are insta snapshots, the
//! output is byte-stable, and the lowered counting rule computes the
//! example's test dates through the engine's own interpreter.
mod support;

use litgraph::cite::SourceCorpus;
use litgraph::clock::{
    compute_due_date, ClockConfig, Date, DayDeadline, Direction, ServiceMethod, Unit,
};
use litigate_lower::{render, Document, LoweredPackage, Lowerer, Options};
use support::load_hir;

fn lowered() -> LoweredPackage {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<&litigate_sema::hir::Package> = packages.iter().collect();
    Lowerer::new(&refs)
        .with_corpus(SourceCorpus::embedded_ref())
        .package(&packages[0], &Options::default())
        .unwrap_or_else(|d| panic!("{}", serde_json::to_string_pretty(&d).unwrap_or_default()))
}

#[test]
fn example_pack_snapshot() {
    let lowered = lowered();
    let Document::Pack(pack) = &lowered.document else {
        panic!("expected a pack");
    };
    let text = render(pack).unwrap_or_default();
    insta::assert_snapshot!("example_pack", text);
    let reread = litgraph::Pack::from_json(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(&reread, pack.as_ref());
    insta::assert_snapshot!(
        "example_pack_map",
        render(&lowered.source_map).unwrap_or_default()
    );
}

#[test]
fn example_clock_snapshot() {
    let lowered = lowered();
    let (clock, map) = lowered.clock.as_ref().unwrap_or_else(|| panic!("no clock"));
    insta::assert_snapshot!("example_clock", render(clock).unwrap_or_default());
    insta::assert_snapshot!("example_clock_map", render(map).unwrap_or_default());
}

#[test]
fn output_is_byte_stable() {
    let (a, b) = (lowered(), lowered());
    assert_eq!(a, b);
    let Document::Pack(pack) = &a.document else {
        panic!("expected a pack");
    };
    assert_eq!(render(pack).ok(), render(pack).ok());
}

fn due(trigger: &str, days: i64, method: Option<ServiceMethod>) -> String {
    let lowered = lowered();
    let (library, _) = lowered.clock.unwrap_or_else(|| panic!("no clock"));
    let clock = library.clock("frcp6").unwrap_or_else(|e| panic!("{e}"));
    let deadline = DayDeadline {
        trigger: Date::parse_iso(trigger).unwrap_or_else(|e| panic!("{e}")),
        days,
        direction: Direction::Forward,
        unit: Unit::Calendar,
        service_method: method,
    };
    compute_due_date(&deadline, &clock, &ClockConfig::default())
        .unwrap_or_else(|e| panic!("{e}"))
        .due_date
}

#[test]
fn lowered_counting_computes_the_example_tests() {
    // rule12.lit: "answer due 21 days after personal service".
    assert_eq!(due("2025-03-03", 21, None), "2025-03-24");
    // rule12.lit: "mail service adds three days".
    assert_eq!(
        due("2025-03-03", 21, Some(ServiceMethod::Mail)),
        "2025-03-27"
    );
    // acme.case: served 2025-03-05, answer due 2025-03-26 (SPEC §12).
    assert_eq!(due("2025-03-05", 21, None), "2025-03-26");
    // A last day on Christmas 2025 (Thursday) rolls to Friday.
    assert_eq!(due("2025-12-04", 21, None), "2025-12-26");
}

proptest::proptest! {
    /// The lowered rule counts `n` days after the trigger and moves only a
    /// closed last day, and only forward to the next open day.
    #[test]
    fn lowered_counting_rolls_only_closed_last_days(offset in 0i64..4000, n in 1i64..90) {
        let lowered = lowered();
        let (library, _) = lowered.clock.unwrap_or_else(|| panic!("no clock"));
        let clock = library.clock("frcp6").unwrap_or_else(|e| panic!("{e}"));
        let trigger = Date::parse_iso("2018-01-01").unwrap_or_else(|e| panic!("{e}")).add_days(offset);
        let deadline = DayDeadline {
            trigger,
            days: n,
            direction: Direction::Forward,
            unit: Unit::Calendar,
            service_method: None,
        };
        let due = compute_due_date(&deadline, &clock, &ClockConfig::default())
            .unwrap_or_else(|e| panic!("{e}"))
            .due_date;
        let due = Date::parse_iso(&due).unwrap_or_else(|e| panic!("{e}"));
        let last = trigger.add_days(n);
        proptest::prop_assert!(!clock.is_closed(due, &[]));
        proptest::prop_assert!(due >= last);
        let between_closed = (0..due.epoch_day() - last.epoch_day())
            .all(|k| clock.is_closed(last.add_days(k), &[]));
        proptest::prop_assert!(between_closed);
    }
}
