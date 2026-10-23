//! Helpers shared by the integration tests.
#![allow(dead_code)]
pub mod diag;

use litigate_syntax::{FileKind, SyntaxKind, SyntaxNode};
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
    write_shape(node, 0, &mut out);
    out
}

fn write_shape(node: &SyntaxNode, depth: usize, out: &mut String) {
    let braced = node
        .children_with_tokens()
        .any(|element| element.kind() == SyntaxKind::LBrace);
    let _ = writeln!(out, "{}{:?}", "  ".repeat(depth), node.kind());
    let mut imports = Vec::new();
    for element in node.children_with_tokens() {
        match element {
            rowan::NodeOrToken::Node(child) if child.kind() == SyntaxKind::ImportDecl => {
                let mut text = String::new();
                write_shape(&child, depth + 1, &mut text);
                imports.push(text);
            }
            rowan::NodeOrToken::Node(child) => {
                imports.sort();
                out.extend(imports.drain(..));
                write_shape(&child, depth + 1, out);
            }
            rowan::NodeOrToken::Token(token) => {
                let kind = token.kind();
                let trivia = matches!(
                    kind,
                    SyntaxKind::Whitespace | SyntaxKind::Comment | SyntaxKind::DocComment
                );
                let separator = braced && kind == SyntaxKind::Comma;
                if !(trivia || separator) {
                    let _ = writeln!(out, "{}{kind:?} {:?}", "  ".repeat(depth + 1), token.text());
                }
            }
        }
    }
    imports.sort();
    out.extend(imports);
}
