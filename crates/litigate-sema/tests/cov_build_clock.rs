//! Holiday calendars, counting rules, deadlines, and transitions lowered by
//! `types::build` (SPEC §5.3-§5.4), one diagnostic path per line of source.
mod common;
use common::{check_body, package, workspace};
use litigate_sema::hir::Declaration;

/// The declarations of an error-free single-module package with `body`.
fn declarations(body: &str) -> Vec<Declaration> {
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    assert_eq!(common::render(&db, ws), Vec::<String>::new());
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter().cloned())
        .collect()
}

#[test]
fn every_holiday_rule_and_observance_lowers() {
    let calendars: Vec<_> = declarations(
        "calendar all {\n\
  a: fixed(2, 29) effective [2000-01-01, ..) observed nearest_weekday\n\
  b: nth(4, thursday, 11) effective [2000-01-01, 2030-01-01) observed next_day\n\
  c: last(monday, 5) effective [2000-01-01, ..)\n\
  d: every(4, 2001, 1, 20) effective [2000-01-01, ..)\n\
  e: once(2001-09-11) effective [2001-09-11, 2001-09-12)\n\
}",
    )
    .into_iter()
    .filter(|d| matches!(d, Declaration::Calendar(c) if c.meta.id == "all"))
    .collect();
    insta::assert_debug_snapshot!(calendars);
}

#[test]
fn holiday_rules_report_their_first_bad_argument() {
    let out = check_body(
        "calendar k {\n\
  a: fixed(2, 30) effective [2000-01-01, ..)\n\
  a: fixed(1, 2) effective [2000-01-01, ..)\n\
  b: fixed(1/2, 1) effective [2000-01-01, ..)\n\
  c: fixed(13, 1) effective [2000-01-01, ..)\n\
  d: fixed(1, true) effective [2000-01-01, ..)\n\
  e: nth(5, monday, 1) effective [2000-01-01, ..)\n\
  f: nth(1, funday, 1) effective [2000-01-01, ..)\n\
  g: nth(1, 3, 1) effective [2000-01-01, ..)\n\
  h: nth(1, monday, 0) effective [2000-01-01, ..)\n\
  i: last(monday.x, 1) effective [2000-01-01, ..)\n\
  j: last(monday, 13) effective [2000-01-01, ..)\n\
  k: every(0, 2000, 1, 1) effective [2000-01-01, ..)\n\
  l: every(4, 0, 1, 1) effective [2000-01-01, ..)\n\
  m: every(4, 2000, 0, 1) effective [2000-01-01, ..)\n\
  n: every(4, 2000, 1, 32) effective [2000-01-01, ..)\n\
  o: once(3) effective [2000-01-01, ..)\n\
  p: fixd(1, 1) effective [2000-01-01, ..)\n\
  q: zzzzzz(1) effective [2000-01-01, ..)\n\
  r: fixed(1) effective [2000-01-01, ..)\n\
  s: fixed(7, 4) effective [2000-01-01, ..) observed zzzzzz\n\
}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn counting_rules_lower_every_option() {
    let countings: Vec<_> = declarations(
        "counting full {\n\
  exclude_trigger_day\n  include_last_day\n  allow_hours\n\
  roll_forward_on [weekend, legal_holiday, inaccessible]\n\
  service_extension { Mail: 3.days, Personal: 1.court_days }\n\
  holidays cal\n  anchor next_business_day\n\
  short_period { below: 11.days, count: business_days }\n\
}",
    )
    .into_iter()
    .filter(|d| matches!(d, Declaration::Counting(c) if c.meta.id == "full"))
    .collect();
    insta::assert_debug_snapshot!(countings);
}

#[test]
fn counting_rules_reject_unknown_keywords_and_shapes() {
    let out = check_body(
        "counting k1 {\n\
  roll_forward_on [weekend, weekend.x, zzzzzz]\n\
  service_extension { Mail: 3.days, Personal: 8.hours, Mail: 4.days }\n\
  holidays cal\n  anchor trigger_dya\n\
  short_period { below: 8.hours, count: court_days }\n\
}\n\
counting k2 { anchor 3 }\n\
counting k3 { holidays nope, short_period { below: 7.days, count: zzzzzz } }\n\
counting k4 { holidays c6, short_period { below: 7.days } }\n\
counting k5 { holidays cal, service_extension { Mail: 1.5.days } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn deadlines_lower_direction_and_options() {
    let deadlines: Vec<_> = declarations(
        "deadline back: 3.days before Served(complaint) counted_by c6 { extendable true, extension_authority \"Court\", note \"n\" }",
    )
    .into_iter()
    .filter(|d| matches!(d, Declaration::Deadline(_)))
    .collect();
    insta::assert_debug_snapshot!(deadlines);
}

#[test]
fn deadlines_report_period_trigger_and_counting_problems() {
    let out = check_body(
        "counting hrs { holidays cal, allow_hours }\n\
deadline h1: 8.hours from Served(complaint) counted_by hrs\n\
deadline n1: 1.5.days from Served(complaint) counted_by c6\n\
deadline n2: 3.days from 3 counted_by c6\n\
deadline n3: 3.days from Served(complaint) counted_by nope\n\
deadline n4: 3.days from Nope(complaint) counted_by c6",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn transitions_need_versions_and_a_final_otherwise() {
    let out = check_body(
        "deadline p: 1.days from Served(complaint) counted_by c6 effective [2000-01-01, 2001-01-01)\n\
deadline p: 2.days from Served(complaint) counted_by c6 effective [2001-01-01, ..)\n\
transition t1 amends p { otherwise -> p@2000-01-01 }\n\
transition t2 amends p@2001-01-01 { otherwise -> p@2000-01-01, unless true -> p@2001-01-01 }\n\
transition t3 amends p@2001-01-01 {}\n\
transition t4 amends p@2001-01-01 { otherwise -> p }",
    );
    insta::assert_debug_snapshot!(out);
}
