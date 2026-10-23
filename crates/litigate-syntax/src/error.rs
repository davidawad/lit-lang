//! Byte-range syntax errors and their conversion to [`Diagnostic`]s.
use litigate_base::{
    Applicability, Diagnostic, DiagnosticCode, Fix, LineIndex, Note, Profile, Span,
};
use rowan::{TextRange, TextSize};

/// A suggested source edit over byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFix {
    /// Byte range replaced by the edit (empty for an insertion).
    pub range: TextRange,
    /// Replacement text.
    pub replacement: String,
    /// Whether the edit can be applied without review.
    pub applicability: Applicability,
}

/// One lexer or parser error (always an L0xxx code).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Primary byte range.
    pub range: TextRange,
    /// Concise explanation.
    pub message: String,
    /// Supporting notes without spans.
    pub notes: Vec<String>,
    /// Suggested edits.
    pub fixes: Vec<TextFix>,
}

impl SyntaxError {
    /// Creates an error with no notes or fixes.
    pub fn new(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self {
            code,
            range,
            message: message.into(),
            notes: Vec::new(),
            fixes: Vec::new(),
        }
    }

    /// Adds a note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds an edit.
    #[must_use]
    pub fn with_fix(
        mut self,
        range: TextRange,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        self.fixes.push(TextFix {
            range,
            replacement: replacement.into(),
            applicability,
        });
        self
    }

    /// Converts to a structured diagnostic for `file`, using `index` to map
    /// byte offsets to one-based lines and columns.
    #[must_use]
    pub fn to_diagnostic(&self, file: &litigate_base::FileId, index: &LineIndex) -> Diagnostic {
        let span = |range: TextRange| span_of(file, index, range);
        Diagnostic {
            severity: self.code.severity(Profile::Dev),
            notes: self
                .notes
                .iter()
                .map(|message| Note {
                    message: message.clone(),
                })
                .collect(),
            fixes: self
                .fixes
                .iter()
                .map(|fix| Fix::single(span(fix.range), fix.replacement.clone(), fix.applicability))
                .collect(),
            ..Diagnostic::error(self.code, span(self.range), self.message.clone())
        }
    }
}

/// Maps a byte range to a one-based, end-exclusive [`Span`].
#[must_use]
pub fn span_of(file: &litigate_base::FileId, index: &LineIndex, range: TextRange) -> Span {
    let (line, col) = index.line_col(usize::from(range.start()));
    let (end_line, end_col) = index.line_col(usize::from(range.end()));
    Span {
        file: file.clone(),
        line,
        col,
        end_line,
        end_col,
    }
}

/// Builds a `TextRange` from `usize` offsets, saturating at `u32::MAX`.
#[must_use]
pub fn range(start: usize, end: usize) -> TextRange {
    let size = |value: usize| TextSize::from(u32::try_from(value).unwrap_or(u32::MAX));
    TextRange::new(size(start), size(end.max(start)))
}
