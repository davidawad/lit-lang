use super::format;
use crate::parse::FileKind;

fn fmt(text: &str) -> String {
    format(text, FileKind::Lit).unwrap_or_else(|error| panic!("{error}: {text}"))
}

#[test]
fn refuses_files_with_syntax_errors() {
    let error = format("module x\nstate", FileKind::Lit).err();
    assert!(error.is_some_and(|e| !e.errors.is_empty()));
}

#[test]
fn sorts_imports_and_separates_items() {
    let out = fmt("module m\nimport b\nimport a\nstart s\ngroup g: \"G\"\ngroup h: \"H\"\n");
    assert_eq!(
        out,
        "module m\n\nimport a\nimport b\n\nstart s\n\ngroup g: \"G\"\ngroup h: \"H\"\n"
    );
}

#[test]
fn cite_before_body_breaks_the_body() {
    let out = fmt("module m\nrule duty r cite FRCP 1 { start s }\n");
    assert_eq!(
        out,
        "module m\n\nrule duty r cite FRCP 1\n{\n  start s\n}\n"
    );
}

#[test]
fn long_bodies_break_and_drop_commas() {
    let long = "x".repeat(90);
    let out = fmt(&format!(
        "module m\nstate s: \"{long}\" {{ group g, note \"n\" }}\n"
    ));
    assert!(out.contains("{\n  group g\n  note \"n\"\n}"), "{out}");
}

#[test]
fn required_commas_survive_breaking() {
    let out = fmt("module m\nenum E { A, B // b\n}\n");
    assert_eq!(out, "module m\n\nenum E {\n  A,\n  B, // b\n}\n");
}

#[test]
fn comments_are_preserved() {
    let src = "// head\nmodule m\n\n/// Doc.\nstate s: \"S\" { // first\n  a \"A\" -> t /* inline */\n}\n";
    let out = fmt(src);
    for needle in ["// head", "/// Doc.", "// first", "/* inline */"] {
        assert!(out.contains(needle), "{needle} lost: {out}");
    }
    assert_eq!(fmt(&out), out);
}

#[test]
fn case_blocks_follow_case_layout() {
    let src = "case c uses p ^1.0.0 as_of 2025-01-01 summary \"s\" parties { a: X Y } \
               facts { n: e }";
    let out = format(src, FileKind::Case).unwrap_or_default();
    assert_eq!(
        out,
        "case c\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n\nparties {\n  a: X Y\n}\n\nfacts { n: e }\n"
    );
}

#[test]
fn the_edition_marker_stays_first() {
    assert_eq!(
        fmt("edition   2026\nmodule m\n"),
        "edition 2026\n\nmodule m\n"
    );
    let case =
        "edition 2026 case c uses p ^1.0.0 as_of 2025-01-01 summary \"s\" parties { a: X Y }";
    assert_eq!(
        format(case, FileKind::Case).as_deref(),
        Ok("edition 2026\ncase c\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n\nparties {\n  a: X Y\n}\n")
    );
}
