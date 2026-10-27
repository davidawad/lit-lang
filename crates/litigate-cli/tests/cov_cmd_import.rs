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

/// `dir/p.json`, the built frcp pack without its extension grants and
/// designated windows (pack v5), which import reports rather than spells
/// (L6002).
fn legacy_frcp(dir: &Path) {
    let built = Path::new(litgraph_packs::PACKS_DIR).join("frcp-civil-procedure.json");
    let mut pack: Value = serde_json::from_str(&std::fs::read_to_string(built).unwrap()).unwrap();
    for edge in pack["edges"].as_array_mut().unwrap() {
        let Some(edge) = edge.as_object_mut() else {
            continue;
        };
        if edge
            .get("deadline")
            .and_then(|d| d.get("designated"))
            .is_some()
        {
            edge.remove("deadline");
        } else if let Some(d) = edge.get_mut("deadline").and_then(Value::as_object_mut) {
            d.remove("extensions");
        }
    }
    write(dir, "p.json", &pack);
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
        diagnostics[i]["labels"]
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
    legacy_frcp(&dir);
    let mut scenario: Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(litgraph_packs::SCENARIOS_DIR).join("frcp-civil-case.json"),
        )
        .unwrap(),
    )
    .unwrap();
    // Negative zero: an exact rational has no sign of zero, and JCS writes
    // `-0` as `0` (RFC 8785 §3.2.2.3).
    scenario["scenario"]["params"]["zero"] = Value::from("ZERO");
    let text = serde_json::to_string_pretty(&scenario)
        .unwrap()
        .replace("\"ZERO\"", "-0.0");
    std::fs::write(dir.join("s.json"), text).unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "s.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(!dir.join("o").exists());
    insta::assert_json_snapshot!("import_difference", run.json["diagnostics"]);
}

/// A number too large for an exact `params` value is caught by the check
/// of the generated source, at its span.
#[test]
fn a_number_the_typed_source_cannot_hold_fails_the_check() {
    let dir = scratch("import-unchecked-number");
    legacy_frcp(&dir);
    let mut scenario: Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(litgraph_packs::SCENARIOS_DIR).join("frcp-civil-case.json"),
        )
        .unwrap(),
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
    insta::assert_json_snapshot!("import_unchecked_number", run.json["diagnostics"]);
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

/// A legacy transform with no typed spelling fails the import itself.
#[test]
fn a_transform_with_no_typed_spelling_fails_the_import() {
    let dir = with_pack("import-no-lowering");
    std::fs::write(
        dir.join("links.json"),
        "{\"instances\": {\"frcp-rule12-example@copy\": {\"pack\": \"frcp-rule12-example\", \
         \"payoff_transform\": \"payoff *\"}}}",
    )
    .unwrap();
    let run = litigate_in(&dir, &["import", "p.json", "links.json", "--out", "o"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("import_untyped_transform", errors(&run.json));
}

/// A scratch tree holding `lit/{core,examples}` and a calibration set of the
/// example pack whose first entry targets `edge`.
fn with_example_package(name: &str, edge: &str) -> PathBuf {
    let dir = scratch(name);
    for pkg in ["core", "examples"] {
        support_cmds::copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    let set = serde_json::json!({
        "id": "rule12-fy", "title": "Rule 12 rates",
        "entries": [{
            "target": "edge-probability", "ref": format!("frcp-rule12-example::{edge}"),
            "value": 0.2, "n": 50, "method": "10 / 50",
            "source": {"title": "Report", "url": "https://example.org/r.pdf", "vintageStart": "2023-10-01", "vintageEnd": "2024-09-30"}
        }]
    });
    write(&dir, "set.json", &set);
    dir
}

/// A set of a pack already under `--out` goes into that package, checked
/// and compared there.
#[test]
fn a_calibration_set_imports_into_the_package_it_calibrates() {
    let dir = with_example_package("import-calibration", "barred");
    let run = litigate_in(&dir, &["import", "set.json", "--out", "."]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let text = std::fs::read_to_string(dir.join("lit/examples/calibration-rule12-fy.lit")).unwrap();
    assert!(text.contains("p barred 0.2 {"), "{text}");
}

/// A target the package does not declare fails the import, unwritten.
#[test]
fn a_calibration_target_the_package_lacks_fails_the_import() {
    let dir = with_example_package("import-calibration-missing", "no-such-edge");
    let run = litigate_in(&dir, &["import", "set.json", "--out", "."]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        errors(&run.json),
        [(
            "L6002".to_owned(),
            "the imported source does not check".to_owned()
        )]
    );
    assert!(!dir.join("lit/examples/calibration-rule12-fy.lit").exists());
}
