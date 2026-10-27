//! Every generated editor file, rendered from its sources under a
//! repository root: the keyword lists ([`crate::keywords`]) and Helix's
//! queries ([`crate::helix`]). `just editor-queries` (the `editor-queries`
//! binary) writes them; `just tree-sitter-generate` and `just lit-sync` run
//! it, so a land's regeneration resolves two branches' keyword additions.

use crate::{helix, keywords};
use anyhow::{anyhow, Context as _, Result};
use std::path::{Path, PathBuf};

fn read(root: &Path, path: &str) -> Result<String> {
    let path = root.join(path);
    std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
}

/// `(path relative to root, expected contents)` of every generated file.
///
/// # Errors
/// When a source cannot be read, or the keyword table or highlights
/// template is malformed.
pub fn expected(root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let table = read(root, keywords::TABLE)?;
    let rows = litigate_syntax::keyword_table::parse(&table)
        .map_err(|message| anyhow!("{}: {message}", keywords::TABLE))?;
    let highlights = keywords::render_highlights(&rows, &read(root, keywords::TEMPLATE)?)?;
    let mut files = vec![
        (
            PathBuf::from(keywords::EMACS),
            keywords::render_emacs(&rows),
        ),
        (PathBuf::from(keywords::HIGHLIGHTS), highlights.clone()),
    ];
    files.extend(helix::expected(root, &highlights)?);
    Ok(files)
}

/// The generated files under `root` whose contents differ from [`expected`].
///
/// # Errors
/// As [`expected`].
pub fn stale(root: &Path) -> Result<Vec<PathBuf>> {
    Ok(expected(root)?
        .into_iter()
        .filter(|(path, text)| {
            std::fs::read_to_string(root.join(path)).ok().as_deref() != Some(text)
        })
        .map(|(path, _)| path)
        .collect())
}

/// Write every generated file under `root`; returns the paths written.
///
/// # Errors
/// As [`expected`], or when a file cannot be written.
pub fn write(root: &Path) -> Result<Vec<PathBuf>> {
    let files = expected(root)?;
    for (path, text) in &files {
        let full = root.join(path);
        std::fs::write(&full, text).with_context(|| format!("writing {}", full.display()))?;
    }
    Ok(files.into_iter().map(|(path, _)| path).collect())
}
