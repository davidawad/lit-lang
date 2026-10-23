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

/// Why an obligation nothing recorded is skipped.
const NOT_APPLICABLE: &str = "not applicable to this artifact";

/// Accumulates outcomes into a report. A checked obligation's findings
/// merge across checks and override any skip, whichever came first; an
/// obligation only skipped keeps its first reason; one never recorded is
/// skipped as not applicable.
#[derive(Debug, Default)]
pub(crate) struct Builder {
    checked: BTreeMap<Obligation, Vec<Diagnostic>>,
    skipped: BTreeMap<Obligation, String>,
}

impl Builder {
    pub fn check(&mut self, name: Obligation, diagnostics: Vec<Diagnostic>) {
        self.checked.entry(name).or_default().extend(diagnostics);
    }

    pub fn skip(&mut self, name: Obligation, reason: &str) {
        self.skipped
            .entry(name)
            .or_insert_with(|| reason.to_owned());
    }

    pub fn finish(mut self, profile: Profile) -> VerificationReport {
        let obligations = Obligation::ALL
            .iter()
            .map(|&name| {
                let reason = self
                    .skipped
                    .remove(&name)
                    .unwrap_or(NOT_APPLICABLE.to_owned());
                self.checked
                    .remove(&name)
                    .map_or_else(|| skipped(name, reason), |found| checked(name, &found))
            })
            .collect();
        VerificationReport {
            obligations,
            profile,
        }
    }
}

/// The finding of an obligation that ran and found `found`.
fn checked(name: Obligation, found: &[Diagnostic]) -> Finding {
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

fn skipped(name: Obligation, reason: String) -> Finding {
    Finding {
        name,
        status: ObligationStatus::Skipped,
        count: 0,
        examples: vec![reason],
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

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::DiagnosticCode as C;

    fn diagnostic(code: C, severity: Severity, line: u32) -> Diagnostic {
        let span = litigate_base::Span {
            line,
            ..crate::diag::start_of("pkgs/demo/main.lit")
        };
        Diagnostic {
            severity,
            ..Diagnostic::error(code, span, format!("finding on line {line}"))
        }
    }

    fn status(b: Builder, name: Obligation) -> Option<ObligationStatus> {
        b.finish(Profile::Dev).finding(name).map(|f| f.status)
    }

    #[test]
    fn an_unrecorded_obligation_is_not_applicable() {
        let report = Builder::default().finish(Profile::Strict);
        assert_eq!(report.obligations.len(), Obligation::ALL.len());
        let f = report.finding(Obligation::GraphExits);
        assert_eq!(f.map(|f| f.status), Some(ObligationStatus::Skipped));
        assert_eq!(
            f.map(|f| f.examples.clone()),
            Some(vec!["not applicable to this artifact".to_owned()])
        );
        assert!(report.is_accepted());
    }

    #[test]
    fn a_check_replaces_an_earlier_skip_and_a_later_skip_is_ignored() {
        let mut b = Builder::default();
        b.skip(Obligation::Holes, "first");
        b.check(Obligation::Holes, Vec::new());
        b.skip(Obligation::Holes, "second");
        assert_eq!(status(b, Obligation::Holes), Some(ObligationStatus::Passed));
        let mut b = Builder::default();
        b.check(Obligation::Holes, Vec::new());
        b.skip(Obligation::Holes, "after");
        assert_eq!(status(b, Obligation::Holes), Some(ObligationStatus::Passed));
    }

    #[test]
    fn a_skipped_obligation_keeps_its_first_reason() {
        let mut b = Builder::default();
        b.skip(Obligation::CaseReplay, "first");
        b.skip(Obligation::CaseReplay, "second");
        let report = b.finish(Profile::Dev);
        let f = report.finding(Obligation::CaseReplay);
        assert_eq!(
            f.map(|f| (f.status, f.examples.clone())),
            Some((ObligationStatus::Skipped, vec!["first".to_owned()]))
        );
    }

    #[test]
    fn checks_merge_and_the_worst_severity_decides() {
        let mut b = Builder::default();
        b.check(
            Obligation::TestsPass,
            vec![diagnostic(C::TestFailed, Severity::Warning, 3)],
        );
        assert_eq!(
            status(b, Obligation::TestsPass),
            Some(ObligationStatus::Warning)
        );
        let mut b = Builder::default();
        b.check(
            Obligation::TestsPass,
            vec![diagnostic(C::TestFailed, Severity::Warning, 3)],
        );
        b.check(
            Obligation::TestsPass,
            vec![diagnostic(C::TestFailed, Severity::Error, 2)],
        );
        let report = b.finish(Profile::Strict);
        let f = report.finding(Obligation::TestsPass);
        assert_eq!(f.map(|f| f.status), Some(ObligationStatus::Failed));
        assert_eq!(f.map(|f| f.count), Some(2));
        assert_eq!(
            f.map(|f| f.examples.clone()),
            Some(vec![
                "pkgs/demo/main.lit:2:1: L5006 finding on line 2".to_owned(),
                "pkgs/demo/main.lit:3:1: L5006 finding on line 3".to_owned(),
            ])
        );
        assert!(!report.is_accepted());
    }

    #[test]
    fn examples_are_capped() {
        let mut b = Builder::default();
        let found = (1..=10)
            .map(|line| diagnostic(C::NoExit, Severity::Error, line))
            .collect();
        b.check(Obligation::GraphExits, found);
        let report = b.finish(Profile::Strict);
        let f = report.finding(Obligation::GraphExits);
        assert_eq!(
            f.map(|f| (f.count, f.examples.len())),
            Some((10, MAX_EXAMPLES))
        );
    }
}
