use litigate_base::Profile;
use serde::{Deserialize, Serialize};

/// Static obligation reported by `litigate check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Obligation {
    /// Types, units, and pattern coverage.
    WellTyped,
    /// Citations resolve to a vendored L0 span.
    CiteResolves,
    /// Version intervals are disjoint.
    EffectiveDisjoint,
    /// Versions cover their declared lifetime.
    EffectiveCovered,
    /// Precedence relations are acyclic.
    PrecedenceAcyclic,
    /// Overrides have explicit delegation.
    DelegationRespected,
    /// Exceptions are disjoint or ordered.
    ExceptionsOrdered,
    /// Chance distributions are exact and exhaustive.
    DistExact,
    /// Every nonterminal has an outgoing edge.
    GraphExits,
    /// Every node is reachable from start or marked `entry`.
    GraphReachable,
    /// Mandatory deadline windows are jointly satisfiable.
    DeadlineSatisfiable,
    /// Case events replay legally.
    CaseReplay,
    /// Inline tests pass.
    TestsPass,
    /// Explicit holes are inventoried.
    Holes,
    /// Every deadline/counting declaration has an inline test.
    RuleWithoutTest,
}

/// Result state of an obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObligationStatus {
    /// Obligation was discharged.
    Passed,
    /// The selected profile permits the finding with a warning.
    Warning,
    /// Obligation failed under the selected profile.
    Failed,
}

/// Evidence for one named verification obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Obligation checked.
    pub name: Obligation,
    /// Outcome under the selected profile.
    pub status: ObligationStatus,
    /// Number of findings (violations, holes, or warnings).
    pub count: usize,
    /// Stable, bounded examples.
    pub examples: Vec<String>,
}

/// Results embedded in every verified litigatable artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    /// All checks, including checks discharged by sema.
    pub obligations: Vec<Finding>,
    /// Profile used to determine warning/error acceptance.
    pub profile: Profile,
}

impl VerificationReport {
    pub(crate) fn new(profile: Profile) -> Self {
        Self {
            obligations: Vec::new(),
            profile,
        }
    }

    pub(crate) fn push(
        &mut self,
        name: Obligation,
        status: ObligationStatus,
        count: usize,
        examples: Vec<String>,
    ) {
        self.obligations.push(Finding {
            name,
            status,
            count,
            examples: examples.into_iter().take(8).collect(),
        });
    }

    /// Whether every obligation is accepted by this report's profile.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.obligations
            .iter()
            .all(|f| f.status != ObligationStatus::Failed)
    }
}
