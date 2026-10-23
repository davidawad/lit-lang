//! Envelopes, exit codes, and human rendering (SPEC §9.2-§9.3).
use crate::cli::Format;
use crate::render;
use litigate_base::{Diagnostic, Envelope, Profile, Severity, WorldTag, SCHEMA_VERSION};
use serde_json::Value;

/// What a command produced: an envelope's parts plus the sources its
/// diagnostics point into (for human rendering).
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Operation-specific result, kept alongside errors (`null`: none).
    pub result: Value,
    /// Every diagnostic, in any order (sorted on output).
    pub diagnostics: Vec<Diagnostic>,
    /// `{compiler_version, language_version, packages, as_of, known_at}`.
    pub provenance: Value,
    /// `(repository-relative path, text)` of files diagnostics may cite.
    pub sources: Vec<(String, String)>,
    /// The profile that graded the diagnostics, for commands taking one.
    pub profile: Option<Profile>,
    /// The command ran but its subject failed without a litigate
    /// diagnostic (a litgraph `ok: false` response): exit 2, `ok` stays
    /// true because compilation itself succeeded.
    pub failed: bool,
}

impl Report {
    /// A report with only a result.
    #[must_use]
    pub fn of(result: Value) -> Self {
        Self {
            result,
            provenance: crate::prov::empty(),
            ..Self::default()
        }
    }

    /// True when an error-severity diagnostic is present.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// A usage or I/O failure (exit 1, no envelope).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure(pub String);

impl Failure {
    /// A failure with `message`.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl From<std::io::Error> for Failure {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<litgraph::Error> for Failure {
    fn from(error: litgraph::Error) -> Self {
        Self(error.to_string())
    }
}

/// Command outcome.
pub type Outcome = Result<Report, Failure>;

/// Text to print and the process exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printed {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code: 0 ok, 2 check failure, 1 usage or I/O.
    pub code: u8,
}

/// Sorts by `(file, line, col, code)` and drops exact duplicates (SPEC §9.2).
#[must_use]
pub fn sorted(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_by(|a, b| {
        let key = |d: &Diagnostic| {
            (
                d.span.file.0.clone(),
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message.clone(),
            )
        };
        key(a).cmp(&key(b))
    });
    diagnostics.dedup();
    diagnostics
}

/// The envelope for `op` (schema v2, SPEC §9.2): the result even when an
/// error exists, and every diagnostic fingerprinted and tagged with the
/// profile and the world (`actual` at the provenance's dates).
#[must_use]
pub fn envelope(op: &str, report: &Report, elapsed_ms: f64) -> Envelope<Value> {
    let date = |key: &str| report.provenance.get(key).and_then(Value::as_str);
    let world = WorldTag::actual(date("as_of"), date("known_at"));
    let mut diagnostics = sorted(report.diagnostics.clone());
    litigate_base::stamp(&mut diagnostics, report.profile, &world);
    Envelope {
        schema_version: SCHEMA_VERSION,
        ok: !report.has_errors(),
        op: op.to_owned(),
        result: (!report.result.is_null()).then(|| report.result.clone()),
        diagnostics,
        provenance: report.provenance.clone(),
        elapsed_ms,
    }
}

/// Renders an outcome.
#[must_use]
pub fn print(op: &str, outcome: Outcome, format: Format, color: bool, elapsed_ms: f64) -> Printed {
    let report = match outcome {
        Ok(report) => report,
        Err(Failure(message)) => {
            return Printed {
                stdout: String::new(),
                stderr: format!("litigate {op}: {message}\n"),
                code: 1,
            }
        }
    };
    let envelope = envelope(op, &report, elapsed_ms);
    let code = if envelope.ok {
        u8::from(report.failed) * 2
    } else {
        2
    };
    let stdout = match format {
        Format::Json => serde_json::to_string(&envelope)
            .map(|mut s| {
                s.push('\n');
                s
            })
            .unwrap_or_default(),
        Format::Human => render::human(&envelope, &report.sources, color),
    };
    Printed {
        stdout,
        stderr: String::new(),
        code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::{DiagnosticCode, Span};
    use proptest::prelude::*;

    fn diagnostic(code: DiagnosticCode, severity: Severity, line: u32) -> Diagnostic {
        let span = Span {
            file: "f.lit".into(),
            line,
            col: 1,
            end_line: line,
            end_col: 1,
        };
        Diagnostic {
            severity,
            ..Diagnostic::error(code, span, "m")
        }
    }

    proptest! {
        /// SPEC §9.2: `ok` is false iff an error exists, `result` is kept
        /// either way, the exit code follows, and diagnostics come out
        /// sorted, fingerprinted, and tagged.
        #[test]
        fn envelope_invariants(items in proptest::collection::vec((0usize..57, 0u8..3, 1u32..50), 0..12), failed: bool) {
            let severities = [Severity::Error, Severity::Warning, Severity::Info];
            let diagnostics: Vec<Diagnostic> = items
                .iter()
                .map(|(c, s, l)| diagnostic(DiagnosticCode::ALL[*c], severities[usize::from(*s)], *l))
                .collect();
            let errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
            let report = Report { result: serde_json::json!({"x": 1}), diagnostics, failed, profile: Some(Profile::Strict), ..Report::default() };
            let printed = print("check", Ok(report), Format::Json, false, 0.0);
            let envelope: serde_json::Value = serde_json::from_str(&printed.stdout).unwrap();
            prop_assert_eq!(envelope["ok"].as_bool(), Some(!errors));
            prop_assert_eq!(&envelope["result"], &serde_json::json!({"x": 1}));
            prop_assert_eq!(envelope["schema_version"].as_u64(), Some(2));
            for d in envelope["diagnostics"].as_array().unwrap() {
                prop_assert_eq!(d["fingerprint"].as_str().map(str::len), Some(16));
                prop_assert_eq!(&d["profile"], "strict");
                prop_assert_eq!(&d["world"]["name"], "actual");
            }
            let expected = if errors || failed { 2 } else { 0 };
            prop_assert_eq!(printed.code, expected);
            let keys: Vec<(u64, String)> = envelope["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| (d["span"]["line"].as_u64().unwrap(), d["code"].as_str().unwrap().to_owned()))
                .collect();
            let mut sorted = keys.clone();
            sorted.sort();
            prop_assert_eq!(keys, sorted);
        }
    }

    #[test]
    fn a_null_result_is_absent_and_the_world_takes_the_provenance_dates() {
        let report = Report {
            result: Value::Null,
            diagnostics: vec![diagnostic(DiagnosticCode::TypeMismatch, Severity::Error, 1)],
            provenance: crate::prov::of(Vec::new(), Some("2024-01-01"), Some("2024-02-01")),
            ..Report::default()
        };
        let envelope = envelope("check", &report, 0.0);
        assert!(envelope.result.is_none() && !envelope.ok);
        let d = &envelope.diagnostics[0];
        assert_eq!(d.profile, None);
        assert_eq!(
            d.world,
            Some(WorldTag::actual(Some("2024-01-01"), Some("2024-02-01")))
        );
    }

    #[test]
    fn io_and_engine_errors_become_their_messages() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        assert_eq!(Failure::from(io), Failure::new("gone"));
        let engine = litgraph::Error::Invalid("bad pack".into());
        assert_eq!(Failure::from(engine), Failure::new("invalid: bad pack"));
    }

    #[test]
    fn failures_exit_1_on_stderr() {
        let printed = print("check", Err(Failure::new("nope")), Format::Json, false, 0.0);
        assert_eq!(printed.code, 1);
        assert!(printed.stdout.is_empty());
        assert_eq!(printed.stderr, "litigate check: nope\n");
    }
}
