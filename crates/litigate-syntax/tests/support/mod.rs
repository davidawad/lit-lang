//! Helpers shared by the integration tests.
#![allow(dead_code)]
pub mod diag;

use litigate_syntax::{FileKind, SyntaxKind, SyntaxNode, SyntaxToken};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.lit` and `.case` file under `lit/` and `cases/`, sorted.
pub fn corpus() -> Vec<(PathBuf, FileKind)> {
    let mut found = Vec::new();
    for dir in ["lit", "cases"] {
        walk(&repo_root().join(dir), &mut found);
    }
    found.sort();
    found
        .into_iter()
        .filter_map(|path| FileKind::from_path(&path).map(|kind| (path, kind)))
        .collect()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Structural signature of a tree: node kinds and significant token text,
/// ignoring trivia, block separator commas, and import order.
pub fn shape(node: &SyntaxNode) -> String {
    let mut out = String::new();
    let forum = module_forum(node);
    write_shape(node, 0, forum.as_deref(), &mut out);
    out
}

fn write_shape(node: &SyntaxNode, depth: usize, forum: Option<&str>, out: &mut String) {
    if node.kind() == SyntaxKind::ForumDefaultDecl {
        return;
    }
    if node.kind() == SyntaxKind::FieldList
        && forum.is_some_and(|default| {
            let fields: Vec<_> = node
                .children()
                .filter(|child| child.kind() == SyntaxKind::Field)
                .collect();
            !fields.is_empty()
                && fields
                    .iter()
                    .all(|field| simple_forum(field).as_deref() == Some(default))
        })
    {
        return;
    }
    if node.kind() == SyntaxKind::Field
        && forum.is_some_and(|default| simple_forum(node).as_deref() == Some(default))
    {
        return;
    }
    let braced = node
        .children_with_tokens()
        .any(|element| element.kind() == SyntaxKind::LBrace);
    let _ = writeln!(out, "{}{:?}", "  ".repeat(depth), node.kind());
    let mut imports = Vec::new();
    for element in node.children_with_tokens() {
        match element {
            rowan::NodeOrToken::Node(child) if child.kind() == SyntaxKind::ImportDecl => {
                let mut text = String::new();
                write_shape(&child, depth + 1, forum, &mut text);
                imports.push(text);
            }
            rowan::NodeOrToken::Node(child) => {
                imports.sort();
                out.extend(imports.drain(..));
                write_shape(&child, depth + 1, forum, out);
            }
            rowan::NodeOrToken::Token(token) => {
                let kind = token.kind();
                let trivia = matches!(
                    kind,
                    SyntaxKind::Whitespace | SyntaxKind::Comment | SyntaxKind::DocComment
                );
                let separator = braced && kind == SyntaxKind::Comma;
                let redundant_id = node.kind() == SyntaxKind::EdgeDecl
                    && kind == SyntaxKind::Ident
                    && redundant_edge_id(node, &token);
                if !(trivia || separator || redundant_id) {
                    let _ = writeln!(out, "{}{kind:?} {:?}", "  ".repeat(depth + 1), token.text());
                }
            }
        }
    }
    imports.sort();
    out.extend(imports);
}

fn module_forum(root: &SyntaxNode) -> Option<String> {
    if let Some(decl) = root
        .children()
        .find(|node| node.kind() == SyntaxKind::ForumDefaultDecl)
    {
        return decl
            .children_with_tokens()
            .filter_map(rowan::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .map(|token| token.text().to_owned());
    }
    let deadlines: Vec<_> = root
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::DeadlineDecl)
        .collect();
    let first = deadlines.first().and_then(|deadline| {
        deadline
            .children()
            .find(|node| node.kind() == SyntaxKind::FieldList)
            .and_then(|fields| fields.children().find_map(|field| simple_forum(&field)))
    })?;
    deadlines
        .iter()
        .all(|deadline| {
            deadline
                .children()
                .find(|node| node.kind() == SyntaxKind::FieldList)
                .and_then(|fields| fields.children().find_map(|field| simple_forum(&field)))
                .as_deref()
                == Some(first.as_str())
        })
        .then_some(first)
}

fn simple_forum(field: &SyntaxNode) -> Option<String> {
    if field.kind() != SyntaxKind::Field
        || !field
            .children_with_tokens()
            .any(|element| element.kind() == SyntaxKind::KwForum)
    {
        return None;
    }
    let reference = field
        .children()
        .find(|child| child.kind() == SyntaxKind::Ref)?;
    let identifiers: Vec<_> = reference
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(|token| token.kind() == SyntaxKind::Ident)
        .collect();
    (identifiers.len() == 1 && reference.children().next().is_none())
        .then(|| identifiers[0].text().to_owned())
}

fn redundant_edge_id(node: &SyntaxNode, token: &SyntaxToken) -> bool {
    let Some(target) = node
        .children()
        .find(|child| child.kind() == SyntaxKind::Ref)
        .and_then(|reference| reference.last_token())
        .map(|target| target.text().to_owned())
    else {
        return false;
    };
    let Some(edge_block) = node.parent() else {
        return false;
    };
    let Some(declaration) = edge_block.parent() else {
        return false;
    };
    let Some(from) = declaration
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .find(|part| part.kind() == SyntaxKind::Ident)
        .map(|part| part.text().to_owned())
    else {
        return false;
    };
    let targets: Vec<_> = edge_block
        .children()
        .filter(|edge| edge.kind() == SyntaxKind::EdgeDecl)
        .filter(|edge| {
            edge.children()
                .find(|child| child.kind() == SyntaxKind::Ref)
                .and_then(|reference| reference.last_token())
                .is_some_and(|last| last.text() == target)
        })
        .collect();
    let expected = if targets.len() > 1 {
        let Some(ordinal) = targets
            .iter()
            .position(|edge| edge.text_range() == node.text_range())
        else {
            return false;
        };
        format!("edge-{from}-{target}-{}", ordinal + 1)
    } else {
        format!("edge-{from}-{target}")
    };
    token.text() == expected
}
