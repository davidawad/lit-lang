//! Inlay hints over matters: the hint sits after the event that triggered
//! the deadline (not an earlier or same-day one), names the counting
//! rule's cite or else the rule, and only appears in the range asked for.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use litigate_lsp::inlay::hints;
use tower_lsp::lsp_types::{InlayHint, InlayHintLabel, InlayHintTooltip, Position, Range};

const CASE: &str = "cases/acme.case";

fn all() -> Range {
    Range::new(Position::new(0, 0), Position::new(999, 0))
}

fn labels(hints: &[InlayHint]) -> Vec<(u32, String)> {
    hints
        .iter()
        .map(|h| match &h.label {
            InlayHintLabel::String(s) => (h.position.line, s.clone()),
            InlayHintLabel::LabelParts(_) => (h.position.line, String::new()),
        })
        .collect()
}

/// The worked matter with its events replaced by `events`.
fn matter(events: &str) -> String {
    let case = support::read("cases/examples/acme.case");
    let start = case.find("events {").unwrap();
    let end = start + case[start..].find("}\n").unwrap() + 2;
    format!("{}events {{\n{events}}}\n{}", &case[..start], &case[end..])
}

fn line_of(text: &str, needle: &str) -> u32 {
    u32::try_from(text[..text.find(needle).unwrap()].matches('\n').count()).unwrap()
}

#[test]
fn the_hint_follows_the_triggering_event() {
    // The answer period's trigger policy is `first`: the period runs from
    // the first service, and neither a later one nor another on the same
    // day restarts it (SPEC §5.3). Same-day services are docketed: events
    // of one date are ordered by docket number (SPEC §5.6).
    let case = matter(
        "  first: 2025-03-01 Served(complaint, Electronic) on widget\n  mailed: 2025-03-05 docket 4 Served(complaint, Mail) on widget\n  handed: 2025-03-05 docket 5 Served(complaint, Personal) on widget\n",
    );
    let root = support::worked("inlay-order", &[(CASE, &case)]);
    let mut host = support::host(&root);
    support::pin(&root, &mut host);
    let a = host.analyze();
    assert_eq!(a.diagnostics[CASE], Vec::new());
    let found = labels(&hints(a, CASE, all()));
    assert_eq!(
        found,
        [(
            line_of(&case, "2025-03-01 Served(complaint, Electronic"),
            "// answer-period due 2025-03-24 (FRCP 6(a) rolled)".to_owned()
        )]
    );
    // A range that ends before the event shows nothing.
    let before = Range::new(Position::new(0, 0), Position::new(1, 0));
    assert!(hints(a, CASE, before).is_empty());
    // Nor does a file that is not a matter.
    assert!(hints(a, "lit/ex/rule12.lit", all()).is_empty());
}

#[test]
fn a_qualified_event_call_is_not_found_in_the_source() {
    // The schedule renders the trigger unqualified; the source is matched
    // as written, so a qualified call carries no hint.
    let case = matter("  2025-03-05 frcp-rule12-example::Served(complaint, Personal) on widget\n");
    let root = support::worked("inlay-qualified", &[(CASE, &case)]);
    let mut host = support::host(&root);
    support::pin(&root, &mut host);
    let a = host.analyze();
    let scheduled: Vec<&str> = a
        .case_of(CASE)
        .unwrap()
        .schedule
        .iter()
        .map(|s| s.trigger.as_str())
        .collect();
    assert_eq!(scheduled, ["Served(complaint, Personal)"]);
    assert!(hints(a, CASE, all()).is_empty());
}

#[test]
fn a_counting_rule_from_another_package_is_cited() {
    // `ptab` counts with `lit/core`'s `rcfc6`: the hint cites the version
    // the schedule selected there, not the bare rule id.
    let mut host = support::host(&support::repo());
    let a = host.analyze();
    let case = "cases/ptab-ipr-defense.case";
    insta::assert_json_snapshot!("ptab_ipr_defense_hints", labels(&hints(a, case, all())));
}

#[test]
fn a_deadline_with_an_unknown_due_date_says_so() {
    // The engine counts no business days: the schedule keeps the deadline,
    // its due date unknown, and the hint says so rather than guessing.
    let module = format!(
        "{}\ndeadline in-business-days: 3.business_days from Served(complaint) counted_by frcp6\n",
        support::read("lit/examples/rule12.lit")
    );
    let case = matter("  2025-03-03 Served(complaint, Personal) on widget\n");
    let root = support::worked(
        "inlay-unknown",
        &[(CASE, &case), ("lit/ex/rule12.lit", &module)],
    );
    let mut host = support::host(&root);
    support::pin(&root, &mut host);
    let a = host.analyze();
    let found = labels(&hints(a, CASE, all()));
    let line = line_of(&case, "2025-03-03 Served(complaint, Personal");
    assert_eq!(
        found,
        [
            (
                line,
                "// answer-period due 2025-03-24 (FRCP 6(a))".to_owned()
            ),
            (
                line,
                "// in-business-days due unknown: `in-business-days` counts business days, which have no engine lowering".to_owned()
            ),
        ]
    );
}

#[test]
fn a_notice_without_its_due_date_hints_the_reason() {
    // The notice's designated date is `?`: the hint names why the due date
    // is unknown, the tooltip has the reason whole (lg-flu.46).
    let mut host = support::host(&support::repo());
    let a = host.analyze();
    let case = "cases/e2e-ttab-unknown.case";
    let found = hints(a, case, all());
    assert_eq!(
        labels(&found),
        [(
            line_of(&support::read(case), "2026-10-21 InstitutionNoticed(?)"),
            "// answer-period due unknown: notice due date not recorded".to_owned()
        )]
    );
    let Some(InlayHintTooltip::MarkupContent(tip)) = &found[0].tooltip else {
        panic!("no tooltip");
    };
    assert_eq!(
        tip.value,
        "`ttab-trademark-trial-and-appeal-board::answer-period` triggered by `InstitutionNoticed(?)` on 2026-10-21: the due date is unknown: notice due date not recorded: `InstitutionNoticed` on 2026-10-21 designates the due date of `answer-period` (its `due`), and the matter records it as `?`; it is the notice's to set, never counted in its place"
    );
}
