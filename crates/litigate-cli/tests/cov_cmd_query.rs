//! `litigate query`: every kind, every filter, paging cursors, packages
//! that do not load, and every failure.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use support_cmds::{in_fixture, litigate_in, scratch};

/// `query --path lit/cov ARGS`.
fn query(args: &[&str]) -> Value {
    let mut all = vec!["query", "--path", "lit/cov"];
    all.extend_from_slice(args);
    let run = in_fixture(&all);
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"].clone()
}

/// The ids of a result's matches.
fn ids(result: &Value) -> Vec<String> {
    result["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn without_path_or_lib_and_no_lit_directory_there_is_nothing_to_search() {
    let run = litigate_in(&scratch("query-empty"), &["query", "nodes"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("query_nothing_to_search", run.stderr);
}

#[test]
fn a_missing_path_is_a_failure() {
    let run = in_fixture(&["query", "nodes", "--path", "no/such/dir"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("no/such/dir"), "{}", run.stderr);
}

#[test]
fn a_cursor_that_is_not_a_number_is_a_failure() {
    let run = in_fixture(&["query", "nodes", "--path", "lit/cov", "--cursor", "next"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("query_bad_cursor", run.stderr);
}

#[test]
fn a_package_that_does_not_load_contributes_nothing() {
    let dir = scratch("query-unloaded");
    std::fs::create_dir_all(dir.join("bad")).unwrap();
    std::fs::write(dir.join("bad/Lit.toml"), "[package]\nname = \"bad\"\n").unwrap();
    std::fs::write(dir.join("bad/x.lit"), "module x\n").unwrap();
    let run = litigate_in(&dir, &["query", "nodes", "--path", "bad"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        run.json["result"],
        json!({"matches": [], "total": 0, "next_cursor": null})
    );
}

#[test]
fn rules_list_their_kind_and_deadlines() {
    let result = query(&["rules"]);
    assert_eq!(result["total"], 2);
    insta::assert_json_snapshot!("query_rules", result);
}

#[test]
fn versions_are_keyed_by_their_start_date() {
    let result = query(&[
        "versions",
        "--match",
        "period",
        "--effective-on",
        "2010-01-01",
    ]);
    assert_eq!(
        ids(&result),
        [
            "cov-pack::amended-answer-period@2009-12-01",
            "cov-pack::answer-period@2009-12-01",
            "cov-pack::brief-period@2009-12-01",
            "cov-pack::reply-period@2009-12-01",
        ]
    );
}

#[test]
fn overrides_list_overriding_relations_only() {
    let result = query(&["overrides"]);
    assert_eq!(ids(&result), ["cov-pack::reply-period"]);
    insta::assert_json_snapshot!("query_overrides", result);
}

#[test]
fn anonymous_edges_to_the_same_node_are_numbered() {
    let result = query(&["edges", "--match", "default-risk->"]);
    assert_eq!(
        ids(&result),
        [
            "cov-pack::default-risk->judgment-entered#0",
            "cov-pack::default-risk->judgment-entered#1",
        ]
    );
}

#[test]
fn labels_match_case_insensitively() {
    let result = query(&["edges", "--match", "MOTION GRANTED"]);
    assert_eq!(ids(&result), ["cov-pack::granted"]);
}

#[test]
fn the_package_filter_takes_a_name_or_a_pack_id() {
    assert_eq!(
        query(&["deadlines", "--package", "cov-example"])["total"],
        4
    );
    assert_eq!(query(&["deadlines", "--package", "cov-pack"])["total"], 4);
    assert_eq!(query(&["deadlines", "--package", "other"])["total"], 0);
}

#[test]
fn the_overrides_filter_keeps_declarations_related_to_the_id() {
    let related = query(&["deadlines", "--overrides", "cov-pack::answer-period"]);
    assert_eq!(ids(&related), ["cov-pack::reply-period"]);
    assert_eq!(query(&["deadlines", "--overrides", "frcp6"])["total"], 0);
    // Holes carry no version metadata, so no relation can match.
    assert_eq!(
        query(&["holes", "--overrides", "answer-period"])["total"],
        0
    );
}

#[test]
fn a_package_without_holes_returns_none_with_or_without_in_force_filter() {
    let dir = scratch("query-holes");
    support_cmds::copy_dir(&support_cmds::repo().join("lit/examples"), &dir.join("ex"));
    let run = litigate_in(&dir, &["query", "holes", "--path", "ex"]);
    assert_eq!(run.json["result"]["total"], 0, "{}", run.stdout);
    let dated = litigate_in(
        &dir,
        &[
            "query",
            "holes",
            "--path",
            "ex",
            "--effective-on",
            "2020-01-01",
        ],
    );
    assert_eq!(dated.json["result"]["total"], 0, "{}", dated.stdout);
}
