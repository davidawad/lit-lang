//! Named verification-obligation contracts.
use serde::{Deserialize, Serialize};

/// Static obligation name emitted by `litigate check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Obligation {
    /// Types, units, and pattern coverage are valid.
    WellTyped,
    /// Every citation resolves to a vendored L0 source span.
    CiteResolves,
    /// In-force versions do not overlap.
    EffectiveDisjoint,
    /// Version coverage contains no unexplained gaps.
    EffectiveCovered,
    /// Precedence relations contain no cycles.
    PrecedenceAcyclic,
    /// Each override is permitted by explicit delegation.
    DelegationRespected,
    /// Exceptions are disjoint or explicitly ordered.
    ExceptionsOrdered,
    /// Every chance distribution is exact and exhaustive.
    DistExact,
    /// Every nonterminal has an outgoing edge.
    GraphExits,
    /// Every node is reachable from start or marked entry.
    GraphReachable,
    /// No state has mutually unsatisfiable mandatory deadlines.
    DeadlineSatisfiable,
    /// Case events replay as legal transitions.
    CaseReplay,
    /// Holes are listed and conform to the selected profile.
    Holes,
}

/// Outcome of one named obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Obligation that was checked.
    pub obligation: Obligation,
    /// Whether it was discharged.
    pub passed: bool,
    /// Human-readable evidence or failure reason.
    pub detail: String,
}
