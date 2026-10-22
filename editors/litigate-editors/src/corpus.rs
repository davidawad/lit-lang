//! The `.lit`/`.case` corpus: every such file in the repository.

use std::path::{Path, PathBuf};

/// Directories that hold build output or third-party code, never sources
/// (the same set `tree-sitter-litigate/tests/conformance.rs` skips).
pub const SKIPPED_DIRS: [&str; 4] = [".git", "target", "node_modules", ".worktrees"];

/// Every `.lit` and `.case` file under `root`, sorted.
///
/// # Errors
/// When a directory cannot be read.
pub fn files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        let name = entry.file_name();
        if kind.is_dir() && !SKIPPED_DIRS.iter().any(|skip| name == *skip) {
            walk(&path, out)?;
        } else if kind.is_file() && is_source(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_source(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "lit" || ext == "case")
}
