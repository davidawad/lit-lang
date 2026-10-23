//! Stable diagnostic structures shared by parser, checker, and CLI.
use crate::{DiagnosticCode, FileId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A half-open source range with one-based line and column coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Fix {
    /// Range replaced by this edit.
    pub span: Span,
    /// Replacement source text.
    pub replacement: String,
    /// Whether the edit can be applied without review.
    pub applicability: Applicability,
}

/// Additional explanation attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Note {
    /// Optional source location for the note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// Human-readable explanation.
    pub message: String,
}

/// One structured compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[cfg(test)]
mod tests {
    use super::{Applicability, Diagnostic, Fix, Note, Severity, Span};
    use crate::DiagnosticCode;

    #[test]
    fn error_diagnostic_defaults_and_optional_fields_round_trip() {
        let span = Span {
            file: "src/main.lit".into(),
            line: 1,
            col: 2,
            end_line: 1,
            end_col: 3,
        };
        let mut diagnostic =
            Diagnostic::error(DiagnosticCode::UnexpectedToken, span.clone(), "bad token");
        assert_eq!(diagnostic.severity, Severity::Error);
        assert!(diagnostic.notes.is_empty());
        assert!(diagnostic.fixes.is_empty());
        let value = serde_json::to_value(&diagnostic).unwrap();
        assert!(value.get("notes").is_none());
        assert!(value.get("fixes").is_none());
        diagnostic.notes.push(Note {
            span: None,
            message: "expected identifier".into(),
        });
        diagnostic.fixes.push(Fix {
            span,
            replacement: "name".into(),
            applicability: Applicability::MachineApplicable,
        });
        let decoded: Diagnostic =
            serde_json::from_value(serde_json::to_value(diagnostic).unwrap()).unwrap();
        assert_eq!(decoded.notes.len(), 1);
        assert_eq!(
            decoded.fixes[0].applicability,
            Applicability::MachineApplicable
        );
    }

    fn error_of(value: serde_json::Value) -> String {
        serde_json::from_value::<Diagnostic>(value)
            .unwrap_err()
            .to_string()
    }

    fn wire() -> serde_json::Value {
        serde_json::json!({
            "code": DiagnosticCode::UnexpectedToken.as_str(),
            "severity": "warning",
            "span": {"file": "a.lit", "line": 1, "col": 1, "end_line": 1, "end_col": 2},
            "message": "m",
        })
    }

    #[test]
    fn wire_form_decodes_with_defaults() {
        let decoded: Diagnostic = serde_json::from_value(wire()).unwrap();
        assert_eq!(decoded.severity, Severity::Warning);
        assert_eq!(decoded.span.file.0, "a.lit");
        assert!(decoded.notes.is_empty());
        assert!(decoded.fixes.is_empty());
        for (text, severity) in [
            ("error", Severity::Error),
            ("info", Severity::Info),
            ("hint", Severity::Hint),
        ] {
            assert_eq!(
                serde_json::from_value::<Severity>(serde_json::json!(text)).unwrap(),
                severity
            );
        }
        for (text, applicability) in [
            ("maybe_incorrect", Applicability::MaybeIncorrect),
            ("has_placeholders", Applicability::HasPlaceholders),
            ("unspecified", Applicability::Unspecified),
        ] {
            assert_eq!(
                serde_json::from_value::<Applicability>(serde_json::json!(text)).unwrap(),
                applicability
            );
        }
    }

    #[test]
    fn malformed_wire_forms_are_rejected_with_the_field_named() {
        let mut bad_code = wire();
        bad_code["code"] = serde_json::json!(101);
        assert!(error_of(bad_code).contains("expected a string"));
        let mut unknown_code = wire();
        unknown_code["code"] = serde_json::json!("L3999");
        assert!(error_of(unknown_code).contains("unknown diagnostic code L3999"));
        let mut severity = wire();
        severity["severity"] = serde_json::json!("fatal");
        assert!(error_of(severity).contains("unknown variant `fatal`"));
        let mut missing = wire();
        missing.as_object_mut().unwrap().remove("span");
        assert!(error_of(missing).contains("missing field `span`"));
        let mut line = wire();
        line["span"]["line"] = serde_json::json!(-1);
        assert!(error_of(line).contains("invalid value"));
        let mut fix = wire();
        fix["fixes"] = serde_json::json!([{
            "span": wire()["span"],
            "replacement": "x",
            "applicability": "always",
        }]);
        assert!(error_of(fix).contains("unknown variant `always`"));
        let mut note = wire();
        note["notes"] = serde_json::json!([{"span": null}]);
        assert!(error_of(note).contains("missing field `message`"));
    }
}
