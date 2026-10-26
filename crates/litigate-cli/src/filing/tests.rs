use super::*;

const HEAD: &str = "case m\nuses p ^0.1.0\nas_of 2025-03-10\nsummary \"S.\"\n\nparties {\n  a: Plaintiff Corporation\n}\n";

fn with_events(events: &str) -> String {
    format!("{HEAD}\nevents {{\n{events}}}\n\nfacts {{ x: y }}\n")
}

#[test]
fn a_filing_goes_before_the_first_later_event() {
    let text = with_events("  2025-03-01 A() on a\n  2025-03-09 C() by a\n");
    let line = line("2025-03-05", "B()", "by", "a");
    let spliced = splice(&text, "2025-03-05", &line);
    assert_eq!(
        spliced,
        with_events("  2025-03-01 A() on a\n  2025-03-05 B() by a\n  2025-03-09 C() by a\n")
    );
    assert!(added(&text, &spliced, &line));
}

#[test]
fn a_filing_on_the_day_of_the_last_event_follows_it() {
    let text = with_events("  2025-03-01 A() on a\n");
    let spliced = splice(&text, "2025-03-01", "2025-03-01 B() by a");
    assert_eq!(
        spliced,
        with_events("  2025-03-01 A() on a\n  2025-03-01 B() by a\n")
    );
}

#[test]
fn a_filing_before_every_event_goes_first() {
    let text = "case m\nevents {\n2025-03-09 C() by a\n}\n";
    assert_eq!(
        splice(text, "2025-03-01", "2025-03-01 B() by a"),
        "case m\nevents {\n  2025-03-01 B() by a\n2025-03-09 C() by a\n}\n"
    );
}

#[test]
fn an_empty_block_gets_the_event() {
    let text = with_events("");
    let spliced = splice(&text, "2025-03-05", "2025-03-05 B() by a");
    assert!(added(&text, &spliced, "2025-03-05 B() by a"), "{spliced}");
}

#[test]
fn a_matter_without_events_gets_a_block_before_its_facts() {
    let text = format!("{HEAD}\nfacts {{ x: y }}\n");
    let spliced = splice(&text, "2025-03-05", "2025-03-05 B() by a");
    assert_eq!(
        spliced,
        format!("{HEAD}\nevents {{\n  2025-03-05 B() by a\n}}\n\nfacts {{ x: y }}\n")
    );
}

#[test]
fn a_matter_without_blocks_gets_one_at_the_end() {
    let spliced = splice(HEAD, "2025-03-05", "2025-03-05 B() by a");
    assert_eq!(
        spliced,
        format!("{HEAD}\nevents {{\n  2025-03-05 B() by a\n}}\n")
    );
    assert!(added(HEAD, &spliced, "2025-03-05 B() by a"));
    let bare = HEAD.trim_end();
    assert_eq!(
        splice(bare, "2025-03-05", "2025-03-05 B() by a"),
        format!("{bare}\n\nevents {{\n  2025-03-05 B() by a\n}}\n")
    );
}

#[test]
fn text_that_is_not_one_event_is_not_added() {
    let text = with_events("  2025-03-01 A() on a\n");
    let two = "2025-03-05 B() by a\n  2025-03-06 C() by a";
    assert!(!added(&text, &splice(&text, "2025-03-05", two), two));
    assert!(!added(&text, &text, "2025-03-01 A() on a"));
}

#[test]
fn as_of_moves_forward_only() {
    let text = with_events("");
    assert!(with_as_of(&text, "2025-04-01").contains("as_of 2025-04-01\n"));
    assert_eq!(with_as_of(&text, "2025-03-01"), text);
    assert_eq!(with_as_of("", "2025-03-01"), "");
}

#[test]
fn a_docketed_filing_goes_before_a_later_docket_number_of_its_date() {
    let text = with_events("  2025-03-05 docket 2 A() on a\n  2025-03-05 docket 1_0 C() by a\n");
    let line = "2025-03-05 docket 7 B() by a";
    assert_eq!(
        splice_docketed(&text, "2025-03-05", Some(7), line),
        with_events(
            "  2025-03-05 docket 2 A() on a\n  2025-03-05 docket 7 B() by a\n  2025-03-05 docket 1_0 C() by a\n"
        )
    );
    assert_eq!(
        splice_docketed(&text, "2025-03-05", Some(11), line),
        splice(&text, "2025-03-05", line)
    );
}
