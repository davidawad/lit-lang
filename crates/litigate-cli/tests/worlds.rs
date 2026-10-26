//! `litigate worlds` (lg-fq2.4; docs/litigate/CHANGE.md §5.1): a matter
//! replayed in every world a `.world` file spans. The Rule 59(e) example
//! (docs/litigate/examples/timelines) is late without the 2009 amendment
//! and timely with it; at a `known_at` before the amendment's order the
//! corpus cannot count the period at all. Runs in memory over the
//! example files and `lit/core`, as the wasm build does.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::LitigateRun;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use support::{chmod, litigate_in, repo, scratch};

const CASE: &str = "docs/litigate/examples/timelines/rule-59e.case";
const WORLD: &str = "docs/litigate/examples/timelines/rule-59e.world";

/// Every file under `dir` (repository-relative), keyed by its path.
fn tree(dir: &str, out: &mut BTreeMap<String, String>) {
    let root = repo();
    let mut todo = vec![root.join(dir)];
    while let Some(d) = todo.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path);
            } else {
                let rel = path.strip_prefix(&root).unwrap().to_string_lossy();
                out.insert(
                    rel.replace('\\', "/"),
                    std::fs::read_to_string(&path).unwrap(),
                );
            }
        }
    }
}

/// The example files, `lit/core`, and `edits` (path, text) over them.
fn files(edits: &[(&str, &str)]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for dir in [
        "lit/core",
        "docs/litigate/examples/timelines",
        "docs/litigate/examples/change",
    ] {
        tree(dir, &mut out);
    }
    for (path, text) in edits {
        out.insert((*path).to_owned(), (*text).to_owned());
    }
    out
}

fn example(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap()
}

/// `worlds CASE --world WORLD --path docs/litigate/examples ARGS` over
/// `files`.
fn worlds(files: BTreeMap<String, String>, args: &[&str]) -> (LitigateRun, Value) {
    let mut all: Vec<String> = [
        "worlds",
        CASE,
        "--world",
        WORLD,
        "--path",
        "docs/litigate/examples",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    all.extend(args.iter().map(|s| (*s).to_owned()));
    let run = litigate_cli::execute_in_memory(files, &all);
    let json = serde_json::from_str(&run.stdout).unwrap_or(Value::Null);
    (run, json)
}

/// Each filing's worlds by timeliness.
fn filings(json: &Value) -> Value {
    json["result"]["filings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["key"].as_str().unwrap().to_owned(), f["worlds"].clone()))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

#[test]
fn the_motion_is_timely_only_in_the_world_with_the_2009_amendment() {
    let (run, json) = worlds(files(&[]), &[]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    insta::assert_json_snapshot!("rule_59e_worlds", json["result"]);
}

#[test]
fn known_at_selects_what_the_corpus_held() {
    // Before the order of March 26, 2009 the corpus cannot count a 2010
    // period (the 2005 FRCP 6 ends December 1, 2009): unjudged in both.
    let (run, json) = worlds(files(&[]), &["--known-at", "2009-01-01"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        filings(&json),
        json!({"motion": {"unjudged": ["w1", "w2"]}})
    );
    assert_eq!(json["result"]["known_at"], json!("2009-01-01"));
    let (_, json) = worlds(
        files(&[]),
        &["--known-at", "2009-06-01", "--as-of", "2010-05-01"],
    );
    assert_eq!(
        filings(&json),
        json!({"motion": {"timely": ["w1"], "late": ["w2"]}})
    );
    assert_eq!(json["result"]["as_of"], json!("2010-05-01"));
}

#[test]
fn layers_and_choices_come_from_the_command_line_too() {
    // A layer in every world, and one varied: four worlds, two outcomes.
    // The FRCP 87 what-if grants over the 2009 version of Rule 59(e), so
    // it brings the amendment with it (a layer brings its dependencies):
    // only the world with neither is the 2007 rule's.
    let (run, json) = worlds(
        files(&[]),
        &[
            "--layer",
            "change-sdny-local-civil-rules",
            "--vary",
            "change-frcp87-what-if",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let outcomes: Vec<(Value, Value)> = json["result"]["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| (o["worlds"].clone(), o["filings"].clone()))
        .collect();
    assert_eq!(
        outcomes,
        [
            (json!(["w1", "w2", "w3"]), json!({"motion": "timely"})),
            (json!(["w4"]), json!({"motion": "late"})),
        ]
    );
    let layers = &json["result"]["worlds"][0]["layers"];
    assert!(layers
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["name"] == "change-sdny-local-civil-rules"));
    // A name already in the world, or not loaded, is a usage error.
    for (args, why) in [
        (
            ["--layer", "change-frcp-2009-amendment"],
            "layer `change-frcp-2009-amendment` is in the world already",
        ),
        (
            ["--vary", "change-frcp-base"],
            "layer `change-frcp-base` is in the world already",
        ),
        (
            ["--vary", "rule-59e-2009"],
            "layer `rule-59e-2009` is not a loaded package",
        ),
    ] {
        let (run, _) = worlds(files(&[]), &args);
        assert_eq!(run.code, 1, "{args:?}: {}", run.stdout);
        assert!(run.stderr.contains(why), "{args:?}: {}", run.stderr);
    }
}

#[test]
fn a_varied_choice_named_like_a_layer_is_taken() {
    // `--vary NAME` makes a choice point named NAME.
    let world = example(WORLD).replace("[choice.rule-59e-2009]", "[choice.change-frcp87-what-if]");
    let (run, _) = worlds(
        files(&[(WORLD, &world)]),
        &["--vary", "change-frcp87-what-if"],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr.contains("is in the world already"),
        "{}",
        run.stderr
    );
}

#[test]
fn more_worlds_than_the_limit_is_a_usage_error() {
    // Twelve choices (4096 worlds; eleven of layers never loaded), and
    // one more varied on the command line.
    let choices = (0..11).fold(String::new(), |mut s, i| {
        let _ = write!(s, "\n[choice.c{i}]\nwithout = []\nwith = [\"l{i}\"]\n");
        s
    });
    let world = format!("{}{choices}", example(WORLD));
    let (run, _) = worlds(
        files(&[(WORLD, &world)]),
        &["--vary", "change-sdny-local-civil-rules"],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr.contains("span 8192 worlds, more than 4096"),
        "{}",
        run.stderr
    );
}

#[test]
fn each_filing_fares_in_each_world() {
    // An undocketed id-less filing (timely in both worlds), the motion, a
    // second motion replay rejects, and a judgment a party enters itself
    // (it answers no deadline); a synthetic layer varied in an actual
    // world leaves that world's law undetermined.
    let case = example(CASE).replace(
        "  motion: 2010-03-25 AlterAmendMoved() by movant-co\n",
        "  2010-03-02 change-frcp-base::JudgmentEntered() by movant-co\n  \
         motion: 2010-03-25 AlterAmendMoved() by movant-co\n  \
         again: 2010-03-26 AlterAmendMoved() by movant-co\n",
    );
    let world = example(WORLD).replace("modality = \"counterfactual\"\n", "");
    let (run, json) = worlds(
        files(&[(CASE, &case), (WORLD, &world)]),
        &["--vary", "change-frcp87-what-if"],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    insta::assert_json_snapshot!("each_filing", filings(&json));
}

#[test]
fn a_world_package_or_matter_that_fails_is_reported() {
    let world = example(WORLD).replace("change-frcp-base", "no-such-layer");
    let (run, json) = worlds(files(&[(WORLD, &world)]), &[]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(json["diagnostics"][0]["code"], json!("L1013"));
    let base = "docs/litigate/examples/change/base/base.lit";
    let broken = example(base).replace("start periods", "start nowhere");
    let (run, json) = worlds(files(&[(base, &broken)]), &[]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["severity"] == "error"));
    let (run, json) = worlds(files(&[(CASE, "case broken\n")]), &[]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(json["result"].is_null());
    // A matter its own lock does not pin fails in every world alike.
    let lock = "docs/litigate/examples/timelines/Lit.lock";
    let (run, json) = worlds(files(&[(lock, "version = 2\n")]), &[]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(json["diagnostics"][0]["code"], json!("L1007"));
}

#[test]
fn bad_arguments_are_usage_errors() {
    let run = |args: &[&str]| {
        let args: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
        litigate_cli::execute_in_memory(files(&[]), &args)
    };
    for (args, why) in [
        (
            vec!["worlds", CASE, "--world", WORLD, "--known-at", "2009-13-01"],
            "2009-13-01",
        ),
        (
            vec!["worlds", "nope.case", "--world", WORLD],
            "nope.case: no such file or directory",
        ),
        (
            vec!["worlds", CASE, "--world", "nope.world"],
            "nope.world: no such file or directory",
        ),
        (vec!["worlds", WORLD, "--world", WORLD], "not a .case file"),
        (vec!["worlds", CASE, "--world", CASE], "not a .world file"),
        (
            vec!["worlds", CASE, "--world", WORLD, "--path", "nowhere"],
            "no such file or directory",
        ),
    ] {
        let out = run(&args);
        assert_eq!(out.code, 1, "{args:?}: {}", out.stdout);
        assert!(out.stderr.contains(why), "{args:?}: {}", out.stderr);
    }
    assert!(Path::new(&repo().join(CASE)).is_file());
}

#[cfg(unix)]
#[test]
fn an_unreadable_world_file_fails_loading() {
    let dir = scratch("worlds-unreadable");
    std::fs::write(dir.join("m.case"), "case m\n").unwrap();
    std::fs::write(dir.join("w.world"), example(WORLD)).unwrap();
    chmod(&dir.join("w.world"), 0o000);
    let run = litigate_in(&dir, &["worlds", "m.case", "--world", "w.world"]);
    chmod(&dir.join("w.world"), 0o644);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("Permission denied"), "{}", run.stderr);
}
