//! Byte ranges inside one file and file-local diagnostic reports that become
//! [`litigate_base::Diagnostic`]s once a [`LineIndex`] is at hand.
use litigate_base::{Applicability, Diagnostic, DiagnosticCode, FileId, LineIndex, Severity, Span};

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

    /// Converts to a located diagnostic: located notes become labels.
    /// Sema codes are all errors except the lint range, which callers
    /// grade by profile.
    #[must_use]
    pub fn to_diagnostic(&self, file: &FileId, index: &LineIndex) -> Diagnostic {
        let span = |range: TextRange| to_span(file, index, range);
        let mut d = Diagnostic::error(self.code, span(self.range), self.message.clone());
        if self.code.is_lint() {
            d.severity = Severity::Warning;
        }
        for (range, message) in &self.notes {
            d = d.note_at(range.map(span), message.clone());
        }
        for (range, replacement, applicability) in &self.fixes {
            d = d.fix(span(*range), replacement.clone(), *applicability);
        }
        d
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

/// Sorts diagnostics by `(file, line, col, code)` (SPEC §9.2), drops
/// exact duplicates, and gives every L3xxx error a derivation.
pub fn sort_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    diagnostics
        .iter_mut()
        .for_each(Diagnostic::ensure_derivation);
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
