//! Reads package trees from disk: the root packages, their `path`
//! dependencies (transitively), and the `core` stdlib when configured. This
//! is the only module in sema that touches the filesystem.
use super::manifest::parse_manifest;
use std::collections::BTreeSet;
use std::io;
use std::path::{Component, Path, PathBuf};

/// One `.lit` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFile {
    /// Package-relative POSIX path.
    pub relative: String,
    /// Contents (invalid UTF-8 decoded lossily; the lexer reports L0003).
    pub text: String,
}

/// One package directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPackage {
    /// Repository-relative POSIX directory (`.` for the root).
    pub root: String,
    /// `Lit.toml` text.
    pub manifest: String,
    /// `Lit.lock` text, if present.
    pub lock: Option<String>,
    /// `.lit` files sorted by relative path.
    pub files: Vec<LoadedFile>,
}

/// Repository-relative POSIX rendering of `path` (as given when outside).
#[must_use]
pub fn relative_to(repo_root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".to_owned()),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    }
}

fn read_lossy(path: &Path) -> io::Result<String> {
    std::fs::read(path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Collects `.lit` files under `dir`, skipping hidden directories and
/// nested packages (directories with their own `Lit.toml`).
fn collect(dir: &Path, base: &Path, out: &mut Vec<LoadedFile>) -> io::Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<_>>()?;
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if !name.starts_with('.') && !path.join("Lit.toml").exists() {
                collect(&path, base, out)?;
            }
        } else if path.extension().is_some_and(|e| e == "lit") {
            out.push(LoadedFile {
                relative: relative_to(base, &path),
                text: read_lossy(&path)?,
            });
        }
    }
    Ok(())
}

/// Loads one package directory.
///
/// # Errors
/// I/O errors reading `Lit.toml` or sources.
pub fn load_package(repo_root: &Path, dir: &Path) -> io::Result<LoadedPackage> {
    let manifest = read_lossy(&dir.join("Lit.toml"))?;
    let lock_path = dir.join("Lit.lock");
    let lock = if lock_path.exists() {
        Some(read_lossy(&lock_path)?)
    } else {
        None
    };
    let mut files = Vec::new();
    collect(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(LoadedPackage {
        root: relative_to(repo_root, dir),
        manifest,
        lock,
        files,
    })
}

/// Loads `roots`, every `path` dependency reachable from them, and the
/// stdlib directory when given. Each directory is loaded once; the result
/// keeps discovery order (roots first).
///
/// # Errors
/// I/O errors; a dependency path that does not exist is left for sema to
/// report as L1006.
pub fn load_tree(
    repo_root: &Path,
    roots: &[PathBuf],
    stdlib: Option<&Path>,
) -> io::Result<Vec<LoadedPackage>> {
    let mut seen = BTreeSet::new();
    let mut queue: Vec<PathBuf> = roots.to_vec();
    queue.extend(stdlib.map(Path::to_path_buf));
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(dir) = queue.get(i).cloned() {
        i += 1;
        let key = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if !seen.insert(key) || !dir.join("Lit.toml").is_file() {
            continue;
        }
        let pkg = load_package(repo_root, &dir)?;
        if let Ok(parsed) = parse_manifest(&pkg.manifest) {
            queue.extend(parsed.dependency_paths.values().map(|p| dir.join(p)));
        }
        out.push(pkg);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_worked_example() -> io::Result<()> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let pkgs = load_tree(&repo, &[repo.join("lit/examples")], None)?;
        let summary: Vec<(String, Vec<String>, bool)> = pkgs
            .into_iter()
            .map(|p| {
                (
                    p.root,
                    p.files.into_iter().map(|f| f.relative).collect(),
                    p.lock.is_some(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [(
                "lit/examples".to_owned(),
                vec!["rule12.lit".to_owned()],
                false
            )]
        );
        Ok(())
    }

    #[test]
    fn relative_paths_are_posix() {
        assert_eq!(
            relative_to(Path::new("/r"), Path::new("/r/lit/core/a.lit")),
            "lit/core/a.lit"
        );
        assert_eq!(relative_to(Path::new("/r"), Path::new("/r")), ".");
    }
}
