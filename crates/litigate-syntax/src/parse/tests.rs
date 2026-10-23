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

/// The kinds of every node under the root, in preorder.
fn node_kinds(parsed: &super::Parse) -> Vec<SyntaxKind> {
    parsed.syntax().descendants().map(|n| n.kind()).collect()
}

#[test]
fn refinement_types_parse_into_their_slot_and_are_reserved() {
    let parsed = parse_lit(
        "module m\ntype Short = {d: Days | d <= 60.days && d > 0.days}\n\
         event Served(on: {t: Date | t >= 2026-01-01}, how: String)\n",
    );
    assert_eq!(codes(&parsed), ["L0009", "L0009"]);
    for error in parsed.errors() {
        assert!(error.message.starts_with(crate::RESERVED_REFINEMENT));
    }
    assert_eq!(
        parsed.errors()[0].range,
        crate::TextRange::new(22.into(), 60.into())
    );
    let kinds = node_kinds(&parsed);
    assert_eq!(
        kinds
            .iter()
            .filter(|k| **k == SyntaxKind::RefinedType)
            .count(),
        2
    );
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    // Nested refinements are reserved at every level.
    let nested = parse_lit("module m\ntype T = {a: {b: Int | b > 0} | a < 9}\n");
    assert_eq!(codes(&nested), ["L0009", "L0009"]);
}

#[test]
fn a_malformed_refinement_type_reports_where_it_breaks() {
    let unclosed = parse_lit("module m\ntype T = {d: Days | d > 0.days\n");
    // One error per position: the unclosed brace (with its fix) wins.
    assert_eq!(codes(&unclosed), ["L0002"]);
    let no_pipe = parse_lit("module m\ntype T = {d: Days}\n");
    assert_eq!(codes(&no_pipe), ["L0009", "L0001"]);
    let no_type = parse_lit("module m\ntype T = {d: | d}\nstart s\n");
    assert_eq!(codes(&no_type), ["L0009", "L0001"]);
    let no_binder = parse_lit("module m\ntype T = {: Days | d}\nstart s\n");
    assert_eq!(codes(&no_binder), ["L0009", "L0001"]);
    let no_colon = parse_lit("module m\ntype T = {d Days | d}\nstart s\n");
    assert_eq!(codes(&no_colon), ["L0009", "L0001"]);
    // Unclosed before the next item: the brace's L0002, then that item.
    let anchored = parse_lit("module m\ntype T = {d: Days | d > 0.days\nstart s\n");
    assert_eq!(codes(&anchored), ["L0002"]);
    assert!(anchored
        .syntax()
        .descendants()
        .any(|n| n.kind() == SyntaxKind::StartDecl));
    let no_param_type = parse_lit("module m\nevent E(x: 1)\n");
    assert_eq!(codes(&no_param_type), ["L0001"]);
    let trailing = parse_lit("module m\ntype T = {d: Days | d e}\nstart s\n");
    assert_eq!(codes(&trailing), ["L0009", "L0001"]);
}

#[test]
fn where_clauses_parse_into_their_slot_and_are_reserved() {
    let parsed = parse_lit(
        "module m\ntype Short = Days where it <= 60.days\n\
         event Served(on: Date where on >= 2026-01-01, how: String)\nstart s\n",
    );
    assert_eq!(codes(&parsed), ["L0009", "L0009"]);
    assert!(parsed.errors()[0]
        .message
        .starts_with(crate::RESERVED_REFINEMENT));
    let kinds = node_kinds(&parsed);
    assert_eq!(
        kinds
            .iter()
            .filter(|k| **k == SyntaxKind::WhereClause)
            .count(),
        2
    );
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    assert!(parsed
        .syntax()
        .descendants_with_tokens()
        .any(|t| t.kind() == SyntaxKind::KwWhere));
    // A clause without its predicate still parses the next item.
    let bare = parse_lit("module m\ntype T = Days where\nstart s\n");
    assert_eq!(codes(&bare), ["L0009", "L0001"]);
}

#[test]
fn where_is_reserved_everywhere_but_only_as_a_whole_word() {
    for source in ["module m\nstart where\n", "module m\ngroup where: \"x\"\n"] {
        let parsed = parse_lit(source);
        assert_eq!(codes(&parsed), ["L0009"], "{source}");
        assert!(parsed.errors()[0].message.contains("`where`"));
    }
    assert!(codes(&parse_case("case where\n")).contains(&"L0009"));
    assert_eq!(crate::REFINEMENT_WORDS, ["where"]);
    assert!(parse_lit("module m\nstart where-filed\ngroup g: \"where\"\n").is_ok());
}

#[test]
fn a_bare_pipe_outside_a_refinement_is_unexpected() {
    let parsed = parse_lit("module m\nstart a | b\n");
    assert!(codes(&parsed).iter().all(|c| *c == "L0001"), "{parsed:?}");
    assert!(!codes(&parsed).is_empty());
}
