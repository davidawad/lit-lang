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

/// A links module with one instance whose patch block is `body`.
fn patched(body: &str) -> super::Parse {
    parse_lit(&format!(
        "module links\ninstance base@one {{\n  pack base\n  patch go {{\n{body}\n  }}\n}}\n"
    ))
}

#[test]
fn guard_operations_parse_into_their_slot_and_are_reserved() {
    let parsed = patched(
        "    note \"narrowed\"\n    when and has_event(Served()) effective [2026-11-01, ..) cite \
         \"Fed. Cir. R. 1\"\n    when or date_of(Served()) < 2026-01-01\n    supersedes when \
         true known_at 2026-10-01 cite ?(\"Order, Dkt. 12\")\n    note \"after\"",
    );
    assert_eq!(codes(&parsed), ["L0010", "L0010", "L0010"]);
    for error in parsed.errors() {
        assert!(error.message.starts_with(crate::RESERVED_GUARD_OP));
    }
    assert!(parsed.errors()[0].message.contains("`and`"));
    assert!(parsed.errors()[1].message.contains("`or`"));
    // The `supersedes when` form is reported over the whole operation.
    let text = parsed.syntax().text().to_string();
    let range = parsed.errors()[2].range;
    assert!(text[range].starts_with("supersedes when true known_at"));
    assert!(text[range].ends_with("Dkt. 12\")"));
    let kinds = node_kinds(&parsed);
    assert_eq!(
        kinds.iter().filter(|k| **k == SyntaxKind::GuardOp).count(),
        3
    );
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    for kw in [
        SyntaxKind::KwAnd,
        SyntaxKind::KwOr,
        SyntaxKind::KwSupersedes,
    ] {
        assert!(parsed
            .syntax()
            .descendants_with_tokens()
            .any(|t| t.kind() == kw));
    }
    // The patch's own fields stay `Field`s beside the reserved slots.
    let fields = parsed
        .syntax()
        .descendants()
        .filter(|n| n.kind() == SyntaxKind::Field)
        .count();
    assert_eq!(fields, 3, "{kinds:?}");
}

#[test]
fn a_malformed_guard_operation_reports_where_it_breaks() {
    let no_when = patched("    supersedes true");
    assert_eq!(codes(&no_when), ["L0010", "L0001"]);
    let bare = patched("    when and");
    assert_eq!(codes(&bare), ["L0010", "L0001"]);
    let bare = patched("    supersedes when");
    assert_eq!(codes(&bare), ["L0010", "L0001"]);
    // A plain `when` is still a field; sema rejects it in a patch (L2007).
    assert!(patched("    when true").is_ok());
}

#[test]
fn and_and_or_are_reserved_everywhere_but_only_as_whole_words() {
    for source in [
        "module m\nstart and\n",
        "module m\ngroup or: \"x\"\n",
        "module m\nstate s: \"S\" {\n  go: \"Go\" -> s when and true\n}\n",
    ] {
        let parsed = parse_lit(source);
        assert!(codes(&parsed).contains(&"L0010"), "{source}");
        let reserved = parsed
            .errors()
            .iter()
            .filter(|e| e.code.as_str() == "L0010");
        for error in reserved {
            assert!(
                error.message.starts_with(crate::RESERVED_GUARD_OP),
                "{source}"
            );
        }
    }
    assert!(codes(&parse_case("case or\n")).contains(&"L0010"));
    assert_eq!(crate::GUARD_OP_WORDS, ["and", "or"]);
    assert!(
        parse_lit("module m\nstart reversed-and-remanded\ngroup or-else: \"and or\"\n").is_ok()
    );
}

/// A module whose only item after `module m` is `item`.
fn limited(item: &str) -> super::Parse {
    parse_lit(&format!("module m\n\n{item}\n\nstart s\n"))
}

#[test]
fn filing_limits_parse_into_their_slot_and_are_reserved() {
    let parsed = limited(
        "/// FRAP 32(a)(7) and 32(f)-(g).\nlimit principal-brief effective [1998-12-01, ..) {\n  \
         words <= 13_000 cite \"FRAP 32(a)(7)(B)(i)\"\n  lines <= 1300\n  pages <= ?(\"local \
         rule\")\n  requires certificate compliance cite \"FRAP 32(g)(1)\"\n  excludes \
         [cover-page, table-of-contents, ?(\"local rule\")] cite \"FRAP 32(f)\"\n  ecf \
         \"brief\"\n  ecf ?(\"district ECF code\")\n} cite \"FRAP 32\" known_at 2026-10-24",
    );
    assert_eq!(codes(&parsed), ["L0011", "L0011", "L0011"]);
    for error in parsed.errors() {
        assert!(error.message.starts_with(crate::RESERVED_FILING));
    }
    for (error, word) in parsed
        .errors()
        .iter()
        .zip(["limit", "certificate", "excludes"])
    {
        assert!(error.message.contains(&format!("`{word}`")), "{word}");
    }
    let kinds = node_kinds(&parsed);
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    let count = |kind| kinds.iter().filter(|k| **k == kind).count();
    assert_eq!(count(SyntaxKind::LimitDecl), 1);
    assert_eq!(count(SyntaxKind::LimitBound), 3);
    assert_eq!(count(SyntaxKind::CertificateReq), 1);
    assert_eq!(count(SyntaxKind::Exclusions), 1);
    assert_eq!(count(SyntaxKind::EcfCode), 2);
    assert_eq!(count(SyntaxKind::VersionClause), 2);
    assert_eq!(count(SyntaxKind::CiteExpr), 4);
    // The doc comment leads the declaration; the next item still parses.
    let limit = parsed
        .syntax()
        .children()
        .find(|n| n.kind() == SyntaxKind::LimitDecl)
        .expect("limit");
    assert!(limit.text().to_string().starts_with("/// FRAP 32"));
    assert!(kinds.contains(&SyntaxKind::StartDecl));
    for kw in [
        SyntaxKind::KwLimit,
        SyntaxKind::KwWords,
        SyntaxKind::KwPages,
        SyntaxKind::KwLines,
        SyntaxKind::KwRequires,
        SyntaxKind::KwCertificate,
        SyntaxKind::KwExcludes,
        SyntaxKind::KwEcf,
    ] {
        assert!(
            parsed
                .syntax()
                .descendants_with_tokens()
                .any(|t| t.kind() == kw),
            "{kw:?}"
        );
    }
}

#[test]
fn a_filing_limit_takes_relations_on_either_side_of_its_body() {
    let parsed = limited(
        "limit local::brief override frap::brief delegable_to [CaseOrder] {\n  words <= ?\n} \
         supersedes local::old-brief cite ?(\"local rule\")",
    );
    assert_eq!(codes(&parsed), ["L0011"]);
    let kinds = node_kinds(&parsed);
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    let relations = kinds.iter().filter(|k| **k == SyntaxKind::Relation).count();
    assert_eq!(relations, 3);
}

#[test]
fn a_malformed_filing_limit_reports_where_it_breaks() {
    for (item, expected) in [
        ("limit", vec!["L0011", "L0001"]),
        ("limit \"brief\"", vec!["L0011", "L0001"]),
        ("limit brief", vec!["L0011", "L0001"]),
        ("limit brief { words 13000 }", vec!["L0011", "L0001"]),
        ("limit brief { words <= }", vec!["L0011", "L0001"]),
        (
            "limit brief { requires compliance }",
            vec!["L0011", "L0001"],
        ),
        (
            "limit brief { requires certificate }",
            vec!["L0011", "L0011", "L0001"],
        ),
        (
            "limit brief { excludes cover-page }",
            vec!["L0011", "L0011", "L0001"],
        ),
        ("limit brief { ecf }", vec!["L0011", "L0001"]),
        ("limit brief { note \"x\" }", vec!["L0011", "L0001"]),
    ] {
        assert_eq!(codes(&limited(item)), expected, "{item}");
    }
}

#[test]
fn filing_words_are_reserved_everywhere_but_only_as_whole_words() {
    for source in [
        "module m\nstart limit\n",
        "module m\ngroup certificate: \"x\"\n",
        "module m\nstate s: \"S\" {\n  go \"Go\" -> s { requires [excludes] }\n}\n",
        "module m\nstate s: \"S\" {\n  go \"Go\" -> s { requires certificate service }\n}\n",
    ] {
        let parsed = parse_lit(source);
        assert!(codes(&parsed).contains(&"L0011"), "{source}");
        let reserved = parsed
            .errors()
            .iter()
            .filter(|e| e.code.as_str() == "L0011");
        for error in reserved {
            assert!(
                error.message.starts_with(crate::RESERVED_FILING),
                "{source}"
            );
        }
    }
    assert!(codes(&parse_case("case limit\n")).contains(&"L0011"));
    assert_eq!(crate::FILING_WORDS, ["limit", "certificate", "excludes"]);
    // `requires` stays the edge field; the body words stay identifiers.
    assert!(parse_lit(
        "module m\nstart no-transcript-certificate\ngroup words: \"limit\"\n\
         state s: \"S\" {\n  go \"Go\" -> s { requires [ecf, pages, lines] }\n}\n"
    )
    .is_ok());
}

#[test]
fn the_spec_filing_limit_example_parses_and_is_only_reserved() {
    let spec = include_str!("../../../../docs/litigate/SPEC.md");
    let section = &spec[spec
        .find("### 3.7 Reserved: filing requirements")
        .expect("§3.7")..];
    let body = section
        .split("```lit\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("a lit example in §3.7");
    let parsed = limited(body.trim_end());
    assert_eq!(codes(&parsed), ["L0011", "L0011", "L0011"], "{body}");
    let kinds = node_kinds(&parsed);
    assert!(!kinds.contains(&SyntaxKind::Error), "{kinds:?}");
    assert!(kinds.contains(&SyntaxKind::Relation));
}
