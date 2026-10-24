//! `check`, `test`, `fmt`, `explain` and exit codes over the SPEC §12
//! worked example (`lit/examples` + `cases/examples`).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{codes, litigate, scratch};

#[test]
fn dev_check_reports_holes_and_replays_acme() {
    let run = litigate(&["check", "lit/examples", "cases/examples"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let found = codes(&run.json);
    assert_eq!(
        found
            .iter()
            .filter(|c| c.0 == "L9002" && c.1 == "warning")
            .count(),
        3
    );
    assert!(
        !found
            .iter()
            .any(|c| matches!(c.0.as_str(), "L9003" | "L9006")),
        "{found:?}"
    );
    assert_eq!(found.len(), 3, "{found:?}");
    let case = &run.json["result"]["cases"][0];
    assert_eq!(case["start"], "frcp-rule12-example::mtd-ruling");
    assert_eq!(case["deadlines"][0]["due"], "2025-03-26");
    insta::assert_json_snapshot!("check_dev", run.json);
}

#[test]
fn strict_check_fails_on_every_hole() {
    let run = litigate(&[
        "check",
        "--profile",
        "strict",
        "lit/examples",
        "cases/examples",
    ]);
    assert_eq!(run.code, 2);
    assert_eq!(run.json["ok"], false);
    // Schema v2 keeps the result alongside errors: the failing obligations.
    let holes = run.json["result"]["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "holes")
        .unwrap();
    assert_eq!(holes["status"], "failed");
    let found = codes(&run.json);
    assert_eq!(
        found
            .iter()
            .filter(|c| c.0 == "L5002" && c.1 == "error")
            .count(),
        3,
        "{found:?}"
    );
    insta::assert_json_snapshot!("check_strict", run.json);
}

#[test]
fn human_output_renders_diagnostics_with_source() {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(support::repo())
        .args([
            "--format",
            "human",
            "check",
            "--profile",
            "strict",
            "lit/examples",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(out.status.code(), Some(2));
    insta::assert_snapshot!("check_strict_human", text);
}

#[test]
fn ttab_check_human_output_groups_library_noise_and_verbose_restores_it() {
    let run = |args: &[&str]| {
        assert_cmd::Command::cargo_bin("litigate")
            .unwrap()
            .current_dir(support::repo())
            .args(args)
            .output()
            .unwrap()
    };
    let default = run(&["--format", "human", "check", "cases/e2e-ttab-short.case"]);
    assert_eq!(default.status.code(), Some(2));
    let default_text = String::from_utf8_lossy(&default.stdout);
    assert!(default_text.contains("designates `answer-period` due 2026-11-05"));
    assert!(
        default_text.contains("ttab-trademark-trial-and-appeal-board: 68 holes; --verbose to show")
    );
    assert!(!default_text.contains("[L9003]"));
    insta::assert_snapshot!("ttab_check_human_default", default_text);

    let verbose = run(&[
        "--format",
        "human",
        "check",
        "--verbose",
        "cases/e2e-ttab-short.case",
    ]);
    assert_eq!(verbose.status.code(), Some(2));
    let verbose_text = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_text.contains("[L9002]"));
    assert!(!verbose_text.contains("--verbose to show"));
    insta::assert_snapshot!("ttab_check_human_verbose", verbose_text);

    let json = run(&["--format", "json", "check", "cases/e2e-ttab-short.case"]);
    let document: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(document["diagnostics"].as_array().unwrap().len(), 69);
    assert!(document["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "L9002"));
}

#[test]
fn shipped_packages_have_no_anonymous_edges() {
    let run = litigate(&["check", "lit"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert!(
        !run.json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "L9003"),
        "{}",
        run.stdout
    );
}

#[test]
fn inline_tests_pass_through_the_engine_clock() {
    let run = litigate(&["test", "lit/examples"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["passed"], 2);
    assert_eq!(run.json["result"]["failed"], 0);
    insta::assert_json_snapshot!("test_examples", run.json["result"]);
}

#[test]
fn a_failing_test_is_l5006() {
    let dir = scratch("failing-test");
    support::copy_dir(&support::repo().join("lit/examples"), &dir.join("pkg"));
    let path = dir.join("pkg/rule12.lit");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("== 2025-03-24", "== 2025-03-25");
    std::fs::write(&path, text).unwrap();
    let run = support::litigate_in(&dir, &["test", "pkg"]);
    assert_eq!(run.code, 2);
    let d = &run.json["diagnostics"][0];
    assert_eq!(d["code"], "L5006");
    assert!(
        d["message"].as_str().unwrap().contains("\"2025-03-24\""),
        "{d}"
    );
    assert_eq!(d["span"]["line"], 103);
}

#[test]
fn examples_are_canonically_formatted() {
    let run = litigate(&["fmt", "--check", "lit/examples", "cases/examples"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert!(run.json["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["changed"] == false));
}

#[test]
fn fmt_rewrites_and_check_reports_l9004() {
    let dir = scratch("fmt");
    std::fs::create_dir_all(dir.join("p")).unwrap();
    std::fs::copy(
        support::repo().join("lit/examples/Lit.toml"),
        dir.join("p/Lit.toml"),
    )
    .unwrap();
    std::fs::write(
        dir.join("p/a.lit"),
        "module   m\nstart s\nstate s: \"S\"  { _ \"x\" -> t }\nterminal t: \"T\" outcome [win]\n",
    )
    .unwrap();
    let check = support::litigate_in(&dir, &["fmt", "--check", "p"]);
    assert_eq!(check.code, 2);
    assert_eq!(
        codes(&check.json),
        [("L9004".to_owned(), "error".to_owned())]
    );
    let fmt = support::litigate_in(&dir, &["fmt", "p"]);
    assert_eq!(fmt.code, 0);
    assert_eq!(fmt.json["result"]["files"][0]["changed"], true);
    assert_eq!(support::litigate_in(&dir, &["fmt", "--check", "p"]).code, 0);
}

#[test]
fn explain_gives_the_long_form() {
    let run = litigate(&["explain", "l3004"]);
    assert_eq!(run.code, 0);
    insta::assert_json_snapshot!("explain_l3004", run.json["result"]);
}

#[test]
fn usage_and_io_errors_exit_1_without_an_envelope() {
    let unknown = litigate(&["explain", "L0000"]);
    assert_eq!(unknown.code, 1);
    assert!(unknown.stdout.is_empty());
    assert!(
        unknown.stderr.contains("not a diagnostic code"),
        "{}",
        unknown.stderr
    );
    assert_eq!(litigate(&["check", "no/such/dir"]).code, 1);
    assert_eq!(litigate(&["frobnicate"]).code, 1);
    assert_eq!(litigate(&["--help"]).code, 0);
}
