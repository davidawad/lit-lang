//! Lowering diagnostics (SPEC §9.1 L6xxx) located at HIR spans.
use crate::num;
use litigate_base::{Diagnostic, DiagnosticCode as C, FileId, Note, Span};
use litigate_sema::hir::{Money, Rational};

/// Collects diagnostics, locating span-less HIR at a fallback span (the
/// package manifest or case file).
pub(crate) struct Sink {
    /// Where a finding without its own span is reported.
    pub fallback: Span,
    /// Findings, in discovery order.
    pub out: Vec<Diagnostic>,
}

impl Sink {
    /// An empty sink reporting span-less findings at the start of `file`.
    pub fn new(file: &str) -> Self {
        Self {
            fallback: start_of(file),
            out: Vec::new(),
        }
    }

    /// Records an error with `code` at `span` (else the fallback).
    pub fn error(&mut self, code: C, span: Option<&Span>, message: impl Into<String>) {
        let diagnostic = self.located(code, span, message);
        self.out.push(diagnostic);
    }

    /// An error with `code` at `span` (else the fallback), not yet recorded.
    fn located(&self, code: C, span: Option<&Span>, message: impl Into<String>) -> Diagnostic {
        let at = span.cloned().unwrap_or_else(|| self.fallback.clone());
        Diagnostic::error(code, at, message)
    }

    /// Records an error with a note.
    pub fn error_with(
        &mut self,
        code: C,
        span: Option<&Span>,
        message: impl Into<String>,
        note: impl Into<String>,
    ) {
        let mut diagnostic = self.located(code, span, message);
        diagnostic.notes.push(Note {
            span: None,
            message: note.into(),
        });
        self.out.push(diagnostic);
    }

    /// L6001: `what` has no lowering.
    pub fn unsupported(&mut self, span: Option<&Span>, what: impl Into<String>) {
        self.error(C::LoweringUnsupported, span, what);
    }

    /// An exact number as binary64; L6004 when it has none.
    pub fn number(&mut self, r: Rational, span: Option<&Span>) -> Option<f64> {
        let value = num::rational(r).filter(|v| v.is_finite());
        if value.is_none() {
            self.error(
                C::NonFiniteNumber,
                span,
                format!(
                    "{}/{} has no finite binary64 value",
                    r.numerator, r.denominator
                ),
            );
        }
        value
    }

    /// Money as major units; L6001 for a currency with no lowering.
    pub fn money(&mut self, m: &Money, span: Option<&Span>) -> Option<f64> {
        let value = num::money(m);
        if value.is_none() {
            self.unsupported(
                span,
                format!("currency `{}` has no lowering (only `usd`)", m.currency),
            );
        }
        value
    }

    /// The diagnostics, sorted (SPEC §9.2), or `Ok(value)` when there are
    /// none.
    pub fn finish<T>(self, value: T) -> Result<T, Vec<Diagnostic>> {
        if self.out.is_empty() {
            Ok(value)
        } else {
            Err(self.into_sorted())
        }
    }

    /// The diagnostics, sorted (SPEC §9.2).
    pub fn into_sorted(mut self) -> Vec<Diagnostic> {
        self.out.sort_by(|a, b| {
            (&a.span.file.0, a.span.line, a.span.col, a.code.as_str()).cmp(&(
                &b.span.file.0,
                b.span.line,
                b.span.col,
                b.code.as_str(),
            ))
        });
        self.out
    }
}

/// An empty span at line 1, column 1 of `file`.
pub(crate) fn start_of(file: &str) -> Span {
    Span {
        file: FileId(file.to_owned()),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}
