//! `lit/links` (SPEC §8.4) builds to a `links.json` semantically equal to
//! `packs/links.json`, and the engine gives the same answers over every
//! composition it routes when fed the generated file instead.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litgraph::api::{handle, Catalog, Request};
use serde_json::{json, Value};
use std::path::Path;
use support::{litigate, repo, scratch};

/// `(file name, contents)` of every `*.json` in `dir`, names prefixed.
fn json_files(dir: &Path, prefix: &str) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (
                format!("{prefix}{name}"),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

/// The shipped packs and clock library, with `links.json` taken from `links`.
fn catalog(links: &str) -> Catalog {
    let packs = repo().join("packs");
    let files = json_files(&packs, "")
        .into_iter()
        .map(|(n, t)| {
            if n == "links.json" {
                (n, links.to_owned())
            } else {
                (n, t)
            }
        })
        .chain(json_files(&packs.join("clock"), "clock/"));
    let catalog = Catalog::from_files("test".into(), files)
        .unwrap()
        .with_embedded_scenarios()
        .unwrap();
    assert!(catalog.load_errors.is_empty(), "{:?}", catalog.load_errors);
    catalog
}

/// Builds `lit/links` into a scratch directory; the generated `links.json`.
fn generated() -> String {
    let out = scratch("links-package");
    let build = litigate(&["build", "lit/links", "--out", out.to_str().unwrap()]);
    assert_eq!(build.code, 0, "{}", build.stdout);
    std::fs::read_to_string(out.join("packs/links.json")).unwrap()
}

#[test]
fn the_links_package_is_semantically_equal_to_packs_links_json() {
    let diff = litigate(&[
        "diff",
        "lit/links",
        "--against",
        "packs/links.json",
        "--check",
    ]);
    assert_eq!(diff.code, 0, "{}", diff.stdout);
    assert_eq!(diff.json["result"]["equal"], json!(true), "{}", diff.stdout);
    assert_eq!(diff.json["result"]["changes"], json!([]), "{}", diff.stdout);
}

#[test]
fn the_links_package_checks_and_its_tests_pass() {
    let check = litigate(&["check", "lit/links"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    let test = litigate(&["test", "lit/links"]);
    assert_eq!(test.code, 0, "{}", test.stdout);
    assert_eq!(test.json["result"]["failed"], json!(0), "{}", test.stdout);
    assert_eq!(test.json["result"]["passed"], json!(3), "{}", test.stdout);
}

/// Every link and instance in the generated file, by id, is the shipped one.
#[test]
fn the_generated_links_compose_every_forum_as_the_shipped_ones_do() {
    let ours = catalog(&generated());
    let theirs = catalog(&std::fs::read_to_string(repo().join("packs/links.json")).unwrap());
    assert_eq!(ours.links, theirs.links);

    let compositions: [(&[&str], &str); 5] = [
        (&["cofc", "cafc"], "cofc::claim-accrues"),
        (
            &["ptab-patent-trial-appeal-board", "cafc"],
            "ptab-patent-trial-appeal-board::patent-granted",
        ),
        (&["itc-337", "cafc"], "itc-337::complaint-filed"),
        (
            &["frcp-civil-procedure", "frap-appellate-procedure", "cafc"],
            "frcp-civil-procedure::pre-filing",
        ),
        (
            &["ttab-trademark-trial-and-appeal-board", "cafc"],
            "ttab-trademark-trial-and-appeal-board::published",
        ),
    ];
    let ops = [
        json!({"op": "validate"}),
        json!({"op": "solve"}),
        json!({"op": "chain"}),
        json!({"op": "structure"}),
    ];
    for (packs, start) in compositions {
        for op in &ops {
            let request: Request = serde_json::from_value(json!({
                "packs": packs,
                "scenario": {"start": start, "mixed": "optimistic", "prob_fill": "uniform"},
                "op": op,
            }))
            .unwrap();
            let a = handle(&request, &ours);
            let b = handle(&request, &theirs);
            assert!(a.ok, "{packs:?} {op}: {:?}", a.error);
            assert_eq!(a.result, b.result, "{packs:?} {op}");
            assert_eq!(
                serde_json::to_value(&a.warnings).unwrap(),
                serde_json::to_value(&b.warnings).unwrap(),
                "{packs:?} {op}"
            );
        }
    }

    // The one named scenario that composes packs runs the same way.
    let request: Request =
        serde_json::from_value(json!({"scenario": "ptab-ipr-defense", "op": {"op": "solve"}}))
            .unwrap();
    let (a, b) = (handle(&request, &ours), handle(&request, &theirs));
    assert!(a.ok, "{:?}", a.error);
    assert_eq!(a.result, b.result);
    assert_ne!(a.result, Some(Value::Null));
}
