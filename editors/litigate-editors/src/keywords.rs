//! Editor keyword lists, generated from the keyword table
//! (`crates/litigate-syntax/keywords.txt`, parsed by
//! [`litigate_syntax::keyword_table`]): the canonical `highlights.scm` from
//! its template `highlights.scm.in`, and Emacs's
//! `litigate-ts-mode-keywords.el`. Helix's copy follows from the canonical
//! query ([`crate::generated`]); Neovim's is a symlink to it.

use anyhow::{bail, Result};
use litigate_syntax::keyword_table::{Highlight, Keyword};
use std::fmt::Write as _;

/// The keyword table, relative to the repository root.
pub const TABLE: &str = "crates/litigate-syntax/keywords.txt";
/// The canonical highlights template, relative to the repository root.
pub const TEMPLATE: &str = "editors/tree-sitter-litigate/queries/highlights.scm.in";
/// The canonical highlights query, relative to the repository root.
pub const HIGHLIGHTS: &str = "editors/tree-sitter-litigate/queries/highlights.scm";
/// Emacs's keyword lists, relative to the repository root.
pub const EMACS: &str = "editors/emacs/litigate-ts-mode-keywords.el";

/// Line width the lists wrap at.
const WIDTH: usize = 80;

/// The table's keywords of class `highlight`, quoted, in table order.
fn quoted(rows: &[Keyword<'_>], highlight: Highlight) -> Vec<String> {
    rows.iter()
        .filter(|row| row.highlight == highlight)
        .map(|row| format!("{:?}", row.text))
        .collect()
}

/// `words` joined by spaces into lines of at most [`WIDTH`] columns, the
/// first starting with `first`, the rest with `rest`.
fn wrap(words: &[String], first: &str, rest: &str) -> String {
    let mut out = String::new();
    let mut line = first.to_owned();
    let mut fresh = true;
    for word in words {
        if !fresh && line.len() + 1 + word.len() > WIDTH {
            out.push_str(&line);
            out.push('\n');
            rest.clone_into(&mut line);
            fresh = true;
        }
        if !fresh {
            line.push(' ');
        }
        line.push_str(word);
        fresh = false;
    }
    out.push_str(&line);
    out
}

/// Render `highlights.scm` from its `template`: the template's first
/// paragraph (its own note) becomes a "generated" note, and each line that
/// is only `{{class}}` becomes that class's keywords at its indentation.
///
/// # Errors
/// A `{{class}}` naming no class, or a class with no keyword (an empty
/// list is not a valid query).
pub fn render_highlights(rows: &[Keyword<'_>], template: &str) -> Result<String> {
    let body = template
        .split_once("\n\n")
        .map_or(template, |(_, body)| body);
    let mut out = format!(
        "; Generated from highlights.scm.in and {TABLE} by\n\
         ; `just editor-queries`. Do not edit; edit those and rerun.\n\n"
    );
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("{{")
            .and_then(|rest| rest.strip_suffix("}}"))
        {
            let Some(highlight) = Highlight::ALL.into_iter().find(|h| h.name() == name) else {
                bail!("{TEMPLATE}: unknown keyword class `{{{{{name}}}}}`");
            };
            let words = quoted(rows, highlight);
            if words.is_empty() {
                bail!("{TEMPLATE}: no `{name}` keyword in {TABLE}");
            }
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push_str(&wrap(&words, indent, indent));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    Ok(out)
}

/// Render `litigate-ts-mode-keywords.el`.
#[must_use]
pub fn render_emacs(rows: &[Keyword<'_>]) -> String {
    let mut out = String::from(
        ";;; litigate-ts-mode-keywords.el --- Keyword lists of litigate-ts-mode  -*- lexical-binding: t; -*-\n\
         \n\
         ;; Copyright (C) 2026 David Awad\n\
         \n\
         ;; Author: David Awad\n\
         ;; URL: https://github.com/davidawad/litgraph\n\
         ;; SPDX-License-Identifier: GPL-3.0-or-later\n\
         \n\
         ;; This file is not part of GNU Emacs.\n\
         \n\
         ;;; Commentary:\n\
         \n\
         ;; Generated from crates/litigate-syntax/keywords.txt by `just\n\
         ;; editor-queries'.  Do not edit; edit keywords.txt and rerun.\n\
         \n\
         ;;; Code:\n",
    );
    let lists: [(&str, &[Highlight], &str); 4] = [
        (
            "declaration",
            &[Highlight::Declaration],
            "Keywords that open a declaration.",
        ),
        (
            "clause",
            &[Highlight::Clause, Highlight::Test],
            "Clause, field and test keywords.",
        ),
        (
            "conditional",
            &[Highlight::Conditional],
            "Conditional keywords.",
        ),
        ("directive", &[Highlight::Directive], "Directive keywords."),
    ];
    for (name, classes, doc) in lists {
        let words: Vec<String> = classes
            .iter()
            .flat_map(|class| quoted(rows, *class))
            .collect();
        let _ = write!(
            out,
            "\n(defconst litigate-ts-mode--{name}-keywords\n{})\n  \"{doc}\")\n",
            wrap(&words, "  '(", "    ")
        );
    }
    out.push_str(
        "\n(provide 'litigate-ts-mode-keywords)\n;;; litigate-ts-mode-keywords.el ends here\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_syntax::keyword_table::parse;

    const TABLE: &str = "KwA a declaration\nKwB b clause\nKwC c test\nKwD d conditional\n\
                         KwE e directive\nKwF f none\n";

    #[test]
    fn placeholders_take_their_class_at_their_indentation() {
        let rows = parse(TABLE).unwrap();
        let out = render_highlights(&rows, "; note\n\n[\n  {{clause}}\n] @keyword\n").unwrap();
        assert!(out.ends_with("\n\n[\n  \"b\"\n] @keyword\n"), "{out}");
        assert!(out.starts_with("; Generated from"), "{out}");
        assert!(!out.contains("; note"), "{out}");
    }

    #[test]
    fn unknown_and_empty_classes_are_errors() {
        let rows = parse(TABLE).unwrap();
        let unknown = render_highlights(&rows, "n\n\n{{bold}}\n").unwrap_err();
        assert!(unknown.to_string().contains("`{{bold}}`"), "{unknown}");
        let rows = parse("KwA a clause\n").unwrap();
        let empty = render_highlights(&rows, "n\n\n{{test}}\n").unwrap_err();
        assert!(empty.to_string().contains("no `test` keyword"), "{empty}");
    }

    #[test]
    fn emacs_clause_list_holds_the_test_keywords() {
        let rows = parse(TABLE).unwrap();
        let out = render_emacs(&rows);
        assert!(
            out.contains("(defconst litigate-ts-mode--clause-keywords\n  '(\"b\" \"c\")\n"),
            "{out}"
        );
        assert!(!out.contains("\"f\""), "{out}");
    }

    #[test]
    fn long_lists_wrap() {
        let words: Vec<String> = (0..30).map(|i| format!("\"word{i}\"")).collect();
        let out = wrap(&words, "  ", "  ");
        assert!(out.lines().count() > 1);
        assert!(out.lines().all(|line| line.len() <= WIDTH), "{out}");
    }
}
