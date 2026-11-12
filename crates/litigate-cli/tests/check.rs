//! `check`, `test`, `fmt`, `explain` and exit codes over the SPEC §12
//! worked example (`lit/examples` + `cases/examples`).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{codes, litigate, scratch};

#[test]
fn dev_check_reports_clean_example_and_replays_acme() {
    let run = litigate(&["check", "lit/examples", "cases/examples"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let found = codes(&run.json);
    assert!(found.is_empty(), "{found:?}");
    assert!(
        !found
            .iter()
            .any(|c| matches!(c.0.as_str(), "L9003" | "L9006")),
        "{found:?}"
    );
    let case = &run.json["result"]["cases"][0];
    assert_eq!(case["start"], "frcp-rule12-example::mtd-ruling");
    assert_eq!(case["deadlines"][0]["due"], "2025-03-26");
    insta::assert_json_snapshot!("check_dev", run.json);
}

#[test]
fn strict_check_accepts_the_hole_free_example() {
    let run = litigate(&[
        "check",
        "--profile",
        "strict",
        "lit/examples",
        "cases/examples",
    ]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["ok"], true);
    // Schema v2 keeps the result alongside errors: the failing obligations.
    let holes = run.json["result"]["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "holes")
        .unwrap();
    assert_eq!(holes["status"], "passed");
    let found = codes(&run.json);
    assert_eq!(
        found
            .iter()
            .filter(|c| c.0 == "L5002" && c.1 == "error")
            .count(),
        0,
        "{found:?}"
    );
    insta::assert_json_snapshot!("check_strict", run.json);
}

#[test]
fn strict_dependency_holes_follow_reachable_declarations() {
    use std::path::Path;
    fn setup(label: &str, file: &str, cite: &str, reason: &str) -> std::path::PathBuf {
        let dir = support::scratch(label);
        let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lit");
        support::copy_dir(&packages.join("core"), &dir.join("lit/core"));
        support::copy_dir(&packages.join("frcp"), &dir.join("lit/frcp"));
        #[cfg(unix)]
        std::os::unix::fs::symlink(support::repo().join("sources"), dir.join("sources")).unwrap();
        #[cfg(not(unix))]
        support::copy_dir(&support::repo().join("sources"), &dir.join("sources"));
        let calendar = dir.join("lit/core").join(file);
        let source = std::fs::read_to_string(&calendar).unwrap();
        let suffix = if file == "calendar.lit" {
            " historical"
        } else {
            ""
        };
        let from = format!("cite \"{cite}\"{suffix}");
        let to = format!("cite ?(\"{reason}\")");
        assert!(source.contains(&from));
        std::fs::write(calendar, source.replacen(&from, &to, 1)).unwrap();
        let formatted = support::litigate_in(&dir, &["fmt", "lit/core"]);
        assert_eq!(formatted.code, 0, "{}", formatted.stdout);
        for target in ["lit/core", "lit/frcp"] {
            let lock = support::litigate_in(&dir, &["lock", target]);
            assert_eq!(lock.code, 0, "{target}: {}", lock.stdout);
        }
        dir
    }
    let run = |dir: &Path| support::litigate_in(dir, &["check", "--profile", "strict", "lit/frcp"]);

    // A state calendar hole is in core but outside FRCP's declaration graph.
    let unused_dir = setup(
        "strict-hole-unreachable",
        "states/alabama.lit",
        "Ala. Code § 1-3-8",
        "unreachable fixture hole",
    );
    let unused = run(&unused_dir);
    assert_eq!(unused.code, 0, "{:?}", unused.json["diagnostics"]);
    let diagnostics = unused.json["diagnostics"].as_array().unwrap();
    let note = diagnostics.iter().find(|d| d["code"] == "L5002").unwrap();
    assert_eq!(note["severity"], "info");
    assert!(note["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["message"].as_str().unwrap().contains("1 holes")));

    // The configured counting rule itself is reachable from FRCP.
    let direct_dir = setup(
        "strict-hole-direct",
        "frcp6.lit",
        "FRCP (2008 ed.) 6 (1963 Advisory Committee Note)",
        "direct fixture hole",
    );
    let direct = run(&direct_dir);
    assert_eq!(direct.code, 2, "{}", direct.stdout);
    assert!(support::codes(&direct.json)
        .iter()
        .any(|(code, severity)| code == "L5002" && severity == "error"));

    // A federal-holiday cite is reached through FRCP's deadline -> frcp6 -> calendar chain.
    let reached_dir = setup(
        "strict-hole-transitive",
        "calendar.lit",
        "5 U.S.C. § 6103(a)",
        "reachable fixture hole",
    );
    let reached = run(&reached_dir);
    assert_eq!(reached.code, 2, "{}", reached.stdout);
    assert!(
        support::codes(&reached.json)
            .iter()
            .any(|(code, severity)| code == "L5002" && severity == "error"),
        "{}",
        reached.stdout
    );
}

#[test]
fn strict_dependency_holes_follow_metadata_less_events_and_types() {
    use std::path::Path;
    let dir = support::scratch("strict-hole-through-event");
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lit");
    support::copy_dir(&packages.join("core"), &dir.join("lit/core"));
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/timeline-package");
    support::copy_dir(&fixture, &dir.join("lit/a"));
    #[cfg(unix)]
    std::os::unix::fs::symlink(support::repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    support::copy_dir(&support::repo().join("sources"), &dir.join("sources"));

    let manifest = dir.join("lit/a/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    let text = text.replace("mover = \"self\"", "plaintiff = \"self\"");
    std::fs::write(
        manifest,
        format!("{text}\n[dependencies]\ncore = {{ version = \"^1.0.0\", path = \"../core\" }}\n"),
    )
    .unwrap();

    let time = dir.join("lit/core/time.lit");
    let source = std::fs::read_to_string(&time).unwrap();
    let source = source.replace(
        "event Served(method: ServiceMethod)",
        "type FixtureMethod = ServiceMethod\nevent Served(method: FixtureMethod) paper fixture-limit",
    );
    assert_ne!(source, std::fs::read_to_string(&time).unwrap());
    std::fs::write(
        time,
        format!(
            "{source}\n{}\n",
            r#"limit fixture-limit { words <= ?("event-reachable fixture hole") }"#
        ),
    )
    .unwrap();

    let authoring = dir.join("lit/a/fixture.lit");
    let source = std::fs::read_to_string(&authoring).unwrap();
    let source = source.replace(
        "module fixture\n",
        "module fixture\n\nimport core.time.{ Served, ServiceMethod }\n",
    );
    let source = source
        .replace("event Started() }", "event Served(Personal) }")
        .replace("by mover", "by plaintiff");
    assert_ne!(source, std::fs::read_to_string(&authoring).unwrap());
    std::fs::write(authoring, source).unwrap();

    for target in ["lit/core", "lit/a"] {
        let fmt = support::litigate_in(&dir, &["fmt", target]);
        assert_eq!(fmt.code, 0, "{}", fmt.stdout);
        let lock = support::litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{}", lock.stdout);
    }
    let run = support::litigate_in(&dir, &["check", "--profile", "strict", "lit/a"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.json["diagnostics"].as_array().unwrap().iter().any(|d| {
            d["code"] == "L5002"
                && d["severity"] == "error"
                && d["span"]["file"]
                    .as_str()
                    .is_some_and(|file| file.contains("core/time.lit"))
                && d["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("event-reachable fixture hole"))
        }),
        "{}",
        run.stdout
    );
}

#[test]
fn strict_dependency_holes_follow_earlier_version_of_reachable_deadline() {
    use std::path::Path;
    let dir = support::scratch("strict-hole-earlier-deadline-version");
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lit");
    support::copy_dir(&packages.join("core"), &dir.join("lit/core"));
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/timeline-package");
    support::copy_dir(&fixture, &dir.join("lit/a"));
    #[cfg(unix)]
    std::os::unix::fs::symlink(support::repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    support::copy_dir(&support::repo().join("sources"), dir.join("sources"));

    let manifest = dir.join("lit/a/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        manifest,
        format!(
            "{}\n[dependencies]\ncore = {{ version = \"^1.0.0\", path = \"../core\" }}\n",
            text.replace("timeline-fixture", "deadline-root")
                .replace("mover = \"self\"", "plaintiff = \"self\"")
        ),
    )
    .unwrap();

    let time = dir.join("lit/core/time.lit");
    let source = std::fs::read_to_string(&time).unwrap();
    let source = source.replacen(
        "module core.time\n",
        "module core.time\n\nimport core.calendar.{ us-federal-holidays }\n",
        1,
    );
    let source = format!(
        "{source}\n\nevent ImportedStart()\n\ncounting hole-count cite ?(\"earlier version hole\")\n{{\n  exclude_trigger_day\n  include_last_day\n  holidays us-federal-holidays\n  anchor trigger_day\n}}\n\ncounting clear-count cite \"Fixture 1\"\n{{\n  exclude_trigger_day\n  include_last_day\n  holidays us-federal-holidays\n  anchor trigger_day\n}}\n\ndeadline imported-period: 10.days after ImportedStart() counted_by hole-count effective [2020-01-01, 2021-01-01) known_at 2020-01-01 cite \"Fixture 2\"\n\ndeadline imported-period: 10.days after ImportedStart() counted_by clear-count effective [2021-01-01, ..) known_at 2021-01-01 cite \"Fixture 2\"\n"
    );
    std::fs::write(time, source).unwrap();

    let root = dir.join("lit/a/fixture.lit");
    let source = std::fs::read_to_string(&root).unwrap();
    let source = source
        .replace("timeline-fixture", "deadline-root")
        .replace(
            "module fixture\n",
            "module fixture\n\nimport core.time.{ ImportedStart, imported-period }\n",
        )
        .replace("by mover", "by plaintiff")
        .replace(
            "event Started() }",
            "event ImportedStart(), within imported-period }",
        );
    assert_ne!(source, std::fs::read_to_string(&root).unwrap());
    std::fs::write(root, source).unwrap();

    for target in ["lit/core", "lit/a"] {
        let fmt = support::litigate_in(&dir, &["fmt", target]);
        assert_eq!(fmt.code, 0, "{target}: {}", fmt.stdout);
        let lock = support::litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{target}: {}", lock.stdout);
    }
    let run = support::litigate_in(&dir, &["check", "--profile", "strict", "lit/a"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| {
                diagnostic["code"] == "L5002"
                    && diagnostic["severity"] == "error"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("earlier version hole"))
            }),
        "{}",
        run.stdout
    );
}

#[test]
fn human_output_renders_diagnostics_with_source() {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .env("LITIGATE_TODAY", "2000-01-01")
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
    assert_eq!(out.status.code(), Some(0));
    insta::assert_snapshot!("check_strict_human", text);
}

#[test]
fn ttab_check_human_output_is_limited_to_the_case_when_core_has_no_holes() {
    let case = support::repo()
        .join("crates/litigate-cli/tests/fixtures/e2e-ttab-short/e2e-ttab-short.case");
    let case = case.to_string_lossy().into_owned();
    let run = |args: &[&str]| {
        assert_cmd::Command::cargo_bin("litigate")
            .unwrap()
            .env("LITIGATE_TODAY", "2000-01-01")
            .current_dir(support::repo())
            .args(args)
            .output()
            .unwrap()
    };
    let default = run(&["--format", "human", "check", &case]);
    assert_eq!(default.status.code(), Some(2));
    let default_text = String::from_utf8_lossy(&default.stdout);
    assert!(default_text.contains("designates `answer-period` due 2026-11-05"));
    assert!(!default_text.contains("core: "));
    assert!(!default_text.contains("holes; --verbose to show"));
    assert!(!default_text.contains("[L9003]"));
    insta::assert_snapshot!("ttab_check_human_default", default_text);

    let verbose = run(&["--format", "human", "check", "--verbose", &case]);
    assert_eq!(verbose.status.code(), Some(2));
    let verbose_text = String::from_utf8_lossy(&verbose.stdout);
    assert!(!verbose_text.contains("[L5002]"));
    assert!(!verbose_text.contains("--verbose to show"));

    let json = run(&["--format", "json", "check", &case]);
    let document: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert!(!document["diagnostics"].as_array().unwrap().is_empty());
    assert!(!document["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "L5002"));
}

#[test]
fn verbose_check_restores_library_diagnostics() {
    let fixture = support::repo().join("crates/litigate-cli/tests/fixtures/library-diagnostic");
    let case = fixture.join("cases/matter.case");
    let lib = fixture.join("lit");
    let run = |format: &str, verbose: bool| {
        let mut command = assert_cmd::Command::cargo_bin("litigate").unwrap();
        command
            .env("LITIGATE_TODAY", "2000-01-01")
            .current_dir(support::repo())
            .args(["--format", format, "check"]);
        if verbose {
            command.arg("--verbose");
        }
        command.arg("--lib").arg(&lib).arg(&case).output().unwrap()
    };

    let default = run("human", false);
    assert_eq!(default.status.code(), Some(0));
    let default_text = String::from_utf8_lossy(&default.stdout);
    assert!(!default_text.contains("[L9002]"), "{default_text}");

    let verbose = run("human", true);
    assert_eq!(verbose.status.code(), Some(0));
    let verbose_text = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_text.contains("[L9002]"), "{verbose_text}");

    let json = run("json", false);
    let document: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert!(document["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "L9002"));
}

#[test]
fn shipped_packages_have_no_anonymous_edges() {
    let run = litigate(&[
        "check",
        "lit/core",
        "lit/links",
        "lit/examples",
        "lit/frcp",
        "lit/frap",
        "lit/frcrimp",
        "lit/bap8",
        "lit/ca1",
        "lit/ca8",
        "lit/cadc",
        "lit/cafc",
        "lit/cofc",
        "lit/itc",
        "lit/mpep",
        "lit/ptab",
        "lit/ttab",
    ]);
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
    assert_eq!(d["span"]["line"], 108);
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
