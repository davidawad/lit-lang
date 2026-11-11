// SPDX-License-Identifier: GPL-3.0-or-later
//! Records the git commit the binary was built from, and whether the
//! working tree had uncommitted changes, so `litigate --version` names the
//! exact build installed (lg-rf4m).
//!
//! Sets `LITIGATE_GIT_COMMIT` (the full sha, or `unknown` outside a git
//! checkout) and `LITIGATE_GIT_TREE` (`clean`, `dirty`, or `unknown`).
//! Builds without a `.git` (a Docker context, a Nix source tarball) can set
//! either variable in the environment to supply it.
//!
//! Also embeds the stdlib packages a matter vendors, `lit/core` and
//! `lit/frcp` (their `Lit.toml`, `Lit.lock` and `.lit` files), as
//! `OUT_DIR/stdlib.rs`, so `litigate vendor` rewrites a vendored copy to
//! exactly the one this binary was built with (lg-70c6).

use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The `lit/` directories embedded as the stdlib `litigate vendor` writes.
const STDLIB: [&str; 2] = ["core", "frcp"];

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let dir = Path::new(&manifest);
    for var in ["LITIGATE_GIT_COMMIT", "LITIGATE_GIT_TREE"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    // Rerun when HEAD moves (checkout, commit) or the index changes
    // (staging; `git status` refreshes it after an edit too).
    for path in ["HEAD", "index", "packed-refs"] {
        if let Some(p) = git(dir, &["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={}", dir.join(p).display());
        }
    }
    if let Some(head) = git(dir, &["symbolic-ref", "-q", "HEAD"]) {
        if let Some(p) = git(dir, &["rev-parse", "--git-path", &head]) {
            println!("cargo:rerun-if-changed={}", dir.join(p).display());
        }
    }

    let commit = std::env::var("LITIGATE_GIT_COMMIT")
        .ok()
        .or_else(|| git(dir, &["rev-parse", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_owned());
    let tree = std::env::var("LITIGATE_GIT_TREE").ok().unwrap_or_else(|| {
        match git(dir, &["status", "--porcelain", "--untracked-files=no"]) {
            Some(s) if s.is_empty() => "clean".to_owned(),
            Some(_) => "dirty".to_owned(),
            None => "unknown".to_owned(),
        }
    });
    println!("cargo:rustc-env=LITIGATE_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=LITIGATE_GIT_TREE={tree}");
    println!(
        "cargo:rustc-env=LITIGATE_BUILD_FINGERPRINT={}",
        fingerprint(&dir.join("../.."))
    );
    embed_stdlib(&dir.join("../../lit"));
}

/// A deterministic identity of compiler inputs, computed while building
/// instead of hashing the hundreds-of-megabytes executable on every run.
fn fingerprint(root: &Path) -> String {
    let mut hash = Sha256::new();
    for relative in [
        "crates",
        "lit",
        "cases",
        "sources",
        "docs/litigate",
        "editors/tree-sitter-litigate",
    ] {
        let dir = root.join(relative);
        println!("cargo:rerun-if-changed={}", dir.display());
        hash_tree(root, &dir, &mut hash);
    }
    hash_root_files(root, &mut hash);
    hash_build_environment(&mut hash);
    hash_rustc(&mut hash);
    hash_external_build_inputs(&mut hash);
    hex_digest(hash)
}

fn hash_tree(root: &Path, dir: &Path, hash: &mut Sha256) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                hash_tree(root, &path, hash);
            }
        } else if path.is_file() {
            hash_file(root, &path, hash);
        }
    }
}

fn hash_file(root: &Path, path: &Path, hash: &mut Sha256) {
    if let Ok(bytes) = std::fs::read(path) {
        let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
        hash_named(hash, &relative, &bytes);
    }
}

fn hash_named(hash: &mut Sha256, name: &str, bytes: &[u8]) {
    hash.update((name.len() as u64).to_be_bytes());
    hash.update(name.as_bytes());
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

fn hash_root_files(root: &Path, hash: &mut Sha256) {
    // Root build metadata and the pinned Rust toolchain are inputs too.
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
    ] {
        let path = root.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        hash_file(root, &path, hash);
    }
}

fn hash_build_environment(hash: &mut Sha256) {
    for name in [
        "TARGET",
        "PROFILE",
        "CARGO_CFG_TARGET_OS",
        "CARGO_CFG_TARGET_ARCH",
    ] {
        hash_environment_value(hash, name);
    }
    let mut features: Vec<_> = std::env::vars()
        .filter(|(name, _)| name.starts_with("CARGO_FEATURE_"))
        .collect();
    features.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, value) in features {
        println!("cargo:rerun-if-env-changed={name}");
        hash_named(hash, &name, value.as_bytes());
    }
}

fn hash_environment_value(hash: &mut Sha256, name: &str) {
    println!("cargo:rerun-if-env-changed={name}");
    match std::env::var_os(name) {
        Some(value) => {
            hash_named(hash, name, b"set:");
            hash_named(hash, name, value.to_string_lossy().as_bytes());
        }
        None => hash_named(hash, name, b"unset"),
    }
}

fn hash_rustc(hash: &mut Sha256) {
    let version = std::env::var_os("RUSTC")
        .and_then(|rustc| std::process::Command::new(rustc).arg("-V").output().ok())
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    hash_named(hash, "rustc -V", version.as_bytes());
}

fn hash_external_build_inputs(hash: &mut Sha256) {
    for name in [
        "LITIGATE_SMT_Z3_ARCHIVE",
        "LITIGATE_SMT_Z3_SOURCE",
        "CC",
        "CXX",
        "CC_x86_64_unknown_linux_musl",
        "CXX_x86_64_unknown_linux_musl",
        "AR_x86_64_unknown_linux_musl",
        "RANLIB_x86_64_unknown_linux_musl",
    ] {
        hash_environment_value(hash, name);
    }
}

fn hex_digest(hash: Sha256) -> String {
    let mut fingerprint = String::with_capacity(64);
    for byte in hash.finalize() {
        let _ = write!(fingerprint, "{byte:02x}");
    }
    fingerprint
}

/// Writes `OUT_DIR/stdlib.rs`: `STDLIB_FILES`, `(DIR/relative path,
/// contents)` of every file the package loader reads in each [`STDLIB`]
/// directory. A build context without `lit/` embeds none.
fn embed_stdlib(lit: &Path) {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("stdlib.rs");
    let mut files = Vec::new();
    for name in STDLIB {
        let dir = lit.join(name);
        println!("cargo:rerun-if-changed={}", dir.display());
        package_files(&dir, &dir, name, &mut files);
    }
    files.sort();
    let mut src = String::from(
        "/// `(DIR/relative path, contents)` of the stdlib packages this binary \
         ships (`lit/core`, `lit/frcp`), as `litigate vendor` writes them.\n\
         pub static STDLIB_FILES: &[(&str, &str)] = &[\n",
    );
    for (key, path) in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
        let _ = writeln!(
            src,
            "    ({key:?}, include_str!({:?})),",
            abs.display().to_string()
        );
    }
    src.push_str("];\n");
    if let Err(e) = std::fs::write(&out, src) {
        panic!("writing {}: {e}", out.display());
    }
}

/// `Lit.toml`, `Lit.lock` and every `.lit` file under `dir`, skipping
/// hidden directories and nested packages (as the package loader does).
fn package_files(dir: &Path, base: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for path in read.filter_map(|e| e.ok().map(|e| e.path())) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if !name.starts_with('.') && !path.join("Lit.toml").is_file() {
                package_files(&path, base, prefix, out);
            }
        } else if name == "Lit.toml"
            || name == "Lit.lock"
            || path.extension().is_some_and(|e| e == "lit")
        {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy();
            out.push((format!("{prefix}/{}", rel.replace('\\', "/")), path));
        }
    }
}

/// `git ARGS` run in `dir`, its trimmed stdout, or `None` when git is
/// missing or fails (not a checkout).
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}
