//! Formatter paths through the public API: refusal, `--check` diagnostics,
//! and layout decisions around citations, comments, and forced breaks.
use litigate_base::FileId;
use litigate_syntax::{check_format, format, parse, FileKind};

fn fmt(text: &str) -> String {
    let once = format(text, FileKind::Lit).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(format(&once, FileKind::Lit).as_deref(), Ok(once.as_str()));
    once
}

#[test]
fn format_errors_count_the_syntax_errors() {
    let text = "module x\nstart 1\nstart 2\n";
    assert_eq!(parse(text, FileKind::Lit).errors().len(), 2);
    let error = format(text, FileKind::Lit).err();
    assert_eq!(
        error.map(|e| e.to_string()).as_deref(),
        Some("cannot format a file with 2 syntax error(s)")
    );
}

#[test]
fn check_format_leaves_unparsable_and_canonical_files_alone() {
    let file = FileId::from("x.lit");
    assert_eq!(check_format("module x\nstate", FileKind::Lit, &file), None);
    assert_eq!(check_format("module x\n", FileKind::Lit, &file), None);
}

#[test]
fn check_format_points_past_a_canonical_prefix() {
    let file = FileId::from("x.lit");
    let diagnostic = check_format("module x", FileKind::Lit, &file);
    let at = diagnostic.as_ref().map(|d| (d.span.line, d.span.col));
    assert_eq!(at, Some((1, 9)));
    let fix = diagnostic.and_then(|d| d.fixes.into_iter().next());
    assert_eq!(fix.map(|f| f.replacement).as_deref(), Some("module x\n"));
}

#[test]
fn a_comma_after_citation_text_stays_on_its_line() {
    let src = "module m\n\ntest \"t\" { expect f(cite FRCP 1, 2) }\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn comments_on_one_line_lead_the_next_item_together() {
    let out = fmt("module m\n/* a */ /* b */\nstart s\n");
    assert_eq!(out, "module m\n\n/* a */\n/* b */\nstart s\n");
}

#[test]
fn comment_layout_inside_items_and_blocks() {
    // `(input, canonical)`, after `module m` and a blank line.
    let cases = [
        // A block comment between tokens stays inline.
        (
            "state s: \"S\" /* c */ { group g }",
            "state s: \"S\" /* c */ { group g }",
        ),
        // A line comment ends the line; what follows is indented.
        (
            "state s: \"S\" // c\n{ group g }",
            "state s: \"S\" // c\n  { group g }",
        ),
        (
            "state s: \"S\"\n// c\n{ group g }",
            "state s: \"S\"\n  // c\n  { group g }",
        ),
        // A blank line before a comment in a block is kept.
        (
            "state s: \"S\" {\n  group g\n\n  // c\n  note \"n\"\n}",
            "state s: \"S\" {\n  group g\n\n  // c\n  note \"n\"\n}",
        ),
        // A comment after the last entry stays before the `}`.
        (
            "state s: \"S\" {\n  group g\n  // dangling\n}",
            "state s: \"S\" {\n  group g\n  // dangling\n}",
        ),
        ("state s: \"S\" {   }", "state s: \"S\" {}"),
        // A block holding only a comment breaks around it.
        ("state s: \"S\" { // c\n}", "state s: \"S\" {\n  // c\n}"),
        // Blank lines between entries vanish when the block prints flat.
        (
            "state s: \"S\" { group g\n\n  note \"n\" }",
            "state s: \"S\" { group g, note \"n\" }",
        ),
    ];
    for (input, canonical) in cases {
        let out = fmt(&format!("module m\n{input}\n"));
        assert_eq!(out, format!("module m\n\n{canonical}\n"), "{input}");
    }
}

#[test]
fn a_comment_after_the_case_header_trails_it() {
    let text = "case c // header\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n";
    assert_eq!(format(text, FileKind::Case).as_deref(), Ok(text));
}

/// The first record fits on its line because the second one, broken by its
/// comment, starts a new line right after its `{`.
#[test]
fn a_record_before_a_broken_record_stays_flat() {
    let out = fmt("module m\ntest \"t\" { expect f({a: 1}, {b: 2 // c\n}) }\n");
    assert_eq!(
        out,
        "module m\n\ntest \"t\" {\n  expect f({ a: 1 }, {\n    b: 2 // c\n  })\n}\n"
    );
}
