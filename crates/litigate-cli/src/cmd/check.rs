//! `litigate check`: every package and matter, every obligation.
use crate::case::{self, CaseCheck};
use crate::cli::Dates;
use crate::clock::Clocks;
use crate::out::{Failure, Outcome, Report};
use crate::verify::{self, Verdict};
use crate::ws::{self, Session};
use litigate_base::{Diagnostic, Profile};
use litigate_verify::Finding;
use serde_json::{json, Value};

/// A completed check.
pub struct Checked {
    /// What was loaded.
    pub session: Session,
    /// Per package (load order), with its name.
    pub verdicts: Vec<(String, Verdict)>,
    /// Per matter.
    pub cases: Vec<CaseCheck>,
    /// Every diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// Obligation findings.
    pub findings: Vec<Finding>,
    /// The profile.
    pub profile: Profile,
    /// Clock fragments.
    pub clocks: Clocks,
}

/// Loads and checks `paths`.
///
/// # Errors
/// Usage or I/O failures.
pub fn checked(
    paths: &[String],
    profile: Profile,
    as_of: Option<&str>,
    lib: &[String],
) -> Result<Checked, Failure> {
    let targets = ws::discover(paths)?;
    let session = Session::load(&targets, lib)?;
    let clocks = Clocks::new(&session.hir());
    let verdicts: Vec<(String, Verdict)> = session
        .packages
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                verify::package(&session, p, &clocks, profile, as_of),
            )
        })
        .collect();
    let cases: Vec<CaseCheck> = session
        .cases
        .iter()
        .map(|c| case::check(&session, c, &clocks, profile))
        .collect();
    let diagnostics: Vec<Diagnostic> = verdicts
        .iter()
        .flat_map(|(_, v)| v.diagnostics.iter().cloned())
        .chain(cases.iter().flat_map(|c| c.diagnostics.iter().cloned()))
        .collect();
    let not_run: Vec<String> = session
        .packages
        .iter()
        .zip(&verdicts)
        .filter(|(_, (_, v))| !v.ran)
        .map(|(p, _)| p.root.clone())
        .collect();
    let findings = verify::findings(&diagnostics, &not_run, cases.len());
    Ok(Checked {
        session,
        verdicts,
        cases,
        diagnostics,
        findings,
        profile,
        clocks,
    })
}

impl Checked {
    /// Repository-relative paths of every checked file.
    #[must_use]
    pub fn files(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .session
            .packages
            .iter()
            .flat_map(|p| p.files.iter().map(|(f, _)| f.clone()))
            .chain(self.cases.iter().map(|c| c.path.clone()))
            .collect();
        files.sort();
        files
    }

    /// Per-package summary.
    #[must_use]
    pub fn packages(&self) -> Vec<Value> {
        self.session
            .packages
            .iter()
            .zip(&self.verdicts)
            .map(|(p, (_, v))| {
                json!({
                    "name": p.name,
                    "version": p.version,
                    "digest": p.digest,
                    "root": p.root,
                    "requested": p.requested,
                    "holes": p.check.holes.len(),
                    "tests": {
                        "passed": v.tests.iter().filter(|t| t.passed).count(),
                        "failed": v.tests.iter().filter(|t| !t.passed).count(),
                    },
                })
            })
            .collect()
    }

    /// Envelope provenance.
    #[must_use]
    pub fn provenance(&self, as_of: Option<&str>, known_at: Option<&str>) -> Value {
        crate::prov::session(&self.session.packages, as_of, known_at)
    }
}

/// `litigate check`.
pub fn check(paths: &[String], profile: Profile, dates: &Dates, lib: &[String]) -> Outcome {
    let c = checked(paths, profile, dates.as_of.as_deref(), lib)?;
    let profile_name = match profile {
        Profile::Dev => "dev",
        Profile::Strict => "strict",
    };
    Ok(Report {
        result: json!({
            "profile": profile_name,
            "files": c.files(),
            "obligations": c.findings,
            "packages": c.packages(),
            "cases": c.cases.iter().map(CaseCheck::summary).collect::<Vec<_>>(),
        }),
        provenance: c.provenance(dates.as_of.as_deref(), dates.known_at.as_deref()),
        sources: c.session.sources(),
        diagnostics: c.diagnostics,
        failed: false,
    })
}
