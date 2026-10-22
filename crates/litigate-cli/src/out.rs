//! Envelopes, exit codes, and human rendering (SPEC §9.2-§9.3).
use crate::cli::Format;
use crate::render;
use litigate_base::{Diagnostic, Envelope, Severity};
use serde_json::Value;

/// What a command produced: an envelope's parts plus the sources its
/// diagnostics point into (for human rendering).
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Operation-specific result (dropped when an error diagnostic exists).
    pub result: Value,
    /// Every diagnostic, in any order (sorted on output).
    pub diagnostics: Vec<Diagnostic>,
    /// `{compiler_version, language_version, packages, as_of, known_at}`.
    pub provenance: Value,
    /// `(repository-relative path, text)` of files diagnostics may cite.
    pub sources: Vec<(String, String)>,
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

/// The envelope for `op`.
#[must_use]
pub fn envelope(op: &str, report: &Report, elapsed_ms: f64) -> Envelope<Value> {
    let ok = !report.has_errors();
    Envelope {
        ok,
        op: op.to_owned(),
        result: ok.then(|| report.result.clone()),
        diagnostics: sorted(report.diagnostics.clone()),
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
