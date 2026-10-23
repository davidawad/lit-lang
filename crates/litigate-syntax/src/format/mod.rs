//! `litigate fmt`: the canonical layout of SPEC §10.
//!
//! Formatting rewrites only trivia, separator commas inside braces, and the
//! order of `import` declarations. Files with syntax errors are refused.
mod build;
mod doc;

use crate::error::SyntaxError;
use crate::parse::{parse, FileKind, SyntaxNode, SyntaxToken};
use crate::syntax_kind::SyntaxKind::{self, CaseFile, Comment, DocComment, ImportDecl, Whitespace};
use build::Builder;
use litigate_base::{Diagnostic, DiagnosticCode, FileId, LineIndex};
use rowan::NodeOrToken;

/// Maximum line width before braced bodies break (SPEC §10).
pub const WIDTH: usize = 100;

/// Formatting refused because the source has syntax errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    /// The L0xxx errors that block formatting.
    pub errors: Vec<SyntaxError>,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot format a file with {} syntax error(s)",
            self.errors.len()
        )
    }
}

impl std::error::Error for FormatError {}

/// Formats `text` canonically.
///
/// # Errors
/// Returns the syntax errors when `text` does not parse cleanly.
pub fn format(text: &str, kind: FileKind) -> Result<String, FormatError> {
    let parsed = parse(text, kind);
    if !parsed.is_ok() {
        return Err(FormatError {
            errors: parsed.errors().to_vec(),
        });
    }
    Ok(format_tree(&parsed.syntax()))
}

/// L0009-style check used by `litigate fmt --check` and the strict profile:
/// `None` when `text` is canonical (or unparsable: syntax errors are
/// reported elsewhere), else an L9004 diagnostic at the first difference.
#[must_use]
pub fn check_format(text: &str, kind: FileKind, file: &FileId) -> Option<Diagnostic> {
    let formatted = format(text, kind).ok()?;
    if formatted == text {
        return None;
    }
    let at = text
        .bytes()
        .zip(formatted.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| text.len().min(formatted.len()));
    let range = crate::error::range(at, at);
    let error = SyntaxError::new(
        DiagnosticCode::NonCanonicalFormat,
        range,
        "file is not in canonical format; run `litigate fmt`",
    )
    .with_fix(
        crate::error::range(0, text.len()),
        formatted,
        litigate_base::Applicability::MachineApplicable,
    );
    Some(error.to_diagnostic(file, &LineIndex::new(text)))
}

/// One top-level item rendered on its own, with attached comments.
struct Item {
    kind: SyntaxKind,
    sort_key: Option<String>,
    leading: Vec<String>,
    body: String,
    trailing: Vec<String>,
}

impl Item {
    /// Lines of the item, not counting doc comments and comments.
    fn single_line(&self) -> bool {
        self.body
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .count()
            <= 1
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for comment in &self.leading {
            out.push_str(comment);
            out.push('\n');
        }
        out.push_str(&self.body);
        for comment in &self.trailing {
            out.push(' ');
            out.push_str(comment);
        }
        out
    }
}

fn format_tree(root: &SyntaxNode) -> String {
    let (mut items, tail) = collect_items(root);
    sort_imports(&mut items);
    let case = root.kind() == CaseFile;
    let mut out = String::new();
    for (index, item) in items.iter().enumerate() {
        if let Some(prev) = index.checked_sub(1).and_then(|i| items.get(i)) {
            let contiguous = if case {
                prev.single_line() && item.single_line()
            } else {
                prev.kind == item.kind && prev.single_line() && item.single_line()
            };
            out.push_str(if contiguous { "\n" } else { "\n\n" });
        }
        out.push_str(&item.render());
    }
    // A clean parse has at least one item (`module` or the `case` header),
    // so the tail always follows one.
    if !tail.is_empty() {
        out.push_str("\n\n");
        out.push_str(&tail.join("\n"));
    }
    // Item bodies and comments end in text, so no blank lines trail.
    let mut text = out
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    text.push('\n');
    text
}

/// Splits the root into items. Root-level comments attach to the next
/// item (or trail the previous one when on its line); the rest form the
/// tail. A `.case` header (`case id`) is its own item; a clean `.case` file
/// always has item nodes after it, and the first one flushes it.
fn collect_items(root: &SyntaxNode) -> (Vec<Item>, Vec<String>) {
    let mut items: Vec<Item> = Vec::new();
    let mut leading: Vec<String> = Vec::new();
    let mut newlines = 0usize;
    let mut header: Option<Builder> = None;
    for element in root.children_with_tokens() {
        match element {
            NodeOrToken::Token(token) if token.kind() == Whitespace => {
                newlines += token.text().matches('\n').count();
            }
            NodeOrToken::Token(token) if matches!(token.kind(), Comment | DocComment) => {
                if let Some(builder) = header.take() {
                    items.push(item(CaseFile, None, std::mem::take(&mut leading), builder));
                }
                let text = token.text().trim_end().to_owned();
                match items.last_mut() {
                    Some(last) if newlines == 0 && leading.is_empty() => {
                        last.trailing.push(text);
                    }
                    _ => {
                        leading.push(text);
                    }
                }
                newlines = 0;
            }
            NodeOrToken::Token(token) => {
                header.get_or_insert_with(Builder::new).token(&token);
                newlines = 0;
            }
            NodeOrToken::Node(node) => {
                if let Some(builder) = header.take() {
                    items.push(item(CaseFile, None, std::mem::take(&mut leading), builder));
                }
                let mut builder = Builder::new();
                builder.node(&node);
                let key = (node.kind() == ImportDecl).then(|| significant_text(&node));
                items.push(item(
                    node.kind(),
                    key,
                    std::mem::take(&mut leading),
                    builder,
                ));
                newlines = 0;
            }
        }
    }
    (items, leading)
}

fn item(kind: SyntaxKind, sort_key: Option<String>, leading: Vec<String>, b: Builder) -> Item {
    Item {
        kind,
        sort_key,
        leading,
        body: doc::print(&b.finish(), WIDTH),
        trailing: Vec::new(),
    }
}

/// Sorts each run of consecutive imports by module path.
fn sort_imports(items: &mut [Item]) {
    for run in items.chunk_by_mut(|a, b| a.kind == ImportDecl && b.kind == ImportDecl) {
        run.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    }
}

fn significant_text(node: &SyntaxNode) -> String {
    node.descendants_with_tokens()
        .filter_map(NodeOrToken::into_token)
        .filter(|token: &SyntaxToken| !crate::parser::is_trivia(token.kind()))
        .map(|token| token.text().to_owned())
        .collect()
}

#[cfg(test)]
mod tests;
