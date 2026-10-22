//! `litigate explain CODE`: the long form of a diagnostic code.
use crate::out::{Failure, Outcome, Report};
use litigate_base::{DiagnosticCode, Profile, Severity};
use serde_json::json;

fn word(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

/// The phase a code's range belongs to (SPEC §9.1).
#[must_use]
pub fn phase(code: DiagnosticCode) -> &'static str {
    match &code.as_str()[..2] {
        "L0" => "syntax",
        "L1" => "names, modules, packages",
        "L2" => "types",
        "L3" => "temporal, authority, precedence",
        "L4" => "case",
        "L5" => "verification",
        "L6" => "lowering and build",
        _ => "lint",
    }
}

/// `{code, explanation, severity:{dev, strict}, phase}`.
#[must_use]
pub fn entry(code: DiagnosticCode) -> serde_json::Value {
    json!({
        "code": code.as_str(),
        "explanation": code.explain(),
        "phase": phase(code),
        "severity": {
            "dev": word(code.severity(Profile::Dev)),
            "strict": word(code.severity(Profile::Strict)),
        },
    })
}

/// `litigate explain`.
pub fn explain(code: &str) -> Outcome {
    let parsed = DiagnosticCode::parse(&code.to_ascii_uppercase()).ok_or_else(|| {
        Failure::new(format!(
            "`{code}` is not a diagnostic code; `litigate describe` lists them"
        ))
    })?;
    Ok(Report::of(entry(parsed)))
}
