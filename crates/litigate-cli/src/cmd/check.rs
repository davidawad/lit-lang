//! `litigate check`: every package and matter, every obligation.
use crate::cli::Dates;
use crate::clock::Clocks;
use crate::matter::{self, CaseCheck};
use crate::out::{Failure, Outcome, Report};
use crate::verdict::{self, Verdict};
use crate::ws::{self, Session};
use litigate_base::{Diagnostic, Profile};
use litigate_lower::{EngineClock, Lowerer, Options};
use litigate_sema::pkg::source::{Disk, Source};
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
}

/// Loads and checks `paths`.
///
/// # Errors
/// Usage or I/O failures.
pub fn checked(
    paths: &[String],
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    checked_in(&Disk, paths, profile, options, lib)
}

/// [`checked`] from `src`.
///
/// # Errors
/// Usage or I/O failures.
pub fn checked_in(
    src: &dyn Source,
    paths: &[String],
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    let targets = ws::discover_in(src, paths)?;
    Ok(check_session(
        Session::load_in(src, &targets, lib)?,
        profile,
        options,
    ))
}

/// Loads and checks `targets`.
///
/// # Errors
/// I/O failures.
pub fn checked_targets(
    targets: &ws::Targets,
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    Ok(check_session(
        Session::load(targets, lib)?,
        profile,
        options,
    ))
}

/// The engine clock matters replay under: every counting rule of `all`
/// lowered. A counting rule that does not lower is reported by its
/// package's verification; replay then counts with the shared library
/// alone.
#[must_use]
pub fn engine(all: &[&litigate_sema::hir::Package]) -> EngineClock {
    EngineClock::lowered(&Lowerer::new(all), all).unwrap_or_else(|_| EngineClock::new([]))
}

/// Checks everything loaded in `session` at the build dates `options`
/// (each defaults to the package's `known_at`; a matter selects at its own
/// `as_of` and `known_at`).
#[must_use]
pub fn check_session(session: Session, profile: Profile, options: &Options) -> Checked {
    let all = session.hir();
    let clocks = Clocks::new(&all);
    let verdicts: Vec<(String, Verdict)> = session
        .packages
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                verdict::package(&session, p, &clocks, profile, options),
            )
        })
        .collect();
    let cases: Vec<CaseCheck> = session
        .cases
        .iter()
        .map(|c| matter::check(&session, c, &clocks, profile))
        .collect();
    let diagnostics: Vec<Diagnostic> = verdicts
        .iter()
        .flat_map(|(_, v)| v.diagnostics.iter().cloned())
        .chain(cases.iter().flat_map(|c| c.diagnostics.iter().cloned()))
        .collect();
    let parts: Vec<&[Finding]> = verdicts
        .iter()
        .map(|(_, v)| v.findings.as_slice())
        .chain(cases.iter().map(|c| c.findings.as_slice()))
        .collect();
    let findings = crate::report::merge(&parts);
    Checked {
        session,
        verdicts,
        cases,
        diagnostics,
        findings,
        profile,
    }
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

/// `litigate check` over `src`.
pub fn check(
    src: &dyn Source,
    paths: &[String],
    profile: Profile,
    dates: &Dates,
    lib: &[String],
    verbose: bool,
) -> Outcome {
    let c = checked_in(src, paths, profile, &dates.options(), lib)?;
    let profile_name = match profile {
        Profile::Dev => "dev",
        Profile::Strict => "strict",
    };
    let own_files: std::collections::BTreeSet<String> = c
        .session
        .packages
        .iter()
        .filter(|p| p.requested)
        .flat_map(|p| p.files.iter().map(|(f, _)| f.clone()))
        .chain(c.session.cases.iter().map(|case| case.path.clone()))
        .collect();
    let package_by_file: std::collections::BTreeMap<String, String> = c
        .session
        .packages
        .iter()
        .flat_map(|p| {
            p.files
                .iter()
                .map(move |(f, _)| (f.clone(), p.name.clone()))
        })
        .collect();
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
        check_render: Some(crate::out::CheckRender {
            verbose,
            own_files,
            package_by_file,
        }),
        profile: Some(profile),
        failed: false,
    })
}
