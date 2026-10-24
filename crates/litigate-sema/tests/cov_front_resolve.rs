//! Signatures, the module graph, and scope lookup (SPEC §4.3) on real
//! `.lit` sources: every L1002 path and the kind nouns of misused names.
mod common;
use common::{check_body, package, render, workspace};

const NODES: &str = "module shared.nodes\nterminal end: \"End\"\ngroup motions: \"Motions\"\n";

#[test]
fn misused_names_are_named_by_their_kind() {
    let body = "deadline dl: 1.days from Served(complaint, Mail) counted_by c6\n\
exception ex rebuts dflt when true\n\
transition tr amends dl { otherwise -> dl }\n\
rule duty rl { }\n\
default dflt when true => 1\n\
default uses-deadline when true => dl\n\
default uses-counting when true => c6\n\
default uses-calendar when true => cal\n\
default uses-exception when true => ex\n\
default uses-transition when true => tr\n\
default uses-rule when true => rl\n\
default uses-node when true => t\n\
state bad: \"Bad\" { _ \"go\" -> dflt }\n";
    insta::assert_debug_snapshot!(check_body(body));
}

#[test]
fn only_a_plain_fact_tag_makes_a_fact_node() {
    use litigate_sema::resolve::sig::ValueKind;
    let src = "module main\nstart a\n\
chance a: \"A\" by Court { tags [other, base.fact], go \"Go\" -> b p 1 }\n\
chance b: \"B\" by Court { tags [fact], go \"Go\" -> c p 1 }\n\
terminal c: \"C\"\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", src)])]);
    let file = ws.packages(&db)[0].files(&db)[0];
    let sig = litigate_sema::db::queries::signature(&db, file);
    let fact = |name: &str| match &sig.values[name].kind {
        ValueKind::Node { fact, .. } => Some(*fact),
        _ => None,
    };
    assert_eq!(
        (fact("a"), fact("b"), fact("c")),
        (Some(false), Some(true), Some(false))
    );
}

#[test]
fn every_value_kind_has_a_noun() {
    use litigate_sema::resolve::sig::ValueKind as K;
    let node = K::Node {
        chance: false,
        terminal: false,
        fact: false,
        edges: Vec::new(),
    };
    let kinds = [
        node,
        K::Deadline,
        K::Counting(std::collections::BTreeSet::new()),
        K::Calendar(false),
        K::Default(None),
        K::Exception(None),
        K::Transition,
        K::Rule,
        K::Instance(None),
    ];
    let nouns: Vec<&str> = kinds.iter().map(K::noun).collect();
    assert_eq!(
        nouns,
        [
            "node",
            "deadline",
            "counting rule",
            "calendar",
            "default",
            "exception",
            "transition",
            "rule",
            "instance"
        ]
    );
}

#[test]
fn a_name_declared_twice_keeps_its_first_kind() {
    let body = "state twice: \"Twice\"\ndefault twice when true => 1\n\
default same when true => 1\ndefault same when true => 2\n";
    insta::assert_debug_snapshot!(check_body(body));
}

#[test]
fn unknown_groups_and_authorities_are_unresolved() {
    let body = "authority Statute > FederalRule\n\
default d delegable_to [FederalRul] when true => 1\n\
state g: \"G\" { group motion }\n";
    insta::assert_debug_snapshot!(check_body(body));
}

#[test]
fn module_paths_and_whole_module_imports() {
    let main = "module main\nimport shared.nodes\nstart s\n\
state s: \"S\" { group motions, _ \"a\" -> shared.nodes.ending, _ \"b\" -> nd, _ \"c\" -> shared.nodes.end }\n\
default inst when true => base@copy\ndefault member when true => base@copy.done\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[("main.lit", main), ("nodes.lit", NODES)],
    )]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn unknown_imports_suggest_visible_modules_only() {
    let main = "module main\nimport shared.node\nimport hidden.nodes\nstart s\nstate s: \"S\"\n";
    let (db, ws) = workspace(&[
        package("demo", &["dep"], &[("main.lit", main)]),
        package("dep", &[], &[("nodes.lit", NODES)]),
        package(
            "hidden",
            &[],
            &[("hidden.lit", "module hidden.node\nstate h: \"H\"\n")],
        ),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn qualified_names_need_one_segment_and_a_declared_instance() {
    let base = "module base\nstart entry\nstate entry: \"E\" { go \"Go\" -> done }\nterminal done: \"D\"\n";
    let links = "module links\n\
link deep \"Deep\": base::done::x -> base::entry\n\
link mixed \"Mixed\": base::done.x -> base::entry\n\
link undeclared \"Undeclared\": base@nope::done -> base::entry\n\
link packless \"Packless\": base@bare::done -> base::entry\n\
instance base @ bare { note \"no pack\" }\n\
instance base @ blank { pack }\n";
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", base)]),
        package("links", &["base"], &[("links.lit", links)]),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}
