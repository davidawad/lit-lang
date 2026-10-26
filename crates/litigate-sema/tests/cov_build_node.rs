//! Nodes and edges lowered by `types::build` (SPEC §5.1, §6 field types),
//! one diagnostic path per line of source.
mod common;
use common::{check_body, package, workspace};
use litigate_sema::hir::Declaration;

/// The node declarations named in `ids` of an error-free package.
fn nodes(body: &str, ids: &[&str]) -> Vec<Declaration> {
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    assert_eq!(common::render(&db, ws), Vec::<String>::new());
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter().cloned())
        .filter(|d| matches!(d, Declaration::Node(n) if ids.contains(&n.meta.id.as_str())))
        .collect()
}

#[test]
fn node_and_edge_fields_lower() {
    let out = nodes(
        "group g: \"G\"\n\
deadline dl: 3.days from Served(complaint) counted_by c6\n\
state rich: \"Rich\" by Plaintiff {\n\
  group g\n  valence caution\n  note \"n\"\n  court_listener_url \"u\"\n  attrs { weight: -1.5 }\n\
  a \"A\" -> done { within dl { extendable false }, effort 2.hours, cost 3.usd, event Served(complaint), duration 13.5.days, valence bad }\n\
  b \"B\" -> done by either { deadline 5.court_days { extendable true, extension_authority \"Court\", counting \"frap26\" }, duration { min 1.days, mode 2.5.days }, action_id \"x\", note \"m\", tags [t], sets [s], clears [c], requires [r], forbids [f], authority cite \"FRCP 6(a)\" }\n\
}\n\
terminal done: \"Done\" outcome [won] { valence neutral, payoff ?(\"tbd\"), payoff_by_flag { won: 5.usd, lost: -2.usd } }\n\
state good: \"Good\" { valence good }",
        &["rich", "done", "good"],
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn node_fields_report_placement_values_and_duplicates() {
    let out = check_body(
        "state v1: \"V\" { valence zzzzzz }\n\
state v2: \"V\" { valence good.x }\n\
state a1: \"A\" { attrs { a: 1, a: 2, b: 99999999999999999999 } }\n\
state p1: \"P\" discretion outcome [x] { payoff 1.usd, payoff_by_flag { a: 1.usd } }\n\
terminal p2: \"P\" { payoff_by_flag { a: 1.005.usd, b: 2.usd } }\n\
state f1: \"F\" { tags [fact] }\n\
state f2: \"F\" { tags [other] }\n\
state f3: \"F\" { tags [fact]\n  a \"A\" -> t p 1/2, b \"B\" -> t p 1/2 }\n\
chance f4: \"F\" { tags [fact]\n  a \"A\" -> t p 1 }\n\
state g1: \"G\" { group nope }\n\
state e1: \"E\" { a \"A\" -> t, a \"B\" -> t }\n\
deadline dl: 3.days from Served(complaint) counted_by c6\n\
state w1: \"W\" { a \"A\" -> t { within dl { counting \"frap26\" } } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn chance_probabilities_sum_to_one_exactly() {
    let out = check_body(
        "chance empty: \"E\" {}\n\
chance holed: \"H\" { a \"A\" -> t p ?, b \"B\" -> t p 1/2, c \"C\" -> t p 3/4 }\n\
chance under: \"U\" { a \"A\" -> t p ?, b \"B\" -> t p 1/2 }\n\
chance huge: \"O\" { a \"A\" -> t p 1/4294967291, b \"B\" -> t p 1/4294967279, c \"C\" -> t p 1/3 }\n\
chance loop: \"L\" { a \"A\" -> loop, b \"B\" -> t p 1 }",
    );
    insta::assert_debug_snapshot!(out);
}

/// Edges gated by `requires`/`forbids` are a distribution per flag
/// context (lg-teqz): `ok` draws one distribution in each, `both` sees two
/// at once when `a` and `b` are both set, `bare` none where neither is,
/// `holed` overflows only where `a` is set, `sparse` enables nothing where
/// `a` is unset (not checked), and `wide` gates on too many flags.
#[test]
fn gated_chance_probabilities_sum_to_one_per_flag_context() {
    let out = check_body(
        "chance ok: \"K\" { a \"A\" -> t p 1/2 { requires [a] }, b \"B\" -> t p 1/2 { requires [a] }, c \"C\" -> t p 1 { forbids [a] } }\n\
chance both: \"B\" { a \"A\" -> t p 1 { requires [a] }, b \"B\" -> t p 1 { requires [b] }, c \"C\" -> t p 1 { forbids [a, b] } }\n\
chance bare: \"N\" { a \"A\" -> t p 1 { requires [a] }, c \"C\" -> t p 1/2 { forbids [a] } }\n\
chance holed: \"H\" { a \"A\" -> t p ? { requires [a] }, b \"B\" -> t p 1 { requires [a] }, c \"C\" -> t p 1/2, d \"D\" -> t p 1/2 { forbids [a] } }\n\
chance sparse: \"S\" { a \"A\" -> t p 1 { requires [a] } }\n\
chance wide: \"W\" { a \"A\" -> t p 1 { requires [g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12] } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn edge_probabilities_are_constants_in_the_unit_interval() {
    let out = check_body(
        "chance s1: \"S\" { a \"A\" -> t p (prob({ Personal: 1/2, Mail: 1/2 }, Mail)) }\n\
chance s2: \"S\" { a \"A\" -> t p (1 == 1) }\n\
chance s3: \"S\" { a \"A\" -> t p (nope) }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn edge_fields_report_their_problems() {
    let out = check_body(
        "deadline dl: 3.days from Served(complaint) counted_by c6\n\
state s1: \"S\" { a \"A\" -> t { replaces [x] } }\n\
state s2: \"S\" { a \"A\" -> t { within dl, deadline 3.days } }\n\
state s3: \"S\" { a \"A\" -> t { within 3 } }\n\
state s4: \"S\" { a \"A\" -> t { deadline 3 } }\n\
state s5: \"S\" { a \"A\" -> t { within c6 } }\n\
state s6: \"S\" { a \"A\" -> t { duration 99999999999999999999.5.days } }\n\
state s7: \"S\" { a \"A\" -> t { duration 3.hours } }\n\
state s8: \"S\" { a \"A\" -> t { duration { min 3.hours, mode 4.days } } }\n\
state s9: \"S\" { a \"A\" -> t { duration { min 3.days } } }\n\
state s10: \"S\" { a \"A\" -> t { duration { mode 3.days, max 2.days } } }\n\
state s11: \"S\" { a \"A\" -> t { effort 3.days, event Nope() } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn fact_guards_are_typed_placed_and_disjoint() {
    let many: Vec<String> = (0..17).map(|i| format!("has_event(E{i}())")).collect();
    let events = (0..17)
        .map(|i| format!("event E{i}()\n"))
        .collect::<Vec<_>>()
        .concat();
    let body = String::from(
        "deadline dl: 3.days from Served(complaint) counted_by c6\n\
chance ok: \"Ok\" { tags [fact]\n  y \"Y\" -> t p 1/2 { when (date_of(Served(complaint, Mail)) <= deadline(dl)) }\n  n \"N\" -> t p 1/2 { when !(date_of(Served(complaint, Mail)) <= deadline(dl)) } }\n\
chance plain: \"P\" { y \"Y\" -> t p 1 { when true } }\n\
state st: \"S\" { y \"Y\" -> t { when true } }\n\
chance anon: \"A\" { tags [fact]\n  _ \"Y\" -> t p 1 { when true } }\n\
chance foreign: \"F\" { tags [fact]\n  y \"Y\" -> t p 1 { when (1 < 2) } }\n\
chance typed: \"T\" { tags [fact]\n  y \"Y\" -> t p 1 { when 3 } }\n\
chance both: \"B\" { tags [fact]\n  y \"Y\" -> t p 1/2 { when has_event(Served(complaint, Mail)) }\n  n \"N\" -> t p 1/2 { when true } }\n\
chance odd: \"O\" { tags [fact]\n  y \"Y\" -> t p 1 { when has_event(3) } }\n\
chance bare: \"B\" { tags [fact]\n  y \"Y\" -> t p 1 { when } }\n",
    ) + &events
        + "chance wide: \"W\" { tags [fact]\n  y \"Y\" -> t p 1/2 { when "
        + &many.join(" && ")
        + " }\n  n \"N\" -> t p 1/2 { when false } }";
    insta::assert_debug_snapshot!(check_body(&body));
}

#[test]
fn information_sets_offer_one_choice() {
    let out = check_body(
        "decision a: \"A\" by Plaintiff info filing { x \"File\" -> t, y \"Wait\" -> t }\n\
decision b: \"B\" by Plaintiff info filing { y \"Wait\" -> t, x \"File\" -> t }\n\
decision c: \"C\" by Plaintiff info filing { x \"File\" -> t }\n\
decision d: \"D\" by Defendant info filing { x \"File\" -> t, y \"Wait\" -> t }\n\
decision e: \"E\" by Plaintiff info other { x \"File\" -> t }\n\
state f: \"F\" info filing { x \"File\" -> t }",
    );
    insta::assert_debug_snapshot!(out);
    let info = nodes(
        "decision a: \"A\" by Plaintiff info filing { x \"File\" -> t }",
        &["a"],
    );
    assert!(matches!(&info[..], [Declaration::Node(n)] if n.info.as_deref() == Some("filing")));
}

#[test]
fn an_extension_authority_hole_leaves_it_undeclared() {
    let out = nodes(
        "deadline dl: 3.days from Served(complaint) counted_by c6\n\
state open: \"S\" by Plaintiff {\n\
  a \"A\" -> done { within dl { extension_authority ?(\"unverified\") } }\n\
}\n\
terminal done: \"Done\" outcome [won]",
        &["open"],
    );
    let [Declaration::Node(n)] = out.as_slice() else {
        panic!("{out:?}");
    };
    let e = &n.edges[0];
    let d = e.deadline.as_ref().unwrap();
    assert_eq!(
        (d.extendable, d.extension_authority.as_deref()),
        (None, None)
    );
    assert_eq!(
        e.holes.get("within.extension_authority"),
        Some(&Some("unverified".to_owned()))
    );
}
