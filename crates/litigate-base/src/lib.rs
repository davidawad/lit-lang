//! Shared source locations, diagnostics, profiles, and JSON envelopes.
mod code;
mod diagnostic;
mod envelope;
mod line_index;

pub use code::DiagnosticCode;
pub use diagnostic::{Applicability, Diagnostic, Fix, Note, Severity, Span};
pub use envelope::{Envelope, LitigateRun, Profile};
pub use line_index::{FileId, LineIndex};
