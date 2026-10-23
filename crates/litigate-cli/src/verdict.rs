//! One package's obligations (SPEC §7), discharged by `litigate-verify`
//! over sema's result and the package's lowering, plus what the CLI checks
//! around them: `Lit.lock` freshness (L1007), canonical formatting in
//! `strict` (L9004), and the anonymous-edge lint (L9003).
use crate::clock::Clocks;
use crate::hirx;
use crate::inline_tests::{self, TestResult};
use crate::report;
use crate::ws::{Pkg, Session};
use litgraph::cite::SourceCorpus;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity};
use litigate_lower::{LoweredPackage, Lowerer, Options};
use litigate_sema::hir;
use litigate_syntax::FileKind;
use litigate_verify::{Finding, PackageInput, Verification};

/// One package's findings.
#[derive(Debug, Clone)]
pub struct Verdict {
    /// Every diagnostic: sema's, the lowering's, verification's, and the
    /// CLI's own, severities set for the profile.
    pub diagnostics: Vec<Diagnostic>,
    /// Inline test results.
    pub tests: Vec<TestResult>,
    /// `litigate-verify`'s report and diagnostics.
    pub verification: Verification,
    /// The report with the CLI's own findings folded in.
    pub findings: Vec<Finding>,
    /// The lowering, absent when sema or lowering failed.
    pub lowered: Option<LoweredPackage>,
}

/// L9003 for every `_` edge (every version): cases and links cannot name it.
#[must_use]
pub fn anonymous_edges(pkg: &hir::Package) -> Vec<Diagnostic> {
    let fallback = inline_tests::nowhere(pkg);
    hirx::nodes_at(pkg, None)
        .into_iter()
        .flat_map(|n| n.edges.iter().map(move |e| (n, e)))
        .filter(|(_, e)| e.id.is_none())
        .map(|(n, e)| Diagnostic {
            severity: Severity::Info,
            ..Diagnostic::error(
                C::AnonymousEdge,
                e.span.clone().unwrap_or_else(|| fallback.clone()),
                format!(
                    "the edge `{} -> {}` has no id, so cases and links cannot name it",
                    n.meta.id,
                    hirx::local(&e.to)
                ),
            )
        })
        .collect()
}

/// `Lit.lock` diagnostics: a lock that exists, or one a package with
/// dependencies needs.
fn lock(session: &Session, pkg: &Pkg) -> Vec<Diagnostic> {
    let has_deps =
        litigate_sema::pkg::manifest::parse_manifest(pkg.input.manifest_text(&session.db))
            .is_ok_and(|m| !m.manifest.dependencies.is_empty());
    if pkg.input.lock_text(&session.db).is_none() && !has_deps {
        return Vec::new();
    }
    litigate_sema::lock_diagnostics(&session.db, session.ws, pkg.input)
}

/// L9004 for every `.lit` file not in canonical form.
fn format(pkg: &Pkg) -> Vec<Diagnostic> {
    pkg.files
        .iter()
        .filter(|(p, _)| {
            std::path::Path::new(p)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("lit"))
        })
        .filter_map(|(p, text)| {
            litigate_syntax::check_format(text, FileKind::Lit, &p.as_str().into())
        })
        .collect()
}

/// Verifies one package at the build dates `options`.
#[must_use]
pub fn package(
    session: &Session,
    pkg: &Pkg,
    clocks: &Clocks,
    profile: Profile,
    options: &Options,
) -> Verdict {
    let all = session.hir();
    let corpus = SourceCorpus::embedded_ref();
    let mut own: Vec<Diagnostic> = pkg.check.diagnostics.clone();
    let locked = lock(session, pkg);
    own.extend(locked.iter().cloned());
    if profile == Profile::Strict {
        own.extend(format(pkg));
    }
    let (verification, lowered, tests) = match &pkg.check.package {
        None => (
            litigate_verify::verify_unchecked(&pkg.check, profile),
            None,
            Vec::new(),
        ),
        Some(hir) => {
            own.extend(anonymous_edges(hir));
            let lowering = Lowerer::new(&all).with_corpus(corpus).package(hir, options);
            if let Err(errors) = &lowering {
                own.extend(errors.iter().cloned());
            }
            let input = PackageInput {
                package: hir,
                check: &pkg.check,
                lowered: lowering.as_ref().map_err(Vec::as_slice),
                packages: &all,
                options,
            };
            let verification = litigate_verify::verify_package(&input, corpus, profile);
            let (tests, _) = inline_tests::run(clocks, hir, &pkg.files, None);
            (verification, lowering.ok(), tests)
        }
    };
    let mut diagnostics: Vec<Diagnostic> = own
        .into_iter()
        .map(|mut d| {
            d.severity = d.code.severity(profile);
            d
        })
        .collect();
    diagnostics.extend(verification.diagnostics.iter().cloned());
    let extra = report::findings_of(&locked);
    let findings = report::merge(&[&verification.report.obligations, &extra]);
    Verdict {
        diagnostics,
        tests,
        verification,
        findings,
        lowered,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{example_with, hir_with};

    #[test]
    fn a_spanless_anonymous_edge_is_reported_at_the_manifest() {
        let mut pkg = hir_with("");
        for decl in &mut pkg.modules[0].declarations {
            if let hir::Declaration::Node(n) = decl {
                n.edges.iter_mut().for_each(|e| e.span = None);
            }
        }
        let found: Vec<(String, u32)> = anonymous_edges(&pkg)
            .into_iter()
            .map(|d| (d.span.file.0, d.span.line))
            .collect();
        assert_eq!(found, vec![("lit/examples/Lit.toml".to_owned(), 1); 2]);
    }

    #[test]
    fn a_lowering_failure_is_a_package_diagnostic() {
        let session = example_with(
            "counting nourl cite XYZ 6(a)\n{\n  exclude_trigger_day\n  include_last_day\n  holidays federal-holidays\n}",
        );
        let clocks = Clocks::new(&session.hir());
        let verdict = package(
            &session,
            &session.packages[0],
            &clocks,
            Profile::Dev,
            &Options::default(),
        );
        assert!(verdict.lowered.is_none());
        let codes: Vec<&str> = verdict
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code.as_str())
            .collect();
        assert_eq!(codes, ["L6001"]);
    }
}
