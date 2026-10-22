//! Operation response envelope and checking profiles.
use crate::Diagnostic;
use serde::{Deserialize, Serialize};

/// Compiler checking profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Development checks permit explicit holes with warnings.
    Dev,
    /// Strict checks reject unresolved holes and incomplete verification.
    Strict,
}

/// JSON response shape shared by all `litigate` operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// Whether the operation completed without errors.
    pub ok: bool,
    /// Operation name.
    pub op: String,
    /// Operation-specific result, absent on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// Structured compiler diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Source and content-hash information.
    pub provenance: serde_json::Value,
    /// Wall time in milliseconds.
    pub elapsed_ms: f64,
}

impl<T> Envelope<T> {
    /// Creates a successful envelope.
    pub fn success(op: impl Into<String>, result: T) -> Self {
        Self {
            ok: true,
            op: op.into(),
            result: Some(result),
            diagnostics: Vec::new(),
            provenance: serde_json::json!({}),
            elapsed_ms: 0.0,
        }
    }
}
