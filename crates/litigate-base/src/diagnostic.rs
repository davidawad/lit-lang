//! Stable diagnostic structures shared by parser, checker, and CLI
//! (diagnostic schema v2, SPEC §9.2).
use crate::{DiagnosticCode, FileId, Profile};
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

/// One text replacement of a [`Fix`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Edit {
    /// Range replaced by this edit, in any file.
    pub span: Span,
    /// Replacement source text.
    pub replacement: String,
}

/// A suggested change: one or more edits applied together or not at all
/// (a rename touches every reference, possibly across files).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Fix {
    /// What the fix does, when the edits alone do not say.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The edits, non-overlapping; at least one.
    pub edits: Vec<Edit>,
    /// Whether the edits can be applied without review.
    pub applicability: Applicability,
}

impl Fix {
    /// A fix of one edit replacing `span` with `replacement`.
    pub fn single(
        span: Span,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        Self {
            message: None,
            edits: vec![Edit {
                span,
                replacement: replacement.into(),
            }],
            applicability,
        }
    }
}

/// A secondary source location with a label saying what it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Label {
    /// The location, in any file.
    pub span: Span,
    /// What is at `span` (for example "the rule it conflicts with").
    pub message: String,
}

/// Additional unlocated explanation attached to a diagnostic; a located
/// one is a [`Label`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Note {
    /// Human-readable explanation.
    pub message: String,
}

/// The world a diagnostic was found in: the layer set and the query
/// coordinates (valid time `as_of`, knowledge time `known_at`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorldTag {
    /// The world's name: `actual` for the packages as loaded.
    pub name: String,
    /// The valid-time coordinate, when the command fixed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    /// The knowledge-time coordinate, when the command fixed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_at: Option<String>,
}

impl WorldTag {
    /// The actual world at the given coordinates.
    #[must_use]
    pub fn actual(as_of: Option<&str>, known_at: Option<&str>) -> Self {
        Self {
            name: "actual".into(),
            as_of: as_of.map(str::to_owned),
            known_at: known_at.map(str::to_owned),
        }
    }
}

/// One step of the proof that an L3xxx error holds: the rule of the
/// resolution semantics applied (SPEC §9.2 names them), the fact it
/// concludes, and the derivations of its premises.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Derivation {
    /// The inference rule, e.g. `defeats` or `winner`.
    pub rule: String,
    /// The fact concluded, e.g. `winner(frcp::answer-due, v2)`.
    pub fact: String,
    /// Where the fact comes from in source, when it has one location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// Derivations of the premises, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub premises: Vec<Derivation>,
}

impl Derivation {
    /// A step concluding `fact` by `rule`, with no premises yet.
    pub fn new(rule: impl Into<String>, fact: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            fact: fact.into(),
            span: None,
            premises: Vec::new(),
        }
    }

    /// Locates the fact at `span`.
    #[must_use]
    pub fn at(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }

    /// Adds a premise.
    #[must_use]
    pub fn premise(mut self, premise: Self) -> Self {
        self.premises.push(premise);
        self
    }
}

/// One structured compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    /// Stable diagnostic identifier; a well-formed code this compiler does
    /// not know still decodes (see [`DiagnosticCode::Unknown`]).
    pub code: DiagnosticCode,
    /// Severity of the finding.
    pub severity: Severity,
    /// Primary source range.
    pub span: Span,
    /// Concise explanation.
    pub message: String,
    /// Labelled secondary locations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<Label>,
    /// Supporting notes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
    /// Suggested changes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    /// For an L3xxx error, the derivation of the defect from the claims.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation: Option<Derivation>,
    /// Stable identity across runs and unrelated edits (set in envelopes):
    /// a hash of the code, file, and message, and the occurrence among
    /// diagnostics equal in those, never of line numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    /// The checking profile that graded the severity (set in envelopes of
    /// commands that take `--profile`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    /// The world the diagnostic was found in (set in envelopes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<WorldTag>,
}

impl Diagnostic {
    /// Constructs an error diagnostic with no labels, notes, or edits.
    pub fn error(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            span,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            fixes: Vec::new(),
            derivation: None,
            fingerprint: None,
            profile: None,
            world: None,
        }
    }

    /// Adds a labelled secondary span.
    #[must_use]
    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    /// Adds an unlocated note.
    #[must_use]
    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(Note {
            message: message.into(),
        });
        self
    }

    /// Adds a located note: a label at `span`, else a note.
    #[must_use]
    pub fn note_at(self, span: Option<Span>, message: impl Into<String>) -> Self {
        match span {
            Some(span) => self.label(span, message),
            None => self.note(message),
        }
    }

    /// Adds a one-edit fix.
    #[must_use]
    pub fn fix(
        mut self,
        span: Span,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        self.fixes
            .push(Fix::single(span, replacement, applicability));
        self
    }

    /// Attaches the derivation of the defect.
    #[must_use]
    pub fn derived(mut self, derivation: Derivation) -> Self {
        self.derivation = Some(derivation);
        self
    }

    /// Gives an L3xxx error without a derivation the one-step derivation
    /// of its code's rule: the message is the fact concluded, and each
    /// label a `premise` located at its span. Checks that know the claims
    /// attach a full tree instead, which this keeps.
    pub fn ensure_derivation(&mut self) {
        let Some(rule) = self.code.derivation_rule() else {
            return;
        };
        if self.severity != Severity::Error || self.derivation.is_some() {
            return;
        }
        let mut derivation =
            Derivation::new(rule, self.message.clone()).at(Some(self.span.clone()));
        for label in &self.labels {
            derivation = derivation.premise(
                Derivation::new("premise", label.message.clone()).at(Some(label.span.clone())),
            );
        }
        self.derivation = Some(derivation);
    }

    /// The fingerprint of this diagnostic as the `occurrence`th (from 0)
    /// among diagnostics with its code, file, and message: 16 hex digits of
    /// FNV-1a-64 over those fields, so it survives edits that move lines.
    #[must_use]
    pub fn fingerprint_of(&self, occurrence: usize) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let occurrence = occurrence.to_string();
        for part in [
            self.code.as_str(),
            &self.span.file.0,
            &self.message,
            &occurrence,
        ] {
            for byte in part.bytes().chain([0]) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
        }
        format!("{hash:016x}")
    }
}

/// Stamps envelope-level fields on sorted `diagnostics`: each one's
/// fingerprint, the profile and world they were found under, and a
/// derivation on every L3xxx error ([`Diagnostic::ensure_derivation`]).
pub fn stamp(diagnostics: &mut [Diagnostic], profile: Option<Profile>, world: &WorldTag) {
    let mut seen: std::collections::BTreeMap<(String, String, String), usize> =
        std::collections::BTreeMap::new();
    for d in diagnostics {
        let key = (
            d.code.as_str().to_owned(),
            d.span.file.0.clone(),
            d.message.clone(),
        );
        let n = seen.entry(key).or_default();
        d.fingerprint = Some(d.fingerprint_of(*n));
        *n += 1;
        d.profile = profile;
        d.world = Some(world.clone());
        d.ensure_derivation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(file: &str, line: u32) -> Span {
        Span {
            file: file.into(),
            line,
            col: 2,
            end_line: line,
            end_col: 3,
        }
    }

    #[test]
    fn error_diagnostic_defaults_and_optional_fields_round_trip() {
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedToken,
            span("src/main.lit", 1),
            "bad token",
        );
        assert_eq!(diagnostic.severity, Severity::Error);
        let value = serde_json::to_value(&diagnostic).unwrap();
        for absent in [
            "labels",
            "notes",
            "fixes",
            "derivation",
            "fingerprint",
            "profile",
            "world",
        ] {
            assert!(value.get(absent).is_none(), "{absent}");
        }
        diagnostic = diagnostic
            .note("expected identifier")
            .note_at(None, "unlocated")
            .note_at(Some(span("src/other.lit", 4)), "declared here")
            .label(span("src/main.lit", 9), "used here")
            .fix(
                span("src/main.lit", 1),
                "name",
                Applicability::MachineApplicable,
            )
            .derived(
                Derivation::new("conflict", "conflict(d, a, b)")
                    .at(Some(span("src/main.lit", 1)))
                    .premise(Derivation::new("winner", "winner(d, a)")),
            );
        diagnostic.fixes.push(Fix {
            message: Some("rename everywhere".into()),
            edits: vec![
                Edit {
                    span: span("src/main.lit", 1),
                    replacement: "b".into(),
                },
                Edit {
                    span: span("src/other.lit", 4),
                    replacement: "b".into(),
                },
            ],
            applicability: Applicability::MaybeIncorrect,
        });
        assert_eq!(diagnostic.notes.len(), 2);
        assert_eq!(diagnostic.labels.len(), 2);
        assert_eq!(diagnostic.labels[0].span.file.0, "src/other.lit");
        let decoded: Diagnostic =
            serde_json::from_value(serde_json::to_value(&diagnostic).unwrap()).unwrap();
        assert_eq!(decoded, diagnostic);
        assert_eq!(decoded.fixes[1].edits.len(), 2);
        let derivation = decoded.derivation.unwrap();
        assert_eq!(derivation.premises[0].rule, "winner");
        assert!(derivation.premises[0].span.is_none());
    }

    #[test]
    fn stamping_fingerprints_by_code_file_message_and_occurrence() {
        let d = |line| Diagnostic::error(DiagnosticCode::TypeMismatch, span("a.lit", line), "m");
        let mut list = vec![d(1), d(7), d(3)];
        list[2].message = "other".into();
        let world = WorldTag::actual(Some("2024-01-01"), None);
        stamp(&mut list, Some(Profile::Strict), &world);
        let prints: Vec<String> = list
            .iter()
            .map(|d| d.fingerprint.clone().unwrap())
            .collect();
        assert_eq!(prints[0], d(40).fingerprint_of(0));
        assert_eq!(prints[1], d(2).fingerprint_of(1));
        assert_ne!(prints[0], prints[1]);
        assert_ne!(prints[0], prints[2]);
        assert!(prints.iter().all(|p| p.len() == 16));
        assert_eq!(list[0].profile, Some(Profile::Strict));
        assert_eq!(list[0].world.as_ref(), Some(&world));
        let value = serde_json::to_value(&list[0]).unwrap();
        assert_eq!(
            value["world"],
            serde_json::json!({"name": "actual", "as_of": "2024-01-01"})
        );
        assert_eq!(value["profile"], "strict");
    }

    #[test]
    fn every_l3_error_gets_a_derivation_and_keeps_its_own() {
        let mut gap = Diagnostic::error(DiagnosticCode::EffectiveGap, span("a.lit", 3), "a gap")
            .label(span("a.lit", 4), "the next version starts here");
        gap.ensure_derivation();
        let derivation = gap.derivation.clone().unwrap();
        assert_eq!(
            (derivation.rule.as_str(), derivation.fact.as_str()),
            ("gap", "a gap")
        );
        assert_eq!(derivation.span, Some(span("a.lit", 3)));
        assert_eq!(derivation.premises[0].rule, "premise");
        assert_eq!(derivation.premises[0].span, Some(span("a.lit", 4)));
        let own = Derivation::new("cycle", "cycle(a, b)");
        let mut kept = Diagnostic::error(DiagnosticCode::PrecedenceCycle, span("a.lit", 1), "m")
            .derived(own.clone());
        let mut warning = Diagnostic::error(DiagnosticCode::InvalidCounting, span("a.lit", 1), "m");
        warning.severity = Severity::Warning;
        let mut other = Diagnostic::error(DiagnosticCode::TypeMismatch, span("a.lit", 1), "m");
        for d in [&mut kept, &mut warning, &mut other] {
            d.ensure_derivation();
        }
        assert_eq!(kept.derivation, Some(own));
        assert!(warning.derivation.is_none());
        assert!(other.derivation.is_none());
        let mut list = vec![Diagnostic::error(
            DiagnosticCode::NoVersionInForce,
            span("a.lit", 1),
            "m",
        )];
        stamp(&mut list, None, &WorldTag::actual(None, None));
        assert_eq!(
            list[0].derivation.as_ref().map(|d| d.rule.as_str()),
            Some("no-winner")
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
        assert!(decoded.labels.is_empty());
        assert!(decoded.fixes.is_empty());
        assert!(decoded.derivation.is_none());
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
    fn a_newer_compilers_code_decodes_and_re_encodes() {
        let mut newer = wire();
        newer["code"] = serde_json::json!("L3999");
        let decoded: Diagnostic = serde_json::from_value(newer.clone()).unwrap();
        assert!(!decoded.code.is_known());
        assert_eq!(serde_json::to_value(&decoded).unwrap(), newer);
    }

    #[test]
    fn malformed_wire_forms_are_rejected_with_the_field_named() {
        let mut bad_code = wire();
        bad_code["code"] = serde_json::json!(101);
        assert!(error_of(bad_code).contains("expected a string"));
        let mut malformed = wire();
        malformed["code"] = serde_json::json!("X3999");
        assert!(error_of(malformed).contains("malformed diagnostic code X3999"));
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
            "edits": [{"span": wire()["span"], "replacement": "x"}],
            "applicability": "always",
        }]);
        assert!(error_of(fix).contains("unknown variant `always`"));
        let mut v1_fix = wire();
        v1_fix["fixes"] = serde_json::json!([{
            "span": wire()["span"],
            "replacement": "x",
            "applicability": "unspecified",
        }]);
        assert!(error_of(v1_fix).contains("missing field `edits`"));
        let mut note = wire();
        note["notes"] = serde_json::json!([{}]);
        assert!(error_of(note).contains("missing field `message`"));
        let mut label = wire();
        label["labels"] = serde_json::json!([{"message": "m"}]);
        assert!(error_of(label).contains("missing field `span`"));
    }
}
