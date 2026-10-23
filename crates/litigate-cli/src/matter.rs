//! `.case` matters (SPEC §3.2, §5.6). `litigate-sema`'s case checker
//! resolves the matter, checks its pins against the `Lit.lock` beside it,
//! and replays its events under the engine clock, run and reported (its
//! `case-replay` obligation) by `litigate-verify`; the matter is then
//! lowered to a litgraph `NamedScenario`.
//!
//! The deadline schedule is a report over the replayed history: every
//! deadline of the used packages whose trigger has occurred, counted by
//! litgraph's clock with its steps (what `explain-value` traces).
use crate::clock::{self, Clocks};
use crate::hirx;
use crate::report;
use crate::ws::{CaseText, Session};
use litigate_base::{Diagnostic, FileId, LineIndex, Profile, Severity};
use litigate_lower::{EngineClock, LoweredCase, Lowerer};
use litigate_sema::case::{Env, Step};
use litigate_sema::hir;
use litigate_sema::pkg::lock::LockFile;
use litigate_sema::temporal::MatterEvent;
use litigate_verify::Finding;
use serde::Serialize;

/// A deadline the events triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Scheduled {
    /// Qualified deadline id.
    pub deadline: String,
    /// The latest triggering event.
    pub trigger: String,
    /// Its date.
    pub trigger_date: String,
    /// Due date.
    pub due: String,
    /// Counting rule.
    pub rule: String,
    /// Counting steps.
    pub steps: Vec<String>,
    /// The deadline version that counted it (`id@from`).
    #[serde(skip)]
    pub version: String,
    /// The `cite` of the counting version that counted it.
    #[serde(skip)]
    pub cite: Option<String>,
    /// Whether the last day rolled forward.
    #[serde(skip)]
    pub rolled: bool,
    /// The service method that added days, if any.
    #[serde(skip)]
    pub method: Option<String>,
}

/// A checked matter.
#[derive(Debug, Clone)]
pub struct CaseCheck {
    /// Repository-relative path.
    pub path: String,
    /// Case HIR with its replayed `start` (present when it resolved).
    pub case: Option<hir::Case>,
    /// The lowered scenario (present when the matter has no errors).
    pub lowered: Option<LoweredCase>,
    /// Every diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// The matter's obligation findings.
    pub findings: Vec<Finding>,
    /// Replay trace.
    pub replay: Vec<Step>,
    /// Deadlines the events triggered, with their due dates.
    pub schedule: Vec<Scheduled>,
    /// Names of the packages it uses, primary first.
    pub packages: Vec<String>,
}

/// The JSON summary `check` reports per matter.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// Path.
    pub path: String,
    /// Case id.
    pub id: Option<String>,
    /// Replayed start (qualified).
    pub start: Option<String>,
    /// Replay trace.
    pub replay: Vec<Step>,
    /// Deadline schedule.
    pub deadlines: Vec<Scheduled>,
}

impl CaseCheck {
    /// The JSON summary.
    #[must_use]
    pub fn summary(&self) -> Summary {
        Summary {
            path: self.path.clone(),
            id: self.case.as_ref().map(|c| c.id.clone()),
            start: self.case.as_ref().and_then(|c| c.start.clone()),
            replay: self.replay.clone(),
            deadlines: self.schedule.clone(),
        }
    }
}

/// The `Lit.lock` path beside matter `path`.
fn lock_path(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/Lit.lock"),
        None => "Lit.lock".to_owned(),
    }
}

/// The matter's lock, or the L1007 for one that does not parse.
fn lock(case: &CaseText) -> Result<Option<LockFile>, Diagnostic> {
    let Some(text) = case.lock.as_deref() else {
        return Ok(None);
    };
    LockFile::parse(text)
        .map(Some)
        .map_err(|r| r.to_diagnostic(&FileId(lock_path(&case.path)), &LineIndex::new(text)))
}

/// Whether the matter's source writes `start`.
fn asserts_start(text: &str) -> bool {
    litigate_syntax::parse_case(text)
        .case_file()
        .is_some_and(|f| f.start().is_some())
}

/// The engine's default start for `case`: its primary package's start, qualified.
fn default_start(all: &[&hir::Package], case: &hir::Case) -> Option<String> {
    let primary = case.uses.first()?;
    all.iter()
        .find(|p| p.manifest.name == primary.name)
        .map(|p| format!("{}::{}", p.manifest.pack_id, p.start))
}

fn is_error(d: &Diagnostic) -> bool {
    d.severity == Severity::Error
}

/// Checks one matter against the loaded packages.
#[must_use]
pub fn check(
    session: &Session,
    case: &CaseText,
    engine: &EngineClock,
    clocks: &Clocks,
    profile: Profile,
) -> CaseCheck {
    let all = session.hir();
    // CLI-side findings: an unreadable lock, and lowering errors.
    let (lock, mut own) = match lock(case) {
        Ok(lock) => (lock, Vec::new()),
        Err(d) => (None, vec![d]),
    };
    let verified = litigate_verify::verify_case_source(
        &case.path,
        &case.text,
        Env {
            packages: &all,
            lock: lock.as_ref(),
            clock: engine,
            profile,
        },
    );
    let mut diagnostics = verified.diagnostics();
    let mut checked_case = verified.checked.case;
    if let Some(c) = checked_case
        .as_mut()
        .filter(|c| c.start == default_start(&all, c) && !asserts_start(&case.text))
    {
        // No start is asserted and the events (none, or only option
        // removals) leave the matter at the engine's default start, the
        // primary package's: it lowers as no `start` at all.
        c.start = None;
    }
    let clean = checked_case
        .as_ref()
        .filter(|_| !own.iter().chain(&diagnostics).any(is_error));
    let lowered = clean.and_then(|c| match Lowerer::new(&all).case(c, &case.path) {
        Ok(lowered) => Some(lowered),
        Err(errors) => {
            own.extend(errors);
            None
        }
    });
    let used: Vec<&hir::Package> = checked_case.as_ref().map_or_else(Vec::new, |c| {
        c.uses
            .iter()
            .filter_map(|pin| all.iter().copied().find(|p| p.manifest.name == pin.name))
            .collect()
    });
    let schedule = match clean {
        Some(c) if lowered.is_some() => schedule(&used, clocks, c),
        _ => Vec::new(),
    };
    let extra = report::findings_of(&own);
    let findings = report::merge(&[&verified.verification.report.obligations, &extra]);
    diagnostics.extend(own);
    if profile == Profile::Strict {
        diagnostics.extend(litigate_syntax::check_format(
            &case.text,
            litigate_syntax::FileKind::Case,
            &case.path.as_str().into(),
        ));
    }
    CaseCheck {
        path: case.path.clone(),
        packages: used.iter().map(|p| p.manifest.name.clone()).collect(),
        case: checked_case,
        lowered,
        diagnostics,
        findings,
        replay: verified.checked.trace,
        schedule,
    }
}

/// Events in date order, ties in source order (SPEC §5.6).
fn history(case: &hir::Case) -> Vec<&hir::Event> {
    let mut events: Vec<&hir::Event> = case.events.iter().collect();
    events.sort_by(|a, b| a.date.cmp(&b.date));
    events
}

/// Every deadline of `packages` with a trigger in the matter's history,
/// counted from its latest trigger under the declarations governing it
/// among the used packages, as replay selects them (`clock::due`: at the
/// matter's `known_at`, with precedence and transitions applied).
fn schedule(packages: &[&hir::Package], clocks: &Clocks, case: &hir::Case) -> Vec<Scheduled> {
    let history = history(case);
    let names: Vec<&str> = packages.iter().map(|p| p.manifest.name.as_str()).collect();
    let world = clocks.scope(&names);
    let events: Vec<MatterEvent<'_>> = history
        .iter()
        .map(|e| MatterEvent {
            date: &e.date,
            action: &e.action,
        })
        .collect();
    let mut out: Vec<Scheduled> = Vec::new();
    for pkg in packages {
        for d in hirx::declarations(pkg) {
            let hir::Declaration::Deadline(d) = d else {
                continue;
            };
            let id = format!("{}::{}", pkg.manifest.pack_id, d.meta.id);
            if out.iter().any(|s| s.deadline == id) {
                continue;
            }
            let Some(at) = history
                .iter()
                .rposition(|e| hirx::matches(&d.trigger, &e.action))
            else {
                continue;
            };
            let known_at = case.known_at.as_deref();
            if let Ok(due) = clock::due(clocks, &world, pkg, &d.meta.id, &events, at, known_at) {
                out.push(Scheduled {
                    deadline: id,
                    trigger: hirx::event_text(&history[at].action),
                    trigger_date: due.trigger,
                    due: due.computed.due_date,
                    rule: due.rule,
                    steps: due.computed.steps,
                    version: due.version,
                    cite: due.cite,
                    rolled: due.computed.last_day_rolled,
                    method: due.method,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lock_sits_beside_the_matter() {
        assert_eq!(
            lock_path("cases/examples/acme.case"),
            "cases/examples/Lit.lock"
        );
        assert_eq!(lock_path("acme.case"), "Lit.lock");
    }

    #[test]
    fn an_unparseable_lock_is_l1007_at_the_lock() {
        let case = CaseText {
            path: "cases/a.case".into(),
            text: String::new(),
            lock: Some("version = 7\n".into()),
        };
        let d = lock(&case).err().map(|d| (d.code.as_str(), d.span.file.0));
        assert_eq!(d, Some(("L1007", "cases/Lit.lock".to_owned())));
    }

    use crate::testkit::{self, ACME};

    /// `matter::check` of the worked example (plus `extra`) and one matter
    /// with `lock`, else the lock it needs.
    fn checked(extra: &str, text: &str, lock: Option<&str>) -> CaseCheck {
        let session = testkit::example_locked(extra, text, lock);
        let all = session.hir();
        let engine = EngineClock::lowered(&Lowerer::new(&all), &all).unwrap();
        check(
            &session,
            &session.cases[0],
            &engine,
            &Clocks::new(&all),
            Profile::Dev,
        )
    }

    fn errors(c: &CaseCheck) -> Vec<(&str, &str)> {
        c.diagnostics
            .iter()
            .filter(|d| is_error(d))
            .map(|d| (d.code.as_str(), d.span.file.0.as_str()))
            .collect()
    }

    #[test]
    fn the_default_start_is_the_primary_package_start() {
        let c = checked("", ACME, None);
        let session = testkit::example_with("");
        let all = session.hir();
        let mut case = c.case.unwrap();
        assert_eq!(
            default_start(&all, &case).as_deref(),
            Some("frcp-rule12-example::time-barred")
        );
        case.uses.clear();
        assert_eq!(default_start(&all, &case), None);
    }

    #[test]
    fn check_reports_an_unparseable_lock() {
        let c = checked("", ACME, Some("version = 7\n"));
        // The unreadable lock, and the matter it then leaves unpinned.
        assert_eq!(
            errors(&c),
            [
                ("L1007", "cases/examples/acme.case"),
                ("L1007", "cases/examples/Lit.lock")
            ]
        );
        assert!(c.lowered.is_none());
    }

    #[test]
    fn a_matter_that_does_not_lower_has_no_scenario_or_schedule() {
        let text = ACME.replace(
            "analysis { params: { rate: 650, opp_rate: 550 } }",
            "analysis { frobnicate: 1 }",
        );
        let c = checked("", &text, None);
        assert_eq!(errors(&c), [("L4007", "cases/examples/acme.case")]);
        assert!(c.lowered.is_none());
        assert!(c.schedule.is_empty());
    }

    #[test]
    fn a_deadline_the_engine_cannot_count_is_left_off_the_schedule() {
        let c = checked(
            "deadline in-business-days: 3.business_days from Served(complaint) counted_by frcp6",
            ACME,
            None,
        );
        let scheduled: Vec<&str> = c.schedule.iter().map(|s| s.deadline.as_str()).collect();
        assert_eq!(scheduled, ["frcp-rule12-example::answer-period"]);
    }
}
