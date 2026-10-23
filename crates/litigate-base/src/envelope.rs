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

#[cfg(test)]
mod tests {
    use super::{Envelope, Profile};

    #[test]
    fn success_envelope_serializes_stable_shape_and_profiles() {
        let envelope = Envelope::success("check", 7_u8);
        assert!(envelope.ok);
        assert_eq!(envelope.op, "check");
        assert_eq!(envelope.result, Some(7));
        assert!(envelope.diagnostics.is_empty());
        assert_eq!(envelope.provenance, serde_json::json!({}));
        assert_eq!(envelope.elapsed_ms, 0.0);

        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(value["result"], 7);
        assert_eq!(
            serde_json::from_value::<Profile>(serde_json::json!("dev")).unwrap(),
            Profile::Dev
        );
        assert_eq!(
            serde_json::from_value::<Profile>(serde_json::json!("strict")).unwrap(),
            Profile::Strict
        );
        assert!(serde_json::from_value::<Profile>(serde_json::json!("unknown")).is_err());
    }

    #[test]
    fn absent_result_is_omitted() {
        let envelope: Envelope<()> = Envelope {
            ok: false,
            op: "check".into(),
            result: None,
            diagnostics: Vec::new(),
            provenance: serde_json::Value::Null,
            elapsed_ms: 1.0,
        };
        assert!(serde_json::to_value(envelope)
            .unwrap()
            .get("result")
            .is_none());
    }
}
