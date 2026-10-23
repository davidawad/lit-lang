//! Docket order (SPEC §5.6, lg-flu.5): events are ordered by date, then
//! `docket` number, never by source order. Same-day events that are not
//! both docketed are unordered and must commute; source must follow
//! docket order; docket numbers grow with the date.
mod cov_case_support;

use cov_case_support::{check, fixture, lock, matter, run, ALPHA_ONLY, STAKES};
use litigate_base::Profile;
use litigate_sema::case::NoClock;
use std::collections::BTreeMap;

fn alpha(events: &[&str]) -> String {
    matter(ALPHA_ONLY, "", events, STAKES)
}

const SERVED: &str = "2010-01-04 docket 1 Served(complaint, Mail) on d";
const ANSWERED: &str = "2010-01-04 docket 2 Answered(complaint) by d";
const FORFEIT_LATE: &str = "2010-01-04 Forfeited(late) by d cite \"FRCP 12(h)\"";
const FORFEIT_REOPEN: &str = "2010-01-04 Forfeited(reopen) by d cite \"FRCP 12(h)\"";

#[test]
fn docket_order_table() {
    let rows: Vec<(&str, Vec<&str>)> = vec![
        (
            "ok: same-day events in docket order",
            vec![SERVED, ANSWERED],
        ),
        (
            "ok: digit separators",
            vec![
                "2010-01-04 docket 1_000 Served(complaint, Mail) on d",
                "2010-01-04 docket 1_001 Answered(complaint) by d",
            ],
        ),
        (
            "out of order: a smaller docket number written later",
            vec![ANSWERED, SERVED],
        ),
        (
            "out of order: dates first, docket numbers then",
            vec![
                "2010-01-08 docket 9 Answered(complaint) by d",
                "2010-01-04 Served(complaint, Mail) on d",
            ],
        ),
        (
            "unordered: same day, undocketed, not commuting",
            vec![
                "2010-01-04 Served(complaint, Mail) on d",
                "2010-01-04 Answered(complaint) by d",
            ],
        ),
        (
            "unordered: one docketed, one not",
            vec![SERVED, "2010-01-04 Answered(complaint) by d"],
        ),
        (
            "unordered: one docket number shared by two events",
            vec![SERVED, "2010-01-04 docket 1 Answered(complaint) by d"],
        ),
        (
            "unordered: a removal beside a move",
            vec!["2010-01-04 Served(complaint, Mail) on d", FORFEIT_LATE],
        ),
        (
            "ok: removals commute undocketed",
            vec![FORFEIT_REOPEN, FORFEIT_LATE],
        ),
        (
            "unordered: a move beside a removal",
            vec![FORFEIT_LATE, "2010-01-04 Served(complaint, Mail) on d"],
        ),
        (
            "ok: docket numbers growing with the dates",
            vec![
                "2010-01-04 docket 5 Served(complaint, Mail) on d",
                "2010-01-08 docket 9 Answered(complaint) by d",
            ],
        ),
        (
            "out of order: consistent docket numbers written backwards",
            vec![
                "2010-01-08 docket 9 Answered(complaint) by d",
                "2010-01-04 docket 5 Served(complaint, Mail) on d",
            ],
        ),
    ];
    snapshot("docket_order_table", &rows);
}

fn snapshot(name: &str, rows: &[(&str, Vec<&str>)]) {
    let got: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, run(&alpha(events), &NoClock, Profile::Dev)))
        .collect();
    insta::assert_debug_snapshot!(name, got);
}

#[test]
fn docket_number_table() {
    let rows: Vec<(&str, Vec<&str>)> = vec![
        (
            "contradiction: a docket number falling with the date",
            vec![
                "2010-01-04 docket 5 Served(complaint, Mail) on d",
                "2010-01-08 docket 3 Answered(complaint) by d",
            ],
        ),
        (
            "contradiction: one docket number on two dates",
            vec![
                "2010-01-04 docket 5 Served(complaint, Mail) on d",
                "2010-01-08 docket 5 Answered(complaint) by d",
            ],
        ),
        (
            "invalid: docket zero",
            vec!["2010-01-04 docket 0 Served(complaint, Mail) on d"],
        ),
        (
            "invalid: a fractional docket",
            vec!["2010-01-04 docket 1.5 Served(complaint, Mail) on d"],
        ),
    ];
    snapshot("docket_number_table", &rows);
}

/// Unordered events that commute replay the same whichever is written
/// first: source order is never consulted.
#[test]
fn commuting_same_day_events_replay_alike_in_any_source_order() {
    let one = run(
        &alpha(&[FORFEIT_REOPEN, FORFEIT_LATE]),
        &NoClock,
        Profile::Dev,
    );
    let two = run(
        &alpha(&[FORFEIT_LATE, FORFEIT_REOPEN]),
        &NoClock,
        Profile::Dev,
    );
    assert_eq!(one, two);
}

#[test]
fn the_docket_number_is_matter_hir() {
    let lock = lock(fixture());
    let checked = check(
        &alpha(&[SERVED, ANSWERED]),
        fixture(),
        Some(&lock),
        &NoClock,
        Profile::Dev,
    );
    let case = checked
        .case
        .unwrap_or_else(|| panic!("{:?}", checked.diagnostics));
    let dockets: Vec<Option<u64>> = case.events.iter().map(|e| e.docket).collect();
    assert_eq!(dockets, [Some(1), Some(2)]);
    let json = serde_json::to_value(&case.events).unwrap_or_default();
    assert_eq!(json[0]["docket"], 1);
    let undocketed = check(
        &alpha(&[FORFEIT_LATE]),
        fixture(),
        Some(&lock),
        &NoClock,
        Profile::Dev,
    );
    let json = serde_json::to_value(undocketed.case.map(|c| c.events)).unwrap_or_default();
    assert!(json[0].get("docket").is_none(), "{json}");
}
