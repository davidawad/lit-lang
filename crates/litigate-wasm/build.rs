// SPDX-License-Identifier: GPL-3.0-or-later
//! Embed the repository's `lit/` library (every `Lit.toml`, `Lit.lock` and
//! `.lit` file) and `cases/` matters (every `.case` and `Lit.lock`),
//! recursively, so the wasm compiles them with no filesystem.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());
    let mut src = String::new();
    for (name, dir, what) in [
        ("EMBEDDED_LIBRARY", "lit", "`lit/` library"),
        ("EMBEDDED_CASES", "cases", "`cases/` matter"),
    ] {
        let root = manifest.join("../..").join(dir);
        let mut files = Vec::new();
        collect(&root, &mut files);
        files.sort();
        let _ = writeln!(
            src,
            "/// `(repository-relative path, contents)` of every embedded {what} file.\npub static {name}: &[(&str, &str)] = &["
        );
        for p in &files {
            let rel = p.strip_prefix(&root).unwrap_or(p);
            let key = format!("{dir}/{}", rel.to_string_lossy().replace('\\', "/"));
            let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
            let _ = writeln!(
                src,
                "    ({key:?}, include_str!({:?})),",
                abs.display().to_string()
            );
        }
        src.push_str("];\n");
    }
    let out_file = out_dir.join("embedded_sources.rs");
    if let Err(e) = std::fs::write(&out_file, src) {
        panic!("writing {}: {e}", out_file.display());
    }
}

/// Every source file under `dir`, skipping hidden entries.
fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    // A missing `lit/` or `cases/` is a packaging bug, not an empty tree.
    let read = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "{} is missing ({e}); litigate-wasm embeds lit/ and cases/",
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
            || path.extension().is_some_and(|e| e == "lit" || e == "case")
        {
            println!("cargo:rerun-if-changed={}", path.display());
            out.push(path);
        }
    }
}
