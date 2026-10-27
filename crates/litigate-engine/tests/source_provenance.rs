// SPDX-License-Identifier: GPL-3.0-or-later
//! Every vendored L0 text (`sources/**/*.txt`) has its current sha256
//! recorded in a provenance record, so an edit to a file without a matching
//! provenance update (or a provenance edit that drops a file's entry) fails
//! here instead of going unnoticed (lg-1jqa).
//!
//! A file's record is `sources/PROVENANCE.md` or a `PROVENANCE.md` /
//! `MANIFEST.json` in its own directory or a directory between it and
//! `sources/` (the per-court subdirectories keep their own
//! `PROVENANCE.md`; `sources/change-corpus/{rules,rulings}` keep a
//! `MANIFEST.json`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const RECORDS: [&str; 2] = ["PROVENANCE.md", "MANIFEST.json"];

fn sources_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sources")
}

fn txt_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read sources dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            txt_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "txt") {
            out.push(path);
        }
    }
}

/// The provenance records that may cover `file`: those in its directory and
/// each ancestor up to and including `root`.
fn records_for(file: &Path, root: &Path) -> Vec<String> {
    let mut texts = Vec::new();
    let mut dir = file.parent();
    while let Some(d) = dir {
        for name in RECORDS {
            if let Ok(text) = fs::read_to_string(d.join(name)) {
                texts.push(text);
            }
        }
        if d == root {
            break;
        }
        dir = d.parent();
    }
    texts
}

#[test]
fn every_vendored_text_has_its_sha256_recorded() {
    let root = sources_root().canonicalize().expect("sources/ exists");
    let mut files = Vec::new();
    txt_files(&root, &mut files);
    files.sort();
    assert!(
        files.len() > 300,
        "expected the full corpus, found {} files",
        files.len()
    );

    let missing: Vec<String> = files
        .iter()
        .filter_map(|file| {
            let digest = format!(
                "{:x}",
                Sha256::digest(fs::read(file).expect("read source text"))
            );
            let recorded = records_for(file, &root).iter().any(|r| r.contains(&digest));
            (!recorded).then(|| format!("{} {digest}", file.strip_prefix(&root).unwrap().display()))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "{} of {} sources/**/*.txt files have no provenance record holding their current \
         sha256 (sources/PROVENANCE.md, or a PROVENANCE.md / MANIFEST.json in the file's \
         directory or an ancestor under sources/); record the hash with the file's \
         retrieval and extraction notes:\n{}",
        missing.len(),
        files.len(),
        missing.join("\n")
    );
}
