//! `litigate fmt`: the canonical layout of SPEC §10.
//!
//! Formatting rewrites only trivia, separator commas inside braces, and the
//! order of `import` declarations. Files with syntax errors are refused.
mod build;
mod doc;

use crate::error::SyntaxError;
use crate::parse::{parse, FileKind, SyntaxNode, SyntaxToken};
use crate::syntax_kind::SyntaxKind::{
    self, CaseFile, Comment, DeadlineDecl, DocComment, Field, FieldList, ForumDefaultDecl, Ident,
    ImportDecl, KwForum, SourceFile, Whitespace,
};
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

/// L9004 check used by `litigate fmt --check` and the strict profile:
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
    let forum = module_forum(root);
    let (mut items, tail) = collect_items(root, forum.as_ref().map(|(name, _)| name.clone()));
    if let Some((name, _insert)) = forum.filter(|(_, insert)| *insert) {
        let position = items
            .iter()
            .rposition(|item| {
                matches!(
                    item.kind,
                    SyntaxKind::ModuleDecl | ImportDecl | SyntaxKind::PackageDecl
                )
            })
            .map_or(0, |index| index + 1);
        items.insert(
            position,
            Item {
                kind: ForumDefaultDecl,
                sort_key: None,
                leading: Vec::new(),
                body: format!("forum {name}"),
                trailing: Vec::new(),
            },
        );
    }
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
fn collect_items(root: &SyntaxNode, forum: Option<String>) -> (Vec<Item>, Vec<String>) {
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
                header
                    .get_or_insert_with(|| Builder::new(forum.clone()))
                    .token(&token);
                newlines = 0;
            }
            NodeOrToken::Node(node) => {
                if let Some(builder) = header.take() {
                    items.push(item(CaseFile, None, std::mem::take(&mut leading), builder));
                }
                let mut builder = Builder::new(forum.clone());
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

/// Returns a module's explicit default forum, or the common deadline forum
/// that formatting can safely hoist. The boolean says whether to insert it.
fn module_forum(root: &SyntaxNode) -> Option<(String, bool)> {
    if root.kind() != SourceFile {
        return None;
    }
    if let Some(decl) = root.children().find(|node| node.kind() == ForumDefaultDecl) {
        let forum = decl
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|token| token.kind() == Ident)?
            .text()
            .to_owned();
        return Some((forum, false));
    }
    let deadlines: Vec<_> = root
        .descendants()
        .filter(|node| node.kind() == DeadlineDecl)
        .collect();
    if deadlines.is_empty() {
        return None;
    }
    let forums: Vec<_> = deadlines
        .iter()
        .map(|deadline| {
            deadline
                .children()
                .find(|node| node.kind() == FieldList)
                .and_then(|fields| {
                    fields
                        .children()
                        .find_map(|field| simple_forum_field(&field))
                })
        })
        .collect();
    let first = forums.first()?.as_ref()?.clone();
    forums
        .iter()
        .all(|forum| forum.as_deref() == Some(first.as_str()))
        .then_some((first, true))
}

/// A bare `forum IDENT` option; adoption citations and comments make it
/// declaration-specific and therefore ineligible for hoisting.
pub(super) fn simple_forum_field(field: &SyntaxNode) -> Option<String> {
    if field.kind() != Field
        || field.parent().is_none_or(|list| {
            list.kind() != FieldList || list.parent().is_none_or(|d| d.kind() != DeadlineDecl)
        })
        || field
            .descendants_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .any(|token| matches!(token.kind(), Comment | DocComment))
    {
        return None;
    }
    let tokens: Vec<_> = field
        .descendants_with_tokens()
        .filter_map(NodeOrToken::into_token)
        .filter(|token| !matches!(token.kind(), Whitespace | Comment | DocComment))
        .collect();
    if tokens.len() == 2 && tokens[0].kind() == KwForum && tokens[1].kind() == Ident {
        Some(tokens[1].text().to_owned())
    } else {
        None
    }
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
