//! The inventory over the repository's own `lit/` (every promoted package
//! in the court table, the elements its build gives) and over small trees
//! for each way a manifest or the court table can be wrong.
#![allow(clippy::unwrap_used, clippy::too_many_lines, clippy::unnecessary_wraps)]

use litigate_feed::inventory::{build_elements, Inventory};
use litigate_feed::watched;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A scratch tree with `files` (path, text).
fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litigate-feed-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

const BASE: &str = r#"
[package]
name = "ca9-court"
pack_id = "ca9-court"
[sources.local]
url = "https://example.test/rules.pdf"
cite_prefixes = ["9th Cir. R."]
[[cite_syntax.rules]]
prefixes = ["FRAP", "Fed. R. App. P."]
"#;

const PRECEDENT: &str = r#"
[package]
name = "bowles"
pack_id = "bowles"
kind = "precedent"
[package.instrument]
court = "scotus"
designation = "551 U.S. 205"
[dependencies]
ca9-court = { version = "^1", path = "../ca9" }
[sources.opinion]
url = "https://www.courtlistener.com/opinion/145721/bowles-v-russell/"
"#;

#[test]
fn every_promoted_package_is_in_the_court_table() {
    let inventory = Inventory::read(&repo()).unwrap();
    assert_eq!(inventory.courts["ca8"], ["ca8-court-of-appeals"]);
    assert_eq!(inventory.courts["uscfc"], ["cofc"]);
    assert!(inventory.precedents.is_empty());
    let ca8 = inventory.package("ca8-court-of-appeals").unwrap();
    assert_eq!(ca8.dir, "lit/ca8");
    assert!(ca8.deps.contains(&"lit/frap".to_owned()));
    assert!(inventory
        .prefix_groups()
        .iter()
        .any(|g| g.contains(&"8th Cir. R.".to_owned())));
}

#[test]
fn the_watched_packages_build_into_elements_with_cites() {
    let root = repo();
    let inventory = Inventory::read(&root).unwrap();
    let names = watched(&inventory);
    assert!(names.contains("frap-appellate-procedure") && names.contains("core"));
    let elements = build_elements(&root, &inventory, &names).unwrap();
    let ca8 = &elements["ca8-court-of-appeals"];
    let notice = ca8.iter().find(|e| e.id == "notice-filed").unwrap();
    assert_eq!(notice.cite.as_deref(), Some("FRAP 3(a)(1)"));
    assert!(!elements["frap-appellate-procedure"].is_empty());
    assert!(build_elements(&root, &inventory, &BTreeSet::new())
        .unwrap()
        .is_empty());
}

#[test]
fn precedents_resolve_through_courtlistener_urls_and_designations() {
    let root = tree(
        "precedent",
        &[
            ("lit/ca9/Lit.toml", BASE),
            ("lit/bowles/Lit.toml", PRECEDENT),
        ],
    );
    let courts = "[packages.ca9-court]\ncourts = [\"ca9\"]\n";
    let inventory = Inventory::read_with(&root, courts).unwrap();
    assert_eq!(inventory.courts["ca9"], ["ca9-court"]);
    let p = &inventory.precedents[0];
    assert_eq!(
        (p.package.as_str(), p.citation.as_deref(), p.cluster),
        ("bowles", Some("551 U.S. 205"), Some(145_721))
    );
    let ca9 = inventory.package("ca9-court").unwrap();
    assert_eq!(
        ca9.prefix_groups,
        [vec!["FRAP", "Fed. R. App. P."], vec!["9th Cir. R."]]
    );
    assert_eq!(ca9.source_urls, ["https://example.test/rules.pdf"]);
    assert_eq!(
        inventory.closure(["bowles"]),
        BTreeSet::from(["bowles".to_owned(), "ca9-court".to_owned()])
    );
    assert!(watched(&inventory).contains("bowles"));
}

#[test]
fn a_wrong_court_table_or_manifest_is_an_error() {
    let root = tree("table", &[("lit/ca9/Lit.toml", BASE)]);
    let err = |courts: &str| Inventory::read_with(&root, courts).unwrap_err();
    assert!(err("[packages]\n").contains("`ca9-court` is not listed"));
    assert!(err("[packages.ca9-court]\n").contains("exactly one"));
    assert!(
        err("[packages.ca9-court]\ncourts = [\"ca9\"]\nnone = \"x\"\n").contains("exactly one")
    );
    assert!(
        err("[packages.ca9-court]\ncourts = [\"ca9\"]\n[packages.gone]\nnone = \"x\"\n")
            .contains("`gone` is not a promoted package")
    );
    assert!(err("[packages.ca9-court]\nwrong = 1\n").starts_with("courts.toml:"));

    let bad = tree("manifest", &[("lit/x/Lit.toml", "[package]\n")]);
    assert!(Inventory::read_with(&bad, "")
        .unwrap_err()
        .contains("Lit.toml"));
    let missing = tree("missing", &[]);
    assert!(Inventory::read_with(&missing, "").is_err());
}

#[test]
fn a_failed_build_is_an_error() {
    let root = tree(
        "build",
        &[
            ("lit/ca9/Lit.toml", BASE),
            ("lit/ca9/broken.lit", "this is not litigate"),
        ],
    );
    let courts = "[packages.ca9-court]\ncourts = [\"ca9\"]\n";
    let inventory = Inventory::read_with(&root, courts).unwrap();
    let err = build_elements(&root, &inventory, &watched(&inventory)).unwrap_err();
    assert!(err.contains("`litigate build lit/ca9"), "{err}");
}
