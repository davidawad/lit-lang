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
  patch go { note \"n\", event Go(), replaces [base::go], within nowhere, deadline 3.days, effort 1.usd }\n\
  patch go { note \"again\" }\n\
  patch go\n}\n\
instance base@two { pack nope }\n\
instance base@three { note \"no pack\" }\n",
    );
    insta::assert_debug_snapshot!(out);
}

/// A `payoff_transform` is a `Money<usd>` expression over `payoff` and a
/// declared `stake`, from literals, `+ - * /` and unary `-`.
#[test]
fn payoff_transforms_are_typed_money_expressions() {
    let rows = [
        (
            "affine",
            "(-(stake - payoff) * 2 / 4 + 1_000.usd)",
            "stake 5.usd",
        ),
        ("undeclared stake", "(stake - payoff)", ""),
        ("unread stake", "(payoff)", "stake 5.usd"),
        ("another name", "(payoff * rate)", ""),
        ("qualified name", "(base::payoff)", ""),
        ("a call", "(max(payoff, 0.usd))", ""),
        ("a comparison", "(payoff > 0.usd)", ""),
        ("negation", "(!payoff)", ""),
        ("number, not money", "(1000 - payoff)", ""),
        ("a ratio, not money", "(payoff / payoff)", ""),
    ];
    let table: Vec<(&str, Vec<String>)> = rows
        .iter()
        .map(|(name, transform, stake)| {
            let text = format!(
                "module links\ninstance base@one {{ pack base, payoff_transform {transform} {stake} }}\n"
            );
            (*name, links(&text))
        })
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn instance_patches_lower_to_edge_fields() {
    let text = "module links\ninstance base@one { pack base, roles { A: self, B: opponent, C: nature }, payoff_transform (stake - payoff), stake 5.usd, patch go { effort 2.hours, cost ?, deadline 3.days { note \"n\" }, valence good, tags [], attrs { w: 1 }, sets [f] } }\ninstance base@two { pack base, payoff_transform (stake - payoff), stake ?(\"the matter's\") }\n";
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

/// A link carries `p` only out of a draw: a chance node or a terminal,
/// never a decision or a state (SPEC §5.1, L2008); nor a `when` guard.
#[test]
fn link_probabilities_leave_only_draws() {
    let base = "module base\nstart entry\nstate entry: \"E\" { go \"Go\" -> draw }\n\
decision pick: \"P\" { go \"Go\" -> done }\n\
chance draw: \"R\" { a \"A\" -> done p 1 }\nterminal done: \"D\"\n";
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", base)]),
        package(
            "links",
            &["base"],
            &[(
                "links.lit",
                "module links\n\
link a \"A\": base::draw -> base::entry p 0\n\
link b \"B\": base::done -> base::entry p 1\n\
link c \"C\": base::pick -> base::entry p 1\n\
link d \"D\": base::entry -> base::done p 1\n\
link e \"E\": base::done -> base::entry { when true }\n",
            )],
        ),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}
