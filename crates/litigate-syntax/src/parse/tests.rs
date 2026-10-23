use super::{parse_case, parse_lit, FileKind};
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
