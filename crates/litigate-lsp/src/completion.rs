//! Completion (`textDocument/completion`): the names a reference at the
//! cursor can resolve to, as the index resolves them (design review §4.4).
//!
//! After `pack::` the declarations of that package; otherwise the
//! declarations, enum variants and actors the file sees (its own, its
//! package's, and those of the packages it may name: dependencies, or the
//! packages a matter `uses`), a matter's parties, and the item keywords of
//! the file's kind.
use crate::analysis::Analysis;
use crate::index::{Def, Space};
use litigate_syntax::{FileKind, SyntaxKind as K, CASE_ITEM_KEYWORDS, LIT_ITEM_KEYWORDS};
use std::collections::BTreeSet;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

/// Whether `c` continues an identifier.
fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// The identifier being written up to byte `offset` of `text`, and the
/// `pack::` qualifier before it.
#[must_use]
pub fn prefix(text: &str, offset: usize) -> (String, Option<String>) {
    let before = text.get(..offset).unwrap_or(text);
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| ident_char(*c))
        .last()
        .map_or(before.len(), |(i, _)| i);
    let word = before[start..].to_owned();
    let qualifier = before[..start].strip_suffix("::").map(|q| {
        let from = q
            .char_indices()
            .rev()
            .take_while(|(_, c)| ident_char(*c))
            .last()
            .map_or(q.len(), |(i, _)| i);
        q[from..].to_owned()
    });
    (word, qualifier.filter(|q| !q.is_empty()))
}

fn kind_of(def: &Def) -> CompletionItemKind {
    match (&def.space, def.kind) {
        (Space::Variant(_), _) => CompletionItemKind::ENUM_MEMBER,
        (Space::Party | Space::Role, _) => CompletionItemKind::VARIABLE,
        (_, K::EnumDecl) => CompletionItemKind::ENUM,
        (_, K::EventDecl) => CompletionItemKind::EVENT,
        (_, K::TypeDecl) => CompletionItemKind::STRUCT,
        (_, K::DeadlineDecl | K::CountingDecl | K::CalendarDecl) => CompletionItemKind::CONSTANT,
        _ => CompletionItemKind::REFERENCE,
    }
}

/// What the kind of `def` is called (`state`, `deadline`, `variant`).
fn noun(def: &Def) -> String {
    match &def.space {
        Space::Variant(e) => format!("variant of {e}"),
        Space::Party => "party".to_owned(),
        Space::Role => "actor".to_owned(),
        _ => format!("{:?}", def.kind)
            .trim_end_matches("Decl")
            .to_lowercase(),
    }
}

/// Completions at byte `offset` of `file` (holding `text`).
#[must_use]
pub fn at(analysis: &Analysis, file: &str, text: &str, offset: usize) -> Vec<CompletionItem> {
    let (word, qualifier) = prefix(text, offset);
    let index = &analysis.index;
    let scope = index.scope(file);
    let qualifier = qualifier.map(|q| index.package_named(&q));
    let sees = |d: &Def| match (&qualifier, &d.package) {
        (Some(q), p) => d.space == Space::Decl && p.as_ref() == Some(q),
        (None, Some(p)) => scope.package.as_ref() == Some(p) || scope.visible.contains(p),
        (None, None) => d.file == file,
    };
    let mut seen = BTreeSet::new();
    let mut items: Vec<CompletionItem> = index
        .defs
        .iter()
        .filter(|d| {
            matches!(
                d.space,
                Space::Decl | Space::Variant(_) | Space::Party | Space::Role
            )
        })
        .filter(|d| d.name.starts_with(&word) && sees(d))
        .filter(|d| seen.insert(d.name.clone()))
        .map(|d| CompletionItem {
            label: d.name.clone(),
            kind: Some(kind_of(d)),
            detail: Some(match &d.package {
                Some(p) => format!("{} · {p}", noun(d)),
                None => noun(d),
            }),
            ..CompletionItem::default()
        })
        .collect();
    if qualifier.is_none() {
        let keywords = match Analysis::kind(file) {
            Some(FileKind::Case) => CASE_ITEM_KEYWORDS,
            _ => LIT_ITEM_KEYWORDS,
        };
        items.extend(
            keywords
                .iter()
                .filter_map(|k| k.text())
                .filter(|k| k.starts_with(&word) && seen.insert((*k).to_owned()))
                .map(|k| CompletionItem {
                    label: k.to_owned(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..CompletionItem::default()
                }),
        );
    }
    items
}

#[cfg(test)]
mod tests {
    use super::prefix;

    #[test]
    fn the_prefix_is_the_identifier_before_the_cursor() {
        let p = |t: &str| prefix(t, t.len());
        assert_eq!(p("x answer-p"), ("answer-p".into(), None));
        assert_eq!(p("counted_by "), (String::new(), None));
        assert_eq!(p("frcp::ans"), ("ans".into(), Some("frcp".into())));
        assert_eq!(p("x core::"), (String::new(), Some("core".into())));
        assert_eq!(p("::x"), ("x".into(), None));
        assert_eq!(p(""), (String::new(), None));
        // An offset past the text reads the whole text.
        assert_eq!(prefix("ab", 9), ("ab".into(), None));
    }
}
