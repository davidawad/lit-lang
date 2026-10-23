//! The temporal and precedence checks over worlds without source: a
//! [`NoSource`] locator (no clause spans, so no fixes), HIR whose spans
//! were stripped (declaration placeholders, rules found by name), a
//! package checked from a position other than first, and HIR edits the
//! front end never produces.
mod cov_rules_support;
mod legal;
use cov_rules_support::{demo, edit, lines, packs, strip, world};
use legal::pkg;
use litigate_sema::hir::{Declaration, Expr, Relation};
use litigate_sema::precedence::{self, conclude, defeasible, Conclusion};
use litigate_sema::temporal::world::NoSource;
use litigate_sema::temporal::Key;
use std::fmt::Write as _;

const BODY: &str = "authority A > B
authority B > A
deadline p: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, ..)
deadline p: 21.days from Served(complaint) counted_by c6 effective [2012-01-01, 2014-01-01)
deadline g: 1.days from Served(complaint) counted_by c6 effective [2009-12-01, 2011-01-01)
deadline g: 2.days from Served(complaint) counted_by c6 effective [2011-01-01, 2012-01-01)
deadline v: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline v: 21.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
transition tr amends v@2016-12-01 { unless 1 < 2 -> v@2009-12-01, otherwise -> v@2016-12-01 }
deadline tg: 1.days from Served(complaint) counted_by c6
deadline a: 2.days from Served(complaint) counted_by c6 notwithstanding tg
deadline b: 3.days from Served(complaint) counted_by c6 notwithstanding tg
default d when has_event(Ordered()) => 1
exception e1 rebuts d when has_event(Answered(complaint)) => 2
exception e2 rebuts d when has_event(Hearing(motion)) => 3 priority 1
exception e3 rebuts d when has_event(Served(complaint, Mail)) => 4 priority 1
deadline brief: 14.days before Hearing(motion) counted_by c6
state w: \"W\" { file \"File\" -> t { within brief } }
rule duty empty {
  state x: \"X\" { _ \"go\" -> t }
}
rule power allow {
  decision m: \"Move?\" by Defendant { _ \"yes\" -> t }
}
";

#[test]
fn without_source_spans_fall_back_and_fixes_disappear() {
    let spanned = demo(BODY);
    let stripped = strip(&spanned);
    let with_spans = lines(&precedence::check(&world(&spanned), &NoSource, 0));
    let without = lines(&precedence::check(&world(&stripped), &NoSource, 0));
    insta::assert_debug_snapshot!((with_spans, without));
}

#[test]
fn rules_are_found_by_name_without_spans() {
    let stripped = strip(&demo(BODY));
    let w = world(&stripped);
    let inside: Vec<String> = w
        .decls
        .iter()
        .filter_map(|d| d.rule.map(|(r, k)| format!("{} in {r} ({k:?})", d.meta.id)))
        .collect();
    assert_eq!(inside, ["x in empty (Duty)", "m in allow (Power)"]);
}

fn rules(name: &str, deps: &[&str], body: &str) -> litigate_sema::pkg::load::LoadedPackage {
    let imports = deps.iter().fold(String::new(), |mut s, d| {
        let _ = writeln!(s, "import {d}.rules.{{tg, d, c6}}");
        s
    });
    let text = format!(
        "module {name}.rules\n{imports}start s\nenum Document {{ complaint }}\nevent Served(document: Document)\nevent Ordered()\nstate s: \"S\"\n{body}\n"
    );
    pkg(
        name,
        "FederalRule",
        "2009-12-01",
        deps,
        &[("rules.lit", &text)],
    )
}

/// A package checked from position 1, after its dependency: pairs whose
/// later member is its own are still its own.
#[test]
fn a_package_checked_after_its_dependency_owns_pairs_it_closes() {
    let dep = "counting c6 { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n\
        deadline tg: 1.days from Served(complaint) counted_by c6\n\
        deadline a: 2.days from Served(complaint) counted_by c6 notwithstanding tg\n\
        default d when has_event(Ordered()) => 1\n\
        exception e0 rebuts d when has_event(Ordered()) => 2";
    let main = "deadline b: 3.days from Served(complaint) counted_by c6 notwithstanding tg\n\
        exception e1 rebuts d when has_event(Ordered()) => 3";
    let packs = packs(&[rules("x", &[], dep), rules("y", &["x"], main)]);
    let w = world(&packs);
    insta::assert_debug_snapshot!(lines(&precedence::check(&w, &NoSource, 1)));
}

fn key(name: &str) -> Key {
    Key {
        pack: "demo".into(),
        name: name.into(),
    }
}

/// Every `has_event` atom is true; any other atom is unknown.
fn all_events(e: &Expr) -> Option<bool> {
    litigate_sema::temporal::condition::has_event(e).map(|_| true)
}

#[test]
fn conclusions_report_what_stops_evaluation() {
    let body = "default d when has_event(Ordered()) => 1
exception e1 rebuts d when has_event(Answered(complaint)) => 2
exception e2 rebuts d when has_event(Hearing(motion)) => 3
default u when has_event(Ordered()) => 1
exception odd undercuts u when 1 < 2
default late when has_event(Ordered()) => 1
rule duty later effective [2012-01-01, ..) {
  deadline z: 1.days from Served(complaint) counted_by c6
  exception soon rebuts late when has_event(Ordered()) => 2
}
exception x1 rebuts x2 when has_event(Ordered()) => 2
exception x2 rebuts x1 when has_event(Ordered()) => 3
deadline dl: 1.days from Served(complaint) counted_by c6
";
    let packs = demo(body);
    let w = world(&packs);
    let show = |name: &str, date: &str| {
        let got = match conclude(&w, &key(name), date, &mut all_events) {
            Ok(Conclusion::Value { value, by }) => {
                format!("{:?} via {}", value.kind, by.join(" > "))
            }
            Ok(other) => format!("{other:?}"),
            Err(e) => e,
        };
        format!("{name} on {date}: {got}")
    };
    insta::assert_debug_snapshot!([
        show("d", "2009-01-01"),
        show("d", "2010-01-01"),
        show("u", "2010-01-01"),
        show("late", "2010-01-01"),
        show("x1", "2010-01-01"),
        show("dl", "2010-01-01"),
    ]);
    assert_eq!(defeasible::exceptions_on(&w, &key("d")).len(), 2);
}

/// An `override` whose target resolves to no declaration (the front end
/// reports it first, L1002) draws no delegation diagnostic.
#[test]
fn an_override_of_nothing_is_left_to_name_resolution() {
    let mut packs = demo("deadline o: 1.days from Served(complaint) counted_by c6 override s");
    edit(&mut packs, |d| {
        if let Declaration::Deadline(x) = d {
            for r in &mut x.meta.relations {
                if let Relation::Override(t) = r {
                    "nowhere".clone_into(t);
                }
            }
        }
    });
    let w = world(&packs);
    assert_eq!(
        lines(&precedence::check(&w, &NoSource, 0)),
        Vec::<String>::new()
    );
}

/// An `authority` declaration ranking nothing (never written by the front
/// end, which lowers `A > B > C` to `A > [B, C]` and `B > [C]`) adds no
/// rank.
#[test]
fn an_authority_ranking_nothing_adds_no_rank() {
    let mut packs = demo("authority A > A");
    assert_eq!(world(&packs).ranks.len(), 1);
    edit(&mut packs, |d| {
        if let Declaration::Authority(a) = d {
            a.outranks.clear();
        }
    });
    let w = world(&packs);
    assert!(w.ranks.is_empty());
    assert_eq!(
        lines(&precedence::check(&w, &NoSource, 0)),
        Vec::<String>::new()
    );
}
