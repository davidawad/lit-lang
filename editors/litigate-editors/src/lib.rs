//! Tooling behind litigate's editor modes (`editors/{emacs,helix,nvim}`).
//!
//! - [`grammar`] builds `tree-sitter-litigate` as a shared library for a
//!   given tree-sitter ABI (the `litigate-grammar` binary).
//! - [`helix`] renders Helix's queries from the canonical ones (the
//!   `helix-queries` binary).
//! - [`corpus`] and [`support`] are shared by the editor tests in `tests/`,
//!   which drive Emacs, Helix and Neovim when they are installed.

pub mod corpus;
pub mod grammar;
pub mod helix;
pub mod support;

use std::path::{Path, PathBuf};

/// The repository root (two levels above this crate), canonicalized when
/// possible so tools that dislike `..` segments (Helix) get a plain path.
#[must_use]
pub fn repo_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::canonicalize(&root).unwrap_or(root)
}

/// `editors/` in the repository.
#[must_use]
pub fn editors_dir() -> PathBuf {
    repo_root().join("editors")
}
