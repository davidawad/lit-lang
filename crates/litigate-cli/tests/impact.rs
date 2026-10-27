//! `litigate test --impact BASE` (lg-8fdj.3): a branch that edits a layer
//! lists every expect test, deadline and verdict that moved, per forum.
//! A scratch repository holds the FRAP change files (docs/litigate/change)
//! and three expect tests; the working tree then shortens the pre-2005
//! FRAP 4(a)(6) reopening window from 7 days to 3, as a layer edit would.
// Test code: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
use support::{copy_dir, litigate_in, repo, scratch};

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

fn write(dir: &Path, path: &str, text: &str) {
    let path = dir.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A repository with `lit/core`, the change files, the raw texts their
/// locks pin, and expect tests for two FRAP rulings and the base layer.
fn fixture(name: &str) -> std::path::PathBuf {
    let dir = scratch(name);
    let root = repo();
    copy_dir(&root.join("lit/core"), &dir.join("lit/core"));
    copy_dir(
        &root.join("docs/litigate/change"),
        &dir.join("docs/litigate/change"),
    );
    for lock in ["lit/core/Lit.lock", "docs/litigate/change/Lit.lock"] {
        let text = std::fs::read_to_string(root.join(lock)).unwrap();
        for path in text.split('"').filter(|s| s.contains(" sources/")) {
            let path = path.split(' ').nth(1).unwrap();
            std::fs::create_dir_all(dir.join(path).parent().unwrap()).unwrap();
            std::fs::copy(root.join(path), dir.join(path)).unwrap();
        }
    }
    let change = "docs/litigate/change";
    for (name, world) in [
        ("bass-5cir-2000", "circuit-appellate-ca5"),
        ("benavides-dccir-1996", "circuit-appellate-cadc"),
    ] {
        write(
            &dir,
            &format!("tests/expect/frap/{name}.toml"),
            &format!(
                "case = \"{change}/{name}.case\"\nworld = \"{change}/worlds/{world}.world\"\npaths = [\"{change}\"]\n"
            ),
        );
    }
    write(
        &dir,
        "tests/expect/frap/layer.toml",
        &format!("package = \"{change}/frap-base\"\n"),
    );
    relock(&dir);
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// The pre-2005 reopening window, 7 days in the layer, made 3.
fn shorten_the_reopening_window(dir: &Path) {
    let path = dir.join("docs/litigate/change/frap-base/appeal.lit");
    let text = std::fs::read_to_string(&path).unwrap();
    let text = text
        .replace(
            "deadline reopen-motion-window: 7.days after",
            "deadline reopen-motion-window: 3.days after",
        )
        .replace(
            "expect deadline(reopen-motion-window) == 1999-06-04",
            "expect deadline(reopen-motion-window) == 1999-06-03",
        );
    std::fs::write(&path, text).unwrap();
    relock(dir);
}

/// Every lock of the change files, against the scratch library.
fn relock(dir: &Path) {
    for pkg in [
        "frcp-base",
        "frcp-2009",
        "frap-base",
        "frap-2005",
        "frap-2009",
        "",
    ] {
        let run = litigate_in(dir, &["lock", &format!("docs/litigate/change/{pkg}")]);
        assert_eq!(run.code, 0, "{}", run.stdout);
    }
}

fn row(case: &str, world: &str, field: &str, before: &str, after: &str) -> Value {
    json!({"forum": "frap", "case": case, "world": world, "field": field, "before": before, "after": after})
}

/// The human report of the edited fixture: one aligned table per forum.
fn assert_human_table(dir: &Path) {
    let human = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(dir)
        .args(["--format", "human", "test", "--impact", "HEAD"])
        .output()
        .unwrap();
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(
        text.contains("6 change(s) across 4 expect test(s)"),
        "{text}"
    );
    assert!(
        text.lines().any(|l| l.split(" | ").map(str::trim).eq([
            "benavides-dccir-1996",
            "w1",
            "verdict filed",
            "timely",
            "late"
        ])),
        "{text}"
    );
    assert!(
        text.contains("\nfrcp\ncase | world | field       | before | after\n"),
        "{text}"
    );
}

#[test]
fn a_layer_edit_lists_every_moved_deadline_verdict_and_expectation() {
    let dir = fixture("impact-edit");
    let unchanged = litigate_in(&dir, &["test", "--impact", "HEAD"]);
    assert_eq!(
        unchanged.code, 0,
        "{}{}",
        unchanged.stdout, unchanged.stderr
    );
    assert_eq!(unchanged.json["result"]["tests"], 3);
    assert_eq!(unchanged.json["result"]["changes"], json!([]));

    shorten_the_reopening_window(&dir);
    write(
        &dir,
        "tests/expect/frcp/new.toml",
        "package = \"docs/litigate/change/frcp-base\"\n",
    );
    let run = litigate_in(&dir, &["test", "--impact", "HEAD"]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    let window = "deadline change-deadline-frap-base::reopen-motion-window";
    let test = "change-deadline-frap-base::before 2005 a motion to reopen is due 7 days after notice of entry, weekends skipped";
    let mut want = vec![
        row(
            "bass-5cir-2000",
            "w1",
            &format!("{window} @1999-05-28"),
            "1999-06-04 discharged",
            "1999-06-03 discharged",
        ),
        row(
            "benavides-dccir-1996",
            "w1",
            &format!("{window} @1993-03-12"),
            "1993-03-19 discharged",
            "1993-03-17 late",
        ),
        row(
            "benavides-dccir-1996",
            "w1",
            "verdict filed",
            "timely",
            "late",
        ),
        row(
            test,
            "-",
            "deadline(reopen-motion-window) == 1999-06-03",
            "-",
            "1999-06-03",
        ),
        row(
            test,
            "-",
            "deadline(reopen-motion-window) == 1999-06-04",
            "1999-06-04",
            "-",
        ),
    ];
    want.push(json!({"forum": "frcp", "case": "new", "world": "-", "field": "expect test", "before": "-", "after": "present"}));
    assert_eq!(run.json["result"]["changes"], json!(want));
    assert_eq!(run.json["result"]["tests"], 4);
    let base = run.json["result"]["base"].as_str().unwrap();
    assert_eq!(base.len(), 40, "the base commit's sha: {base}");

    assert_human_table(&dir);
}

#[test]
fn a_base_that_is_not_a_revision_is_a_usage_error() {
    let dir = fixture("impact-usage");
    let run = litigate_in(&dir, &["test", "--impact", "no-such-rev"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("no-such-rev"), "{}", run.stderr);
    let sub = dir.join("tests");
    let run = litigate_in(&sub, &["test", "--impact", "HEAD"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("repository root"), "{}", run.stderr);
}
