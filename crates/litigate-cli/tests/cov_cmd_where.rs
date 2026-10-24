//! `litigate where`: search roots, relations and exceptions that override
//! an id, clock and links lowering, version filters, cite fallback,
//! ambiguity, and every failure.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::json;
use support_cmds::{in_fixture, litigate_in, repo, scratch};

#[test]
fn without_path_or_lib_and_no_lit_directory_there_is_nothing_to_search() {
    let run = litigate_in(&scratch("where-empty"), &["where", "answer-period"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("where_nothing_to_search", run.stderr);
}

#[test]
fn a_missing_path_is_a_failure() {
    let run = in_fixture(&["where", "answer-period", "--path", "no/such/dir"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("no/such/dir"), "{}", run.stderr);
}

#[test]
fn notwithstanding_relations_override_a_deadline() {
    let run = in_fixture(&["where", "answer-period", "--path", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["kind"], "deadline");
    assert_eq!(result["overrides"][0]["id"], "reply-period");
    assert_eq!(result["overrides"][0]["relation"], "notwithstanding");
    assert_eq!(result["overrides"].as_array().unwrap().len(), 1);
    insta::assert_json_snapshot!("where_overridden_deadline", result);
}

#[test]
fn an_exception_overrides_its_default() {
    let run = in_fixture(&["where", "cov-example::answer-days", "--path", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["id"], "cov-pack::answer-days");
    assert_eq!(result["kind"], "default");
    insta::assert_json_snapshot!("where_default_with_exception", result["overrides"]);
}

#[test]
fn a_grant_is_found_by_its_id() {
    let run = in_fixture(&["where", "emergency", "--path", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["kind"], "grant");
}

#[test]
fn a_counting_rule_lowers_into_the_clock_file() {
    let run = in_fixture(&["where", "frcp6", "--path", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        run.json["result"]["lowered"],
        json!(["clock:/countingRules/0"])
    );
    // The rule's cite is located like a node's (SPEC §8.6).
    let source = &run.json["result"]["source"];
    assert_eq!(source["text"], json!("FRCP 6(a)"));
    assert_eq!(source["sections"][0]["heading"], json!("6"));
}

#[test]
fn a_cited_node_reports_its_l0_sections() {
    let run = in_fixture(&["where", "time-barred", "--path", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["lowered"], json!(["/nodes/0"]));
    insta::assert_json_snapshot!("where_cited_node_source", run.json["result"]["source"]);
}

#[test]
fn a_version_date_selects_the_versions_in_force() {
    let on = in_fixture(&[
        "where",
        "cov-pack::answer-period@2010-01-01",
        "--path",
        "lit/cov",
    ]);
    assert_eq!(on.json["result"]["versions"].as_array().unwrap().len(), 1);
    let before = in_fixture(&["where", "answer-period@2001-01-01", "--path", "lit/cov"]);
    assert_eq!(before.code, 0, "{}", before.stdout);
    assert_eq!(before.json["result"]["versions"], json!([]));
    assert!(before.json["result"]["declaration_span"].is_null());
    // Ten characters after `@` but not a date: part of the id.
    let not_a_date = in_fixture(&["where", "answer-period@abcdefghij", "--path", "lit/cov"]);
    assert_eq!(not_a_date.code, 1, "{}", not_a_date.stdout);
}

#[test]
fn a_pack_prefix_that_names_no_package_finds_nothing() {
    let run = in_fixture(&["where", "other-pack::answer-period", "--path", "lit/cov"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("where_names_nothing", run.stderr);
}

#[test]
fn an_id_in_two_packages_is_ambiguous() {
    let run = litigate_in(
        &repo(),
        &[
            "where",
            "answer-period",
            "--path",
            "lit/examples",
            "--path",
            "crates/litigate-cli/tests/fixtures/cov-cmds/lit/cov",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["kind"], "ambiguous");
    let ids: Vec<&str> = result["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "cov-pack::answer-period",
            "frcp-rule12-example::answer-period"
        ]
    );
}

/// `lit/examples` as `frcp/` plus a links package instancing it, with
/// `patch` (an instance field) when given.
fn links_root(name: &str, patch: &str) -> std::path::PathBuf {
    let root = scratch(name);
    support_cmds::copy_dir(&repo().join("lit/examples"), &root.join("frcp"));
    std::fs::create_dir_all(root.join("links")).unwrap();
    std::fs::write(
        root.join("links/Lit.toml"),
        "[package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\n\
         in_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n[dependencies]\n\
         frcp-rule12-example = { version = \"^0.1.0\", path = \"../frcp\" }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("links/links.lit"),
        format!("module links\n\ninstance frcp-rule12-example@copy {{ pack frcp-rule12-example{patch} }}\n"),
    )
    .unwrap();
    root
}

#[test]
fn an_instance_lowers_into_the_links_file() {
    let root = links_root("where-links", "");
    let run = litigate_in(&root, &["where", "frcp-rule12-example@copy", "--path", "."]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["kind"], "instance");
    assert_eq!(result["package"], "links");
    assert_eq!(
        result["lowered"],
        json!(["/instances/frcp-rule12-example@copy"])
    );
}

#[test]
fn a_package_that_does_not_lower_has_no_pointers() {
    let root = links_root("where-bad-links", ", patch nowhere { note \"x\" }");
    let run = litigate_in(&root, &["where", "frcp-rule12-example@copy", "--path", "."]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["kind"], "instance");
    assert_eq!(result["lowered"], json!([]));
    assert!(result["source"].is_null());
}
