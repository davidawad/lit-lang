//! Closed diagnostic-code registry and short explanations (SPEC §9.1).
use crate::{Profile, Severity};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! registry {
    ($count:literal; $($variant:ident = $code:literal => $explain:literal,)+) => {
        /// Stable diagnostic codes. Numeric ranges are reserved by compiler phase.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum DiagnosticCode {
            $(
                #[doc = concat!($explain, " (", $code, ").")]
                $variant,
            )+
        }

        impl DiagnosticCode {
            /// Complete code registry in stable (ascending code) order.
            pub const ALL: [Self; $count] = [$(Self::$variant,)+];

            /// Returns the stable five-character code.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            /// Returns the stable short explanation used by `litigate explain`.
            #[must_use]
            pub const fn explain(self) -> &'static str {
                match self {
                    $(Self::$variant => $explain,)+
                }
            }

            /// Parses a stable diagnostic code string.
            #[must_use]
            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

registry! { 57;
    UnexpectedToken = "L0001" => "The token is not valid at this position in the grammar",
    UnterminatedLiteral = "L0002" => "A string, block comment, or other delimited literal was not closed",
    InvalidToken = "L0003" => "The source contains a character sequence that is not a token",
    InvalidDate = "L0004" => "A date literal is not a valid proleptic Gregorian YYYY-MM-DD date",
    InvalidNumber = "L0005" => "A numeric literal has misplaced separators, an unknown unit suffix, or overflows",
    DanglingDocComment = "L0006" => "A documentation comment is not followed by a declaration",
    DuplicateName = "L1001" => "A declaration name is duplicated in the same namespace",
    UnresolvedName = "L1002" => "A referenced declaration or imported module could not be resolved",
    ImportCycle = "L1003" => "Module imports must form an acyclic dependency graph",
    InvalidManifest = "L1004" => "Lit.toml is missing a required key or has an invalid value",
    PackageMismatch = "L1005" => "A package declaration disagrees with Lit.toml or the file's package",
    UndeclaredDependency = "L1006" => "An import or use names a package that is not a declared dependency",
    StaleLock = "L1007" => "Lit.lock is missing or does not match the resolved package graph",
    InvalidStart = "L1008" => "A package must declare exactly one start node",
    UndeclaredActor = "L1009" => "An actor is not declared in the package's roles table",
    UnboundCite = "L1010" => "A citation does not resolve through a declared source binding",
    TypeMismatch = "L2001" => "The expression type does not match the required type",
    InvalidOperation = "L2002" => "This operation is not permitted for the operand domain types",
    InvalidDistribution = "L2003" => "A distribution must cover every enum variant exactly once and sum to one",
    UnitMismatch = "L2004" => "Day units or currencies are mixed without an explicit conversion",
    NonExhaustiveMatch = "L2005" => "A match does not cover every variant, or has an unreachable arm",
    ArithmeticError = "L2006" => "Exact arithmetic overflowed, divided by zero, or left its domain",
    InvalidField = "L2007" => "A field is unknown, duplicated, or not allowed on this declaration kind",
    InvalidProbability = "L2008" => "An edge probability is outside [0,1], on a self edge, or missing at a chance node",
    EffectiveOverlap = "L3001" => "Two versions of the same declaration overlap in in-force time",
    EffectiveGap = "L3002" => "A declaration version gap is not explicitly declared",
    UnresolvedPrecedence = "L3003" => "Conflicting rules remain unordered after relations and canons are applied",
    MissingDelegation = "L3004" => "An override requires an explicit delegation from the higher-authority rule",
    ExceptionConflict = "L3005" => "Exceptions overlap without an explicit priority or disjointness proof",
    PrecedenceCycle = "L3006" => "Authority or precedence relations form a cycle",
    NoVersionInForce = "L3007" => "No version of the declaration is in force at the selected date",
    InvalidTransition = "L3008" => "A transition's arms are not total or select an unknown version",
    InvalidCounting = "L3009" => "A counting rule or calendar is inconsistent or references unknown data",
    InvalidCaseEvent = "L4001" => "The event is out of order or is not a legal transition from the replayed posture",
    DeadlineMissed = "L4002" => "The event missed its governing deadline without an excusing event",
    InvalidCaseFact = "L4003" => "The fact is not a fact-tagged node or names an edge that is not its own",
    MissingStake = "L4004" => "A reachable terminal needs a value or an explicit unvalued marker",
    InvalidParty = "L4005" => "A party, party role, or perspective is undeclared or ambiguous",
    InvalidBelief = "L4006" => "An observation is not at a chance node or names an edge that is not its own",
    InvalidAnalysis = "L4007" => "An analysis field is unknown, has a dedicated block, or has the wrong shape",
    StartConflict = "L4008" => "An explicit start disagrees with the posture derived from the events",
    VerificationFailed = "L5001" => "A required static verification obligation was not discharged",
    HoleInStrict = "L5002" => "Strict profile does not permit unresolved holes",
    UnreachableNode = "L5003" => "A node is not reachable from start and is not tagged entry",
    NoExit = "L5004" => "A non-terminal node has no outgoing edge",
    DeadlineUnsatisfiable = "L5005" => "Mandatory deadlines on a reachable state have an empty intersection",
    TestFailed = "L5006" => "An inline test assertion does not hold",
    LoweringUnsupported = "L6001" => "The source construct has no lowering in the target litgraph schema",
    SemanticDiff = "L6002" => "Generated output is not semantically equal to the expected document",
    InvalidEngineExpression = "L6003" => "An embedded litgraph expression string does not parse",
    NonFiniteNumber = "L6004" => "An exact value has no finite binary64 representation",
    OutputError = "L6005" => "A generated file could not be written atomically",
    RuleWithoutTest = "L9001" => "A deadline or counting rule should include an inline test",
    HolePresent = "L9002" => "The source explicitly leaves this value unknown",
    AnonymousEdge = "L9003" => "An anonymous edge cannot be referenced by cases or links",
    NonCanonicalFormat = "L9004" => "The file is not in litigate fmt canonical form",
    UnusedDeclaration = "L9005" => "The declaration or import is never used",
}

impl DiagnosticCode {
    /// Severity of this code under `profile` (SPEC §9.1 severity table).
    /// Every non-lint code is an error in both profiles.
    #[must_use]
    pub const fn severity(self, profile: Profile) -> Severity {
        match (self, profile) {
            (Self::AnonymousEdge, _) => Severity::Info,
            (Self::UnusedDeclaration, _)
            | (
                Self::RuleWithoutTest | Self::HolePresent | Self::NonCanonicalFormat,
                Profile::Dev,
            ) => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// True for the lint range (L9xxx).
    #[must_use]
    pub fn is_lint(self) -> bool {
        self.as_str().starts_with("L9")
    }
}

impl Serialize for DiagnosticCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// A string enum of every stable code, so generated bindings see the closed set.
impl schemars::JsonSchema for DiagnosticCode {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DiagnosticCode".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let codes: Vec<&str> = Self::ALL.iter().map(|c| c.as_str()).collect();
        schemars::json_schema!({
            "description": "Stable diagnostic code (`litigate explain <code>`).",
            "type": "string",
            "enum": codes,
        })
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
mod tests;
