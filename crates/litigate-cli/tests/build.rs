//! `build`, `diff`, `run`, `where`, `query`, `explain-value` end to end.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::collections::BTreeMap;
use std::path::Path;
use support::{litigate, repo, scratch};

fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    out
}

#[test]
fn two_builds_are_byte_identical() {
    let (a, b) = (scratch("build-a"), scratch("build-b"));
    for out in [&a, &b] {
        let run = litigate(&[
            "build",
            "lit/examples",
            "cases/examples",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(run.code, 0, "{}", run.stdout);
    }
    let (ta, tb) = (tree(&a), tree(&b));
    assert_eq!(ta, tb);
    let names: Vec<&String> = ta.keys().collect();
    insta::assert_json_snapshot!("build_files", names);
    for (name, bytes) in &ta {
        let text = String::from_utf8(bytes.clone()).unwrap();
        assert!(text.ends_with("}\n"), "{name}");
        assert!(
            !text.contains(a.to_str().unwrap()),
            "{name} carries an absolute path"
        );
    }
    let litigatable: serde_json::Value =
        serde_json::from_slice(&ta["litigatables/frcp-rule12-example.litigatable.json"]).unwrap();
    assert_eq!(litigatable["ir"]["manifest"]["name"], "frcp-rule12-example");
    assert!(litigatable["content_hash"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(litigatable["source_map"]["entries"]
        .as_array()
        .is_some_and(|e| !e.is_empty()));
    insta::assert_snapshot!(
        "built_scenario",
        String::from_utf8(ta["scenarios/acme-v-widget.json"].clone()).unwrap()
    );
}

#[test]
fn build_result_lists_outputs_and_hash() {
    let out = scratch("build-result");
    let run = litigate(&[
        "build",
        "lit/examples",
        "cases/examples",
        "--out",
        out.to_str().unwrap(),
    ]);
    let mut result = run.json["result"].clone();
    let text = serde_json::to_string(&result)
        .unwrap()
        .replace(out.to_str().unwrap(), "OUT");
    result = serde_json::from_str(&text).unwrap();
    insta::assert_json_snapshot!("build_result", result);
}

#[test]
fn strict_build_writes_nothing() {
    let out = scratch("build-strict");
    let run = litigate(&[
        "build",
        "--profile",
        "strict",
        "lit/examples",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(run.code, 2);
    assert!(tree(&out).is_empty());
}

#[test]
fn diff_against_the_built_pack_is_empty_and_a_change_is_l6002() {
    let out = scratch("diff");
    litigate(&["build", "lit/examples", "--out", out.to_str().unwrap()]);
    let pack = out.join("packs/frcp-rule12-example.json");
    let same = litigate(&["diff", "lit/examples", "--against", pack.to_str().unwrap()]);
    assert_eq!(same.code, 0, "{}", same.stdout);
    assert_eq!(same.json["result"]["equal"], true);
    let text = std::fs::read_to_string(&pack)
        .unwrap()
        .replace("\"Answer filed\"", "\"Answer lodged\"");
    std::fs::write(&pack, text).unwrap();
    let changed = litigate(&["diff", "lit/examples", "--against", pack.to_str().unwrap()]);
    assert_eq!(changed.code, 2);
    let errors: Vec<&serde_json::Value> = changed.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "L6002");
    assert_eq!(
        errors[0]["message"],
        "changed at /nodes/answered/label: \"Answer filed\" -> \"Answer lodged\""
    );
}

#[test]
fn diff_of_two_dates_shows_the_amendment() {
    let run = litigate(&[
        "diff",
        "crates/litigate-cli/tests/fixtures/versioned",
        "--as-of",
        "2008-06-01",
        "--to-as-of",
        "2010-06-01",
    ]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["equal"], false);
    insta::assert_json_snapshot!("diff_2009_amendment", run.json["result"]["changes"]);
}

#[test]
fn run_returns_litgraphs_response_from_the_replayed_start() {
    let run = litigate(&["run", "cases/examples/acme.case", "--op", "chain"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let response = &run.json["result"];
    assert_eq!(response["ok"], true);
    assert_eq!(response["op"], "chain");
    assert_eq!(
        response["result"]["start"]["id"],
        "frcp-rule12-example::mtd-ruling"
    );
    let bad = litigate(&[
        "run",
        "cases/examples/acme.case",
        "--op",
        r#"{"op":"solve","nope":1}"#,
    ]);
    assert_eq!(bad.code, 1, "{}", bad.stdout);
}

#[test]
fn explain_value_traces_a_deadline_to_its_rule_and_source() {
    let run = litigate(&["explain-value", "cases/examples/acme.case", "answer-period"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["value"], "2025-03-26");
    insta::assert_json_snapshot!("explain_value_answer_period", run.json["result"]);
    let edge = litigate(&[
        "explain-value",
        "--case",
        "cases/examples/acme.case",
        "--value",
        "move-dismiss",
    ]);
    assert_eq!(edge.json["result"]["type"], "Edge");
    assert_eq!(edge.json["result"]["value"]["due"], "2025-03-26");
}

#[test]
fn where_resolves_declarations_cites_and_codes() {
    let decl = litigate(&["where", "frcp-rule12-example::answer-period"]);
    assert_eq!(decl.code, 0, "{}", decl.stdout);
    insta::assert_json_snapshot!("where_answer_period", decl.json["result"]);
    let cite = litigate(&["where", "FRCP 12(b)"]);
    assert_eq!(cite.json["result"]["kind"], "cite");
    assert_eq!(
        cite.json["result"]["cited_by"][0]["id"],
        "frcp-rule12-example::mtd-ruling"
    );
    assert_eq!(
        litigate(&["where", "L5002"]).json["result"]["kind"],
        "diagnostic"
    );
    let edge = litigate(&["where", "move-dismiss"]);
    assert_eq!(edge.json["result"]["kind"], "edge");
    assert_eq!(edge.json["result"]["lowered"][0], "/edges/3");
}

#[test]
fn query_pages_and_filters() {
    let first = litigate(&["query", "nodes", "--limit", "3"]);
    assert_eq!(first.json["result"]["total"], 7);
    assert_eq!(first.json["result"]["next_cursor"], "3");
    let rest = litigate(&["query", "nodes", "--limit", "10", "--cursor", "3"]);
    assert_eq!(rest.json["result"]["matches"].as_array().unwrap().len(), 4);
    assert!(rest.json["result"]["next_cursor"].is_null());
    let holes = litigate(&["query", "holes"]);
    assert_eq!(holes.json["result"]["total"], 3);
    let cited = litigate(&["query", "cites", "--cite", "12(b)"]);
    assert_eq!(
        cited.json["result"]["matches"][0]["id"],
        "frcp-rule12-example::mtd-ruling"
    );
    let edges = litigate(&["query", "edges", "--match", "judgment-entered"]);
    insta::assert_json_snapshot!("query_edges", edges.json["result"]);
    assert_eq!(
        litigate(&["query", "deadlines", "--effective-on", "2000-01-01"]).json["result"]["total"],
        0
    );
    let _ = repo();
}
