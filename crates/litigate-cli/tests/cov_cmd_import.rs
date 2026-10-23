//! `litigate import`: unreadable and invalid inputs, generated source that
//! does not check (attributed to the inputs owning it), does not lower,
//! lowers to a different document or kind, and output that cannot be written.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support_cmds::{errors, litigate_in, repo, scratch, stable};

/// A scratch directory holding `p.json`, the built worked-example pack.
fn with_pack(name: &str) -> PathBuf {
    let dir = scratch(name);
    let build = dir.join("build");
    let run = litigate_in(
        &repo(),
        &["build", "lit/examples", "--out", build.to_str().unwrap()],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    std::fs::copy(
        build.join("packs/frcp-rule12-example.json"),
        dir.join("p.json"),
    )
    .unwrap();
    dir
}

/// Writes JSON `value` to `dir/name`.
fn write(dir: &Path, name: &str, value: &Value) {
    std::fs::write(dir.join(name), serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// `p.json` with `id` replaced.
fn pack_with_id(dir: &Path, name: &str, id: &str) {
    let mut pack: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("p.json")).unwrap()).unwrap();
    pack["id"] = Value::from(id);
    write(dir, name, &pack);
}

#[test]
fn an_unreadable_input_is_a_failure() {
    let dir = scratch("import-missing");
    let run = litigate_in(&dir, &["import", "none.json", "--out", "o"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("none.json"), "{}", run.stderr);
}

#[test]
fn an_input_of_no_known_kind_is_reported() {
    let dir = scratch("import-unknown");
    std::fs::write(dir.join("x.json"), "{\"title\": \"neither\"}").unwrap();
    let run = litigate_in(&dir, &["import", "x.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("import_unknown_kind", stable(&run.json));
}

#[test]
fn links_without_a_pack_do_not_check() {
    let dir = scratch("import-links-alone");
    std::fs::write(dir.join("links.json"), "{\"instances\": {}}").unwrap();
    let run = litigate_in(&dir, &["import", "links.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(!dir.join("o").exists());
    insta::assert_json_snapshot!("import_links_do_not_check", run.json["diagnostics"]);
}

#[test]
fn check_errors_are_attributed_to_the_input_generating_them() {
    let dir = with_pack("import-attributed");
    pack_with_id(&dir, "bad.json", "Not_Kebab");
    let built = repo().join("crates/litigate-cli/tests/fixtures/cov-cmds");
    let scenario = litigate_in(
        &built,
        &[
            "build",
            "cases/cov.case",
            "--out",
            dir.join("s").to_str().unwrap(),
        ],
    );
    assert_eq!(scenario.code, 0, "{}", scenario.stdout);
    std::fs::copy(dir.join("s/scenarios/cov-matter.json"), dir.join("s.json")).unwrap();
    let run = litigate_in(&dir, &["import", "bad.json", "s.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let diagnostics = run.json["diagnostics"].as_array().unwrap();
    let files: Vec<&str> = diagnostics
        .iter()
        .map(|d| d["span"]["file"].as_str().unwrap())
        .collect();
    assert_eq!(files, ["bad.json", "s.json"]);
    let notes = |i: usize| -> Vec<String> {
        diagnostics[i]["notes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["span"]["file"].as_str().unwrap().to_owned())
            .collect()
    };
    assert!(
        notes(0).iter().all(|f| f.starts_with("lit/")),
        "{:?}",
        notes(0)
    );
    assert!(
        notes(1).iter().all(|f| f.starts_with("cases/")),
        "{:?}",
        notes(1)
    );
    insta::assert_json_snapshot!("import_attributed", run.json["diagnostics"]);
}

#[test]
fn a_pack_and_instance_only_links_import_and_are_written() {
    let dir = with_pack("import-instances");
    std::fs::write(
        dir.join("links.json"),
        "{\"instances\": {\"frcp-rule12-example@copy\": {\"pack\": \"frcp-rule12-example\"}}}",
    )
    .unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "links.json", "--out", "o"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert!(dir.join("o/lit/links/links.lit").is_file());
    assert!(dir.join("o/lit/frcp/Lit.toml").is_file());
    insta::assert_json_snapshot!("import_instances", run.json["result"]);
}

#[test]
fn output_that_cannot_be_written_is_a_failure() {
    let dir = with_pack("import-unwritable");
    std::fs::write(dir.join("o"), "a file, not a directory").unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "--out", "o"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("o/lit/frcp"), "{}", run.stderr);
}

#[test]
fn a_number_without_a_lossless_spelling_is_a_difference() {
    let dir = scratch("import-difference");
    std::fs::copy(
        repo().join("packs/frcp-civil-procedure.json"),
        dir.join("p.json"),
    )
    .unwrap();
    let mut scenario: Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("scenarios/frcp-civil-case.json")).unwrap(),
    )
    .unwrap();
    // Written as text: as binary64 the integer would already be rounded.
    scenario["scenario"]["params"]["huge"] = Value::from("HUGE");
    let text = serde_json::to_string_pretty(&scenario)
        .unwrap()
        .replace("\"HUGE\"", "123456789012345678901");
    std::fs::write(dir.join("s.json"), text).unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "s.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(!dir.join("o").exists());
    insta::assert_json_snapshot!("import_difference", run.json["diagnostics"]);
}

#[test]
fn a_pack_named_links_collides_with_the_links_document() {
    let dir = with_pack("import-collision");
    pack_with_id(&dir, "links-pack.json", "links");
    std::fs::write(
        dir.join("links.json"),
        "{\"instances\": {\"links@copy\": {\"pack\": \"links\"}}}",
    )
    .unwrap();
    let run = litigate_in(
        &dir,
        &["import", "links-pack.json", "links.json", "--out", "o"],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("import_kind_collision", errors(&run.json));
}

#[test]
fn links_that_do_not_lower_have_no_counterpart() {
    let dir = with_pack("import-no-lowering");
    std::fs::write(
        dir.join("links.json"),
        "{\"instances\": {\"frcp-rule12-example@copy\": {\"pack\": \"frcp-rule12-example\", \
         \"patch_edges\": {\"answer\": {\"to\": 7}}}}}",
    )
    .unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "links.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("import_no_lowering", errors(&run.json));
}
