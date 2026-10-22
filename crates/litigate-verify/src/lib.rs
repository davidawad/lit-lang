//! Verification obligations for compiled litigate packages (SPEC §7).
mod graph;
mod report;

pub use graph::verify_graph;
pub use report::{Finding, Obligation, ObligationStatus, VerificationReport};

use litgraph::{
    cite::{CiteOutcome, SourceCorpus},
    model::Pack,
};
use litigate_base::{Diagnostic, DiagnosticCode, Profile};
use litigate_sema::{hir, types::build::PackageCheck};

/// A lowered document together with its reproducible verification report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Litigatable<T> {
    /// Lowered engine IR.
    pub ir: T,
    /// Verification results for this artifact.
    pub verification: VerificationReport,
    /// Source map sidecar, when available.
    pub source_map: Option<litigate_lower::SourceMap>,
}

/// Verify a checked package and its lowered pack, aggregating sema diagnostics.
///
/// The sema package check carries typing, temporal, precedence, and distribution
/// diagnostics; graph and citation checks use the exact lowered engine schema.
#[must_use]
pub fn verify_package(
    package: &hir::Package,
    pack: &Pack,
    check: &PackageCheck,
    corpus: &SourceCorpus,
    profile: Profile,
) -> VerificationReport {
    let mut report = verify_graph(pack, corpus, profile);
    add_sema_findings(&mut report, &check.diagnostics);
    add_holes(&mut report, check, profile);
    add_rule_tests(&mut report, package, profile);
    let dist_failures = distribution_failures(package);
    report.push(
        Obligation::DistExact,
        if dist_failures.is_empty() {
            ObligationStatus::Passed
        } else {
            ObligationStatus::Failed
        },
        dist_failures.len(),
        dist_failures,
    );
    report.profile = profile;
    report
}

fn add_sema_findings(report: &mut VerificationReport, diagnostics: &[Diagnostic]) {
    for finding in findings_from_diagnostics(diagnostics) {
        if let Some(existing) = report
            .obligations
            .iter_mut()
            .find(|f| f.name == finding.name)
        {
            if finding.status == ObligationStatus::Failed {
                existing.status = ObligationStatus::Failed;
                existing.count += finding.count;
                existing.examples.extend(finding.examples);
                existing.examples.truncate(8);
            }
        } else {
            report.obligations.push(finding);
        }
    }
}

fn add_holes(report: &mut VerificationReport, check: &PackageCheck, profile: Profile) {
    let holes: Vec<String> = check
        .holes
        .iter()
        .map(|hole| {
            format!(
                "{}:{}:{} ({})",
                hole.span.file.0,
                hole.span.line,
                hole.span.col,
                hole.reason.as_deref().unwrap_or("reason not supplied")
            )
        })
        .collect();
    report.push(
        Obligation::Holes,
        profile_status(holes.len(), profile),
        holes.len(),
        holes,
    );
}

fn add_rule_tests(report: &mut VerificationReport, package: &hir::Package, profile: Profile) {
    let tested: std::collections::BTreeSet<&str> = package
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(|d| match d {
            hir::Declaration::Test(test) => test.cite.as_deref(),
            _ => None,
        })
        .collect();
    let missing: Vec<String> = package
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(|d| match d {
            hir::Declaration::Deadline(deadline)
                if !has_test(deadline.meta.cite.as_deref(), &tested) =>
            {
                Some(deadline.meta.id.clone())
            }
            hir::Declaration::Counting(counting)
                if !has_test(counting.meta.cite.as_deref(), &tested) =>
            {
                Some(counting.meta.id.clone())
            }
            _ => None,
        })
        .collect();
    report.push(
        Obligation::RuleWithoutTest,
        profile_status(missing.len(), profile),
        missing.len(),
        missing,
    );
}

fn has_test(cite: Option<&str>, tested: &std::collections::BTreeSet<&str>) -> bool {
    cite.is_some_and(|cite| tested.contains(cite))
}

fn profile_status(count: usize, profile: Profile) -> ObligationStatus {
    match (count, profile) {
        (0, _) => ObligationStatus::Passed,
        (_, Profile::Dev) => ObligationStatus::Warning,
        (_, Profile::Strict) => ObligationStatus::Failed,
    }
}

/// Add the sema replay result for a checked case to a package report.
pub fn add_case_replay(report: &mut VerificationReport, diagnostics: &[Diagnostic]) {
    let failures: Vec<String> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.code,
                DiagnosticCode::InvalidCaseEvent
                    | DiagnosticCode::DeadlineMissed
                    | DiagnosticCode::InvalidCaseFact
                    | DiagnosticCode::InvalidParty
                    | DiagnosticCode::MissingStake
                    | DiagnosticCode::InvalidBelief
                    | DiagnosticCode::StartConflict
            )
        })
        .map(|d| d.message.clone())
        .collect();
    report.push(
        Obligation::CaseReplay,
        if failures.is_empty() {
            ObligationStatus::Passed
        } else {
            ObligationStatus::Failed
        },
        diagnostics
            .iter()
            .filter(|d| {
                matches!(
                    d.code,
                    DiagnosticCode::InvalidCaseEvent
                        | DiagnosticCode::DeadlineMissed
                        | DiagnosticCode::InvalidCaseFact
                        | DiagnosticCode::InvalidParty
                        | DiagnosticCode::MissingStake
                        | DiagnosticCode::InvalidBelief
                        | DiagnosticCode::StartConflict
                )
            })
            .count(),
        failures,
    );
}

fn findings_from_diagnostics(diagnostics: &[Diagnostic]) -> Vec<Finding> {
    DIAGNOSTIC_GROUPS
        .iter()
        .map(|(obligation, codes)| {
            let examples: Vec<String> = diagnostics
                .iter()
                .filter(|d| codes.contains(&d.code))
                .map(|d| d.message.clone())
                .take(8)
                .collect();
            let count = diagnostics
                .iter()
                .filter(|d| codes.contains(&d.code))
                .count();
            Finding {
                name: *obligation,
                status: if examples.is_empty() {
                    ObligationStatus::Passed
                } else {
                    ObligationStatus::Failed
                },
                count,
                examples,
            }
        })
        .collect()
}

const DIAGNOSTIC_GROUPS: [(Obligation, &[DiagnosticCode]); 10] = [
    (
        Obligation::WellTyped,
        &[
            DiagnosticCode::TypeMismatch,
            DiagnosticCode::InvalidOperation,
            DiagnosticCode::InvalidField,
            DiagnosticCode::InvalidDistribution,
            DiagnosticCode::UnitMismatch,
            DiagnosticCode::NonExhaustiveMatch,
            DiagnosticCode::ArithmeticError,
            DiagnosticCode::InvalidProbability,
            DiagnosticCode::InvalidCounting,
        ],
    ),
    (
        Obligation::EffectiveDisjoint,
        &[DiagnosticCode::EffectiveOverlap],
    ),
    (
        Obligation::EffectiveCovered,
        &[DiagnosticCode::EffectiveGap],
    ),
    (
        Obligation::PrecedenceAcyclic,
        &[DiagnosticCode::PrecedenceCycle],
    ),
    (
        Obligation::DelegationRespected,
        &[DiagnosticCode::MissingDelegation],
    ),
    (
        Obligation::ExceptionsOrdered,
        &[DiagnosticCode::ExceptionConflict],
    ),
    (
        Obligation::DeadlineSatisfiable,
        &[DiagnosticCode::DeadlineUnsatisfiable],
    ),
    (
        Obligation::DistExact,
        &[
            DiagnosticCode::InvalidDistribution,
            DiagnosticCode::InvalidProbability,
        ],
    ),
    (
        Obligation::CaseReplay,
        &[
            DiagnosticCode::InvalidCaseEvent,
            DiagnosticCode::DeadlineMissed,
            DiagnosticCode::InvalidCaseFact,
            DiagnosticCode::InvalidParty,
            DiagnosticCode::MissingStake,
            DiagnosticCode::InvalidBelief,
            DiagnosticCode::StartConflict,
        ],
    ),
    (Obligation::TestsPass, &[DiagnosticCode::TestFailed]),
];

fn distribution_failures(package: &hir::Package) -> Vec<String> {
    package
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(|d| {
            let hir::Declaration::Node(node) = d else {
                return None;
            };
            let hir::NodeKind::Chance { .. } = node.kind else {
                return None;
            };
            let mut probabilities = Vec::new();
            let mut has_hole = false;
            for edge in &node.edges {
                match edge.probability.as_ref() {
                    Some(hir::Authored::Known(probability)) => {
                        probabilities.push(*probability);
                    }
                    _ => has_hole = true,
                }
            }
            (has_hole || !distribution_is_exact(&probabilities)).then(|| node.meta.id.clone())
        })
        .collect()
}

fn distribution_is_exact(probabilities: &[hir::Rational]) -> bool {
    let mut sum = Some((0_i128, 1_i128));
    for probability in probabilities {
        let numerator = i128::from(probability.numerator);
        let denominator = i128::from(probability.denominator);
        sum = sum.and_then(|(n, d)| {
            n.checked_mul(denominator)
                .and_then(|left| {
                    numerator
                        .checked_mul(d)
                        .and_then(|right| left.checked_add(right))
                })
                .zip(d.checked_mul(denominator))
        });
    }
    sum.is_some_and(|(numerator, denominator)| numerator == denominator)
}

/// Resolve every cite emitted by the pack against its bound L0 source files.
pub(crate) fn cite_failures(pack: &Pack, corpus: &SourceCorpus) -> Vec<String> {
    litgraph::cite::check_pack(pack, corpus)
        .into_iter()
        .filter_map(|check| match check.outcome {
            CiteOutcome::Unresolvable { reason } => Some(format!("{}: {}", check.at, reason)),
            CiteOutcome::Verified { .. } | CiteOutcome::OutOfScope => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::{FileId, Span};

    fn diagnostic(code: DiagnosticCode) -> Diagnostic {
        Diagnostic::error(
            code,
            Span {
                file: FileId("fixture.lit".to_owned()),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 2,
            },
            code.as_str(),
        )
    }

    #[test]
    fn holes_are_warnings_in_dev_and_failures_in_strict() {
        assert_eq!(profile_status(1, Profile::Dev), ObligationStatus::Warning);
        assert_eq!(profile_status(1, Profile::Strict), ObligationStatus::Failed);
        assert_eq!(profile_status(0, Profile::Strict), ObligationStatus::Passed);
    }

    #[test]
    fn inline_rule_test_cites_cover_their_rules() {
        let tested = std::collections::BTreeSet::from(["FRCP 12"]);
        assert!(has_test(Some("FRCP 12"), &tested));
        assert!(!has_test(Some("FRCP 13"), &tested));
        assert!(!has_test(None, &tested));
    }

    #[test]
    fn exact_chance_probabilities_sum_as_rationals() {
        let thirds = [
            hir::Rational {
                numerator: 1,
                denominator: 3,
            },
            hir::Rational {
                numerator: 2,
                denominator: 3,
            },
        ];
        let short = [
            hir::Rational {
                numerator: 1,
                denominator: 2,
            },
            hir::Rational {
                numerator: 1,
                denominator: 3,
            },
        ];
        assert!(distribution_is_exact(&thirds));
        assert!(!distribution_is_exact(&short));
    }

    #[test]
    fn sema_obligations_report_clean_and_failing_results() {
        let passed = findings_from_diagnostics(&[]);
        insta::assert_json_snapshot!(passed);

        let diagnostics: Vec<Diagnostic> = DIAGNOSTIC_GROUPS
            .iter()
            .map(|(_, codes)| diagnostic(codes[0]))
            .collect();
        let failed = findings_from_diagnostics(&diagnostics);
        insta::assert_json_snapshot!(failed);
    }

    #[test]
    fn case_replay_includes_event_and_posture_failures() {
        let mut report = VerificationReport::new(Profile::Strict);
        add_case_replay(&mut report, &[diagnostic(DiagnosticCode::DeadlineMissed)]);
        insta::assert_json_snapshot!(report);
    }
}
