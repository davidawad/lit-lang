//! Stable declaration identity (SPEC §4.3): a node or edge renamed with
//! `formerly` keeps its identity, so a matter written against the old id
//! still resolves (L9007 with the fix), and a former id that collides is
//! L1001.
#![allow(clippy::unwrap_used)]
mod cov_case_support;
use cov_case_support::{check, compile, lock, matter, package, render, At};
use litigate_base::Profile;

const RENAMED: &str = "module alpha
start intake
enum Document { complaint }
event Served(document: Document)
state intake: \"Intake\" by Plaintiff {
  formerly [start-node]
  serve \"Serve\" -> responded { event Served(complaint) formerly [served] }
  _ \"Drop\" -> responded
}
chance ruling: \"Ruling\" {
  formerly [old-ruling, older-ruling] tags [fact]
  won \"Won\" -> responded p 1/2 { formerly [granted] }
  lost \"Lost\" -> responded p 1/2
}
terminal responded: \"Responded\" { formerly [answered] }
";

fn renamed() -> Vec<litigate_sema::hir::Package> {
    compile(&[package("alpha", &[], "", RENAMED)])
}

#[test]
fn formerly_lowers_to_hir_on_nodes_and_edges() {
    let pkgs = renamed();
    let nodes: Vec<(String, Vec<String>, Vec<Vec<String>>)> = pkgs[0].modules[0]
        .declarations
        .iter()
        .filter_map(|d| match d {
            litigate_sema::hir::Declaration::Node(n) => Some((
                n.meta.id.clone(),
                n.formerly.clone(),
                n.edges.iter().map(|e| e.formerly.clone()).collect(),
            )),
            _ => None,
        })
        .collect();
    insta::assert_debug_snapshot!(nodes);
}

#[test]
fn a_matter_using_old_ids_resolves_with_a_fix() {
    let pkgs = renamed();
    let text = matter(
        "alpha ^1.0.0",
        "",
        &[],
        "facts { old-ruling: granted }\nstakes { alpha::answered: 0.usd }\nbeliefs { observe older-ruling: { granted: 2, lost: 1 } }",
    );
    let checked = check(
        &text,
        &pkgs,
        Some(&lock(&pkgs)),
        &At("2011-01-01"),
        Profile::Strict,
    );
    insta::assert_debug_snapshot!(render(&checked.diagnostics));
    let case = checked.case.unwrap();
    assert_eq!(
        case.facts.get("alpha::ruling").map(String::as_str),
        Some("alpha::won")
    );
    assert_eq!(case.observations[0].outcomes[0].0, "alpha::won");
}

#[test]
fn a_former_id_cannot_be_current_or_claimed_twice() {
    let mut db = litigate_sema::LitDatabase::default();
    let text = "module alpha
start a
state a: \"A\" { formerly [b, gone, a] go \"Go\" -> b { formerly [stay] } stay \"Stay\" -> a }
state c: \"C\" { formerly [gone] x \"X\" -> b { formerly [y] } y2 \"Y\" -> b { formerly [y] } }
terminal b: \"B\"
";
    let ws =
        litigate_sema::db::inputs::create_workspace(&mut db, &[package("alpha", &[], "", text)]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let messages: Vec<String> = checks[0]
        .diagnostics
        .iter()
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    insta::assert_debug_snapshot!(messages);
}

#[test]
fn an_anonymous_edge_has_no_former_id() {
    let mut db = litigate_sema::LitDatabase::default();
    let text = "module alpha\nstart a\nstate a: \"A\" { _ \"Go\" -> b { formerly [go] } }\nterminal b: \"B\"\n";
    let ws =
        litigate_sema::db::inputs::create_workspace(&mut db, &[package("alpha", &[], "", text)]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let messages: Vec<String> = checks[0]
        .diagnostics
        .iter()
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    insta::assert_debug_snapshot!(messages);
}

#[test]
fn a_lock_pinning_a_version_that_is_not_loaded_is_stale() {
    let pkgs = renamed();
    let mut pinned = lock(&pkgs);
    pinned.packages[0].version = "1.5.0".into();
    let text = matter("alpha ^1.0.0", "", &[], "stakes { responded: 0.usd }");
    let checked = check(&text, &pkgs, Some(&pinned), &At("2011-01-01"), Profile::Dev);
    insta::assert_debug_snapshot!(render(&checked.diagnostics));
}
