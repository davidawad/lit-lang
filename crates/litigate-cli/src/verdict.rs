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
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
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

/// True when the top-level declaration holding `span` carries the
/// `/// imported` marker (a doc line directly above its header).
#[must_use]
pub fn imported(span: &Span, sources: &[(String, String)]) -> bool {
    let Some((_, text)) = sources.iter().find(|(p, _)| *p == span.file.0) else {
        return false;
    };
    let lines: Vec<&str> = text.lines().collect();
    let at = usize::try_from(span.line).unwrap_or(1).saturating_sub(1);
    let header = (0..=at.min(lines.len().saturating_sub(1)))
        .rev()
        .find(|&i| {
            let l = lines[i];
            !l.is_empty()
                && !l.starts_with(char::is_whitespace)
                && !l.starts_with(['{', '}'])
                && !l.starts_with("//")
        });
    header.is_some_and(|h| {
        lines[..h]
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .any(|l| l.trim() == "/// imported")
    })
}

/// SPEC §7's `dev` exception: an unresolved cite (L1010) on a declaration
/// `litigate import` marked `/// imported` is a warning, and
/// `cite-resolves` is judged on the adjusted severities.
fn imported_cites(
    verification: &mut Verification,
    sema: &[Diagnostic],
    sources: &[(String, String)],
) {
    let mut changed = false;
    for d in &mut verification.diagnostics {
        if d.code == C::UnboundCite && d.severity == Severity::Error && imported(&d.span, sources) {
            d.severity = Severity::Warning;
            changed = true;
        }
    }
    if !changed {
        return;
    }
    let cites: Vec<Diagnostic> = sema
        .iter()
        .chain(&verification.diagnostics)
        .filter(|d| d.code == C::UnboundCite)
        .cloned()
        .collect();
    let adjusted = report::findings_of(&cites);
    for f in &mut verification.report.obligations {
        if let Some(a) = adjusted.iter().find(|a| a.name == f.name) {
            *f = a.clone();
        }
    }
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
            let mut verification = litigate_verify::verify_package(&input, corpus, profile);
            if profile == Profile::Dev {
                imported_cites(&mut verification, &pkg.check.diagnostics, &pkg.files);
            }
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

    fn at(line: u32) -> Span {
        Span {
            file: "a.lit".into(),
            line,
            col: 3,
            end_line: line,
            end_col: 4,
        }
    }

    #[test]
    fn the_marker_covers_its_whole_declaration() {
        let text = "/// imported\nnode a: \"A\"\n{\n  e -> b {\n    authority cite X 1\n  }\n}\n\nnode b: \"B\" {\n}\n";
        let sources = [("a.lit".to_owned(), text.to_owned())];
        assert!(imported(&at(2), &sources));
        assert!(imported(&at(5), &sources));
        assert!(!imported(&at(9), &sources));
        assert!(!imported(&at(5), &[]));
    }
}
