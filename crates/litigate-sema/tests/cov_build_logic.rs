//! Defaults, exceptions, tests, types, links, instances, and declaration
//! metadata built by `types::build` (SPEC §5.2, §5.5, §6, §8.4).
mod common;
use common::{check_body, package, render, workspace};
use litigate_sema::hir::Declaration;

#[test]
fn exceptions_check_target_conclusion_and_priority() {
    let out = check_body(
        "default base when true => 1\n\
exception ok rebuts base when true => 2 priority 1\n\
exception chained undercuts ok when false\n\
exception e1 rebuts c6 when true => 1\n\
exception e2 undercuts base when true => 1\n\
exception e3 rebuts base when true\n\
exception e4 rebuts base when true => 1 priority 1.5\n\
exception e5 rebuts base when true => 1 priority 99999999999\n\
exception e6 rebuts nope when true => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn test_blocks_take_events_and_comparisons() {
    let out = check_body(
        "test \"t\" cite FRCP 6 {\n\
  given 3 on 2025-01-01\n\
  expect true\n\
  expect nope\n\
}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn type_aliases_enums_and_events_are_well_formed() {
    let out = check_body(
        "type A = B\ntype B = Money<usd>\ntype C = D\ntype D = C\ntype L = Dist<Method>\ntype U = Nope\n\
enum Twice { a, a }\nenum Empty {}\n\
event Dup(x: Method, x: Method)\nevent Bad(x: Nope)",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn rule_kinds_lower() {
    let text = format!(
        "{}rule liability l {{\n  terminal lost: \"Lost\"\n}}\nrule power p {{\n  decision m: \"Move?\" by Defendant {{ _ \"yes\" -> t }}\n}}\n",
        common::PRELUDE
    );
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    let rules: Vec<_> = litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter())
        .filter_map(|d| match d {
            Declaration::Rule(r) => Some((r.meta.id.clone(), r.rule_kind, r.declarations.clone())),
            _ => None,
        })
        .collect();
    insta::assert_debug_snapshot!(rules);
}

const BASE: &str = "module base\nstart entry\nstate entry: \"E\" { go \"Go\" -> done, _ \"Anon\" -> done }\nterminal done: \"D\"\ndefault fact when true => 1\n";

fn links(text: &str) -> Vec<String> {
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", BASE)]),
        package("links", &["base"], &[("links.lit", text)]),
    ]);
    render(&db, ws)
}

#[test]
fn links_replace_only_existing_edges() {
    let out = links(
        "module links\n\
link a \"A\": base::done -> base::entry { replaces [base::go, base, nope::go, base::gone, base::fact.x, \"base::entry->done#1\", \"nope\", \"base::entry\", \"base::nowhere->done#1\", \"base::fact->done#1\", \"zz::entry->done#1\"] }\n\
link b \"B\": nope::x -> base::entry by Whoever\n",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn instances_check_pack_probabilities_roles_and_patches() {
    let out = links(
        "module links\n\
instance base@one { pack base, probabilities { go: 1/3, bad: 3/2 }, roles { A: self, B: opponent, C: nature, D: judge }, note \"n\", remove_edges [go], payoff_transform \"x\",\n\
  patch go { \"a\": 1.5, \"b\": \"s\", \"c\": true, \"d\": [1, false], \"e\": { \"f\": unset }, \"g\": 1_000 }\n\
  patch go\n}\n\
instance base@two { pack nope }\n\
instance base@three { note \"no pack\" }\n",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn instance_patches_lower_to_json() {
    let text = "module links\ninstance base@one { pack base, roles { A: self, B: opponent, C: nature }, patch go { \"a\": 1.5, \"b\": \"s\", \"c\": true, \"d\": [1, false], \"e\": { \"f\": unset } } }\n";
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", BASE)]),
        package("links", &["base"], &[("links.lit", text)]),
    ]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    let instances: Vec<_> = litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter())
        .filter(|d| matches!(d, Declaration::Instance(_)))
        .cloned()
        .collect();
    insta::assert_debug_snapshot!(instances);
}
