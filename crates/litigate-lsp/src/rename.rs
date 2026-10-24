//! Rename (`textDocument/prepareRename`, `textDocument/rename`): a
//! declaration, every version of it, and every reference to it across
//! modules, packages and the matters that use them, as one multi-file edit
//! (design review §4.3-§4.4).
//!
//! What can be renamed is what the index resolves: declarations, edges,
//! enum variants, a matter's parties, and local names. A module is named by
//! its path and an actor by its `Lit.toml` key, so neither is renamed here.
use crate::analysis::Analysis;
use crate::index::Space;
use litigate_syntax::{SyntaxKind as K, TextRange, TextSize};
use std::collections::BTreeMap;

/// Why `name` cannot be a new name, if it cannot: it must be one
/// identifier, not a keyword or a reserved word.
///
/// # Errors
/// The reason, as a sentence.
pub fn valid(name: &str) -> Result<(), String> {
    let (tokens, errors) = litigate_syntax::lexer::lex(name);
    let one_ident = matches!(tokens.as_slice(), [t] if t.kind == K::Ident);
    if one_ident && errors.is_empty() && K::from_keyword(name).is_none() {
        Ok(())
    } else {
        Err(format!(
            "`{name}` is not an identifier a declaration can have"
        ))
    }
}

/// The definition to rename at `offset` of `file`, if one is there and
/// can be renamed.
fn target(analysis: &Analysis, file: &str, offset: TextSize) -> Option<usize> {
    analysis
        .index
        .at(file, offset)
        .filter(|&i| !matches!(analysis.index.defs[i].space, Space::Module | Space::Role))
}

/// The identifier being renamed at `offset` of `file` (its range) and its
/// name, for `prepareRename`.
#[must_use]
pub fn prepare(analysis: &Analysis, file: &str, offset: TextSize) -> Option<(TextRange, String)> {
    let def = &analysis.index.defs[target(analysis, file, offset)?];
    let range = analysis
        .index
        .uses
        .iter()
        .find(|u| u.file == file && u.range.contains_inclusive(offset))
        .map_or(def.name_range, |u| u.range);
    Some((range, def.name.clone()))
}

/// The ranges to replace, by file, to rename what is at `offset` of
/// `file`: every version of the definition (the definitions of its name in
/// its namespace and package) and every reference to one of them.
#[must_use]
pub fn ranges(
    analysis: &Analysis,
    file: &str,
    offset: TextSize,
) -> BTreeMap<String, Vec<TextRange>> {
    let Some(i) = target(analysis, file, offset) else {
        return BTreeMap::new();
    };
    let index = &analysis.index;
    let def = &index.defs[i];
    let versions: Vec<usize> = index
        .defs
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            d.name == def.name
                && d.space == def.space
                && d.package == def.package
                && (d.package.is_some() || d.file == def.file)
        })
        .map(|(j, _)| j)
        .collect();
    let mut out: BTreeMap<String, Vec<TextRange>> = BTreeMap::new();
    for &j in &versions {
        let d = &index.defs[j];
        out.entry(d.file.clone()).or_default().push(d.name_range);
        for u in index.references(j) {
            out.entry(u.file.clone()).or_default().push(u.range);
        }
    }
    for edits in out.values_mut() {
        edits.sort_by_key(|r| r.start());
        edits.dedup();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::valid;

    #[test]
    fn a_new_name_is_one_identifier() {
        assert_eq!(valid("answer-period"), Ok(()));
        assert_eq!(valid("12b-motion"), Ok(()));
        for bad in ["", "two words", "a::b", "state", "cite", "\"x\"", "1"] {
            assert!(valid(bad).is_err(), "{bad}");
        }
        // A reserved word lexes as an identifier with an error.
        let reserved = litigate_syntax::RESERVED_WORDS[0];
        assert!(valid(reserved).is_err(), "{reserved}");
    }
}
