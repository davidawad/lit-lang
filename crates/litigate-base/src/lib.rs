//! Shared source locations, diagnostics, profiles, and JSON envelopes.
mod code;
mod diagnostic;
mod envelope;
mod line_index;
pub mod timeline;

pub use code::{DiagnosticCode, UnknownCode};
pub use diagnostic::{
    stamp, Applicability, Derivation, Diagnostic, Edit, Fix, Label, Note, Severity, Span, WorldTag,
};
pub use envelope::{Envelope, LitigateRun, Profile, SCHEMA_VERSION};
pub use line_index::{FileId, LineIndex};
