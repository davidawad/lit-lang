use super::{parse_case, parse_lit, Edition, FileKind};
use crate::SyntaxKind;
use std::path::Path;

#[test]
fn file_kind_from_path_needs_a_known_utf8_extension() {
    assert_eq!(
        FileKind::from_path(Path::new("a/b.lit")),
        Some(FileKind::Lit)
    );
    assert_eq!(
        FileKind::from_path(Path::new("m.case")),
        Some(FileKind::Case)
    );
    assert_eq!(FileKind::from_path(Path::new("notes.md")), None);
    assert_eq!(FileKind::from_path(Path::new("Makefile")), None);
}

#[cfg(unix)]
#[test]
fn file_kind_from_path_rejects_a_non_utf8_extension() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(OsStr::from_bytes(b"x.\xFF"));
    assert_eq!(FileKind::from_path(path), None);
}

#[test]
fn parse_records_its_grammar_and_green_tree() {
    let lit = parse_lit("module m\n");
    assert_eq!(lit.kind(), FileKind::Lit);
    assert_eq!(lit.green().to_string(), "module m\n");
    assert_eq!(lit.green().kind(), crate::SyntaxKind::SourceFile.into());
    let case = parse_case("case c\n");
    assert_eq!(case.kind(), FileKind::Case);
}

fn codes(parsed: &super::Parse) -> Vec<&'static str> {
    parsed.errors().iter().map(|e| e.code.as_str()).collect()
}

#[test]
fn an_unmarked_file_is_the_default_edition() {
    assert_eq!(parse_lit("module m\n").edition(), Edition::E2026);
    assert_eq!(Edition::default(), Edition::E2026);
    assert_eq!(Edition::ALL, [Edition::E2026]);
    assert_eq!(Edition::from_marker("2026"), Some(Edition::E2026));
    assert_eq!(Edition::from_marker("2025"), None);
}

#[test]
fn the_edition_marker_leads_either_grammar() {
    let lit = parse_lit("edition 2026\n\nmodule m\n");
    assert!(lit.is_ok(), "{:?}", lit.errors());
    assert_eq!(lit.edition(), Edition::E2026);
    let decl = lit.syntax().first_child().map(|n| n.kind());
    assert_eq!(decl, Some(SyntaxKind::EditionDecl));
    let case = parse_case("edition 2026\ncase c\n");
    assert_eq!(case.edition(), Edition::E2026);
    assert_eq!(
        case.syntax().first_child().map(|n| n.kind()),
        Some(SyntaxKind::EditionDecl)
    );
}

#[test]
fn an_unknown_or_missing_edition_is_an_error() {
    let unknown = parse_lit("edition 2027\nmodule m\n");
    assert_eq!(codes(&unknown), ["L0008"]);
    assert!(unknown.errors()[0].message.contains("`2026`"));
    assert_eq!(unknown.edition(), Edition::E2026);
    let missing = parse_lit("edition\nmodule m\n");
    assert_eq!(codes(&missing), ["L0001"]);
    assert_eq!(missing.edition(), Edition::E2026);
}

#[test]
fn the_edition_marker_is_only_valid_first() {
    let late = parse_lit("module m\nedition 2026\n");
    assert_eq!(codes(&late), ["L0001"]);
    let after_case = parse_case("case c\nedition 2026\n");
    assert!(codes(&after_case).contains(&"L0001"), "{after_case:?}");
}

#[test]
fn reserved_words_are_not_identifiers() {
    for word in crate::RESERVED_WORDS {
        let parsed = parse_lit(&format!("module m\nstart {word}\n"));
        assert_eq!(codes(&parsed), ["L0007"], "{word}");
        assert!(parsed.errors()[0].message.contains(word));
    }
    // Only the whole word is reserved, and only outside strings.
    assert!(parse_lit("module m\nstart world-court\ngroup g: \"world\"\n").is_ok());
}
