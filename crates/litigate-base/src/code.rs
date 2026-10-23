//! Closed diagnostic-code registry and short explanations (SPEC §9.1).
//!
//! Decoding is tolerant: a well-formed code (`L` and four digits) this
//! compiler does not know decodes to [`DiagnosticCode::Unknown`] and
//! re-encodes unchanged, so an older reader can carry a newer compiler's
//! output (SPEC §9.2).
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
            /// A well-formed code this compiler does not know: emitted by a
            /// newer compiler, or retired. Never emitted by this one.
            Unknown(UnknownCode),
        }

        impl DiagnosticCode {
            /// Complete code registry in stable (ascending code) order.
            pub const ALL: [Self; $count] = [$(Self::$variant,)+];

            /// Returns the stable five-character code.
            #[must_use]
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $code,)+
                    Self::Unknown(code) => code.as_str(),
                }
            }

            /// Returns the stable short explanation used by `litigate explain`.
            #[must_use]
            pub const fn explain(self) -> &'static str {
                match self {
                    $(Self::$variant => $explain,)+
                    Self::Unknown(_) => UNKNOWN_EXPLANATION,
                }
            }

            /// Parses a code this compiler knows (never [`Self::Unknown`]).
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

registry! { 63;
    UnexpectedToken = "L0001" => "The token is not valid at this position in the grammar",
    UnterminatedLiteral = "L0002" => "A string, block comment, or other delimited literal was not closed",
    InvalidToken = "L0003" => "The source contains a character sequence that is not a token",
    InvalidDate = "L0004" => "A date literal is not a valid proleptic Gregorian YYYY-MM-DD date",
    InvalidNumber = "L0005" => "A numeric literal has misplaced separators, an unknown unit suffix, or overflows",
    DanglingDocComment = "L0006" => "A documentation comment is not followed by a declaration",
    ReservedWord = "L0007" => "A word reserved for a future language construct is used as an identifier",
    UnknownEdition = "L0008" => "The edition marker names an edition this compiler does not know",
    ReservedRefinement = "L0009" => "Refinement syntax (`{x: T | p}` or a `where` clause) is reserved for the SMT tier and not yet accepted",
    ReservedGuardOp = "L0010" => "A typed guard operation in a patch (`when and p`, `when or p`, `supersedes when p`) is reserved and not yet accepted",
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
    InvalidProbability = "L2008" => "An edge probability is outside [0,1], on a self edge, on an edge that is not a draw's, or missing at a chance node",
    EffectiveOverlap = "L3001" => "Two versions of the same declaration overlap in in-force time",
    EffectiveGap = "L3002" => "A declaration version gap is not explicitly declared",
    UnresolvedPrecedence = "L3003" => "Conflicting rules remain unordered after relations and canons are applied",
    MissingDelegation = "L3004" => "A displacing relation (override, notwithstanding, supersedes, revives) requires an explicit delegation from the higher-authority rule",
    ExceptionConflict = "L3005" => "Exceptions overlap without an explicit priority or disjointness proof",
    PrecedenceCycle = "L3006" => "Authority or precedence relations form a cycle",
    NoVersionInForce = "L3007" => "No version of the declaration is in force at the selected date",
    InvalidTransition = "L3008" => "A transition's arms are not total or select an unknown version",
    InvalidCounting = "L3009" => "A counting rule or calendar is inconsistent or references unknown data",
    InvalidCaseEvent = "L4001" => "The event is out of order or is not a legal transition from the replayed posture",
    DeadlineMissed = "L4002" => "The event missed the due date of the obligation it discharges",
    InvalidCaseFact = "L4003" => "The fact is not a fact-tagged node or names an edge that is not its own",
    MissingStake = "L4004" => "A reachable terminal needs a value or an explicit unvalued marker",
    InvalidParty = "L4005" => "A party, party role, or perspective is undeclared or ambiguous",
    InvalidBelief = "L4006" => "An observation is not at a chance node or names an edge that is not its own",
    InvalidAnalysis = "L4007" => "An analysis field is unknown, has a dedicated block, or has the wrong shape",
    StartConflict = "L4008" => "An explicit start disagrees with the posture derived from the events",
    ObligationBreached = "L4009" => "A duty's obligation was neither discharged nor transformed by its due date: at as_of it is breached",
    TimelinessUnknown = "L4010" => "An act discharged an obligation whose designated due date is unknown: it is judged neither timely nor late",
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

/// What `explain` says of an [`DiagnosticCode::Unknown`] code.
const UNKNOWN_EXPLANATION: &str =
    "A code this compiler does not know: emitted by a newer compiler, or retired";

/// A well-formed diagnostic code outside this compiler's registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnknownCode(u16);

/// `L0000L0001…L9999`: the text of every well-formed code, so an unknown
/// one still has a `&'static str`.
static CODE_TEXT: [u8; 50_000] = {
    const DIGITS: &[u8; 10] = b"0123456789";
    let mut text = [0_u8; 50_000];
    let mut n = 0;
    while n < 10_000 {
        let at = n * 5;
        text[at] = b'L';
        text[at + 1] = DIGITS[n / 1000];
        text[at + 2] = DIGITS[n / 100 % 10];
        text[at + 3] = DIGITS[n / 10 % 10];
        text[at + 4] = DIGITS[n % 10];
        n += 1;
    }
    text
};

impl UnknownCode {
    /// The code's five-character text.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        let at = usize::from(self.0) * 5;
        std::str::from_utf8(&CODE_TEXT[at..at + 5]).unwrap_or("L????")
    }
}

impl DiagnosticCode {
    /// Decodes `value`: a known code, else [`Self::Unknown`] when it is
    /// well-formed (`L` and four ASCII digits), else `None`.
    #[must_use]
    pub fn decode(value: &str) -> Option<Self> {
        Self::parse(value).or_else(|| {
            let digits = value
                .strip_prefix('L')
                .filter(|d| d.len() == 4 && d.bytes().all(|b| b.is_ascii_digit()))?;
            digits.parse().ok().map(|n| Self::Unknown(UnknownCode(n)))
        })
    }

    /// Whether this compiler knows the code.
    #[must_use]
    pub const fn is_known(self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    /// Severity of this code under `profile` (SPEC §9.1 severity table).
    /// Every non-lint code is an error in both profiles.
    #[must_use]
    pub const fn severity(self, profile: Profile) -> Severity {
        match (self, profile) {
            (Self::AnonymousEdge, _) => Severity::Info,
            (Self::UnusedDeclaration | Self::ObligationBreached | Self::TimelinessUnknown, _)
            | (
                Self::RuleWithoutTest | Self::HolePresent | Self::NonCanonicalFormat,
                Profile::Dev,
            ) => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// The rule of the resolution semantics an error with this code
    /// concludes (SPEC §9.2): `Some` for every known L3xxx code.
    #[must_use]
    pub const fn derivation_rule(self) -> Option<&'static str> {
        Some(match self {
            Self::EffectiveOverlap => "overlap",
            Self::EffectiveGap => "gap",
            Self::UnresolvedPrecedence | Self::ExceptionConflict => "conflict",
            Self::MissingDelegation => "unlicensed",
            Self::PrecedenceCycle => "cycle",
            Self::NoVersionInForce => "no-winner",
            Self::InvalidTransition => "transition",
            Self::InvalidCounting => "counting",
            _ => return None,
        })
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

/// Every known code, or any other well-formed code, so generated bindings
/// see the registry and still accept a newer compiler's codes.
impl schemars::JsonSchema for DiagnosticCode {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DiagnosticCode".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let codes: Vec<&str> = Self::ALL.iter().map(|c| c.as_str()).collect();
        schemars::json_schema!({
            "description": "Stable diagnostic code (`litigate explain <code>`); codes outside the registry are a newer compiler's.",
            "anyOf": [
                {"type": "string", "enum": codes},
                {"type": "string", "pattern": "^L[0-9]{4}$"},
            ],
        })
    }
}

impl<'de> Deserialize<'de> for DiagnosticCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::decode(&value)
            .ok_or_else(|| serde::de::Error::custom(format!("malformed diagnostic code {value}")))
    }
}

#[cfg(test)]
mod tests;
