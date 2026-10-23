//! File-local reports become located diagnostics (SPEC §9.2): lint codes
//! are warnings, every other sema code is an error.
use litigate_base::{Applicability, DiagnosticCode as C, FileId, LineIndex, Severity};
use litigate_sema::db::span::{Report, TextRange};

#[test]
fn lint_reports_are_warnings_and_keep_notes_and_fixes() {
    let text = "module m\nstart ?\n";
    let index = LineIndex::new(text);
    let file = FileId("m.lit".to_owned());
    let hole = TextRange::new(15, 16);
    let lint = Report::new(C::HolePresent, hole, "a hole")
        .note("fill it in")
        .note_at(hole, "here")
        .fix(hole, "s", Applicability::MaybeIncorrect)
        .to_diagnostic(&file, &index);
    assert_eq!(lint.severity, Severity::Warning);
    assert_eq!((lint.span.line, lint.span.col), (2, 7));
    let notes: Vec<_> = lint
        .notes
        .iter()
        .map(|n| (n.span.as_ref().map(|s| (s.line, s.col)), n.message.as_str()))
        .collect();
    assert_eq!(notes, [(None, "fill it in"), (Some((2, 7)), "here")]);
    assert_eq!(lint.fixes[0].replacement, "s");
    let error = Report::new(C::UnresolvedName, hole, "no such name").to_diagnostic(&file, &index);
    assert_eq!(error.severity, Severity::Error);
}
