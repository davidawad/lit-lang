//! Declared event ids (SPEC §5.6, docs/litigate/TIMELINE.md): a declared
//! id is the event's timeline key; an event without one is L9006 (info)
//! with a fix inserting a generated id no other line uses; a repeated id
//! is L1001.
mod cov_case_support;

use cov_case_support::{fixture, lock, matter, render, ALPHA_ONLY, STAKES};
use litigate_base::{DiagnosticCode as C, Profile, Severity};
use litigate_sema::case::{check_source, CheckedCase, Env, NoClock};
use litigate_sema::hir::{Event, EventParty, Expr, ExprKind, Type};

const RULED: &str = "Ruled(2010-01-01, true, 3.5, 5.usd) by p";

fn checked(events: &[&str]) -> CheckedCase {
    let packages: Vec<_> = fixture().iter().collect();
    let lock = lock(fixture());
    let text = matter(ALPHA_ONLY, "", events, STAKES);
    check_source(
        "cases/t.case",
        &text,
        Env {
            packages: &packages,
            lock: Some(&lock),
            clock: &NoClock,
            profile: Profile::Strict,
        },
    )
}

/// The L9006 and L1001 diagnostics, rendered.
fn ids(events: &[&str]) -> Vec<String> {
    let c = checked(events);
    let wanted: Vec<_> = c
        .diagnostics
        .into_iter()
        .filter(|d| matches!(d.code, C::UndeclaredEventId | C::DuplicateName))
        .collect();
    render(&wanted)
}

#[test]
fn a_declared_id_is_the_events_timeline_key() {
    let c = checked(&[&format!("ruling: 2010-01-04 {RULED}")]);
    assert!(c.ok(), "{:?}", c.diagnostics);
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
    let case = c.case.expect("resolved");
    assert_eq!(case.events[0].id.as_deref(), Some("ruling"));
    assert_eq!(case.events[0].key(), "event:Ruled@ruling");
}

#[test]
fn an_event_without_an_id_is_info_and_still_checks() {
    let c = checked(&[&format!("2010-01-04 {RULED}")]);
    assert!(c.ok(), "{:?}", c.diagnostics);
    let d = &c.diagnostics[0];
    assert_eq!((d.code, d.severity), (C::UndeclaredEventId, Severity::Info));
    let case = c.case.expect("resolved");
    assert_eq!(case.events[0].id, None);
    assert_eq!(case.events[0].key(), "event:Ruled@2010-01-04");
}

#[test]
fn generated_ids_skip_every_declared_and_generated_id() {
    insta::assert_debug_snapshot!(ids(&[
        &format!("2010-01-04 {RULED}"),
        &format!("ruled-1: 2010-01-05 {RULED}"),
        &format!("2010-01-06 docket 2 alpha::{RULED}"),
        "2010-01-07 Answered(complaint) by p",
        "2010-01-08 Answered(complaint) by p",
    ]));
}

#[test]
fn a_repeated_id_is_a_duplicate_name() {
    let events = [
        &*format!("ruling: 2010-01-04 {RULED}"),
        &*format!("ruling: 2010-01-05 {RULED}"),
    ];
    insta::assert_debug_snapshot!(ids(&events));
    assert!(checked(&events).case.is_none());
}

#[test]
fn a_line_with_a_syntax_error_gets_no_generated_id() {
    // No call, and an id with no date: both are already syntax errors.
    assert_eq!(
        ids(&["2010-01-04 42 by p", &format!("x: {RULED}")]),
        Vec::<String>::new()
    );
}

fn event(callee: &str, id: Option<&str>, docket: Option<u64>) -> Event {
    Event {
        id: id.map(str::to_owned),
        date: "2010-01-04".into(),
        docket,
        action: Expr {
            ty: Type::Unit,
            kind: ExprKind::Call {
                callee: callee.into(),
                args: Vec::new(),
            },
            span: None,
        },
        relation: EventParty::By,
        party: "p".into(),
        authority: None,
    }
}

#[test]
fn keys_use_the_id_else_the_date_and_docket() {
    assert_eq!(
        event("alpha::Ruled", Some("r"), Some(3)).key(),
        "event:Ruled@r"
    );
    assert_eq!(
        event("Ruled", None, Some(3)).key(),
        "event:Ruled@2010-01-04/3"
    );
    assert_eq!(
        event("a.b.Ruled", None, None).key(),
        "event:Ruled@2010-01-04"
    );
    let mut e = event("Ruled", None, None);
    e.action.kind = ExprKind::Ref { target: "x".into() };
    assert_eq!(e.key(), "event:@2010-01-04");
}
