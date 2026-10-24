// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
use super::*;
use litigate_sema::case::Status;
use proptest::prelude::*;

fn scheduled(deadline: &str, due: Option<&str>, status: Status) -> Scheduled {
    Scheduled {
        deadline: deadline.to_owned(),
        trigger: "Served(complaint, Personal)".to_owned(),
        trigger_date: "2025-03-05".to_owned(),
        due: due.map(str::to_owned),
        unknown: None,
        designated: None,
        rule: "frcp6".to_owned(),
        steps: vec!["FRCP 6(a): exclude the trigger date 2025-03-05".to_owned()],
        version: String::new(),
        cite: None,
        rolled: false,
        method: None,
        bearer: None,
        floor: None,
        extended_to: None,
        extended_under: None,
        request_made: None,
        status,
        trigger_event: 0,
        closed_by: None,
    }
}

fn day(iso: &str) -> Date {
    Date::parse_iso(iso).unwrap()
}

fn cal(remind: &[u32]) -> Calendar<'_> {
    Calendar {
        case: "acme-v-widget",
        path: "cases/acme.case",
        as_of: "2025-03-10",
        remind,
    }
}

/// The calendar with its `PRODID` (the compiler version) masked.
fn stable(ics: &str) -> String {
    ics.replace(env!("CARGO_PKG_VERSION"), "<version>")
}

#[test]
fn text_escapes_backslash_semicolon_comma_and_newlines() {
    assert_eq!(escape("a\\b;c,d\r\ne"), "a\\\\b\\;c\\,d\\ne");
}

#[test]
fn a_line_of_75_octets_is_not_folded_and_76_is() {
    let exact = "x".repeat(75);
    assert_eq!(fold(&exact), format!("{exact}\r\n"));
    let over = "x".repeat(76);
    assert_eq!(fold(&over), format!("{exact}\r\n x\r\n"));
}

#[test]
fn folding_never_splits_a_character() {
    let line = format!("{}§", "x".repeat(74));
    assert_eq!(fold(&line), format!("{}\r\n §\r\n", "x".repeat(74)));
}

#[test]
fn an_unknown_due_date_says_why_and_is_never_guessed() {
    let mut s = scheduled("p::answer-period", None, Status::Open);
    assert_eq!(due_of(&s), Err("the due date is unknown".to_owned()));
    s.unknown = Some("the notice gives no date".to_owned());
    assert_eq!(due_of(&s), Err("the notice gives no date".to_owned()));
    s.due = Some("2025-02-30".to_owned());
    assert!(due_of(&s)
        .unwrap_err()
        .starts_with("due date `2025-02-30` is not a date"));
}

#[test]
fn an_extension_moves_the_due_date() {
    let mut s = scheduled("p::answer-period", Some("2025-03-26"), Status::Open);
    assert_eq!(due_of(&s), Ok(day("2025-03-26")));
    s.extended_to = Some("2025-04-25".to_owned());
    assert_eq!(due_of(&s), Ok(day("2025-04-25")));
}

#[test]
fn the_uid_depends_on_what_caused_the_deadline_only() {
    let a = scheduled("p::answer-period", Some("2025-03-26"), Status::Open);
    let mut moved = a.clone();
    moved.due = Some("2025-03-27".to_owned());
    moved.status = Status::Discharged;
    assert_eq!(uid("m", &a), uid("m", &moved));
    let mut other = a.clone();
    other.trigger_date = "2025-03-06".to_owned();
    assert_ne!(uid("m", &a), uid("m", &other));
    assert_ne!(uid("m", &a), uid("n", &a));
    assert!(uid("m", &a).ends_with("@litigate"));
}

#[test]
fn a_calendar_holds_one_all_day_event_per_deadline_in_due_order() {
    let mut late = scheduled("p::reply-period", Some("2025-04-01"), Status::Lapsed);
    late.extended_to = Some("2025-04-08".to_owned());
    late.floor = Some("2025-03-31".to_owned());
    late.bearer = Some("widget".to_owned());
    let mut open = scheduled("p::answer-period", Some("2025-03-26"), Status::Open);
    open.cite = Some("FRCP 6(a)".to_owned());
    let rows = [
        Row {
            due: due_of(&late).unwrap(),
            scheduled: &late,
            authority: None,
        },
        Row {
            due: due_of(&open).unwrap(),
            scheduled: &open,
            authority: Some("FRCP 12(a)(1)(A)(i)"),
        },
    ];
    let (ics, events) = render(&cal(&[7, 1]), &rows);
    assert!(ics.lines().all(|l| l.len() <= 76), "{ics}");
    assert!(ics.ends_with("END:VCALENDAR\r\n"));
    let order: Vec<&str> = events.iter().map(|e| e.deadline.as_str()).collect();
    assert_eq!(order, ["p::answer-period", "p::reply-period"]);
    assert_eq!(events[1].due, "2025-04-08");
    insta::assert_snapshot!("ics_two_deadlines", stable(&ics));
}

#[test]
fn an_empty_schedule_is_an_empty_calendar() {
    let (ics, events) = render(&cal(&[]), &[]);
    assert!(events.is_empty());
    assert!(!ics.contains("VEVENT"), "{ics}");
}

proptest! {
    #[test]
    fn folded_lines_fit_and_unfold_to_the_original(line in "[ -~§é😀]{0,300}") {
        let folded = fold(&line);
        prop_assert!(folded.ends_with("\r\n"));
        for physical in folded.trim_end_matches("\r\n").split("\r\n") {
            prop_assert!(physical.len() <= LINE_OCTETS, "{physical:?}");
        }
        let unfolded = folded.trim_end_matches("\r\n").replace("\r\n ", "");
        prop_assert_eq!(unfolded, line);
    }

    #[test]
    fn escaped_text_has_no_bare_separators(text in "[ -~\n]{0,80}") {
        let escaped = escape(&text);
        prop_assert!(!escaped.contains('\n'));
        let mut chars = escaped.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                prop_assert!(matches!(chars.next(), Some('\\' | ';' | ',' | 'n')));
            } else {
                prop_assert!(c != ';' && c != ',');
            }
        }
    }
}
