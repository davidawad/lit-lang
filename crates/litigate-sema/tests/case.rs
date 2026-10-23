//! `.case` checking (SPEC §5.6): replay success and failure tables over
//! the SPEC §12 worked example and the FRCP 2009 amendment fixture,
//! deadlines counted by the engine clock over the lowered counting rules.
mod case_support;

use case_support::{
    acme, check_2009, check_example, check_with, examples, examples_clock, lock, matter_2009,
    render,
};
use litigate_base::Profile;
use litigate_lower::Lowerer;
use litigate_sema::hir::Package;
use std::collections::BTreeMap;

/// `acme.case` with `edits` applied (`(from, to)`, each must occur).
fn edited(edits: &[(&str, &str)]) -> String {
    edits.iter().fold(acme(), |text, (from, to)| {
        assert!(text.contains(from), "acme.case has no {from:?}");
        text.replacen(from, to, 1)
    })
}

fn row(text: &str, profile: Profile) -> Vec<String> {
    let checked = check_example(text, profile);
    let mut out = render(&checked.diagnostics);
    if checked.ok() {
        let start = checked.case.and_then(|c| c.start).unwrap_or_default();
        out.push(format!("ok: start {start}"));
    }
    out
}

#[test]
fn the_replay_trace_ends_where_the_posture_does() {
    let checked = check_example(&acme(), Profile::Dev);
    insta::assert_json_snapshot!("acme_replay_trace", checked.trace);
    let last = checked.trace.last().map(|s| s.to.as_str());
    assert_eq!(last, checked.case.and_then(|c| c.start).as_deref());
    let illegal = check_example(
        &edited(&[(
            "2025-03-20 MovedToDismiss(complaint) by widget",
            "2025-03-20 Answered(complaint) by acme",
        )]),
        Profile::Dev,
    );
    assert!(!illegal.ok());
    assert!(illegal.trace.len() < checked.trace.len());
}

#[test]
fn a_terminal_whose_pack_authors_a_payoff_needs_no_stake() {
    let stakes = "stakes { judgment-entered: -2_000_000.usd, dismissed: 0.usd }";
    let authored = check_example(
        &edited(&[(stakes, "stakes { judgment-entered: -2_000_000.usd }")]),
        Profile::Dev,
    );
    assert!(authored.ok(), "{:#?}", render(&authored.diagnostics));
    let hole = check_example(
        &edited(&[(stakes, "stakes { dismissed: 0.usd }")]),
        Profile::Dev,
    );
    let codes: Vec<&str> = hole.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["L4004"], "{:#?}", render(&hole.diagnostics));
}

#[test]
fn an_instance_ref_names_its_base_packs_node() {
    let stakes = "dismissed: 0.usd }";
    let instance = check_example(
        &edited(&[(
            stakes,
            "dismissed: 0.usd, frcp-rule12-example@appeal::dismissed: 1.usd }",
        )]),
        Profile::Dev,
    );
    assert!(instance.ok(), "{:#?}", render(&instance.diagnostics));
    let unknown = check_example(
        &edited(&[(stakes, "dismissed: 0.usd, nope@appeal::dismissed: 1.usd }")]),
        Profile::Dev,
    );
    assert!(!unknown.ok());
}

#[test]
fn the_worked_example_replays_to_the_motion_ruling() {
    let checked = check_example(&acme(), Profile::Strict);
    assert!(checked.ok(), "{:#?}", render(&checked.diagnostics));
    let case = checked.case.unwrap_or_else(|| panic!("resolved"));
    assert_eq!(
        case.start.as_deref(),
        Some("frcp-rule12-example::mtd-ruling")
    );
    insta::assert_json_snapshot!("acme_case_hir", case);
    let refs: Vec<&Package> = examples().iter().collect();
    let lowered = Lowerer::new(&refs)
        .case(&case, "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    assert_eq!(
        lowered.scenario.scenario.start.as_deref(),
        Some("frcp-rule12-example::mtd-ruling")
    );
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn example_replay_table() {
    let served = "2025-03-05 Served(complaint, Personal) on widget";
    let moved = "2025-03-20 MovedToDismiss(complaint) by widget";
    let rows: Vec<(&str, String, Profile)> = vec![
        ("ok: the worked example", acme(), Profile::Dev),
        (
            "ok: mail service adds three days and rolls off the weekend",
            edited(&[
                ("Served(complaint, Personal)", "Served(complaint, Mail)"),
                ("2025-03-20 MovedToDismiss", "2025-03-31 MovedToDismiss"),
            ]),
            Profile::Dev,
        ),
        (
            "ok: answering instead leaves only judgment reachable",
            edited(&[(moved, "2025-03-26 Answered(complaint) by widget")]),
            Profile::Dev,
        ),
        (
            "ok: no events, replay stays at the package start",
            edited(&[(served, ""), (moved, "")]),
            Profile::Dev,
        ),
        (
            "missed: one day after the answer period",
            edited(&[("2025-03-20 MovedToDismiss", "2025-03-27 MovedToDismiss")]),
            Profile::Dev,
        ),
        (
            "missed: mail service, one day after the rolled due date",
            edited(&[
                ("Served(complaint, Personal)", "Served(complaint, Mail)"),
                ("2025-03-20 MovedToDismiss", "2025-04-01 MovedToDismiss"),
            ]),
            Profile::Dev,
        ),
        (
            "order: events out of date order",
            edited(&[(served, "@@"), (moved, served), ("@@", moved)]),
            Profile::Dev,
        ),
        (
            "typestate: nothing moves a pending ruling",
            edited(&[(
                moved,
                &format!("{moved}\n  2025-03-25 Answered(complaint) by widget"),
            )]),
            Profile::Dev,
        ),
        (
            "typestate: the plaintiff cannot answer",
            edited(&[(moved, "2025-03-20 Answered(complaint) by acme")]),
            Profile::Dev,
        ),
        (
            "typestate: an event after as_of",
            edited(&[("2025-03-20 MovedToDismiss", "2025-07-01 MovedToDismiss")]),
            Profile::Dev,
        ),
        (
            "events: unknown constructor",
            edited(&[("MovedToDismiss(complaint)", "MovedToDismis(complaint)")]),
            Profile::Dev,
        ),
        (
            "events: incomplete arguments",
            edited(&[("Served(complaint, Personal)", "Served(complaint)")]),
            Profile::Dev,
        ),
        (
            "events: not a variant",
            edited(&[("Served(complaint, Personal)", "Served(complaint, Fax)")]),
            Profile::Dev,
        ),
        (
            "parties: undeclared actor",
            edited(&[("acme: Plaintiff", "acme: Plantiff")]),
            Profile::Dev,
        ),
        (
            "parties: undeclared event party",
            edited(&[("by widget", "by wigdet")]),
            Profile::Dev,
        ),
        (
            "parties: duplicate party",
            edited(&[(
                "widget: Defendant Corporation",
                "widget: Defendant Corporation\n  widget: Defendant Corporation",
            )]),
            Profile::Dev,
        ),
        (
            "facts: not the node's own edge",
            edited(&[("time-barred: not-barred", "time-barred: granted")]),
            Profile::Dev,
        ),
        (
            "facts: not a fact node",
            edited(&[("time-barred: not-barred", "mtd-ruling: granted")]),
            Profile::Dev,
        ),
        (
            "stakes: a reachable terminal without a stake",
            edited(&[("judgment-entered: -2_000_000.usd, ", "")]),
            Profile::Dev,
        ),
        (
            "stakes: unvalued is fine in dev",
            edited(&[("dismissed: 0.usd", "dismissed: unvalued")]),
            Profile::Dev,
        ),
        (
            "stakes: unvalued is an error in strict",
            edited(&[("dismissed: 0.usd", "dismissed: unvalued")]),
            Profile::Strict,
        ),
        (
            "stakes: only terminals take stakes",
            edited(&[("dismissed: 0.usd", "dismissed: 0.usd, respond: 5.usd")]),
            Profile::Dev,
        ),
        (
            "beliefs: not a chance node",
            edited(&[("observe mtd-ruling", "observe respond")]),
            Profile::Dev,
        ),
        (
            "beliefs: not the node's own outcome",
            edited(&[("granted: 3", "barred: 3")]),
            Profile::Dev,
        ),
        (
            "start: an explicit start the events move past",
            edited(&[("perspective widget", "start respond\n\nperspective widget")]),
            Profile::Dev,
        ),
        (
            "start: an explicit start that agrees",
            edited(&[
                (
                    "perspective widget",
                    "start mtd-ruling\n\nperspective widget",
                ),
                (served, ""),
                (moved, ""),
            ]),
            Profile::Dev,
        ),
        (
            "names: unknown node",
            edited(&[("observe mtd-ruling", "observe mtd-ruing")]),
            Profile::Dev,
        ),
        (
            "uses: requirement the lock does not satisfy",
            edited(&[("^0.1.0", "^0.2.0")]),
            Profile::Dev,
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, text, profile)| (*name, row(text, *profile)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn bowles_order_cannot_extend_the_statutory_appeal_limit() {
    // Source: sources/change-corpus/rulings/MANIFEST.json, entry
    // `bowles-scotus-2007`, and its vendored text. The order was entered
    // 2004-02-10; Rule 4(a)(6)/28 U.S.C. § 2107(c) allowed 14 days, through
    // 2004-02-24. The district court's 17-day date was 2004-02-27; filing
    // on 2004-02-26 was held untimely by the Supreme Court.
    let root = case_support::repo_root();
    let manifest =
        std::fs::read_to_string(root.join("sources/change-corpus/rulings/MANIFEST.json"))
            .expect("rulings manifest");
    let ruling =
        std::fs::read_to_string(root.join("sources/change-corpus/rulings/bowles-scotus-2007.txt"))
            .expect("Bowles opinion text");
    assert!(manifest.contains("\"id\": \"bowles-scotus-2007\""));
    assert!(manifest.contains("\"file\": \"bowles-scotus-2007.pdf\""));
    assert!(ruling.contains("notice of appeal on February 26"));
    assert!(ruling.contains("deadline had actually been February 24"));
    assert!(ruling.contains("deprived the Sixth Circuit of jurisdiction"));

    // Replay the pattern on the fixture's answer period: served
    // 2010-04-01, due 2010-04-22 (21 days); the court may extend it by at
    // most 30 days (to 2010-05-22) but orders 2010-05-25, and the filing
    // falls between the lawful and the ordered dates. The order exceeds
    // its capability's bound and is rejected, so the filing stays missed.
    let checked = check_2009(&matter_2009(&[
        "2010-04-01 Served(complaint, Personal) on widget",
        "2010-04-20 AnswerExtended(2010-05-25) by court cite FRCP 6(b)(1)(A)",
        "2010-05-24 MovedToDismiss(complaint) by widget",
    ]));
    let rendered = render(&checked.diagnostics);
    assert!(
        rendered
            .iter()
            .any(|d| d.contains("L4001") && d.contains("exceeds the bound")),
        "the overlong order must be rejected: {rendered:#?}"
    );
    assert!(
        rendered.iter().any(|d| d.contains("L4002")),
        "the Bowles filing must remain untimely: {rendered:#?}"
    );
    // The same order within the bound makes the filing timely.
    let lawful = check_2009(&matter_2009(&[
        "2010-04-01 Served(complaint, Personal) on widget",
        "2010-04-20 AnswerExtended(2010-05-21) by court cite FRCP 6(b)(1)(A)",
        "2010-05-21 MovedToDismiss(complaint) by widget",
    ]));
    assert!(lawful.ok(), "{:#?}", render(&lawful.diagnostics));
}

#[test]
fn pins_must_match_the_lock() {
    let mut stale = lock(examples());
    stale.packages[0].digest = format!("sha256:{}", "0".repeat(64));
    let table: BTreeMap<&str, Vec<String>> = [("no lock", None), ("stale digest", Some(&stale))]
        .into_iter()
        .map(|(name, lock)| {
            let checked = check_with(&acme(), examples(), lock, examples_clock(), Profile::Dev);
            assert!(!checked.ok());
            (name, render(&checked.diagnostics))
        })
        .collect();
    insta::assert_debug_snapshot!(table);
}

fn row_2009(events: &[&str]) -> Vec<String> {
    let checked = check_2009(&matter_2009(events));
    let mut out = render(&checked.diagnostics);
    if checked.ok() {
        let start = checked.case.and_then(|c| c.start).unwrap_or_default();
        out.push(format!("ok: start {start}"));
    }
    out
}

/// A matter straddling the December 1, 2009 amendments: FRCP 86(a)(2)
/// transitions pick the version for each event, and the counting rule's
/// own versions count each period.
#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn amendment_replay_table() {
    let served = "2009-11-20 Served(complaint, Personal) on widget";
    let rows: Vec<(&str, Vec<&str>)> = vec![
        (
            "pending: the amended 21 days govern (due 2009-12-11)",
            vec![served, "2009-12-11 MovedToDismiss(complaint) by widget"],
        ),
        (
            "pending: an order keeps the prior 20 days (due 2009-12-10), missed",
            vec![
                served,
                "2009-11-25 OrderedPriorRules() by court",
                "2009-12-11 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "pending: the prior rules, extended by a cited order within its bound",
            vec![
                served,
                "2009-11-25 OrderedPriorRules() by court",
                "2009-12-09 AnswerExtended(2009-12-14) by court cite FRCP 6(b)(1)(A)",
                "2009-12-11 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "pending: an uncited extension is not an order",
            vec![
                served,
                "2009-11-25 OrderedPriorRules() by court",
                "2009-12-09 AnswerExtended(2009-12-14) by court",
                "2009-12-11 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "pending: an extension before the trigger has nothing to extend",
            vec![
                "2009-11-18 AnswerExtended(2009-12-14) by court cite FRCP 6(b)(1)(A)",
                served,
                "2009-11-25 OrderedPriorRules() by court",
                "2009-12-11 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "pending: an event named Extended declares no effect and is illegal",
            vec![
                served,
                "2009-12-09 Extended(complaint) by court cite FRCP 6(b)(1)(A)",
            ],
        ),
        (
            "before: 2007 counting skips weekends and Veterans Day after denial (due 2009-11-23)",
            vec![
                "2009-10-01 Served(complaint, Personal) on widget",
                "2009-10-15 MovedToDismiss(complaint) by widget",
                "2009-11-06 Denied(motion) by court",
                "2009-11-23 Answered(complaint) by widget",
            ],
        ),
        (
            "before: one day late after denial",
            vec![
                "2009-10-01 Served(complaint, Personal) on widget",
                "2009-10-15 MovedToDismiss(complaint) by widget",
                "2009-11-06 Denied(motion) by court",
                "2009-11-24 Answered(complaint) by widget",
            ],
        ),
        (
            "before: mail adds 3 days and Sunday rolls to Monday (due 2009-10-26)",
            vec![
                "2009-10-02 Served(complaint, Mail) on widget",
                "2009-10-26 Answered(complaint) by widget",
            ],
        ),
        (
            "after: Christmas rolls the 21st day to Monday (due 2009-12-28)",
            vec![
                "2009-12-04 Served(complaint, Personal) on widget",
                "2009-12-28 Answered(complaint) by widget",
            ],
        ),
        (
            "after: one day late",
            vec![
                "2009-12-04 Served(complaint, Personal) on widget",
                "2009-12-29 Answered(complaint) by widget",
            ],
        ),
        (
            "typestate: answering before service",
            vec!["2009-12-04 Answered(complaint) by widget"],
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, row_2009(events)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// A matter built as HIR (no source) is checked the same way; findings
/// fall back to a placeholder location naming the case.
#[test]
fn hir_without_source_is_checked_with_placeholder_spans() {
    let resolved = litigate_sema::case::resolve(
        "cases/examples/acme.case",
        &acme(),
        &examples().iter().collect::<Vec<_>>(),
        Some(&lock(examples())),
    );
    let mut case = resolved
        .case
        .unwrap_or_else(|| panic!("{:#?}", resolved.diagnostics));
    case.events[1].date = "2025-03-27".into();
    let refs: Vec<&Package> = examples().iter().collect();
    let lock = lock(examples());
    let checked = litigate_sema::case::check(
        case,
        &litigate_sema::case::CaseSpans::default(),
        litigate_sema::case::Env {
            packages: &refs,
            lock: Some(&lock),
            clock: examples_clock(),
            profile: Profile::Dev,
        },
    );
    let codes: Vec<(String, &str)> = checked
        .diagnostics
        .iter()
        .map(|d| (d.span.file.0.clone(), d.code.as_str()))
        .collect();
    assert_eq!(codes, [("<case acme-v-widget>".to_owned(), "L4002")]);
}
