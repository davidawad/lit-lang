//! `litigate query provenance`: basis labels, filters, targets and rendering.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::Value;
use std::path::Path;
use support_cmds::{fixture, litigate_in, repo};

fn run(target: &str, extra: &[&str]) -> support_cmds::Run {
    let mut args = vec!["query", "provenance", target];
    args.extend_from_slice(extra);
    litigate_in(&fixture().join("../provenance-query"), &args)
}

fn rows(result: &Value) -> &[Value] {
    result["result"]["rows"].as_array().unwrap()
}

#[test]
fn provenance_lists_every_basis_and_authored_source_metadata() {
    let run = run("lit/provenance", &[]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let rows = rows(&run.json);
    let bases: Vec<&str> = rows
        .iter()
        .map(|row| row["basis"].as_str().unwrap())
        .collect();
    for basis in [
        "hole",
        "placeholder",
        "estimate",
        "expert",
        "calibrated",
        "sourced",
    ] {
        assert!(bases.contains(&basis), "{basis}: {}", run.stdout);
    }
    assert!(rows.iter().any(|row| row["source"] == "Rule 1"));
    assert!(rows.iter().any(|row| row["vintage"] == "2026-02-01"));
    for basis in ["sourced", "calibrated", "expert", "estimate", "placeholder"] {
        assert!(rows
            .iter()
            .any(|row| { row["basis"] == basis && row["vintage"].is_null() }));
        assert!(rows
            .iter()
            .any(|row| { row["basis"] == basis && row["vintage"].is_string() }));
    }
    assert!(rows
        .iter()
        .all(|row| row["span"].as_str().unwrap().contains(':')));
    assert!(rows.windows(2).all(|pair| {
        let rank = |row: &Value| match row["basis"].as_str().unwrap() {
            "hole" => 0,
            "placeholder" => 1,
            "estimate" => 2,
            "expert" => 3,
            "calibrated" => 4,
            _ => 5,
        };
        rank(&pair[0]) <= rank(&pair[1])
    }));
}

#[test]
fn provenance_basis_filter_and_no_vintage_filter_work_together() {
    let placeholder = run("lit/provenance/provenance.lit", &["--basis", "placeholder"]);
    assert_eq!(placeholder.code, 0, "{}", placeholder.stdout);
    assert_eq!(placeholder.json["result"]["total"], 2);
    assert!(rows(&placeholder.json)
        .iter()
        .all(|row| row["basis"] == "placeholder"));

    let estimate = run("lit/provenance", &["--basis", "estimate", "--no-vintage"]);
    assert_eq!(estimate.code, 0, "{}", estimate.stdout);
    assert_eq!(estimate.json["result"]["total"], 4);
    assert!(rows(&estimate.json)
        .iter()
        .all(|row| row["vintage"].is_null()));
}

#[test]
fn provenance_accepts_a_case_and_lists_its_packages() {
    let run = run("cases/provenance.case", &["--basis", "sourced"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert!(!rows(&run.json).is_empty());
    assert!(rows(&run.json)
        .iter()
        .all(|row| row["package"] == "provenance-query"));
}

#[test]
fn provenance_human_output_is_a_table_and_describe_lists_the_form() {
    let fixture = repo().join("crates/litigate-cli/tests/fixtures/provenance-query");
    let human = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(&fixture)
        .args(["--format", "human", "query", "provenance", "lit/provenance"])
        .output()
        .unwrap();
    assert!(human.status.success());
    let output = String::from_utf8_lossy(&human.stdout);
    assert!(
        output.contains("PACKAGE") && output.contains("BASIS"),
        "{output}"
    );

    let describe = litigate_in(Path::new(&fixture), &["describe"]);
    assert_eq!(describe.code, 0, "{}", describe.stdout);
    let query = describe.json["result"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cmd| cmd["name"] == "query")
        .unwrap();
    assert!(query["args"].as_array().unwrap().iter().any(|arg| {
        arg["name"] == "kind"
            && arg["values"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value == "provenance"))
    }));
}
