//! `litigate check`: every package and matter, every obligation.
use crate::cli::Dates;
use crate::matter::CaseCheck;
use crate::out::{Failure, Outcome, Report};
use crate::queries::{case_check, package_verdict, Build};
use crate::verdict::Verdict;
use crate::ws::{self, Session};
use litigate_base::{Diagnostic, Profile};
use litigate_lower::Options;
use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::change::world::{check_world, WorldCheck};
use litigate_sema::pkg::source::{Disk, Source};
use litigate_verify::{Finding, Obligation};
use serde_json::{json, Value};

/// A completed check.
pub struct Checked {
    /// What was loaded.
    pub session: Session,
    /// Per package (load order), with its name.
    pub verdicts: Vec<(String, Verdict)>,
    /// Per matter.
    pub cases: Vec<CaseCheck>,
    /// Per world (docs/litigate/CHANGE.md §5).
    pub worlds: Vec<WorldCheck>,
    /// Every diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// Obligation findings.
    pub findings: Vec<Finding>,
    /// The profile.
    pub profile: Profile,
    /// Every package and matter was checked: false when
    /// [`check_session_with`] was cancelled part way, and `verdicts` and
    /// `cases` hold only those checked before.
    pub complete: bool,
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

/// Loads and checks `targets` from `src`.
///
/// # Errors
/// I/O failures.
pub fn checked_targets(
    src: &dyn Source,
    targets: &ws::Targets,
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    Ok(check_session(
        Session::load_in(src, targets, lib)?,
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
/// `as_of` and `known_at`), through the session database's queries
/// ([`crate::queries`]): a session checked again after an edit re-runs only
/// what the edit reaches.
#[must_use]
pub fn check_session(session: Session, profile: Profile, options: &Options) -> Checked {
    check_session_with(session, profile, options, &|| false)
}

/// Checks a session, stopping between packages and matters when cancelled.
#[must_use]
pub fn check_session_with(
    mut session: Session,
    profile: Profile,
    options: &Options,
    cancelled: &dyn Fn() -> bool,
) -> Checked {
    let inputs = session.case_inputs();
    let build = Build::of(&session.db, profile, options);
    let verdicts: Vec<(String, Verdict)> = session
        .packages
        .iter()
        .take_while(|_| !cancelled())
        .map(|p| {
            let verdict = if let Some(cache) = session.cache.as_ref() {
                crate::queries::package_verdict_with_cache(
                    &session.db,
                    session.ws,
                    p.input,
                    build,
                    Some(cache),
                )
            } else {
                package_verdict(&session.db, session.ws, p.input, build).clone()
            };
            (p.name.clone(), verdict)
        })
        .collect();
    // A matter is checked at its own dates.
    let cases: Vec<CaseCheck> = inputs
        .iter()
        .take_while(|_| !cancelled())
        .map(|c| {
            let mut check = case_check(&session.db, session.ws, *c, build).clone();
            crate::matter::upgrade_lock(&session, &mut check);
            check
        })
        .collect();
    let worlds: Vec<WorldCheck> = session
        .worlds
        .iter()
        .take_while(|_| !cancelled())
        .map(|w| check_world(&session.db, session.ws, &w.path, &w.text, profile))
        .collect();
    let complete = verdicts.len() == session.packages.len()
        && cases.len() == inputs.len()
        && worlds.len() == session.worlds.len();
    let diagnostics: Vec<Diagnostic> = verdicts
        .iter()
        .flat_map(|(_, v)| v.diagnostics.iter().cloned())
        .chain(cases.iter().flat_map(|c| c.diagnostics.iter().cloned()))
        .chain(worlds.iter().flat_map(|w| w.diagnostics.iter().cloned()))
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
        worlds,
        diagnostics,
        findings,
        profile,
        complete,
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
            .chain(self.worlds.iter().map(|w| w.path.clone()))
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
                // Written holes and the cites no amendment history proves.
                let holes = v.findings.iter().find(|f| f.name == Obligation::Holes);
                json!({
                    "name": p.name,
                    "version": p.version,
                    "digest": p.digest,
                    "root": p.root,
                    "requested": p.requested,
                    "holes": holes.map_or(0, |f| f.count),
                    "tests": {
                        "passed": v.tests.iter().filter(|t| t.passed).count(),
                        "failed": v.tests.iter().filter(|t| !t.passed).count(),
                    },
                })
            })
            .collect()
    }

    /// Judges every matter's deadlines at `today` ([`crate::matter::judge_at`]),
    /// reporting the L4016 of each whose `as_of` is older.
    pub fn judge_at(&mut self, today: &str) {
        for case in &mut self.cases {
            let text = self
                .session
                .cases
                .iter()
                .find(|c| c.path == case.path)
                .map_or("", |c| c.text.as_str());
            self.diagnostics
                .extend(crate::matter::judge_at(case, text, today));
        }
    }

    /// Envelope provenance.
    #[must_use]
    pub fn provenance(&self, as_of: Option<&str>, known_at: Option<&str>) -> Value {
        crate::prov::session(&self.session.packages, as_of, known_at)
    }
}

/// A world's summary: its file, the world, and its layers as it has them.
fn world_summary(w: &WorldCheck) -> Value {
    json!({"path": w.path, "world": w.world, "layers": w.layers})
}

/// `litigate check` over `src`, judging deadlines at `today` when given.
pub fn check(
    src: &dyn Source,
    paths: &[String],
    profile: Profile,
    (dates, today): (&Dates, Option<&str>),
    lib: &[String],
    verbose: bool,
) -> Outcome {
    let today = today.map(super::timeline::date).transpose()?;
    let mut c = checked_in(src, paths, profile, &dates.options(), lib)?;
    if let Some(today) = today {
        c.judge_at(today);
    }
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
        .chain(c.session.worlds.iter().map(|w| w.path.clone()))
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
    let mut result = json!({
        "profile": profile_name,
        "files": c.files(),
        "obligations": c.findings,
        "packages": c.packages(),
        "cases": c.cases.iter().map(CaseCheck::summary).collect::<Vec<_>>(),
    });
    if !c.worlds.is_empty() {
        result["worlds"] = c.worlds.iter().map(world_summary).collect();
    }
    Ok(Report {
        result,
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
