//! `case-replay` for matters (SPEC §5.6, §7): [`verify_case`] and
//! [`verify_case_source`] run sema's case checker (resolution, pins,
//! parties, event replay under the versions governing each date, deadlines
//! on the engine clock, facts, stakes, beliefs) and report its diagnostics
//! against their obligations. Only `unvalued` stakes in `dev` are added
//! here, as warnings: sema reports them in `strict` alone (L4004).
use crate::diag;
use crate::report::{Builder, Obligation as O};
use crate::{sema, Verification};
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity};
use litigate_sema::case::{self, CaseSpans, CheckedCase, Env};
use litigate_sema::hir;

/// A matter checked by sema and verified.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedCase {
    /// Every obligation's result. Its `diagnostics` are the ones this
    /// crate adds; the case checker's are in `checked`.
    pub verification: Verification,
    /// The case checker's result: the matter with its replayed posture in
    /// `start`, and its diagnostics.
    pub checked: CheckedCase,
}

impl VerifiedCase {
    /// Whether the profile accepts the matter: no obligation failed.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.verification.is_accepted()
    }

    /// The case checker's and verification's diagnostics together, sorted
    /// (SPEC §9.1).
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut all: Vec<Diagnostic> = self
            .checked
            .diagnostics
            .iter()
            .chain(&self.verification.diagnostics)
            .cloned()
            .collect();
        all.sort_by(diag::order);
        all
    }
}

/// Checks and verifies `.case` source `text` (repository-relative `file`)
/// against `env`'s packages, lock, and clock under `env.profile`.
#[must_use]
pub fn verify_case_source(file: &str, text: &str, env: Env<'_>) -> VerifiedCase {
    let checked = case::check_source(file, text, env);
    // Spans are only needed for the dev `unvalued` warnings; resolve again
    // only when there are any, so the common path checks once.
    let spans = checked
        .case
        .as_ref()
        .filter(|c| env.profile == Profile::Dev && has_unvalued(c))
        .map(|_| case::resolve(file, text, env.packages, env.lock).spans)
        .unwrap_or_default();
    verified(checked, &spans, env.profile)
}

/// Checks and verifies a matter already in HIR; `spans` locate its parts
/// ([`CaseSpans::default`] when it has no source: findings then name the
/// case).
#[must_use]
pub fn verify_case(case: hir::Case, spans: &CaseSpans, env: Env<'_>) -> VerifiedCase {
    let checked = case::check(case, spans, env);
    verified(checked, spans, env.profile)
}

fn verified(checked: CheckedCase, spans: &CaseSpans, profile: Profile) -> VerifiedCase {
    let mut b = Builder::default();
    sema::aggregate(&mut b, &checked.diagnostics);
    let added = if let Some(case) = &checked.case {
        let added = unvalued(case, spans, profile);
        b.check(O::CaseReplay, added.clone());
        added
    } else {
        b.skip(O::CaseReplay, "not checked: the matter did not resolve");
        Vec::new()
    };
    for name in O::ALL {
        b.skip(
            name,
            "a matter is checked for replay only; its packages carry the rest",
        );
    }
    VerifiedCase {
        verification: Verification::new(b, added, profile),
        checked,
    }
}

fn has_unvalued(case: &hir::Case) -> bool {
    case.stakes.values().any(|s| *s == hir::Stake::Unvalued)
}

/// In `dev`, a warning for each `unvalued` stake (sema rejects them in
/// `strict` itself).
fn unvalued(case: &hir::Case, spans: &CaseSpans, profile: Profile) -> Vec<Diagnostic> {
    if profile != Profile::Dev {
        return Vec::new();
    }
    case.stakes
        .iter()
        .filter(|(_, s)| **s == hir::Stake::Unvalued)
        .map(|(node, _)| {
            let d = diag::emit(
                C::MissingStake,
                profile,
                spans.at(spans.stakes.get(node), &case.id),
                format!("stake `{node}` is `unvalued`"),
            );
            diag::note(
                Diagnostic {
                    severity: Severity::Warning,
                    ..d
                },
                None,
                "`strict` requires a value for every terminal",
            )
        })
        .collect()
}
