//! `litigate diff`: JSON files, directories of JSON (nested, paired by
//! identity or path, with unpairable and incomparable documents), package
//! and matter directories, sides that do not build, and every failure.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{copy_dir, errors, in_fixture, litigate_in, repo, scratch, stable};

/// A scratch directory with `a/`, the built worked example, and `b/`, a copy.
fn built_twice(name: &str) -> PathBuf {
    let dir = scratch(name);
    let out = dir.join("a");
    let run = litigate_in(
        &repo(),
        &["build", "lit/examples", "--out", out.to_str().unwrap()],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    copy_dir(&out, &dir.join("b"));
    dir
}

/// Edits the pack JSON under `root`.
fn edit_pack(root: &Path, edit: impl FnOnce(&mut Value)) {
    let path = root.join("packs/frcp-rule12-example.json");
    let mut pack: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut pack);
    std::fs::write(&path, serde_json::to_string_pretty(&pack).unwrap()).unwrap();
}

/// Retitles the pack, relabels `answered` and removes `dismissed`.
fn three_changes(pack: &mut Value) {
    pack["title"] = json!("Changed title");
    let nodes = pack["nodes"].as_array_mut().unwrap();
    nodes.retain(|n| n["id"] != "dismissed");
    for node in nodes.iter_mut().filter(|n| n["id"] == "answered") {
        node["label"] = json!("Answer lodged");
    }
}

#[test]
fn one_side_alone_is_a_failure() {
    let run = in_fixture(&["diff", "lit/cov"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("diff_one_side", run.stderr);
}

#[test]
fn a_json_file_pairs_with_the_one_pack_a_package_builds() {
    let dir = built_twice("diff-file-package");
    std::fs::copy(
        dir.join("a/packs/frcp-rule12-example.json"),
        dir.join("p.json"),
    )
    .unwrap();
    let examples = repo().join("lit/examples");
    let run = litigate_in(&dir, &["diff", "p.json", examples.to_str().unwrap()]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["equal"], true);
    assert_eq!(run.json["result"]["files"].as_array().unwrap().len(), 1);
}

#[test]
fn two_json_files_compare_directly() {
    let dir = built_twice("diff-two-files");
    edit_pack(&dir.join("b"), three_changes);
    let run = litigate_in(
        &dir,
        &[
            "diff",
            "a/packs/frcp-rule12-example.json",
            "b/packs/frcp-rule12-example.json",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    insta::assert_json_snapshot!("diff_two_files", stable(&run.json));
}

#[test]
fn check_makes_every_change_an_error_with_or_without_an_owner() {
    let dir = built_twice("diff-check");
    edit_pack(&dir.join("b"), three_changes);
    let run = litigate_in(&dir, &["diff", "a", "b", "--check"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("diff_check", run.json["diagnostics"]);
}

#[test]
fn directories_pair_nested_documents_by_identity_or_path() {
    let dir = built_twice("diff-directories");
    let (a, b) = (dir.join("a"), dir.join("b"));
    // Paired by path: a document of no kind on both sides cannot be compared.
    std::fs::write(a.join("notes.json"), "{}").unwrap();
    std::fs::write(b.join("notes.json"), "{}").unwrap();
    // Not JSON: keyed by path; present on one side only.
    std::fs::write(a.join("broken.JSON"), "not json").unwrap();
    // Packs without an id, and other files, on one side only.
    std::fs::create_dir_all(b.join("extra/deeper")).unwrap();
    std::fs::write(b.join("extra/deeper/anonymous.json"), "{\"nodes\": []}").unwrap();
    std::fs::write(b.join("extra/scenario.json"), "{\"summary\": \"no id\"}").unwrap();
    std::fs::write(b.join("extra/readme.txt"), "ignored").unwrap();
    // The pack itself is renamed: it pairs by its id, not its file name.
    std::fs::rename(
        b.join("packs/frcp-rule12-example.json"),
        b.join("packs/renamed.json"),
    )
    .unwrap();
    let run = litigate_in(&dir, &["diff", "a", "b"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_json_snapshot!("diff_directories_errors", errors(&run.json));
    std::fs::remove_file(a.join("notes.json")).unwrap();
    std::fs::remove_file(b.join("notes.json")).unwrap();
    let paired = litigate_in(&dir, &["diff", "a", "b"]);
    assert_eq!(paired.code, 0, "{}", paired.stdout);
    insta::assert_json_snapshot!("diff_directories", paired.json["result"]);
}

#[test]
fn a_file_against_a_directory_of_two_packs_pairs_nothing() {
    let dir = built_twice("diff-unpaired");
    std::fs::copy(
        dir.join("b/packs/frcp-rule12-example.json"),
        dir.join("b/packs/second.json"),
    )
    .unwrap();
    edit_pack(&dir.join("b"), |p| p["id"] = json!("second-pack"));
    let run = litigate_in(
        &dir,
        &["diff", "a/packs/clock/frcp-rule12-example.json", "b/packs"],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["equal"], false);
    assert_eq!(result["files"], json!([]));
    assert_eq!(
        result["unmatched"],
        json!([
            "",
            "clock/frcp-rule12-example.json",
            "pack:frcp-rule12-example",
            "pack:second-pack"
        ])
    );
}

#[test]
fn two_empty_directories_share_no_documents() {
    let dir = scratch("diff-empty");
    std::fs::create_dir_all(dir.join("a")).unwrap();
    std::fs::create_dir_all(dir.join("b")).unwrap();
    let run = litigate_in(&dir, &["diff", "a", "b"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("diff_nothing_to_compare", run.stderr);
}

#[test]
fn an_unreadable_json_file_is_a_failure() {
    let dir = scratch("diff-unreadable-file");
    std::fs::write(dir.join("a.json"), [0xff, 0xfe, 0x00]).unwrap();
    let run = litigate_in(&dir, &["diff", "a.json", "a.json"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("a.json"), "{}", run.stderr);
}

#[test]
fn an_unreadable_file_in_a_directory_is_a_failure() {
    let dir = scratch("diff-unreadable-nested");
    std::fs::create_dir_all(dir.join("a/sub")).unwrap();
    std::fs::write(dir.join("a/sub/x.json"), [0xff, 0xfe, 0x00]).unwrap();
    let run = litigate_in(&dir, &["diff", "a", "a"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(!run.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn unreadable_directories_are_a_failure() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("diff-unreadable-dirs");
    let locked = |path: &Path| {
        std::fs::create_dir_all(path).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000)).unwrap();
    };
    locked(&dir.join("top"));
    locked(&dir.join("outer/inner"));
    let top = litigate_in(&dir, &["diff", "top", "top"]);
    let nested = litigate_in(&dir, &["diff", "outer", "outer"]);
    for path in [dir.join("top"), dir.join("outer/inner")] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    assert_eq!(top.code, 1, "{}", top.stdout);
    assert_eq!(nested.code, 1, "{}", nested.stdout);
}

#[test]
fn a_side_that_does_not_build_reports_its_errors() {
    let dir = scratch("diff-broken-side");
    copy_dir(&repo().join("lit/examples"), &dir.join("pkg"));
    std::fs::write(dir.join("pkg/broken.lit"), "module broken\n\nstate {\n").unwrap();
    let run = litigate_in(&dir, &["diff", "pkg", "--to-as-of", "2020-01-01"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none());
    assert!(!errors(&run.json).is_empty());
}

#[test]
fn a_library_directory_builds_its_packages() {
    let run = in_fixture(&["diff", "lit", "lit/cov"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["equal"], true);
    assert_eq!(run.json["result"]["files"].as_array().unwrap().len(), 2);
}

#[test]
fn a_directory_of_matters_builds_its_scenarios() {
    let dir = scratch("diff-cases");
    let out = dir.join("out");
    let built = in_fixture(&["build", "cases/cov.case", "--out", out.to_str().unwrap()]);
    assert_eq!(built.code, 0, "{}", built.stdout);
    let expected = out.join("scenarios/cov-matter.json");
    let run = in_fixture(&["diff", "cases", "--against", expected.to_str().unwrap()]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["equal"], true);
    assert_eq!(run.json["result"]["files"][0]["kind"], "scenario");
}

#[test]
fn a_side_that_does_not_exist_is_a_failure() {
    let run = in_fixture(&["diff", "lit/cov", "no/such/side"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("diff_missing_side", run.stderr);
}
