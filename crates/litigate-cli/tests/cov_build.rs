//! `build` beyond the worked example: several packages, lowering that
//! fails only at the build's `known_at`, and an output directory that
//! cannot be written.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{litigate_in, located, repo, scratch};

#[test]
fn several_packages_combine_their_litigatable_hashes() {
    let out = scratch("build-several");
    let run = litigate_in(
        &repo(),
        &[
            "build",
            "--out",
            out.to_str().unwrap(),
            "lit/examples",
            "lit/core",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    let hashes: Vec<&str> = result["litigatables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["content_hash"].as_str().unwrap())
        .collect();
    assert_eq!(hashes.len(), 2);
    let combined = result["litigatable_hash"].as_str().unwrap();
    assert!(combined.starts_with("sha256:"), "{combined}");
    assert!(
        !hashes.contains(&combined),
        "{combined} is one package's hash"
    );
}

#[test]
fn lowering_at_an_early_known_at_fails_the_build() {
    let out = scratch("build-known-at");
    let run = litigate_in(
        &repo(),
        &[
            "build",
            "--known-at",
            "2000-01-01",
            "--out",
            out.to_str().unwrap(),
            "lit/examples",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    let errors: Vec<_> = located(&run.json)
        .into_iter()
        .filter(|d| d.1 == "error")
        .collect();
    insta::assert_debug_snapshot!(errors);
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
}

#[test]
fn an_unwritable_output_is_an_io_failure() {
    let dir = scratch("build-unwritable");
    std::fs::write(dir.join("out"), "a file, not a directory").unwrap();
    let run = litigate_in(
        &dir,
        &[
            "build",
            "--out",
            "out",
            &repo().join("lit/examples").to_string_lossy(),
        ],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!(run.stderr, @"litigate build: out/litigatables: Not a directory (os error 20)");
}
