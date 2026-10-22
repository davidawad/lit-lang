//! Obligations discharged by earlier phases: each sema (or lowering)
//! diagnostic is attributed to the obligation it violates (SPEC §7 codes).
use crate::report::{Builder, Obligation};
use litigate_base::{Diagnostic, DiagnosticCode as C};

/// Obligations whose result comes entirely from sema diagnostics.
pub(crate) const SEMA: [Obligation; 7] = [
    Obligation::WellTyped,
    Obligation::EffectiveDisjoint,
    Obligation::EffectiveCovered,
    Obligation::PrecedenceAcyclic,
    Obligation::DelegationRespected,
    Obligation::ExceptionsOrdered,
    Obligation::DistExact,
];

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
        C::HoleInStrict | C::HolePresent => Obligation::Holes,
        C::RuleWithoutTest => Obligation::RuleWithoutTest,
        C::AnonymousEdge | C::NonCanonicalFormat | C::UnusedDeclaration => return None,
        c if c.as_str().starts_with("L4") => Obligation::CaseReplay,
        _ => Obligation::WellTyped,
    })
}

/// Records every diagnostic against its obligation, and a clean result for
/// each sema obligation that has none.
pub(crate) fn aggregate(builder: &mut Builder, diagnostics: &[Diagnostic]) {
    for obligation in SEMA {
        builder.check(obligation, Vec::new());
    }
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
        for code in C::ALL {
            let lint = code.as_str().starts_with("L9");
            assert!(
                lint || obligation_of(code).is_some(),
                "{} has no obligation",
                code.as_str()
            );
        }
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
