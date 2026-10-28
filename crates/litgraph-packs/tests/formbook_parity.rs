// SPDX-License-Identifier: GPL-3.0-or-later
//! Formbook parity (lg-u561.1): every court formbook supports has a
//! lit-lang package or outline. formbook's court list is vendored at a
//! pinned sha (tests/fixtures/formbook-courts.json, refreshed by `just
//! formbook-courts`); tests/fixtures/formbook-forums.json maps each
//! court's formbook path to the package directory that models it and that
//! package's forum id; tests/fixtures/formbook-missing.txt lists the
//! courts with none yet, and the parity beads shrink it to empty.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// formbook path -> court name, from the vendored court list.
fn courts() -> BTreeMap<String, String> {
    let list = json("tests/fixtures/formbook-courts.json");
    let mut courts = BTreeMap::new();
    for court in list["courts"].as_array().expect("courts is an array") {
        let path = court["path"].as_str().expect("court path").to_owned();
        let name = court["name"].as_str().expect("court name").to_owned();
        assert!(
            courts.insert(path.clone(), name).is_none(),
            "{path} listed twice"
        );
    }
    courts
}

/// formbook path -> (package directory, forum id).
fn forums() -> BTreeMap<String, (String, String)> {
    let map = json("tests/fixtures/formbook-forums.json");
    map.as_object()
        .expect("formbook-forums.json is an object")
        .iter()
        .map(|(path, entry)| {
            let field = |key: &str| {
                entry[key]
                    .as_str()
                    .unwrap_or_else(|| panic!("{path}: no {key}"))
                    .to_owned()
            };
            (path.clone(), (field("package"), field("forum")))
        })
        .collect()
}

fn missing() -> BTreeSet<String> {
    read("tests/fixtures/formbook-missing.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_formbook_court_has_a_package_or_outline() {
    let courts = courts();
    let forums = forums();
    let missing = missing();
    let mut errors = Vec::new();
    for (path, name) in &courts {
        match (forums.contains_key(path), missing.contains(path)) {
            (false, false) => errors.push(format!("unmapped: {path} ({name})")),
            (true, true) => errors.push(format!(
                "mapped but still in formbook-missing.txt (delete the line): {path}"
            )),
            _ => {}
        }
    }
    for path in forums.keys().chain(&missing) {
        if !courts.contains_key(path) {
            errors.push(format!("not a formbook court at the pinned sha: {path}"));
        }
    }
    for (path, (package, forum)) in &forums {
        let manifest = format!("{package}/Lit.toml");
        let Ok(text) = std::fs::read_to_string(repo().join(&manifest)) else {
            errors.push(format!("{path}: no package at {manifest}"));
            continue;
        };
        let toml: toml::Table = text.parse().unwrap_or_else(|e| panic!("{manifest}: {e}"));
        let declared = toml["package"].get("forum").and_then(toml::Value::as_str);
        if declared != Some(forum.as_str()) {
            errors.push(format!(
                "{path}: {manifest} declares forum {declared:?}, the map says {forum:?}"
            ));
        }
    }
    assert!(
        errors.is_empty(),
        "formbook parity: {} problem(s) (map a court in tests/fixtures/formbook-forums.json):\n  {}",
        errors.len(),
        errors.join("\n  ")
    );
}
