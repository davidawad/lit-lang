//! Stable diagnostic structures shared by parser, checker, and CLI.
use crate::{DiagnosticCode, FileId};
use serde::{Deserialize, Serialize};

/// A half-open source range with one-based line and column coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Repository-relative source file identifier.
    pub file: FileId,
    /// Start line (one-based).
    pub line: u32,
    /// Start column (one-based, Unicode scalar count).
    pub col: u32,
    /// End line (one-based).
    pub end_line: u32,
    /// End column (one-based, exclusive).
    pub end_col: u32,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// An error prevents a successful check.
    Error,
    /// A warning describes a permitted but questionable construct.
    Warning,
    /// An informational diagnostic.
    Info,
    /// A low-priority editor hint.
    Hint,
}

/// Applicability of an automated edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    /// Safe to apply automatically.
    MachineApplicable,
    /// Plausible edit that requires human review.
    MaybeIncorrect,
    /// Edit contains placeholders that must be filled in.
    HasPlaceholders,
    /// No applicability guarantee is made.
    Unspecified,
}

/// A source edit suggested by a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    /// Range replaced by this edit.
    pub span: Span,
    /// Replacement source text.
    pub replacement: String,
    /// Whether the edit can be applied without review.
    pub applicability: Applicability,
}

/// Additional explanation attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Optional source location for the note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// Human-readable explanation.
    pub message: String,
}

/// One structured compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Closed, stable diagnostic identifier.
    pub code: DiagnosticCode,
    /// Severity of the finding.
    pub severity: Severity,
    /// Primary source range.
    pub span: Span,
    /// Concise explanation.
    pub message: String,
    /// Supporting notes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
    /// Suggested edits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    /// Constructs an error diagnostic with no notes or edits.
    pub fn error(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            span,
            message: message.into(),
            notes: Vec::new(),
            fixes: Vec::new(),
        }
    }
}
