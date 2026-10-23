//! Byte ranges inside one file and file-local diagnostic reports that become
//! [`litigate_base::Diagnostic`]s once a [`LineIndex`] is at hand.
use litigate_base::{
    Applicability, Diagnostic, DiagnosticCode, FileId, Fix, LineIndex, Note, Severity, Span,
};

/// Half-open byte range `[start, end)` in one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TextRange {
    /// First byte.
    pub start: u32,
    /// One past the last byte.
    pub end: u32,
}

impl TextRange {
    /// Range from `start` to `end`.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Smallest range covering both.
    #[must_use]
    pub fn cover(self, other: Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Empty range at this range's end.
    #[must_use]
    pub const fn at_end(self) -> Self {
        Self::new(self.end, self.end)
    }
}

/// A diagnostic whose locations are byte ranges in the reporting file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Stable code.
    pub code: DiagnosticCode,
    /// Primary range.
    pub range: TextRange,
    /// Message.
    pub message: String,
    /// Notes, optionally located in the same file.
    pub notes: Vec<(Option<TextRange>, String)>,
    /// Suggested edits in the same file.
    pub fixes: Vec<(TextRange, String, Applicability)>,
    /// A consequence of an error already reported elsewhere (a name whose
    /// import failed): it still fails the module but is not shown.
    pub suppressed: bool,
}

impl Report {
    /// New error report.
    pub fn new(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self {
            code,
            range,
            message: message.into(),
            notes: Vec::new(),
            fixes: Vec::new(),
            suppressed: false,
        }
    }

    /// Adds an unlocated note.
    #[must_use]
    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push((None, message.into()));
        self
    }

    /// Adds a located note.
    #[must_use]
    pub fn note_at(mut self, range: TextRange, message: impl Into<String>) -> Self {
        self.notes.push((Some(range), message.into()));
        self
    }

    /// Adds a suggested edit.
    #[must_use]
    pub fn fix(
        mut self,
        range: TextRange,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        self.fixes.push((range, replacement.into(), applicability));
        self
    }

    /// Converts to a located diagnostic. Sema codes are all errors except
    /// the lint range, which callers grade by profile.
    #[must_use]
    pub fn to_diagnostic(&self, file: &FileId, index: &LineIndex) -> Diagnostic {
        let span = |range: TextRange| to_span(file, index, range);
        Diagnostic {
            code: self.code,
            severity: if self.code.is_lint() {
                Severity::Warning
            } else {
                Severity::Error
            },
            span: span(self.range),
            message: self.message.clone(),
            notes: self
                .notes
                .iter()
                .map(|(range, message)| Note {
                    span: range.map(span),
                    message: message.clone(),
                })
                .collect(),
            fixes: self
                .fixes
                .iter()
                .map(|(range, replacement, applicability)| Fix {
                    span: span(*range),
                    replacement: replacement.clone(),
                    applicability: *applicability,
                })
                .collect(),
        }
    }
}

/// Converts a byte range to a one-based line/column span.
#[must_use]
pub fn to_span(file: &FileId, index: &LineIndex, range: TextRange) -> Span {
    let (line, col) = index.line_col(range.start as usize);
    let (end_line, end_col) = index.line_col(range.end as usize);
    Span {
        file: file.clone(),
        line,
        col,
        end_line,
        end_col,
    }
}

/// Sorts diagnostics by `(file, line, col, code)` (SPEC §9.2) and drops
/// exact duplicates.
pub fn sort_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.sort_by(|a, b| {
        (&a.span.file, a.span.line, a.span.col, a.code, &a.message).cmp(&(
            &b.span.file,
            b.span.line,
            b.span.col,
            b.code,
            &b.message,
        ))
    });
    diagnostics.dedup();
}
