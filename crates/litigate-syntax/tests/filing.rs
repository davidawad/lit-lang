//! Filing requirements (SPEC §3.7) through the public parser: every entry
//! shape, well-formed or broken where the grammar can break, parses into a
//! `LimitDecl` with exactly the errors expected.
use litigate_syntax::{parse_lit, SyntaxKind};

/// `(limit body, diagnostic codes)`.
pub const SHAPES: &[(&str, &[&str])] = &[
    ("words <= 13_000 cite \"FRAP 32(a)(7)(B)(i)\"", &[]),
    ("pages <= ?", &[]),
    ("lines <= ?(\"local rule\")", &[]),
    ("words <=", &["L0001"]),
    ("words 1", &["L0001"]),
    ("requires certificate compliance", &[]),
    ("requires certificate compliance for [words, lines]", &[]),
    ("requires certificate compliance for words", &["L0001"]),
    ("requires certificate", &["L0001"]),
    ("requires compliance", &["L0001"]),
    (
        "excludes [cover-page, ?(\"local rule\")] cite \"FRAP 32(f)\"",
        &[],
    ),
    ("excludes cover-page", &["L0001"]),
    ("ecf \"brief\"", &[]),
    ("ecf ?(\"district event\")", &[]),
    ("ecf", &["L0001"]),
    ("note \"x\"", &["L0001"]),
];

#[test]
fn every_entry_shape_parses_with_exactly_its_errors() {
    for (entry, expected) in SHAPES {
        let parsed = parse_lit(&format!("module m\nlimit brief {{ {entry} }}\n"));
        let codes: Vec<&str> = parsed.errors().iter().map(|e| e.code.as_str()).collect();
        assert_eq!(&codes, expected, "{entry}");
        assert!(
            parsed
                .syntax()
                .descendants()
                .any(|n| n.kind() == SyntaxKind::LimitDecl),
            "{entry}"
        );
    }
}
