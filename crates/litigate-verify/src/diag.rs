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
