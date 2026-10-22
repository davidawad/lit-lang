//! Where each part of a matter was written, so replay diagnostics point at
//! the `.case` source rather than at the HIR.
//!
//! [`hir::Case`](crate::hir::Case) is a shared contract without spans, so
//! the resolver records them here beside it. A matter built directly as HIR
//! (no source) uses [`CaseSpans::default`], and every diagnostic falls back
//! to a file-level placeholder.
use litigate_base::{Applicability, Diagnostic, DiagnosticCode, FileId, Fix, Note, Span};
use std::collections::BTreeMap;

/// Spans of one event line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventSpans {
    /// The whole line.
    pub whole: Option<Span>,
    /// The date.
    pub date: Option<Span>,
    /// The event constructor call.
    pub action: Option<Span>,
    /// The party after `by`/`on`.
    pub party: Option<Span>,
}

/// Source locations of a resolved matter, parallel to its HIR.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaseSpans {
    /// The `case id` header (the fallback for everything else).
    pub header: Option<Span>,
    /// `uses` items, in order.
    pub uses: Vec<Option<Span>>,
    /// `as_of` date.
    pub as_of: Option<Span>,
    /// The whole `start` item and its node reference.
    pub start: Option<(Span, Span)>,
    /// `perspective` target.
    pub perspective: Option<Span>,
    /// Party lines and their actor references, in order.
    pub parties: Vec<(Option<Span>, Option<Span>)>,
    /// Event lines, in source order (parallel to `Case::events`).
    pub events: Vec<EventSpans>,
    /// A fix that sorts the events block by date, when it is out of order.
    pub sort_events: Option<(Span, String)>,
    /// `facts` entries by qualified node: (node reference, edge reference).
    pub facts: BTreeMap<String, (Span, Span)>,
    /// `stakes` entries by qualified terminal.
    pub stakes: BTreeMap<String, Span>,
    /// Where a missing stake is inserted in the `stakes` block, with the
    /// text that goes before and after the new entry.
    pub stakes_end: Option<(Span, String, String)>,
    /// `observe` entries, in order: (node reference, outcome references).
    pub observations: Vec<(Option<Span>, Vec<Span>)>,
}

impl CaseSpans {
    /// `span`, else the header, else a placeholder naming `id`.
    #[must_use]
    pub fn at(&self, span: Option<&Span>, id: &str) -> Span {
        span.or(self.header.as_ref())
            .cloned()
            .unwrap_or_else(|| nowhere(id))
    }

    /// The best span for event `i`'s `part` (falls back to the line).
    #[must_use]
    pub fn event(&self, i: usize, part: fn(&EventSpans) -> Option<&Span>, id: &str) -> Span {
        let line = self.events.get(i);
        self.at(
            line.and_then(part).or(line.and_then(|l| l.whole.as_ref())),
            id,
        )
    }
}

/// A placeholder span for matters without source.
#[must_use]
pub fn nowhere(id: &str) -> Span {
    Span {
        file: FileId(format!("<case {id}>")),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}

/// Builder helpers on [`Diagnostic`] for case checks.
pub(super) trait Build: Sized {
    fn note(self, message: impl Into<String>) -> Self;
    fn note_at(self, span: Option<Span>, message: impl Into<String>) -> Self;
    fn fix(self, span: Span, replacement: impl Into<String>, a: Applicability) -> Self;
}

impl Build for Diagnostic {
    fn note(self, message: impl Into<String>) -> Self {
        self.note_at(None, message)
    }

    fn note_at(mut self, span: Option<Span>, message: impl Into<String>) -> Self {
        self.notes.push(Note {
            span,
            message: message.into(),
        });
        self
    }

    fn fix(mut self, span: Span, replacement: impl Into<String>, a: Applicability) -> Self {
        self.fixes.push(Fix {
            span,
            replacement: replacement.into(),
            applicability: a,
        });
        self
    }
}

/// An error diagnostic.
pub(super) fn error(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(code, span, message)
}
