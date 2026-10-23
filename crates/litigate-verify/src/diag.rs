//! Diagnostic construction for verification findings.
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Note, Profile, Span};
use std::cmp::Ordering;

/// SPEC §9.1 order: `(file, line, col, code)`.
pub(crate) fn order(a: &Diagnostic, b: &Diagnostic) -> Ordering {
    (&a.span.file.0, a.span.line, a.span.col, a.code)
        .cmp(&(&b.span.file.0, b.span.line, b.span.col, b.code))
        .then_with(|| a.message.cmp(&b.message))
}

/// A diagnostic for `code` at `span`, with the code's `profile` severity.
pub(crate) fn emit(
    code: DiagnosticCode,
    profile: Profile,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        severity: code.severity(profile),
        ..Diagnostic::error(code, span, message)
    }
}

/// Adds a note, optionally located.
pub(crate) fn note(
    mut d: Diagnostic,
    span: Option<&Span>,
    message: impl Into<String>,
) -> Diagnostic {
    d.notes.push(Note {
        span: span.cloned(),
        message: message.into(),
    });
    d
}

/// Where a finding without its own span is reported: line 1 of `file`.
pub(crate) fn start_of(file: &str) -> Span {
    Span {
        file: FileId(file.to_owned()),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}

/// The empty span at the end of `span`.
pub(crate) fn end_of(span: &Span) -> Span {
    Span {
        line: span.end_line,
        col: span.end_col,
        ..span.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::DiagnosticCode as C;

    fn at(file: &str, line: u32, code: C, message: &str) -> Diagnostic {
        let span = Span {
            line,
            end_line: line + 1,
            end_col: 4,
            ..start_of(file)
        };
        Diagnostic::error(code, span, message)
    }

    #[test]
    fn diagnostics_order_by_file_line_column_code_then_message() {
        let mut all = [
            at("b.lit", 1, C::NoExit, "z"),
            at("a.lit", 2, C::NoExit, "z"),
            at("a.lit", 1, C::UnreachableNode, "z"),
            at("a.lit", 1, C::NoExit, "y"),
            at("a.lit", 1, C::NoExit, "x"),
        ];
        all.sort_by(order);
        let keys: Vec<(String, u32, &str, String)> = all
            .iter()
            .map(|d| {
                (
                    d.span.file.0.clone(),
                    d.span.line,
                    d.code.as_str(),
                    d.message.clone(),
                )
            })
            .collect();
        let want = [
            ("a.lit", 1, "L5003", "z"),
            ("a.lit", 1, "L5004", "x"),
            ("a.lit", 1, "L5004", "y"),
            ("a.lit", 2, "L5004", "z"),
            ("b.lit", 1, "L5004", "z"),
        ]
        .map(|(f, l, c, m)| (f.to_owned(), l, c, m.to_owned()));
        assert_eq!(keys, want);
    }

    #[test]
    fn findings_take_the_profile_severity_and_located_notes() {
        let span = start_of("Lit.toml");
        let d = emit(C::RuleWithoutTest, Profile::Dev, span.clone(), "untested");
        assert_eq!(d.severity, litigate_base::Severity::Warning);
        let d = note(d, Some(&span), "here");
        assert_eq!(d.notes.first().and_then(|n| n.span.clone()), Some(span));
        let end = end_of(&at("a.lit", 3, C::NoExit, "z").span);
        assert_eq!((end.line, end.col, end.end_line, end.end_col), (4, 4, 4, 4));
    }
}
