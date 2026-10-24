//! Obligations discharged by earlier phases: each sema (or lowering)
//! diagnostic is attributed to the obligation it violates (SPEC §7 codes).
use crate::report::{Builder, Obligation};
use litigate_base::{Diagnostic, DiagnosticCode as C};

/// The obligation `code` violates, or `None` for a lint with no
/// obligation (anonymous edges, unused declarations, formatting).
///
/// Syntax, name, type, and lowering errors all mean the program is not a
/// well-typed program the engine can run, so each counts against
/// `well-typed`: no error diagnostic ever leaves every obligation passed.
#[must_use]
pub fn obligation_of(code: C) -> Option<Obligation> {
    Some(match code {
        C::UnboundCite => Obligation::CiteResolves,
        C::InvalidDistribution => Obligation::DistExact,
        C::EffectiveOverlap => Obligation::EffectiveDisjoint,
        C::EffectiveGap | C::NoVersionInForce | C::InvalidTransition => {
            Obligation::EffectiveCovered
        }
        C::PrecedenceCycle => Obligation::PrecedenceAcyclic,
        C::MissingDelegation => Obligation::DelegationRespected,
        C::ExceptionConflict | C::UnresolvedPrecedence => Obligation::ExceptionsOrdered,
        C::UnreachableNode => Obligation::GraphReachable,
        C::NoExit => Obligation::GraphExits,
        C::DeadlineUnsatisfiable => Obligation::DeadlineSatisfiable,
        C::TestFailed => Obligation::TestsPass,
        C::NoOptionToComplete | C::DeadTransition | C::ObligationUnconsumed => {
            Obligation::WorkflowSound
        }
        C::HoleInStrict | C::HolePresent => Obligation::Holes,
        C::RuleWithoutTest => Obligation::RuleWithoutTest,
        C::AnonymousEdge | C::NonCanonicalFormat | C::UnusedDeclaration => return None,
        c if c.as_str().starts_with("L4") => Obligation::CaseReplay,
        _ => Obligation::WellTyped,
    })
}

/// Records every diagnostic against its obligation. A sema obligation with
/// none passes only on the evidence `evidence::hir` records from the HIR;
/// without HIR it is skipped, never passed by the absence of diagnostics.
pub(crate) fn aggregate(builder: &mut Builder, diagnostics: &[Diagnostic]) {
    for d in diagnostics {
        if let Some(obligation) = obligation_of(d.code) {
            builder.check(obligation, vec![d.clone()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_non_lint_code_has_an_obligation() {
        let orphans: Vec<&str> = C::ALL
            .into_iter()
            .filter(|code| obligation_of(*code).is_none())
            .map(C::as_str)
            .filter(|code| !code.starts_with("L9"))
            .collect();
        assert_eq!(orphans, Vec::<&str>::new());
    }

    #[test]
    fn lints_without_an_obligation_are_not_aggregated() {
        let span = crate::diag::start_of("Lit.toml");
        let mut b = Builder::default();
        aggregate(
            &mut b,
            &[
                Diagnostic::error(C::AnonymousEdge, span.clone(), "anonymous"),
                Diagnostic::error(C::TypeMismatch, span, "mismatch"),
            ],
        );
        let report = b.finish(litigate_base::Profile::Strict);
        let counts: Vec<(Obligation, usize)> = report
            .obligations
            .iter()
            .filter(|f| f.count > 0)
            .map(|f| (f.name, f.count))
            .collect();
        assert_eq!(counts, [(Obligation::WellTyped, 1)]);
    }

    #[test]
    fn codes_follow_the_spec_table() {
        let table = [
            (C::TypeMismatch, Obligation::WellTyped),
            (C::UnboundCite, Obligation::CiteResolves),
            (C::EffectiveOverlap, Obligation::EffectiveDisjoint),
            (C::EffectiveGap, Obligation::EffectiveCovered),
            (C::PrecedenceCycle, Obligation::PrecedenceAcyclic),
            (C::MissingDelegation, Obligation::DelegationRespected),
            (C::ExceptionConflict, Obligation::ExceptionsOrdered),
            (C::InvalidDistribution, Obligation::DistExact),
            (C::NoExit, Obligation::GraphExits),
            (C::UnreachableNode, Obligation::GraphReachable),
            (C::DeadlineUnsatisfiable, Obligation::DeadlineSatisfiable),
            (C::TestFailed, Obligation::TestsPass),
            (C::NoOptionToComplete, Obligation::WorkflowSound),
            (C::DeadTransition, Obligation::WorkflowSound),
            (C::ObligationUnconsumed, Obligation::WorkflowSound),
            (C::DeadlineMissed, Obligation::CaseReplay),
            (C::StartConflict, Obligation::CaseReplay),
            (C::HoleInStrict, Obligation::Holes),
            (C::HolePresent, Obligation::Holes),
            (C::RuleWithoutTest, Obligation::RuleWithoutTest),
        ];
        for (code, obligation) in table {
            assert_eq!(obligation_of(code), Some(obligation), "{}", code.as_str());
        }
    }
}
