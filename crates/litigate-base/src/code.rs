//! Closed diagnostic-code registry and short explanations.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Stable diagnostic codes. Numeric ranges are reserved by compiler phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// Unexpected token or malformed syntax (L0001).
    UnexpectedToken,
    /// Unterminated string or comment (L0002).
    UnterminatedLiteral,
    /// Invalid lexical token (L0003).
    InvalidToken,
    /// Duplicate or invalid declaration name (L1001).
    DuplicateName,
    /// Unresolved name or import (L1002).
    UnresolvedName,
    /// Import cycle (L1003).
    ImportCycle,
    /// Type mismatch (L2001).
    TypeMismatch,
    /// Invalid operation for domain type (L2002).
    InvalidOperation,
    /// Distribution is not exact or exhaustive (L2003).
    InvalidDistribution,
    /// Effective intervals overlap (L3001).
    EffectiveOverlap,
    /// Effective interval coverage has an unexplained gap (L3002).
    EffectiveGap,
    /// Rule conflict has no declared or canonical winner (L3003).
    UnresolvedPrecedence,
    /// Override lacks delegation (L3004).
    MissingDelegation,
    /// Exception ordering is ambiguous (L3005).
    ExceptionConflict,
    /// Invalid, out-of-order, or unreachable case event (L4001).
    InvalidCaseEvent,
    /// Case event violates a deadline (L4002).
    DeadlineMissed,
    /// Missing or invalid matter fact (L4003).
    InvalidCaseFact,
    /// Required terminal valuation is absent (L4004).
    MissingStake,
    /// Named verification obligation failed (L5001).
    VerificationFailed,
    /// Strict profile rejects an unresolved hole (L5002).
    HoleInStrict,
    /// HIR construct cannot be represented in the target schema (L6001).
    LoweringUnsupported,
    /// Generated output differs semantically from expected output (L6002).
    SemanticDiff,
    /// Deadline or counting declaration lacks an inline test (L9001).
    RuleWithoutTest,
    /// Declaration contains an explicitly permitted unknown (L9002).
    HolePresent,
}

impl DiagnosticCode {
    /// Complete code registry in stable order.
    pub const ALL: [Self; 24] = [
        Self::UnexpectedToken,
        Self::UnterminatedLiteral,
        Self::InvalidToken,
        Self::DuplicateName,
        Self::UnresolvedName,
        Self::ImportCycle,
        Self::TypeMismatch,
        Self::InvalidOperation,
        Self::InvalidDistribution,
        Self::EffectiveOverlap,
        Self::EffectiveGap,
        Self::UnresolvedPrecedence,
        Self::MissingDelegation,
        Self::ExceptionConflict,
        Self::InvalidCaseEvent,
        Self::DeadlineMissed,
        Self::InvalidCaseFact,
        Self::MissingStake,
        Self::VerificationFailed,
        Self::HoleInStrict,
        Self::LoweringUnsupported,
        Self::SemanticDiff,
        Self::RuleWithoutTest,
        Self::HolePresent,
    ];

    /// Returns the stable five-character code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "L0412",
            Self::UnterminatedLiteral => "L0002",
            Self::InvalidToken => "L0003",
            Self::DuplicateName => "L1001",
            Self::UnresolvedName => "L1002",
            Self::ImportCycle => "L1003",
            Self::TypeMismatch => "L2001",
            Self::InvalidOperation => "L2002",
            Self::InvalidDistribution => "L2003",
            Self::EffectiveOverlap => "L3001",
            Self::EffectiveGap => "L3002",
            Self::UnresolvedPrecedence => "L3003",
            Self::MissingDelegation => "L3004",
            Self::ExceptionConflict => "L3005",
            Self::InvalidCaseEvent => "L4001",
            Self::DeadlineMissed => "L4002",
            Self::InvalidCaseFact => "L4003",
            Self::MissingStake => "L4004",
            Self::VerificationFailed => "L5001",
            Self::HoleInStrict => "L5002",
            Self::LoweringUnsupported => "L6001",
            Self::SemanticDiff => "L6002",
            Self::RuleWithoutTest => "L9001",
            Self::HolePresent => "L9002",
        }
    }

    /// Returns the stable short explanation used by `litigate explain`.
    #[must_use]
    pub const fn explain(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "The token is not valid at this position in the grammar.",
            Self::UnterminatedLiteral => {
                "A string, block comment, or other delimited literal was not closed."
            }
            Self::InvalidToken => "The source contains a character sequence that is not a token.",
            Self::DuplicateName => "A declaration name is duplicated in the same namespace.",
            Self::UnresolvedName => {
                "A referenced declaration or imported module could not be resolved."
            }
            Self::ImportCycle => "Module imports must form an acyclic dependency graph.",
            Self::TypeMismatch => "The expression type does not match the required type.",
            Self::InvalidOperation => {
                "This operation is not permitted for the operand domain types."
            }
            Self::InvalidDistribution => {
                "A distribution must cover every enum variant exactly once and sum to one."
            }
            Self::EffectiveOverlap => {
                "Two versions of the same declaration overlap in in-force time."
            }
            Self::EffectiveGap => "A declaration version gap is not explicitly declared.",
            Self::UnresolvedPrecedence => {
                "Conflicting rules remain unordered after relations and canons are applied."
            }
            Self::MissingDelegation => {
                "An override requires an explicit delegation from the higher-authority rule."
            }
            Self::ExceptionConflict => {
                "Exceptions overlap without an explicit priority or disjointness proof."
            }
            Self::InvalidCaseEvent => {
                "The event is out of order or is not a legal transition from the replayed posture."
            }
            Self::DeadlineMissed => {
                "The event missed its governing deadline without an excusing event."
            }
            Self::InvalidCaseFact => {
                "The fact is absent from the forum fact schema or has an invalid variant."
            }
            Self::MissingStake => {
                "A reachable terminal needs a value or an explicit unvalued marker."
            }
            Self::VerificationFailed => {
                "A required static verification obligation was not discharged."
            }
            Self::HoleInStrict => "Strict profile does not permit unresolved holes.",
            Self::LoweringUnsupported => {
                "The source construct has no lowering in the target litgraph schema."
            }
            Self::SemanticDiff => {
                "Generated output is not semantically equal to the expected document."
            }
            Self::RuleWithoutTest => "A deadline or counting rule should include an inline test.",
            Self::HolePresent => "The source explicitly leaves this value unknown.",
        }
    }

    /// Parses a stable diagnostic code string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "L0412" => Self::UnexpectedToken,
            "L0002" => Self::UnterminatedLiteral,
            "L0003" => Self::InvalidToken,
            "L1001" => Self::DuplicateName,
            "L1002" => Self::UnresolvedName,
            "L1003" => Self::ImportCycle,
            "L2001" => Self::TypeMismatch,
            "L2002" => Self::InvalidOperation,
            "L2003" => Self::InvalidDistribution,
            "L3001" => Self::EffectiveOverlap,
            "L3002" => Self::EffectiveGap,
            "L3003" => Self::UnresolvedPrecedence,
            "L3004" => Self::MissingDelegation,
            "L3005" => Self::ExceptionConflict,
            "L4001" => Self::InvalidCaseEvent,
            "L4002" => Self::DeadlineMissed,
            "L4003" => Self::InvalidCaseFact,
            "L4004" => Self::MissingStake,
            "L5001" => Self::VerificationFailed,
            "L5002" => Self::HoleInStrict,
            "L6001" => Self::LoweringUnsupported,
            "L6002" => Self::SemanticDiff,
            "L9001" => Self::RuleWithoutTest,
            "L9002" => Self::HolePresent,
            _ => return None,
        })
    }
}

impl Serialize for DiagnosticCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DiagnosticCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown diagnostic code {value}")))
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticCode;
    #[test]
    fn code_registry_round_trips_and_explains() {
        let code = DiagnosticCode::MissingDelegation;
        assert_eq!(DiagnosticCode::parse(code.as_str()), Some(code));
        assert!(code.explain().contains("delegation"));
        assert_eq!(DiagnosticCode::parse("L3999"), None);
    }
}
