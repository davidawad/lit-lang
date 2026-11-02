//! Effective intervals, version sets, supersession, revival, `as_of`, and
//! transitions (SPEC §5.4), one small program per rule.
mod legal;
use legal::{pkg, render, table, with_world, workspace, PRELUDE};
use litigate_sema::hir::{Declaration, Expr};
use litigate_sema::temporal::{self, governing, Key, MatterEvent, World};

const P20: &str = "deadline p: 20.days from Served(complaint) counted_by c6";
const P21: &str = "deadline p: 21.days from Served(complaint) counted_by c6";

fn v(decl: &str, clauses: &str) -> String {
    format!("{decl} {clauses}\n")
}

#[test]
fn state_counting_variant_is_required_for_local_deadlines() {
    let source = "module main\nstart s\ncounting frbp9006 effective [2025-01-01, ..) cite \"FRBP 9006(a)\" { exclude_trigger_day, include_last_day, roll_forward_on [weekend], holidays cal, service_extension { Personal: 0.days, Electronic: 0.days, Mail: 0.days, Clerk: 0.days, OtherConsented: 0.days, CommercialCarrier: unrecognized, PrisonMail: unrecognized, MailForeign: unrecognized, Express: unrecognized, ExpressForeign: unrecognized }, allow_hours, end_of_month clamp }\ncounting frbp9006-georgia effective [2025-01-01, ..) cite \"FRBP 9006(a)\" { exclude_trigger_day, include_last_day, roll_forward_on [weekend], holidays cal, forward_holidays cal cite \"FRBP 9006(a)(6)(C)\", service_extension { Personal: 0.days, Electronic: 0.days, Mail: 0.days, Clerk: 0.days, OtherConsented: 0.days, CommercialCarrier: unrecognized, PrisonMail: unrecognized, MailForeign: unrecognized, Express: unrecognized, ExpressForeign: unrecognized }, allow_hours, end_of_month clamp }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\nevent Served()\nstate s: \"S\" { _ \"go\" -> t }\nterminal t: \"T\"\ndeadline local: 30.days after Served() counted_by frbp9006\n";
    let bad = legal::pkg_counting("demo", "frbp9006-georgia", source);
    let (db, ws) = workspace(&[bad]);
    let diagnostics = render(&db, ws);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.contains("L3011") && d.contains("local")),
        "{diagnostics:#?}"
    );

    let good_source = source.replace("counted_by frbp9006\n", "counted_by frbp9006-georgia\n");
    let good = legal::pkg_counting("demo", "frbp9006-georgia", &good_source);
    let (db, ws) = workspace(&[good]);
    let diagnostics = render(&db, ws);
    assert!(
        !diagnostics.iter().any(|d| d.contains("L3011")),
        "{diagnostics:#?}"
    );
}

#[test]
fn imported_core_state_counting_variant_checks_bare_imported_base() {
    let state = "module core.states.georgia\ncounting frbp9006-georgia effective [2025-01-01, ..) { holidays state-cal, forward_holidays state-cal cite \"FRBP 9006(a)(6)(C)\" }\ncalendar state-cal { new-year: fixed(1, 1) effective [2025-01-01, ..) }\n";
    let district = "module main\nimport core.frbp9006.{ frbp9006 }\nstart s\nevent Served()\nstate s: \"S\" { _ \"go\" -> t }\nterminal t: \"T\"\ndeadline local: 30.days after Served() counted_by frbp9006\n";
    let mut bad = legal::pkg_counting("demo", "frbp9006-georgia", district);
    bad.files.extend(legal::pkg("core", "LocalRule", "2025-01-01", &[], &[("states/georgia.lit", state), ("frbp9006.lit", "module core.frbp9006\ncounting frbp9006 effective [2025-01-01, ..) { holidays base-cal }\ncalendar base-cal { new-year: fixed(1, 1) effective [2025-01-01, ..) }\n")]).files);
    let (db, ws) = workspace(&[bad]);
    let diagnostics = render(&db, ws);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.contains("L3011") && d.contains("local")),
        "{diagnostics:#?}"
    );

    let passing = district.replace("counted_by frbp9006", "counted_by frbp9006-georgia");
    let mut good = legal::pkg_counting("demo", "frbp9006-georgia", &passing);
    good.files.extend(legal::pkg("core", "LocalRule", "2025-01-01", &[], &[("states/georgia.lit", state), ("frbp9006.lit", "module core.frbp9006\ncounting frbp9006 effective [2025-01-01, ..) { holidays base-cal }\ncalendar base-cal { new-year: fixed(1, 1) effective [2025-01-01, ..) }\n")]).files);
    let (db, ws) = workspace(&[good]);
    let diagnostics = render(&db, ws);
    assert!(
        !diagnostics.iter().any(|d| d.contains("L3011")),
        "{diagnostics:#?}"
    );
}

#[test]
fn version_sets_are_disjoint_and_cover_their_lifetime() {
    let succession =
        v(P20, "effective [2009-12-01, 2016-12-01)") + &v(P21, "effective [2016-12-01, ..)");
    let overlap = v(P20, "effective [2009-12-01, ..)") + &v(P21, "effective [2016-12-01, ..)");
    let nested =
        v(P20, "effective [2009-12-01, ..)") + &v(P21, "effective [2012-01-01, 2013-01-01)");
    let gap = v(P20, "effective [2009-12-01, 2012-01-01)") + &v(P21, "effective [2013-01-01, ..)");
    let gap_revived = v(P20, "effective [2009-12-01, 2012-01-01)")
        + &v(P21, "effective [2013-01-01, ..) revives p@2009-12-01");
    let repealed = v(P20, "effective [2009-12-01, 2015-01-01)");
    let repealed_by_successor = v(P20, "effective [2009-12-01, 2015-01-01)")
        + "deadline q: 30.days from Served(complaint) counted_by c6 effective [2015-01-01, ..) supersedes p\n";
    let suspended_without_revival = v(P20, "effective [2009-12-01, 2012-01-01)")
        + &v(P21, "effective [2014-01-01, ..)")
        + "deadline q: 30.days from Served(complaint) counted_by c6 effective [2012-01-01, 2014-01-01) supersedes p\n";
    let late_start = v(P21, "effective [2016-12-01, ..)");
    let self_supersession = v(P20, "effective [2009-12-01, 2016-12-01)")
        + &v(P21, "effective [2016-12-01, ..) supersedes p@2016-12-01");
    let succession_documented = v(P20, "effective [2009-12-01, 2016-12-01)")
        + &v(P21, "effective [2016-12-01, ..) supersedes p@2009-12-01");
    insta::assert_debug_snapshot!(table(&[
        ("01 consecutive versions", &succession),
        ("02 overlap: forgot to close the old version", &overlap),
        ("03 overlap: a version inside another", &nested),
        ("04 undeclared gap", &gap),
        ("05 gap declared by revives", &gap_revived),
        ("06 repeal with nothing after it", &repealed),
        (
            "07 repeal declared by a superseding rule",
            &repealed_by_successor
        ),
        (
            "08 suspension needs no revives when a later version follows",
            &suspended_without_revival
        ),
        ("09 a declaration may begin after its package", &late_start),
        ("10 a version cannot supersede itself", &self_supersession),
        (
            "11 a version may document what it supersedes",
            &succession_documented
        ),
    ]));
}

const TRANSITION_OK: &str = "transition amend-2016 amends p@2016-12-01 cite \"FRCP 86(a)(2)(B)\"
{
  unless has_event(Ordered()) -> p@2009-12-01
  otherwise -> p@2016-12-01
}
";

#[test]
fn transitions_choose_between_the_amended_version_and_its_predecessor() {
    let base = v(P20, "effective [2009-12-01, 2016-12-01)") + &v(P21, "effective [2016-12-01, ..)");
    let well_formed = base.clone() + TRANSITION_OK;
    let first_version =
        base.clone() + "transition amend amends p@2009-12-01 { otherwise -> p@2009-12-01 }\n";
    let other_decl = base.clone()
        + "deadline q: 1.days from Served(complaint) counted_by c6 effective [2009-12-01, ..)\ntransition amend amends p@2016-12-01 { unless has_event(Ordered()) -> q@2009-12-01, otherwise -> p@2016-12-01 }\n";
    let ancient = v(P20, "effective [2009-12-01, 2012-01-01)")
        + &v(P20.replace("20.days", "19.days").as_str(), "effective [2012-01-01, 2016-12-01)")
        + &v(P21, "effective [2016-12-01, ..)")
        + "transition amend amends p@2016-12-01 { unless has_event(Ordered()) -> p@2009-12-01, otherwise -> p@2016-12-01 }\n";
    let not_history = base.clone()
        + "transition amend amends p@2016-12-01 { unless 1 < 2 -> p@2009-12-01, otherwise -> p@2016-12-01 }\n";
    let twice = base.clone()
        + TRANSITION_OK
        + &TRANSITION_OK.replace("transition amend-2016", "transition amend-again");
    insta::assert_debug_snapshot!(table(&[
        ("1 FRCP 86(a)-shaped transition", &well_formed),
        ("2 amends a first version", &first_version),
        ("3 arm selects another declaration", &other_decl),
        ("4 arm selects a version that is not adjacent", &ancient),
        ("5 condition outside the matter history", &not_history),
        ("6 two transitions for one amendment", &twice),
    ]));
}

fn demo(body: &str) -> (litigate_sema::LitDatabase, litigate_sema::Workspace) {
    let text = format!("{PRELUDE}{body}\n");
    workspace(&[pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )])
}

fn key(name: &str) -> Key {
    Key {
        pack: "demo".into(),
        name: name.into(),
    }
}

/// The amount of the deadline version `as_of` selects, or the rendered
/// L3007 diagnostic.
fn amount_as_of(world: &World<'_>, name: &str, date: &str, known: Option<&str>) -> String {
    let at = litigate_sema::temporal::world::nowhere("query");
    match temporal::as_of(world, &key(name), date, known, at) {
        Ok(i) => match world.decls[i].decl {
            Declaration::Deadline(d) => format!("{}.days", d.amount),
            _ => "not a deadline".into(),
        },
        Err(d) => {
            let notes: Vec<&str> = d
                .labels
                .iter()
                .map(|l| l.message.as_str())
                .chain(d.notes.iter().map(|n| n.message.as_str()))
                .collect();
            format!("{} {} [{}]", d.code.as_str(), d.message, notes.join("; "))
        }
    }
}

#[test]
fn as_of_is_bitemporal_and_honours_supersession() {
    let body = v(P20, "effective [2009-12-01, 2016-12-01) known_at 2009-04-01")
        + &v(P21, "effective [2016-12-01, ..) known_at 2016-04-28")
        + "deadline r: 7.days from Served(complaint) counted_by c6\n\
           deadline q: 9.days from Served(complaint) counted_by c6 effective [2012-01-01, 2014-01-01) supersedes r\n\
           deadline u: 7.days from Served(complaint) counted_by c6\n\
           deadline w: 9.days from Served(complaint) counted_by c6 effective [2012-01-01, 2014-01-01) supersedes u revives u\n\
           deadline k: 7.days from Served(complaint) counted_by c6 known_at 2009-04-01\n\
           deadline k2: 9.days from Served(complaint) counted_by c6 effective [2012-01-01, ..) known_at 2011-06-01 supersedes k\n";
    let (db, ws) = demo(&body);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    with_world(&db, ws, |world| {
        let got: Vec<String> = [
            ("p", "2016-11-30", None),
            ("p", "2016-12-01", None),
            ("p", "2017-01-03", Some("2016-01-01")),
            ("p", "2017-01-03", Some("2016-04-28")),
            ("p", "2009-11-30", None),
            ("q", "2011-06-01", None),
            ("r", "2011-12-31", None),
            ("r", "2013-01-01", None),
            ("r", "2015-01-01", None),
            ("u", "2013-01-01", None),
            ("u", "2015-01-01", None),
            ("k", "2013-01-01", Some("2010-01-01")),
            ("k", "2013-01-01", None),
        ]
        .iter()
        .map(|(n, d, k)| {
            format!(
                "{n} as_of {d} known_at {k:?}: {}",
                amount_as_of(world, n, d, *k)
            )
        })
        .collect();
        insta::assert_debug_snapshot!(got);
    });
}

#[test]
fn pending_matters_follow_the_transition_and_new_matters_the_new_rule() {
    let body = v(P20, "effective [2009-12-01, 2016-12-01)")
        + &v(P21, "effective [2016-12-01, ..)")
        + TRANSITION_OK;
    let events = "test \"histories\" {
  given Served(complaint, Mail) on 2016-11-20
  given Answered(complaint) on 2016-12-12
  given Ordered() on 2016-12-02
  given Served(complaint, Personal) on 2017-01-04
}
";
    let (db, ws) = demo(&(body + events));
    assert_eq!(render(&db, ws), Vec::<String>::new());
    let checks = litigate_sema::check_workspace(&db, ws);
    let package = checks[0].package.as_ref().expect("compiles");
    let givens: Vec<(String, Expr)> = package
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Test(t) => Some(
                t.givens
                    .iter()
                    .map(|(e, d)| (d.clone(), e.clone()))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();
    let world = World::from_packages(&[package]);
    let ev = |i: usize| MatterEvent::on(&givens[i].0, &givens[i].1);
    let rule = |events: &[MatterEvent<'_>], at: usize| match governing(
        &world,
        &key("p"),
        events,
        at,
        None,
    ) {
        Ok(g) => {
            let Declaration::Deadline(d) = world.decls[g.version].decl else {
                return "not a deadline".to_owned();
            };
            let via = g
                .via
                .map_or("in force on the event date".to_owned(), |(t, arm)| {
                    format!("transition `{}` arm {arm}", world.decls[t].meta.id)
                });
            format!("{}.days ({via})", d.amount)
        }
        Err(e) => format!("{e:?}"),
    };
    let pending = [ev(0), ev(1)];
    let ordered = [ev(0), ev(2), ev(1)];
    let fresh = [ev(3)];
    let before = [ev(0)];
    insta::assert_debug_snapshot!([
        format!(
            "served 2016-11-20, answer 2016-12-12: {}",
            rule(&pending, 1)
        ),
        format!(
            "same, court ordered prior rules 2016-12-02: {}",
            rule(&ordered, 2)
        ),
        format!("served 2017-01-04: {}", rule(&fresh, 0)),
        format!("served 2016-11-20 itself: {}", rule(&before, 0)),
    ]);
}
