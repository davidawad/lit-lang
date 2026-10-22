//! The obligations `litigate check` discharges (SPEC §7), over sema's
//! diagnostics plus the HIR checks below. This is the CLI's bridge until
//! `litigate-verify` grows an evaluator: it only reads HIR, so moving it
//! there is mechanical.
pub mod cites;
pub mod graph;

use crate::clock::Clocks;
use crate::inline_tests::{self, TestResult};
use crate::ws::{Pkg, Session};
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
use litigate_syntax::FileKind;
use litigate_verify::{Finding, Obligation};

/// One package's findings.
#[derive(Debug, Clone, Default)]
pub struct Verdict {
    /// Every diagnostic, severities set for the profile.
    pub diagnostics: Vec<Diagnostic>,
    /// Inline test results.
    pub tests: Vec<TestResult>,
    /// False when sema produced no HIR, so HIR obligations did not run.
    pub ran: bool,
}

/// Sets `d`'s severity for `profile` (SPEC §9.1), with the §7 exception:
/// in `dev`, an unresolved cite on an `/// imported` declaration warns.
#[must_use]
pub fn adjust(mut d: Diagnostic, profile: Profile, sources: &[(String, String)]) -> Diagnostic {
    d.severity = d.code.severity(profile);
    if d.code == C::UnboundCite && profile == Profile::Dev && cites::imported(&d.span, sources) {
        d.severity = Severity::Warning;
    }
    d
}

/// Spans of `cite ?(..)` holes: holes written right after `cite`.
fn cite_holes(pkg: &Pkg) -> Vec<Span> {
    pkg.check
        .holes
        .iter()
        .filter(|h| {
            pkg.files.iter().any(|(path, text)| {
                *path == h.span.file.0
                    && text
                        .lines()
                        .nth(usize::try_from(h.span.line).unwrap_or(1).saturating_sub(1))
                        .is_some_and(|line| {
                            let col = usize::try_from(h.span.col).unwrap_or(1).saturating_sub(1);
                            let before: String = line.chars().take(col).collect();
                            before.trim_end().ends_with("cite")
                        })
            })
        })
        .map(|h| h.span.clone())
        .collect()
}

/// A hole as L9002 (`dev`) or L5002 (`strict`).
#[must_use]
pub fn hole(span: &Span, reason: Option<&str>, profile: Profile) -> Diagnostic {
    let why = reason.map_or_else(|| "no reason given".to_owned(), |r| format!("\"{r}\""));
    match profile {
        Profile::Dev => Diagnostic::error(C::HolePresent, span.clone(), format!("hole: {why}")),
        Profile::Strict => Diagnostic::error(
            C::HoleInStrict,
            span.clone(),
            format!("strict profile does not permit the hole {why}"),
        ),
    }
}

/// Verifies one package.
#[must_use]
pub fn package(
    session: &Session,
    pkg: &Pkg,
    clocks: &Clocks,
    profile: Profile,
    as_of: Option<&str>,
) -> Verdict {
    let mut out: Vec<Diagnostic> = pkg.check.diagnostics.clone();
    let has_deps =
        litigate_sema::pkg::manifest::parse_manifest(pkg.input.manifest_text(&session.db))
            .is_ok_and(|m| !m.manifest.dependencies.is_empty());
    if pkg.input.lock_text(&session.db).is_some() || has_deps {
        out.extend(litigate_sema::lock_diagnostics(
            &session.db,
            session.ws,
            pkg.input,
        ));
    }
    out.extend(
        pkg.check
            .holes
            .iter()
            .map(|h| hole(&h.span, h.reason.as_deref(), profile)),
    );
    if profile == Profile::Strict {
        out.extend(
            pkg.files
                .iter()
                .filter(|(p, _)| p.ends_with(".lit"))
                .filter_map(|(p, text)| {
                    litigate_syntax::check_format(text, FileKind::Lit, &p.as_str().into())
                }),
        );
    }
    let mut tests = Vec::new();
    let ran = pkg.check.package.is_some();
    if let Some(hir) = &pkg.check.package {
        let fallback = inline_tests::nowhere(hir);
        let date = as_of.unwrap_or(&hir.manifest.in_force.known_at);
        out.extend(graph::anonymous_edges(hir, &fallback));
        if !hir.manifest.links {
            out.extend(graph::shape(hir, date, &fallback));
            out.extend(graph::satisfiable(hir, date, &fallback));
        }
        out.extend(cites::untested(hir, &fallback));
        let deps: Vec<&litigate_sema::hir::Package> = session
            .hir()
            .into_iter()
            .filter(|d| hir.manifest.dependencies.contains_key(&d.manifest.name))
            .collect();
        out.extend(cites::resolve(hir, &deps, &cite_holes(pkg), &fallback));
        let (results, failures) = inline_tests::run(clocks, hir, &pkg.files, None);
        tests = results;
        out.extend(failures);
    }
    let sources = pkg.files.clone();
    Verdict {
        diagnostics: out
            .into_iter()
            .map(|d| adjust(d, profile, &sources))
            .collect(),
        tests,
        ran,
    }
}

/// The codes whose errors fail `obligation`.
fn fails(obligation: Obligation, code: C) -> bool {
    let s = code.as_str();
    match obligation {
        Obligation::WellTyped => s.starts_with("L2") && code != C::InvalidDistribution,
        Obligation::CiteResolves => code == C::UnboundCite,
        Obligation::EffectiveDisjoint => code == C::EffectiveOverlap,
        Obligation::EffectiveCovered => matches!(
            code,
            C::EffectiveGap | C::NoVersionInForce | C::InvalidTransition
        ),
        Obligation::PrecedenceAcyclic => {
            matches!(code, C::PrecedenceCycle | C::UnresolvedPrecedence)
        }
        Obligation::DelegationRespected => code == C::MissingDelegation,
        Obligation::ExceptionsOrdered => code == C::ExceptionConflict,
        Obligation::DistExact => code == C::InvalidDistribution,
        Obligation::GraphExits => code == C::NoExit,
        Obligation::GraphReachable => code == C::UnreachableNode,
        Obligation::DeadlineSatisfiable => code == C::DeadlineUnsatisfiable,
        Obligation::CaseReplay => s.starts_with("L4"),
        Obligation::TestsPass => code == C::TestFailed,
        Obligation::Holes => matches!(code, C::HolePresent | C::HoleInStrict),
    }
}

/// Every obligation in SPEC §7 order.
pub const OBLIGATIONS: [Obligation; 14] = [
    Obligation::WellTyped,
    Obligation::CiteResolves,
    Obligation::EffectiveDisjoint,
    Obligation::EffectiveCovered,
    Obligation::PrecedenceAcyclic,
    Obligation::DelegationRespected,
    Obligation::ExceptionsOrdered,
    Obligation::DistExact,
    Obligation::GraphExits,
    Obligation::GraphReachable,
    Obligation::DeadlineSatisfiable,
    Obligation::TestsPass,
    Obligation::CaseReplay,
    Obligation::Holes,
];

/// Findings from all diagnostics. `not_run` names packages whose HIR
/// obligations could not run; `cases` is the number of matters checked.
#[must_use]
pub fn findings(diagnostics: &[Diagnostic], not_run: &[String], cases: usize) -> Vec<Finding> {
    OBLIGATIONS
        .iter()
        .map(|&obligation| {
            let hits: Vec<&Diagnostic> = diagnostics
                .iter()
                .filter(|d| fails(obligation, d.code))
                .collect();
            let errors: Vec<&&Diagnostic> = hits
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .collect();
            let hir_based = !matches!(obligation, Obligation::WellTyped | Obligation::CaseReplay);
            let (passed, detail) = match errors.first() {
                Some(first) => (
                    false,
                    format!(
                        "{} failure(s); first {}:{}:{}: {}",
                        errors.len(),
                        first.span.file.0,
                        first.span.line,
                        first.span.col,
                        first.message
                    ),
                ),
                None if hir_based && !not_run.is_empty() => (
                    false,
                    format!("not run for {}: the package has errors", not_run.join(", ")),
                ),
                None if obligation == Obligation::CaseReplay && cases == 0 => {
                    (true, "no matters checked".to_owned())
                }
                None if hits.is_empty() => (true, "discharged".to_owned()),
                None => (true, format!("discharged with {} warning(s)", hits.len())),
            };
            Finding {
                obligation,
                passed,
                detail,
            }
        })
        .collect()
}
