// SPDX-License-Identifier: GPL-3.0-or-later
//! Embed the shared clock library (`clock/`: `lit/core` lowered, the one
//! generated file committed, because lowering needs the litigate compiler,
//! a dependent of this crate; `just lit-sync` regenerates it and
//! `lit-sync-check` fails while it is stale) and the `sources/` directory
//! into the library so the binary works standalone (no directory needed at
//! runtime). `LITGRAPH_SOURCES` still overrides at runtime. The packs,
//! calibration sets and named scenarios are built from `lit/` and `cases/`
//! by the `litgraph-packs` crate, which embeds those.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());

    embed_dir(
        &manifest.join("clock"),
        "json",
        Some("clock/"),
        "EMBEDDED_CLOCK",
        "`(clock/ file name, contents)` of the shared clock library: `lit/core`'s counting rules and holiday calendars, lowered.",
        &out_dir.join("embedded_clock.rs"),
    );
    // Without `embed-sources`, embed an empty corpus (point at a directory
    // that has no `*.txt`), keeping the static's shape identical.
    let sources_dir = if std::env::var_os("CARGO_FEATURE_EMBED_SOURCES").is_some() {
        manifest.join("../../sources")
    } else {
        manifest.join("src")
    };
    embed_dir(
        &sources_dir,
        "txt",
        Some("sources/"),
        "SOURCES_EMBEDDED",
        "`(repo-relative path, contents)` of every embedded L0 source text file, matching how `Pack.sources[].path` names them.",
        &out_dir.join("embedded_sources.rs"),
    );
}

/// Embed every `*.<ext>` file inside `dir` recursively as a
/// `pub static <ident>: &[(&str, &str)]`, key optionally prefixed with
/// `key_prefix` (e.g. `"sources/"` so the key matches `Pack.sources[].path`
/// exactly), written to `out_file`.
fn embed_dir(
    dir: &Path,
    ext: &str,
    key_prefix: Option<&str>,
    ident: &str,
    doc: &str,
    out_file: &Path,
) {
    println!("cargo:rerun-if-changed={}", dir.display());
    // A missing directory is a packaging bug (a Dockerfile or nix source
    // filter that forgot to copy it), not an empty library: fail the build
    // instead of silently shipping a binary with nothing embedded.
    let read = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "{} is missing ({e}); every directory build.rs embeds must be in the build \
             context (Dockerfile COPY, flake.nix dataDirs)",
            dir.display()
        )
    });
    let mut entries = Vec::new();
    collect_files(read, ext, &mut entries);
    entries.sort();
    let mut src = format!("/// {doc}\npub static {ident}: &[(&str, &str)] = &[\n");
    for p in &entries {
        println!("cargo:rerun-if-changed={}", p.display());
        let name = p.strip_prefix(dir).unwrap_or(p).to_string_lossy();
        let name = name.replace(std::path::MAIN_SEPARATOR, "/");
        let key = key_prefix.map_or_else(|| name.clone(), |prefix| format!("{prefix}{name}"));
        let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
        let _ = writeln!(
            src,
            "    ({key:?}, include_str!({:?})),",
            abs.display().to_string()
        );
    }
    src.push_str("];\n");
    if let Err(e) = std::fs::write(out_file, src) {
        panic!("writing {}: {e}", out_file.display());
    }
}

fn collect_files(read: std::fs::ReadDir, ext: &str, entries: &mut Vec<PathBuf>) {
    for path in read.filter_map(|entry| entry.ok().map(|entry| entry.path())) {
        if path.is_dir() {
            if let Ok(children) = std::fs::read_dir(&path) {
                collect_files(children, ext, entries);
            }
        } else if path.extension().is_some_and(|x| x == ext) {
            entries.push(path);
        }
    }
}
