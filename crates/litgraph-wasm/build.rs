// SPDX-License-Identifier: GPL-3.0-or-later
//! Embed the repository's `lit/` library (every `Lit.toml`, `Lit.lock` and
//! `.lit` file, recursively) so `litigate check`/`build` in wasm find the
//! library packages a matter `uses` with no filesystem.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());
    let lit = manifest.join("../../lit");
    let mut files = Vec::new();
    collect(&lit, &mut files);
    files.sort();
    let mut src = String::from(
        "/// `(repository-relative path, contents)` of every embedded `lit/` library file.\npub static EMBEDDED_LIBRARY: &[(&str, &str)] = &[\n",
    );
    for p in &files {
        let rel = p.strip_prefix(&lit).unwrap_or(p);
        let key = format!("lit/{}", rel.to_string_lossy().replace('\\', "/"));
        let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
        let _ = writeln!(
            src,
            "    ({key:?}, include_str!({:?})),",
            abs.display().to_string()
        );
    }
    src.push_str("];\n");
    let out_file = out_dir.join("embedded_library.rs");
    if let Err(e) = std::fs::write(&out_file, src) {
        panic!("writing {}: {e}", out_file.display());
    }
}

/// Every library file under `dir`, skipping hidden entries.
fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    // A missing `lit/` is a packaging bug, not an empty library.
    let read = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "{} is missing ({e}); litgraph-wasm embeds lit/",
            dir.display()
        )
    });
    for path in read.filter_map(|e| e.ok().map(|e| e.path())) {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect(&path, out);
        } else if name == "Lit.toml"
            || name == "Lit.lock"
            || path.extension().is_some_and(|e| e == "lit")
        {
            println!("cargo:rerun-if-changed={}", path.display());
            out.push(path);
        }
    }
}
