//! Helix's copies of the canonical queries.
//!
//! Helix names highlight scopes differently from the common tree-sitter
//! convention (`@constant.numeric` for `@number`, `@namespace` for
//! `@module`; see <https://docs.helix-editor.com/themes.html>), uses
//! `@indent`/`@outdent` for indentation, and highlights a local reference
//! with the scope named by its definition. So
//! `editors/helix/queries/litigate/{highlights,indents,locals,injections}.scm`
//! are rendered from `editors/tree-sitter-litigate/queries` by renaming
//! captures. `textobjects.scm` is hand-written and Helix-only.

use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

/// The generated query files.
pub const GENERATED: [&str; 4] = [
    "highlights.scm",
    "indents.scm",
    "locals.scm",
    "injections.scm",
];

/// Capture renames for one file: `(canonical, Some(helix))` renames,
/// `(canonical, None)` drops the capture.
type Renames = &'static [(&'static str, Option<&'static str>)];

const HIGHLIGHTS: Renames = &[
    ("keyword.conditional", Some("keyword.control.conditional")),
    ("string.escape", Some("constant.character.escape")),
    ("comment.documentation", Some("comment.line.documentation")),
    ("number", Some("constant.numeric")),
    ("boolean", Some("constant.builtin.boolean")),
    ("module", Some("namespace")),
    ("type.definition", Some("type")),
    ("property", Some("variable.other.member")),
    ("function.call", Some("function")),
];

const INDENTS: Renames = &[
    ("indent.begin", Some("indent")),
    ("indent.branch", Some("outdent")),
    ("indent.end", None),
];

const LOCALS: Renames = &[
    (
        "local.definition.parameter",
        Some("local.definition.variable.parameter"),
    ),
    ("local.definition.var", Some("local.definition.label")),
    (
        "local.definition.field",
        Some("local.definition.variable.other.member"),
    ),
    ("local.definition.import", Some("local.definition.variable")),
];

/// `editors/helix/queries/litigate`.
#[must_use]
pub fn queries_dir() -> PathBuf {
    crate::repo_root().join(DIR)
}

/// Render the Helix copy of canonical query `file` from its `source` text.
#[must_use]
pub fn render(file: &str, source: &str) -> String {
    let renames: Renames = match file {
        "highlights.scm" => HIGHLIGHTS,
        "indents.scm" => INDENTS,
        "locals.scm" => LOCALS,
        _ => &[],
    };
    let mut out = format!(
        "; Generated from editors/tree-sitter-litigate/queries/{file} by\n\
         ; `just editor-queries`. Do not edit; edit the source and rerun.\n\n"
    );
    // Helix has no @indent.auto: strings and block comments keep their text.
    for line in source.lines().filter(|line| !line.contains("@indent.auto")) {
        out.push_str(&rename_line(line, renames));
        out.push('\n');
    }
    out
}

/// Rename every `@capture` in `line` whose whole name is in `renames`.
fn rename_line(line: &str, renames: Renames) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find('@') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let len = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
            .unwrap_or(after.len());
        let name = &after[..len];
        match renames.iter().find(|(from, _)| *from == name) {
            Some((_, Some(to))) => {
                out.push('@');
                out.push_str(to);
            }
            Some((_, None)) => {
                let kept = out.trim_end().len();
                out.truncate(kept);
            }
            None => {
                out.push('@');
                out.push_str(name);
            }
        }
        rest = &after[len..];
    }
    out.push_str(rest);
    out
}

/// Helix's directory, relative to the repository root.
pub const DIR: &str = "editors/helix/queries/litigate";

/// Render every generated file under `root` from the canonical queries,
/// taking `highlights` as the canonical `highlights.scm` (itself generated,
/// [`crate::keywords`]): `(path relative to root, expected contents)`.
///
/// # Errors
/// When a canonical query cannot be read.
pub fn expected(root: &Path, highlights: &str) -> Result<Vec<(PathBuf, String)>> {
    let canonical = root.join("editors/tree-sitter-litigate/queries");
    GENERATED
        .iter()
        .map(|file| {
            let source = if *file == "highlights.scm" {
                highlights.to_owned()
            } else {
                let path = canonical.join(file);
                std::fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?
            };
            Ok((Path::new(DIR).join(file), render(file, &source)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_whole_capture_names_only() {
        assert_eq!(
            rename_line("(number) @number @numberish @module", HIGHLIGHTS),
            "(number) @constant.numeric @numberish @namespace"
        );
    }

    #[test]
    fn dropped_captures_take_their_space() {
        assert_eq!(
            rename_line("[ \"}\" ] @indent.branch @indent.end", INDENTS),
            "[ \"}\" ] @outdent"
        );
    }

    #[test]
    fn predicates_keep_their_capture_names() {
        assert_eq!(
            rename_line("(#any-of? @function.builtin \"len\")", HIGHLIGHTS),
            "(#any-of? @function.builtin \"len\")"
        );
    }

    #[test]
    fn indent_auto_lines_are_dropped() {
        let out = render(
            "indents.scm",
            "(a) @indent.begin\n[ (string) ] @indent.auto\n",
        );
        assert!(out.ends_with("(a) @indent\n"), "{out}");
    }
}
