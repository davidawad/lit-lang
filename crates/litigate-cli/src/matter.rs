//! `.case` matters (SPEC §3.2, §5.6). `litigate-sema`'s case checker
//! resolves the matter, checks its pins against the `Lit.lock` beside it,
//! and replays its events under the engine clock, run and reported (its
//! `case-replay` obligation) by `litigate-verify`; the matter is then
//! lowered to a litgraph `NamedScenario`.
//!
//! The deadline schedule is a report over the replayed history: every
//! obligation instance replay created, counted by litgraph's clock with
//! its steps (what `explain-value` traces), with how it ended.
use crate::clock::{self, Clocks};
use crate::hirx;
use crate::report;
use crate::ws::{lock_path, CaseText, Session};
use litigate_base::{Diagnostic, FileId, LineIndex, Profile, Severity};
use litigate_lower::{LoweredCase, Lowerer};
use litigate_sema::case::{Env, Obligation, Pending, Status, Step};
use litigate_sema::hir;
use litigate_sema::pkg::lock::LockFile;
use litigate_sema::temporal::MatterEvent;
use litigate_verify::{Finding, VerifiedCase};
use serde::Serialize;

/// An obligation instance the events created (SPEC §5.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Scheduled {
    /// Qualified deadline id.
    pub deadline: String,
    /// The triggering event.
    pub trigger: String,
    /// Its date.
    pub trigger_date: String,
    /// Due date; `None` when it is unknown (`unknown` says why): a
    /// designated date the notice does not give, or one that cannot be
    /// counted. Never a guess in its place.
    pub due: Option<String>,
    /// Why the due date is unknown: with `status` `unknown`, for a
    /// designated date the trigger leaves `?`, the reason names the
    /// trigger ("notice due date not recorded: ...").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown: Option<String>,
    /// The date the trigger designates, before any roll (`designated p`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated: Option<String>,
    /// Counting rule; empty when the due date is unknown.
    #[serde(skip_serializing_if = "String::is_empty")]
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
    /// The party bearing it, when the deadline names a bearer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    /// The due date it may not end before: the `later_of` instance it
    /// absorbed (FRCP 15(a)(3), FRAP 4(a)(3)); `due` is the counted date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// The due date a cited `extends` event moved it to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_to: Option<String>,
    /// The cause standard of the grant that licensed the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_under: Option<hir::CauseStandard>,
    /// When the request the licensing grant read was made, distinct from
    /// the order's date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_made: Option<String>,
    /// The filed request awaiting the order a grant of its deadline
    /// licenses (a stipulation, a motion to extend): its pending move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<Pending>,
    /// How replay ended it, or `open` at `as_of`; `unknown`, never
    /// `open` or `lapsed`, when the trigger left its designated due date
    /// unknown.
    pub status: Status,
    /// Source index of the trigger event in the matter's `events`.
    #[serde(skip)]
    pub trigger_event: usize,
    /// Source index of the event that ended it, if one did.
    #[serde(skip)]
    pub closed_by: Option<usize>,
}

/// Dates a matter is checked at instead of its own `as_of` / `known_at`.
#[derive(Debug, Clone, Copy, Default)]
pub struct At<'a> {
    /// Replaces the matter's `as_of`.
    pub as_of: Option<&'a str>,
    /// Replaces the matter's `known_at`.
    pub known_at: Option<&'a str>,
}

/// A checked matter.
#[derive(Debug, Clone, PartialEq)]
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
    /// The marking at `as_of` (SPEC §5.6): the node of each concurrent
    /// track, the replayed `start` first; empty when replay did not reach
    /// `as_of`.
    pub marking: Vec<String>,
    /// Deadlines the events triggered, with their due dates (empty unless
    /// the matter lowers).
    pub schedule: Vec<Scheduled>,
    /// Every deadline the events triggered, whether or not the clock
    /// counts it and whether or not the matter has errors.
    pub counted: Vec<Scheduled>,
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

/// The matter's lock, or the L1007 for one that does not parse.
fn lock(case: &CaseText) -> Result<Option<LockFile>, Diagnostic> {
    let Some(text) = case.lock.as_deref() else {
        return Ok(None);
    };
    LockFile::parse(text)
        .map(Some)
        .map_err(|r| r.to_diagnostic(&FileId(lock_path(&case.path)), &LineIndex::new(text)))
}

/// Gives the L1007 of `check`'s lock, when it is in an older format, the
/// fix that upgrades it: the lock `litigate lock DIR` writes for the
/// session's matters in that directory. `None` when there is nothing to
/// upgrade.
pub fn upgrade_lock(session: &Session, check: &mut CaseCheck) -> Option<()> {
    let file = lock_path(&check.path);
    let dir = file.strip_suffix("/Lit.lock").unwrap_or(".");
    let upgraded = || {
        session
            .matter_lock(session.cases.iter().filter(|c| lock_path(&c.path) == file))
            .render()
    };
    let text = session
        .cases
        .iter()
        .find(|c| c.path == check.path)
        .and_then(|c| c.lock.as_deref())?;
    let report = litigate_sema::pkg::lock::upgrade(text, dir, upgraded)?;
    let fixed = report.to_diagnostic(&FileId(file.clone()), &LineIndex::new(text));
    for d in check
        .diagnostics
        .iter_mut()
        // A matter check reports only the lock's own L1007 in the lock.
        .filter(|d| d.span.file == fixed.span.file)
    {
        let (old, new) = (report::example(d), report::example(&fixed));
        check
            .findings
            .iter_mut()
            .flat_map(|f| &mut f.examples)
            .filter(|e| **e == old)
            .for_each(|e| e.clone_from(&new));
        *d = Diagnostic {
            severity: d.severity,
            ..fixed.clone()
        };
    }
    Some(())
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

/// Checks and verifies `case` under `env` in a world of `layers`, at
/// `at`'s dates where given.
fn verify<'a>(case: &CaseText, env: Env<'a>, layers: &[&'a hir::Package], at: At) -> VerifiedCase {
    if at.as_of.is_none() && at.known_at.is_none() && layers.is_empty() {
        return litigate_verify::verify_case_source(&case.path, &case.text, env);
    }
    let resolved = litigate_sema::case::resolve(&case.path, &case.text, env.packages, env.lock);
    match resolved.case {
        Some(mut hcase) => {
            if let Some(as_of) = at.as_of {
                as_of.clone_into(&mut hcase.as_of);
            }
            if let Some(known_at) = at.known_at {
                hcase.known_at = Some(known_at.to_owned());
            }
            let mut verified =
                litigate_verify::verify_case_under(hcase, &resolved.spans, env, layers);
            verified.checked.diagnostics.extend(resolved.diagnostics);
            verified
        }
        // It does not resolve: the source path reports why.
        None => litigate_verify::verify_case_source(&case.path, &case.text, env),
    }
}

/// Checks one matter against the packages it sees: the closure of what
/// it `uses` ([`Session::scope`]). Any other loaded package, checked or
/// not, cannot change its diagnostics, replay or schedule.
#[must_use]
pub fn check(session: &Session, case: &CaseText, clocks: &Clocks, profile: Profile) -> CaseCheck {
    check_at(session, case, clocks, profile, At::default())
}

/// [`check`] with the matter's dates replaced by `at`'s.
#[must_use]
pub fn check_at(
    session: &Session,
    case: &CaseText,
    clocks: &Clocks,
    profile: Profile,
    at: At,
) -> CaseCheck {
    let all = session.hir_of(&session.scope(case));
    check_packages(&all, case, clocks, profile, at)
}

/// Checks a matter against an already scoped package set.
#[must_use]
pub fn check_packages(
    all: &[&hir::Package],
    case: &CaseText,
    clocks: &Clocks,
    profile: Profile,
    at: At,
) -> CaseCheck {
    check_under(all, &[], case, clocks, profile, at)
}

/// [`check_packages`] in a world (docs/litigate/CHANGE.md §5) whose
/// `layers` in force, among `all`, join the law the matter's packages see:
/// replay judges its events, and the schedule counts its deadlines, under
/// the versions the layers make govern.
#[must_use]
pub fn check_under<'a>(
    all: &[&'a hir::Package],
    layers: &[&'a hir::Package],
    case: &CaseText,
    clocks: &Clocks,
    profile: Profile,
    at: At,
) -> CaseCheck {
    let engine = crate::cmd::check::engine(all);
    // CLI-side findings: an unreadable lock, and lowering errors.
    let (lock, mut own) = match lock(case) {
        Ok(lock) => (lock, Vec::new()),
        Err(d) => (None, vec![d]),
    };
    let env = Env {
        packages: all,
        lock: lock.as_ref(),
        clock: &engine,
        profile,
    };
    let verified = verify(case, env, layers, at);
    let mut diagnostics = verified.diagnostics();
    let mut checked_case = verified.checked.case;
    if let Some(c) = checked_case
        .as_mut()
        .filter(|c| c.start == default_start(all, c) && !asserts_start(&case.text))
    {
        // No start is asserted and the events (none, or only option
        // removals) leave the matter at the engine's default start, the
        // primary package's: it lowers as no `start` at all.
        c.start = None;
    }
    let clean = checked_case
        .as_ref()
        .filter(|_| !own.iter().chain(&diagnostics).any(is_error));
    let lowered = clean.and_then(|c| match Lowerer::new(all).case(c, &case.path) {
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
    // Every instance replay created is counted, errors or not (a late act
    // is an error, and its deadline still has a due date); the schedule is
    // that of a matter that lowers.
    let counted = match checked_case.as_ref() {
        Some(c) => schedule(&used, layers, clocks, c, &verified.checked.obligations),
        None => Vec::new(),
    };
    let schedule = if lowered.is_some() {
        counted.clone()
    } else {
        Vec::new()
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
        marking: verified.checked.marking,
        schedule,
        counted,
    }
}

/// Every obligation instance replay created (SPEC §5.6), in creation
/// order, counted from its own trigger under the declaration governing it
/// then among the used packages and the world's `layers` (`clock::due`:
/// precedence and transitions applied).
fn schedule(
    packages: &[&hir::Package],
    layers: &[&hir::Package],
    clocks: &Clocks,
    case: &hir::Case,
    obligations: &[Obligation],
) -> Vec<Scheduled> {
    let names: Vec<&str> = packages
        .iter()
        .chain(layers)
        .map(|p| p.manifest.name.as_str())
        .collect();
    let world = clocks.scope(&names);
    let mut out: Vec<Scheduled> = Vec::new();
    for pkg in packages {
        let prefix = format!("{}::", pkg.manifest.pack_id);
        for o in obligations
            .iter()
            .filter(|o| o.deadline.starts_with(&prefix))
        {
            let trigger = &case.events[o.trigger];
            let local = &o.deadline[prefix.len()..];
            let event = MatterEvent::on(&trigger.date, &trigger.action);
            let due = clock::due(
                clocks,
                &world,
                pkg,
                local,
                std::slice::from_ref(&event),
                0,
                case.known_at.as_deref(),
            );
            let mut scheduled = Scheduled {
                deadline: o.deadline.clone(),
                trigger: hirx::event_text(&trigger.action),
                trigger_date: trigger.date.clone(),
                due: None,
                unknown: None,
                designated: None,
                rule: String::new(),
                steps: Vec::new(),
                version: String::new(),
                cite: None,
                rolled: false,
                method: None,
                bearer: o.bearer.clone(),
                floor: o.floor.clone(),
                extended_to: o.extended_to.clone(),
                extended_under: o.extended_under,
                request_made: o.request_made.clone(),
                pending: o.pending.clone(),
                status: o.status,
                trigger_event: o.trigger,
                closed_by: o.closed_by,
            };
            match due {
                Ok(due) => {
                    scheduled.due = Some(due.computed.due_date);
                    scheduled.designated = due.designated;
                    scheduled.rule = due.rule;
                    scheduled.steps = due.computed.steps;
                    scheduled.version = due.version;
                    scheduled.cite = due.cite;
                    scheduled.rolled = due.computed.last_day_rolled;
                    // A method the rule recognizes for no days added none.
                    scheduled.method = due.method.filter(|_| due.computed.service_days_added);
                }
                Err(why) => scheduled.unknown = Some(why),
            }
            out.push(scheduled);
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
        check(
            &session,
            &session.cases[0],
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
            "analysis { params: { rate: 650.usd / 1.hours, opp_rate: 550.usd / 1.hours } }",
            "analysis { frobnicate: 1 }",
        );
        let c = checked("", &text, None);
        assert_eq!(errors(&c), [("L4007", "cases/examples/acme.case")]);
        assert!(c.lowered.is_none());
        assert!(c.schedule.is_empty());
    }

    /// `matter::check_at` of the worked example with matter `text` at `at`.
    fn checked_at(text: &str, at: At) -> CaseCheck {
        let session = testkit::example_locked("", text, None);
        let all = session.hir();
        check_at(
            &session,
            &session.cases[0],
            &Clocks::new(&all),
            Profile::Dev,
            at,
        )
    }

    #[test]
    fn dates_replace_the_matter_s_own() {
        let c = checked_at(
            ACME,
            At {
                as_of: None,
                known_at: Some("2025-01-01"),
            },
        );
        let case = c.case.unwrap();
        assert_eq!(
            (case.as_of.as_str(), case.known_at.as_deref()),
            ("2025-06-01", Some("2025-01-01"))
        );
        let c = checked_at(
            ACME,
            At {
                as_of: Some("2025-07-01"),
                known_at: None,
            },
        );
        let case = c.case.unwrap();
        assert_eq!(
            (case.as_of.as_str(), case.known_at.as_deref()),
            ("2025-07-01", None)
        );
        // A matter that does not resolve reports why, at any date.
        let c = checked_at(
            "case broken\n",
            At {
                as_of: Some("2025-06-01"),
                known_at: None,
            },
        );
        assert!(c.case.is_none());
        assert!(!errors(&c).is_empty());
    }

    #[test]
    fn a_deadline_the_engine_cannot_count_is_scheduled_unknown() {
        let c = checked(
            "deadline in-business-days: 3.business_days from Served(complaint) counted_by frcp6",
            ACME,
            None,
        );
        let scheduled: Vec<(&str, Option<&str>, bool)> = c
            .schedule
            .iter()
            .map(|s| (s.deadline.as_str(), s.due.as_deref(), s.unknown.is_some()))
            .collect();
        assert_eq!(
            scheduled,
            [
                (
                    "frcp-rule12-example::answer-period",
                    Some("2025-03-26"),
                    false
                ),
                ("frcp-rule12-example::in-business-days", None, true),
            ]
        );
    }
}
