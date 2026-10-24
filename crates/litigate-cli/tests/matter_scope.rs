//! A matter sees only the closure of what it `uses` (lg-flu.49): a package
//! it does not depend on is not loaded for its `check`, `timeline` or
//! `run`, so that package's errors never appear in the matter's output nor
//! set its `ok: false`, and in a workspace check (which still reports
//! every package) it cannot change the matter's replay or schedule. `run`
//! composes a links package only when it joins two packages the matter
//! uses; a dependency of that links package outside the matter's closure
//! is loaded for it but never reported.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate, litigate_in, located, repo, scratch};

const TTAB: &str = "cases/ttab-opposition-applicant-defense.case";
const PTAB: &str = "cases/ptab-ipr-defense.case";
/// `lit/links` and every package it composes.
const FORUMS: [&str; 10] = [
    "cafc", "cofc", "core", "frap", "frcp", "itc", "links", "mpep", "ptab", "ttab",
];

/// A package `scotus` the matters do not use, broken: the worked example
/// with its FRCP source bound to another prefix, so its counting rule has
/// no source `url` and does not lower (L6001), though it checks in sema.
fn broken(dir: &Path) {
    let to = dir.join("lit/scotus");
    copy_dir(&repo().join("lit/examples"), &to);
    let manifest = std::fs::read_to_string(to.join("Lit.toml"))
        .unwrap()
        .replace("\"frcp-rule12-example\"", "\"scotus\"")
        .replace(
            "cite_prefixes = [\"FRCP\"]",
            "cite_prefixes = [\"Sup. Ct. R.\"]",
        );
    std::fs::write(to.join("Lit.toml"), manifest).unwrap();
    let module = to.join("rule12.lit");
    let text = std::fs::read_to_string(&module)
        .unwrap()
        .replace("frcp-rule12-example", "scotus");
    std::fs::write(&module, text).unwrap();
    let lock = litigate_in(dir, &["lock", "lit/scotus"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
}

/// `lit/<packages>` and the matters `cases`, locked, beside the broken package.
fn workspace(name: &str, packages: &[&str], cases: &[&str]) -> PathBuf {
    let dir = scratch(name);
    for pkg in packages {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    broken(&dir);
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    for case in cases {
        std::fs::copy(repo().join(case), dir.join(case)).unwrap();
    }
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

/// Files of `json`'s diagnostics under `lit/scotus` (by any relative spelling).
fn from_scotus(json: &Value) -> Vec<String> {
    located(json)
        .into_iter()
        .map(|(_, _, file, _)| file)
        .filter(|f| f.contains("scotus/"))
        .collect()
}

/// `(code, file)` of `json`'s errors.
fn errors(json: &Value) -> Vec<(String, String)> {
    located(json)
        .into_iter()
        .filter(|d| d.1 == "error")
        .map(|(code, _, file, _)| (code, file))
        .collect()
}

/// Names in `json`'s provenance.
fn provenance(json: &Value) -> Vec<String> {
    json["provenance"]["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_owned())
        .collect()
}

/// `json`'s matters and the diagnostics in them.
fn matters(json: &Value) -> Value {
    let own: Vec<&Value> = json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["span"]["file"].as_str().unwrap().starts_with("cases/"))
        .collect();
    serde_json::json!([json["result"]["cases"], own])
}

#[test]
fn a_matter_using_only_ttab_is_ok_beside_a_broken_package() {
    let dir = workspace("scope-ttab", &["core", "ttab"], &[TTAB]);
    for args in [
        &["check", TTAB][..],
        &["timeline", TTAB],
        &["run", TTAB, "--op", "chain"],
        &["run", TTAB, "--op", "chain", "--no-links"],
    ] {
        let out = litigate_in(&dir, args);
        assert_eq!(out.code, 0, "{args:?}: {}", out.stdout);
        assert_eq!(out.json["ok"], true, "{args:?}");
        assert_eq!(from_scotus(&out.json), Vec::<String>::new(), "{args:?}");
        assert_eq!(
            provenance(&out.json),
            ["core", "ttab-trademark-trial-and-appeal-board"],
            "{args:?}"
        );
    }
}

#[test]
fn the_broken_package_fails_a_workspace_check_and_leaves_the_matter_alone() {
    let dir = workspace("scope-workspace", &["core", "ttab"], &[TTAB]);
    for paths in [&["lit"][..], &["lit", "cases"], &["lit/scotus"]] {
        let out = litigate_in(&dir, &[&["check"][..], paths].concat());
        assert_eq!(out.code, 2, "{paths:?}: {}", out.stdout);
        assert_eq!(out.json["ok"], false);
        assert_eq!(
            errors(&out.json),
            [("L6001".to_owned(), "lit/scotus/rule12.lit".to_owned())],
            "{paths:?}"
        );
    }
    // The workspace check reports the matter exactly as checking it alone
    // does: the broken package's counting rule does not empty its clock.
    let alone = litigate_in(&dir, &["check", TTAB]);
    let whole = litigate_in(&dir, &["check", "lit", "cases"]);
    assert_eq!(matters(&whole.json), matters(&alone.json));
}

#[test]
fn run_composes_links_between_used_packages_without_reporting_their_other_dependencies() {
    let dir = workspace("scope-links", &FORUMS, &[TTAB, PTAB]);
    let manifest = dir.join("lit/links/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap().replace(
        "[dependencies]\n",
        "[dependencies]\nscotus = { version = \"^0.1.0\", path = \"../scotus\" }\n",
    );
    std::fs::write(&manifest, text).unwrap();
    let lock = litigate_in(&dir, &["lock", "lit/links"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    // `lit/links` now depends on the broken package: it joins ptab and
    // cafc, so the PTAB matter composes it, yet still runs as on the repo.
    let run = litigate_in(&dir, &["run", PTAB, "--op", "chain"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(from_scotus(&run.json), Vec::<String>::new());
    assert!(provenance(&run.json).contains(&"links".to_owned()));
    assert!(!provenance(&run.json).contains(&"scotus".to_owned()));
    let shipped = litigate(&["run", PTAB, "--op", "chain"]);
    assert_eq!(
        run.json["result"]["result"],
        shipped.json["result"]["result"]
    );
    // It joins only one package the TTAB matter uses: not loaded at all.
    let ttab = litigate_in(&dir, &["run", TTAB, "--op", "chain"]);
    assert_eq!(ttab.code, 0, "{}", ttab.stdout);
    assert!(!provenance(&ttab.json).contains(&"links".to_owned()));
}

#[test]
fn a_links_package_a_matter_composes_fails_it_closed() {
    let dir = workspace("scope-links-broken", &FORUMS, &[PTAB]);
    let manifest = dir.join("lit/links/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap().replace(
        "[dependencies]\n",
        "[dependencies]\nnowhere = { version = \"^0.1.0\", path = \"../nowhere\" }\n",
    );
    std::fs::write(&manifest, text).unwrap();
    let run = litigate_in(&dir, &["run", PTAB, "--op", "chain"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none());
    assert!(
        errors(&run.json)
            .iter()
            .all(|(_, file)| file.starts_with("lit/links/")),
        "{:?}",
        errors(&run.json)
    );
    // Without links it is not loaded.
    let plain = litigate_in(&dir, &["run", PTAB, "--op", "chain", "--no-links"]);
    assert!(!provenance(&plain.json).contains(&"links".to_owned()));
}
