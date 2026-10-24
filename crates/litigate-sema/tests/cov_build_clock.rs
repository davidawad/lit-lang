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
  service_extension { Personal: 1.court_days, Electronic: 0.days, Mail: 3.days, Clerk: unrecognized, OtherConsented: unrecognized, CommercialCarrier: unrecognized, PrisonMail: unrecognized, MailForeign: unrecognized, Express: unrecognized, ExpressForeign: unrecognized }\n\
  holidays cal\n  anchor next_business_day\n\
  forward_holidays cal cite \"FRCP 6(a)(6)(C)\"\n\
  declared_holidays closures cite \"FRCP 6(a)(6)(B)\"\n\
  forums [frcp, frap]\n\
  short_period { below: 11.days, count: business_days }\n\
}\n\
calendar closures { eo: once(2018-12-05) effective [2018-12-05, 2018-12-06) }",
    )
    .into_iter()
    .filter(|d| matches!(d, Declaration::Counting(c) if c.meta.id == "full"))
    .collect();
    insta::assert_debug_snapshot!(countings);
}

/// A rule with no `service_extension` names no service method: all are
/// unrecognized, and its forums default to its package's.
#[test]
fn a_rule_without_a_service_map_recognizes_no_method() {
    let rules: Vec<_> = declarations("")
        .into_iter()
        .filter_map(|d| match d {
            Declaration::Counting(c) if c.meta.id == "c6" => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(rules[0].forums, ["demo"]);
    assert_eq!(rules[0].service_extensions.len(), 10);
    assert!(rules[0]
        .service_extensions
        .values()
        .all(|t| *t == litigate_sema::hir::ServiceTime::Unrecognized));
}

/// `service_extension` is total over the closed method set; each value is
/// `N.days` or `unrecognized`.
#[test]
fn service_maps_are_total_over_known_methods() {
    let out = check_body(
        "counting s1 { holidays cal, service_extension { Personl: 3.days, Zzzzzzzz: 1.days, Mail: unrecognised, Personal: unrecognized.x, Electronic: unrecognized } }\n\
counting s3 { holidays cal, service_extension }\n\
counting s2 { holidays cal, service_extension { Personal: 0.days, Electronic: 0.days, Mail: 3.days, Clerk: 3.days, OtherConsented: 3.days, CommercialCarrier: 0.days, PrisonMail: 0.days, MailForeign: 0.days, Express: 0.days, ExpressForeign: unrecognized } }",
    );
    insta::assert_debug_snapshot!(out);
}

/// `forward_holidays` and `declared_holidays` need the rule closing on them;
/// a declared calendar lists `once(date)` days only.
#[test]
fn closing_calendars_need_a_cite_and_declared_days() {
    let out = check_body(
        "calendar closures { eo: once(2018-12-05) effective [2018-12-05, 2018-12-06) }\n\
calendar observed-closures { eo: once(2018-12-05) effective [2018-12-05, 2018-12-06) observed nearest_weekday }\n\
counting f1 { holidays cal, forward_holidays cal }\n\
counting f2 { holidays cal, declared_holidays cal cite \"FRCP 6(a)(6)(B)\" }\n\
counting f3 { holidays cal, declared_holidays observed-closures cite ?(\"no order is vendored\") }\n\
counting f4 { holidays cal, declared_holidays closures }\n\
counting f5 { holidays cal, forward_holidays nope cite \"FRCP 6(a)(6)(C)\" }\n\
counting f6 { holidays cal, forward_holidays }",
    );
    insta::assert_debug_snapshot!(out);
}

/// A deadline is a period of its package's forum (or the one it names):
/// its counting rule must compute that forum's periods.
#[test]
fn a_deadline_is_counted_by_its_own_forums_rule() {
    let out = check_body(
        "counting appellate effective [2000-01-01, 2020-01-01) { holidays cal, forums [frap, cafc] }\n\
counting appellate effective [2020-01-01, ..) { holidays cal, forums [scotus] }\n\
deadline own: 3.days from Served(complaint) counted_by c6\n\
deadline named: 3.days from Served(complaint) counted_by appellate { forum scotus }\n\
deadline borrowed: 3.days from Served(complaint) counted_by appellate\n\
deadline unknown: 3.days from Served(complaint) counted_by nope",
    );
    insta::assert_debug_snapshot!(out);
}

/// `forum F cite "X" [quote "..."]` declares the rule adopting `F`'s
/// period, with the words adopting it; a cite hole leaves it undeclared
/// and quotes nothing.
#[test]
fn a_named_forum_carries_its_adopting_rule() {
    let adoptions: Vec<_> = declarations(
        "counting appellate { holidays cal, forums [frap, demo] }\n\
deadline quoted: 3.days from Served(complaint) counted_by appellate { forum frap cite \"28 U.S.C. § 2522\" quote \"within the time\" }\n\
deadline cited: 3.days from Served(complaint) counted_by appellate { forum frap cite \"FRAP 4\" historical }\n\
deadline bare: 3.days from Served(complaint) counted_by appellate { forum frap }",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Deadline(x) => Some((x.meta.id, x.adoption)),
        _ => None,
    })
    .collect();
    insta::assert_debug_snapshot!(adoptions);
    let out = check_body(
        "counting appellate { holidays cal, forums [frap, demo] }\n\
deadline why: 3.days from Served(complaint) counted_by appellate { forum frap cite ?(\"unvendored\") }\n\
deadline unsaid: 3.days from Served(complaint) counted_by appellate { forum frap cite ? }\n\
deadline quoted: 3.days from Served(complaint) counted_by appellate { forum frap cite ?(\"unvendored\") quote \"words\" }",
    );
    insta::assert_debug_snapshot!(out);
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
fn designated_deadlines_lower_the_designating_parameter() {
    let deadlines: Vec<_> = declarations(
        "event Noticed(document: Document, due: Date)\n\
deadline answer: 30.days from Noticed() counted_by c6 { designated due }\n\
deadline counterclaim: 30.days from Noticed() counted_by c6 { designated due, interpretation \"measured from mailing\" }",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Deadline(d) => d.designated,
        _ => None,
    })
    .collect();
    insta::assert_debug_snapshot!(deadlines);
}

#[test]
fn designated_deadlines_report_bad_designations() {
    let out = check_body(
        "event Noticed(document: Document, due: Date)\n\
deadline d1: 30.days before Noticed() counted_by c6 { designated due }\n\
deadline d2: 30.court_days from Noticed() counted_by c6 { designated due }\n\
deadline d3: 30.days from Noticed() counted_by c6 { designated nope }\n\
deadline d4: 30.days from Noticed() counted_by c6 { designated document }\n\
deadline d5: 30.days from 3 counted_by c6 { designated due }\n\
deadline d6: 30.days from Nope() counted_by c6 { designated due }\n\
deadline d7: 30.days from Document() counted_by c6 { designated due }\n\
deadline d8: 30.days from Noticed() counted_by c6 { designated due.x }\n\
deadline d9: 30.days from Noticed() counted_by c6 { interpretation \"measured from mailing\" }\n\
deadline d10: 30.days from Noticed() counted_by c6 { designated nope, interpretation \"measured from mailing\" }",
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

/// A chained period runs from one end of another deadline's period,
/// `deadline(d).nominal` or `.adjusted`; it never says neither.
#[test]
fn a_chained_period_names_which_end_it_runs_from() {
    let chains: Vec<_> = declarations(
        "deadline base: 30.days from Served(complaint) counted_by c6\n\
deadline from-nominal: 90.days from deadline(base).nominal counted_by c6\n\
deadline from-adjusted: 10.days after deadline(from-nominal).adjusted counted_by c6",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Deadline(d) if d.chain.is_some() => Some((d.meta.id, d.chain, d.trigger)),
        _ => None,
    })
    .collect();
    insta::assert_debug_snapshot!(chains);
}

#[test]
fn a_chain_must_name_one_deadline_and_one_end() {
    let out = check_body(
        "deadline base: 30.days from Served(complaint) counted_by c6\n\
deadline bare: 90.days from deadline(base) counted_by c6\n\
deadline misspelt: 90.days from deadline(base).nominl counted_by c6\n\
deadline unknown-end: 90.days from deadline(base).zzzzzzzz counted_by c6\n\
deadline itself: 90.days from deadline(itself).nominal counted_by c6\n\
deadline two: 90.days from deadline(base, base).nominal counted_by c6\n\
deadline literal: 90.days from deadline(3) counted_by c6\n\
deadline nowhere: 90.days from deadline(nope).nominal counted_by c6\n\
deadline not-a-deadline: 90.days from deadline(c6).nominal counted_by c6\n\
deadline event-field: 90.days from Served(complaint).nominal counted_by c6",
    );
    insta::assert_debug_snapshot!(out);
}
