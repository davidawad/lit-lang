//! `coverage`: the generated docs/COVERAGE.md for this repository is
//! current, and on a scratch repository the table's rows, statuses and
//! counts, writing once, `--check` on a stale table, and failures.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::path::Path;
use support::{litigate, litigate_in, scratch};

/// Writes `text` at `root/relative`, creating its directories.
fn put(root: &Path, relative: &str, text: impl AsRef<[u8]>) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A valid `Lit.toml` for package `name` in forum `forum`, then `extra`.
fn manifest(name: &str, forum: &str, extra: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nlanguage = 1\n\
         pack_id = \"{name}\"\nforum = \"{forum}\"\nauthority = \"StateRule\"\n\
         title = \"{name}\"\nin_force = \"2025-01-01\"\nknown_at = \"2025-01-01\"\n\n\
         [package.roles]\napplicant = \"self\"\ncourt = \"nature\"\n\n{extra}"
    )
}

const MODULE: &str = r#"
rule duty answer effective [2025-01-01, ..) cite "Rule 1"
{
  deadline answer-period: 20.days after Served() counted_by frcp6 cite "Rule 1(a)"
}
deadline reply-period: 10.days after Served() counted_by frcp6 cite "Rule 2"
test "answer-period" {
  given Served() on 2025-01-06
  expect deadline(answer-period) == 2025-01-27
}
state served: "Served" {
  court_listener_url "https://www.courtlistener.com/opinion/1/a-v-b/"
  note "court_listener_url in a note is not a cite"
}
// court_listener_url "https://www.courtlistener.com/opinion/2/c-v-d/"
"#;

/// A scratch repository: a federal package, a promoted Florida package
/// (and Florida's outlines, which it supersedes), a Delaware outline, a
/// federal-district and a national outline.
fn repository(name: &str) -> std::path::PathBuf {
    let root = scratch(name);
    put(&root, "lit/fed/Lit.toml", manifest("fed", "frcp", ""));
    put(&root, "lit/fed/notes.md", "not a module");
    put(&root, "lit/fed/README", "not a module");
    put(&root, "lit/fed/sub/m.lit", MODULE);
    // A field keyword with no string after it binds nothing.
    put(
        &root,
        "lit/fed/sub/z.lit",
        "state s: \"S\" { court_listener_url",
    );
    let sources = "[sources.bowles]\ntitle = \"Bowles v. Russell, 551 U.S. 205 (2007)\"\n\
                   url = \"https://www.govinfo.gov/app/details/USREPORTS-551/USREPORTS-551-205\"\n\n\
                   [sources.rules]\ntitle = \"Florida Rules of Civil Procedure\"\n\
                   url = \"https://www.floridabar.org/\"\n\n\
                   [sources.untitled]\nurl = \"https://www.flcourts.gov/\"\n";
    put(
        &root,
        "lit/fl-state-civil/Lit.toml",
        manifest("fl-state-civil", "fl-state-civil", sources),
    );
    put(&root, "lit/fl-state-civil/m.lit", MODULE);
    let outline = |dir: &str, forum: &str| {
        put(
            &root,
            &format!("outlines/{dir}/Lit.toml"),
            manifest(&format!("outline-{forum}"), forum, ""),
        );
        put(&root, &format!("outlines/{dir}/outline.lit"), MODULE);
    };
    outline("eleventh-circuit/florida/state-civil", "fl-state-civil");
    outline(
        "eleventh-circuit/florida/state-appellate",
        "fl-state-appellate",
    );
    outline("eleventh-circuit/florida/federal-sd-fla", "sd-fla");
    outline("third-circuit/delaware/state-civil", "de-superior-civil");
    outline("national/tax-court", "tax-court");
    root
}

#[test]
fn the_committed_table_is_current() {
    let run = litigate(&["coverage", "--check"]);
    assert_eq!(
        run.code, 0,
        "docs/COVERAGE.md is stale: run `just coverage`"
    );
    assert_eq!(run.json["result"]["changed"], false);
}

#[test]
fn a_scratch_repository_is_tabled_once_then_left_alone() {
    let root = repository("coverage-rows");
    let first = litigate_in(&root, &["coverage"]);
    assert_eq!(first.code, 0, "{}", first.stderr);
    let result = &first.json["result"];
    assert_eq!(result["changed"], true);
    assert_eq!(result["promoted_states"], 1);
    insta::assert_json_snapshot!(result["federal"], @r#"
    [
      {
        "dir": "lit/fed",
        "name": "fed",
        "rules": 3,
        "precedents": 1,
        "tests": 1
      }
    ]
    "#);
    let rows = result["jurisdictions"].as_array().unwrap();
    assert_eq!(rows.len(), 56);
    let row = |name: &str| rows.iter().find(|r| r["name"] == name).unwrap().clone();
    insta::assert_json_snapshot!([row("Florida"), row("Delaware"), row("Guam")], @r#"
    [
      {
        "name": "Florida",
        "kind": "state",
        "status": "promoted",
        "packages": [
          "lit/fl-state-civil"
        ],
        "rules": 3,
        "precedents": 2,
        "tests": 1
      },
      {
        "name": "Delaware",
        "kind": "state",
        "status": "outline",
        "packages": [
          "outlines/third-circuit/delaware/state-civil"
        ],
        "rules": 3,
        "precedents": 1,
        "tests": 1
      },
      {
        "name": "Guam",
        "kind": "territory",
        "status": "none",
        "packages": [],
        "rules": 0,
        "precedents": 0,
        "tests": 0
      }
    ]
    "#);
    let table = std::fs::read_to_string(root.join("docs/COVERAGE.md")).unwrap();
    assert!(table.starts_with("# Coverage\n\n**States with a promoted package: 1 of 50.**\n"));
    assert!(table.contains("| fed | `lit/fed` | 3 | 1 | 1 |\n"));
    assert!(table.contains("| Florida | state | promoted | 3 | 2 | 1 | `lit/fl-state-civil` |\n"));
    assert!(table.contains("| District of Columbia | district | none | 0 | 0 | 0 |  |\n"));
    let again = litigate_in(&root, &["coverage"]);
    assert_eq!(again.json["result"]["changed"], false);
    let check = litigate_in(&root, &["coverage", "--check"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
}

#[test]
fn check_exits_2_on_a_stale_table_and_writes_nothing() {
    let root = repository("coverage-stale");
    put(&root, "docs/COVERAGE.md", "# Coverage\n\nedited by hand\n");
    let run = litigate_in(&root.join("lit"), &["coverage", "..", "--check"]);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert_eq!(run.json["ok"], true);
    assert_eq!(run.json["result"]["changed"], true);
    let table = std::fs::read_to_string(root.join("docs/COVERAGE.md")).unwrap();
    assert_eq!(table, "# Coverage\n\nedited by hand\n");
}

/// The usage error `coverage` exits 1 with on `root`.
fn failure(root: &Path) -> String {
    let run = litigate_in(root, &["coverage"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    run.stderr
}

#[test]
fn an_unreadable_repository_is_a_usage_error() {
    let missing = scratch("coverage-missing");
    assert!(failure(&missing).starts_with("litigate coverage: ./lit: "));
    put(&missing, "lit/fed/Lit.toml", manifest("fed", "frcp", ""));
    assert!(failure(&missing).starts_with("litigate coverage: ./outlines: "));
    put(&missing, "outlines/national/bad/Lit.toml", "");
    insta::assert_snapshot!(failure(&missing), @"litigate coverage: outlines/national/bad/Lit.toml: invalid manifest");

    let manifest_root = repository("coverage-bad-manifest");
    put(&manifest_root, "lit/bad/Lit.toml", "[package]\nname = 1\n");
    insta::assert_snapshot!(failure(&manifest_root), @"litigate coverage: lit/bad/Lit.toml: invalid manifest");

    let bytes_root = repository("coverage-bad-bytes");
    put(&bytes_root, "lit/bad/Lit.toml", [0xff_u8, 0xfe]);
    assert!(failure(&bytes_root).starts_with("litigate coverage: ./lit/bad/Lit.toml: "));

    let module_root = repository("coverage-bad-module");
    put(&module_root, "lit/fed/bad.lit", [0xff_u8, 0xfe]);
    assert!(failure(&module_root).starts_with("litigate coverage: ./lit/fed/bad.lit: "));

    let unknown_root = repository("coverage-unknown");
    put(
        &unknown_root,
        "outlines/x/atlantis/state-civil/Lit.toml",
        manifest("outline-atlantis", "atlantis", ""),
    );
    insta::assert_snapshot!(failure(&unknown_root), @"litigate coverage: outlines/x/atlantis/state-civil: `atlantis` is no US state, DC or territory");

    let unwritable = repository("coverage-unwritable");
    put(&unwritable, "docs", "a file, not a directory");
    let message = failure(&unwritable);
    assert!(message.starts_with("litigate coverage: "), "{message}");
}
