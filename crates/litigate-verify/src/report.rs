//! The `VerificationReport` shape (SPEC §7) and how findings are derived
//! from diagnostics: an obligation fails on any error-severity diagnostic,
//! warns on warnings only, and passes on none.
use litigate_base::{Diagnostic, Profile, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Most examples a finding carries.
pub const MAX_EXAMPLES: usize = 8;

/// A named obligation of `litigate check`, in SPEC §7 table order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Obligation {
    /// Types, units, and pattern coverage.
    WellTyped,
    /// Citations resolve to a binding and, with a vendored path, its heading.
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
    /// Every node is reachable from start or an `entry` node.
    GraphReachable,
    /// Mandatory deadline windows are jointly satisfiable.
    DeadlineSatisfiable,
    /// Inline tests pass.
    TestsPass,
    /// Case events replay legally.
    CaseReplay,
    /// Explicit holes are inventoried.
    Holes,
    /// Every deadline and counting rule is exercised by an inline test.
    RuleWithoutTest,
}

impl Obligation {
    /// Every obligation, in report order.
    pub const ALL: [Self; 15] = [
        Self::WellTyped,
        Self::CiteResolves,
        Self::EffectiveDisjoint,
        Self::EffectiveCovered,
        Self::PrecedenceAcyclic,
        Self::DelegationRespected,
        Self::ExceptionsOrdered,
        Self::DistExact,
        Self::GraphExits,
        Self::GraphReachable,
        Self::DeadlineSatisfiable,
        Self::TestsPass,
        Self::CaseReplay,
        Self::Holes,
        Self::RuleWithoutTest,
    ];
}

/// Result state of an obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObligationStatus {
    /// Discharged.
    Passed,
    /// Accepted by the profile with warnings.
    Warning,
    /// Failed under the profile.
    Failed,
    /// Not checked for this artifact; the example says why (it does not
    /// apply, or an earlier phase failed and the artifact never existed).
    Skipped,
}

/// Evidence for one named obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Obligation checked.
    pub name: Obligation,
    /// Outcome under the selected profile.
    pub status: ObligationStatus,
    /// Number of findings (violations, holes, or warnings).
    pub count: usize,
    /// At most [`MAX_EXAMPLES`] examples, `file:line:col: CODE message`.
    pub examples: Vec<String>,
}

/// Results embedded in every litigatable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Every obligation of [`Obligation::ALL`], in that order.
    pub obligations: Vec<Finding>,
    /// Profile that decided warning versus error.
    pub profile: Profile,
}

impl VerificationReport {
    /// Whether no obligation failed.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.obligations
            .iter()
            .all(|f| f.status != ObligationStatus::Failed)
    }

    /// The finding for `name`.
    #[must_use]
    pub fn finding(&self, name: Obligation) -> Option<&Finding> {
        self.obligations.iter().find(|f| f.name == name)
    }
}

/// What one check found for its obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Ran; these diagnostics are its findings (none means passed).
    Checked(Vec<Diagnostic>),
    /// Did not run, for this reason.
    Skipped(String),
}

/// Accumulates outcomes into a report. Obligations never recorded are
/// skipped as not applicable; checked outcomes of one obligation merge.
#[derive(Debug, Default)]
pub(crate) struct Builder {
    outcomes: BTreeMap<Obligation, Outcome>,
}

impl Builder {
    pub fn check(&mut self, name: Obligation, diagnostics: Vec<Diagnostic>) {
        match self
            .outcomes
            .entry(name)
            .or_insert(Outcome::Checked(Vec::new()))
        {
            Outcome::Checked(found) => found.extend(diagnostics),
            skipped @ Outcome::Skipped(_) => *skipped = Outcome::Checked(diagnostics),
        }
    }

    pub fn skip(&mut self, name: Obligation, reason: &str) {
        self.outcomes
            .entry(name)
            .or_insert_with(|| Outcome::Skipped(reason.to_owned()));
    }

    pub fn finish(self, profile: Profile) -> VerificationReport {
        let obligations = Obligation::ALL
            .iter()
            .map(|&name| finding(name, self.outcomes.get(&name)))
            .collect();
        VerificationReport {
            obligations,
            profile,
        }
    }
}

fn finding(name: Obligation, outcome: Option<&Outcome>) -> Finding {
    match outcome {
        Some(Outcome::Checked(found)) => {
            let mut sorted: Vec<&Diagnostic> = found.iter().collect();
            sorted.sort_by(|a, b| crate::diag::order(a, b));
            let status = if found.iter().any(|d| d.severity == Severity::Error) {
                ObligationStatus::Failed
            } else if found.iter().any(|d| d.severity == Severity::Warning) {
                ObligationStatus::Warning
            } else {
                ObligationStatus::Passed
            };
            Finding {
                name,
                status,
                count: found.len(),
                examples: sorted.into_iter().take(MAX_EXAMPLES).map(example).collect(),
            }
        }
        Some(Outcome::Skipped(reason)) => skipped(name, reason),
        None => skipped(name, "not applicable to this artifact"),
    }
}

fn skipped(name: Obligation, reason: &str) -> Finding {
    Finding {
        name,
        status: ObligationStatus::Skipped,
        count: 0,
        examples: vec![reason.to_owned()],
    }
}

fn example(d: &Diagnostic) -> String {
    format!(
        "{}:{}:{}: {} {}",
        d.span.file.0,
        d.span.line,
        d.span.col,
        d.code.as_str(),
        d.message
    )
}
