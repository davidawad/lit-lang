//! `litigate lint-filing CASE EVENT --date DATE --by PARTY` (DESIGN §10,
//! the filing linter): replay the matter with a proposed filing added and
//! report what the engine would make of it, before it is filed. The
//! filing is an error when replay rejects it: not a legal move from the
//! marking on that date, a move on none of the matter's concurrent tracks
//! (L4001, SPEC §5.6), or past the due date of the obligation it
//! discharges (L4002). Otherwise the result lists the obligations it
//! discharges and the deadlines it triggers, with their due dates.
//!
//! Each obligation open before the filing carries its `moves`, the
//! actions that protect it (SPEC §9.3): the acts out of any track of the
//! matter's marking that discharge it (`file`), the extension grants of the
//! version counting it, each with the request it rests on (`extend`),
//! and the order events that exercise them (`orders`). Proposing one of
//! them through `lint-filing` judges it.
//!
//! The matter itself must check clean first, but for the missing
//! lowering (L6001) of a package with concurrent tracks: replay, not the
//! build, judges a filing. Diagnostics are then every
//! error of the matter with the filing, and the warnings the filing adds
//! to the matter as of the filing date. The filing's own line is written
//! here, not by the user. Its id is taken from the same L9006 fix that
//! `litigate fix` would apply, keeping its timeline key stable.
use super::check::check_session;
use crate::filing;
use crate::hirx;
use crate::ics;
use crate::matter::{CaseCheck, Scheduled};
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Diagnostic, DiagnosticCode, Profile, Severity};
use litigate_engine::clock::Date;
use litigate_lower::Options;
use litigate_sema::case::{Status, Step};
use litigate_sema::hir::{self, DeadlineWindow, Declaration, EventEffect, Extension};
use serde_json::{json, Value};

/// What `lint-filing` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` file.
    pub case: String,
    /// The event as a `.case` writes it (`Answered(complaint)`).
    pub event: String,
    /// The filing date.
    pub date: String,
    /// `by` (the party acts) or `on` (the party is served).
    pub mode: &'static str,
    /// The party.
    pub party: String,
    /// Library directories.
    pub lib: Vec<String>,
}

/// `(deadline, trigger, trigger date)`: one obligation instance.
fn key(s: &Scheduled) -> (&str, &str, &str) {
    (&s.deadline, &s.trigger, &s.trigger_date)
}

/// An obligation instance with its due date, or why it has none.
fn entry(s: &Scheduled) -> Value {
    let due = ics::due_of(s);
    json!({
        "deadline": s.deadline, "trigger": s.trigger, "trigger_date": s.trigger_date,
        "due": due.clone().ok().map(Date::to_iso), "unknown": due.err(),
        "status": ics::status_name(s),
    })
}

/// One `extend` grant as a move: who exercises it, under what cause
/// standard, in which window (`before_expiry`: on or before the current
/// due date, judged on the request's date when it names one;
/// `after_expiry`: after it, on motion; `any`), the request it rests on
/// (`on M()`), and a bounded grant's ceiling; `None` for a lock.
fn grant(e: &Extension) -> Option<Value> {
    let (by, standard, window, motion, until) = match e {
        Extension::Bounded {
            by,
            until,
            after_expiry,
            standard,
            motion,
            ..
        } => (
            by,
            *standard,
            if *after_expiry {
                "any"
            } else {
                "before_expiry"
            },
            motion,
            serde_json::to_value(until).ok(),
        ),
        Extension::Discretionary {
            by,
            standard,
            window,
            motion,
            ..
        } => (
            by,
            Some(*standard),
            match window {
                hir::ExtensionWindow::BeforeExpiry => "before_expiry",
                hir::ExtensionWindow::AfterExpiryOnMotion => "after_expiry",
            },
            motion,
            None,
        ),
        Extension::NonExtendable { .. } => return None,
    };
    Some(json!({
        "by": by, "standard": standard, "window": window,
        "request": motion.as_ref().map(|m| hirx::event_text(&m.event)), "until": until,
    }))
}

/// The version of `s`'s deadline that counted it, among `law`.
fn counted_by<'p>(law: &[&'p hir::Package], s: &Scheduled) -> Option<&'p hir::Deadline> {
    law.iter()
        .filter(|p| s.deadline.starts_with(&format!("{}::", p.manifest.pack_id)))
        .flat_map(|p| hirx::deadlines(p, &s.deadline))
        .find(|d| format!("{}@{}", d.meta.id, d.meta.effective.from) == s.version)
}

/// The events of `law` that extend `deadline` (`extends d to p`).
fn orders(law: &[&hir::Package], deadline: &str) -> Vec<String> {
    law.iter()
        .flat_map(|p| hirx::declarations(p))
        .filter_map(|d| match d {
            Declaration::Event(hir::EventType {
                name,
                effect: Some(EventEffect::Extends { deadline: d, .. }),
                ..
            }) if hirx::local(d) == hirx::local(deadline) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The node versions of `case`'s replayed `marking` in force on its
/// `as_of`, one per concurrent track (SPEC §5.6), each with its pack id.
fn posture<'p>(
    law: &[&'p hir::Package],
    case: &hir::Case,
    marking: &[String],
) -> Vec<(&'p str, &'p hir::Node)> {
    marking
        .iter()
        .filter_map(|m| m.split_once("::"))
        .flat_map(|(pack, node)| {
            law.iter()
                .filter(move |p| p.manifest.pack_id == pack)
                .flat_map(|p| {
                    hirx::nodes_at(law, p, &case.as_of, case.known_at.as_deref())
                        .into_iter()
                        .map(|n| (p.manifest.pack_id.as_str(), n))
                })
                .filter(move |(_, n)| n.meta.id == node)
        })
        .collect()
}

/// The moves that protect `s`: the acts out of the marking (`nodes`)
/// that discharge it, the grants that extend it, and their orders.
fn moves(law: &[&hir::Package], nodes: &[(&str, &hir::Node)], s: &Scheduled) -> Value {
    let file: Vec<Value> = nodes
        .iter()
        .flat_map(|(pack, n)| n.edges.iter().map(move |e| (pack, e)))
        .filter(|(_, e)| {
            matches!(&e.deadline, Some(hir::EdgeDeadline {
                window: DeadlineWindow::Declared { deadline }, ..
            }) if hirx::local(deadline) == hirx::local(&s.deadline))
        })
        .map(|(pack, e)| {
            json!({
                "edge": e.id.as_ref().map(|id| format!("{pack}::{id}")),
                "event": e.event.as_ref().map(hirx::event_text),
            })
        })
        .collect();
    let extend: Vec<Value> = counted_by(law, s)
        .into_iter()
        .flat_map(|d| d.extend.iter().filter_map(grant))
        .collect();
    json!({"file": file, "extend": extend, "orders": orders(law, &s.deadline)})
}

/// Whether `d` bears on the filing. A package with concurrent tracks has
/// no lowering (L6001) but checks and replays, and replay is what judges
/// a filing, so that error neither blocks the lint nor is reported.
fn judged(d: &Diagnostic) -> bool {
    d.code != DiagnosticCode::LoweringUnsupported
}

/// The obligation instances of `c`: every instance replay counted when
/// nothing but a missing lowering is wrong with the matter, else its
/// schedule (empty unless it lowers).
fn scheduled(c: &CaseCheck) -> &[Scheduled] {
    if c.diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && judged(d))
    {
        &c.schedule
    } else {
        &c.counted
    }
}

/// Whether `known` already has a diagnostic like `d` (spans move with
/// the inserted line, so they are not compared).
fn known(known: &[Diagnostic], d: &Diagnostic) -> bool {
    let like = |x: &Diagnostic| (x.code, x.severity, x.message.clone());
    known.iter().any(|k| like(k) == like(d))
}

/// Every error of the matter with the filing (`filed`), and the warnings
/// it adds to the matter without it (`at`).
pub(super) fn added(at: &CaseCheck, filed: &CaseCheck) -> Vec<Diagnostic> {
    filed
        .diagnostics
        .iter()
        .filter(|d| judged(d))
        .filter(|d| match d.severity {
            Severity::Error => true,
            _ => !known(&at.diagnostics, d),
        })
        .cloned()
        .collect()
}

fn filing_line_number(text: &str, line: &str) -> Result<u32, Failure> {
    text.lines()
        .collect::<Vec<_>>()
        .into_iter()
        .rposition(|candidate_line| candidate_line.trim() == line)
        .and_then(|index| u32::try_from(index).ok())
        .and_then(|index| index.checked_add(1))
        .ok_or_else(|| Failure::new("could not locate the proposed filing"))
}

fn added_line(at: &str, filed: &str, line: &str) -> Result<(), Failure> {
    if filing::added(at, filed, line) {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "`{line}` is not one event of a .case `events` block"
        )))
    }
}

fn generated_id(
    diagnostics: &[Diagnostic],
    filing_line: u32,
    date: &str,
) -> Result<String, Failure> {
    diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UndeclaredEventId && d.span.line == filing_line)
        .flat_map(|d| d.fixes.iter())
        .flat_map(|f| f.edits.iter())
        .find_map(|e| {
            e.replacement
                .strip_suffix(date)
                .and_then(|s| s.strip_suffix(": "))
        })
        .map(str::to_owned)
        .ok_or_else(|| Failure::new("could not generate a stable id for the proposed filing"))
}

/// Add the machine-generated event id to a proposed filing (with docket
/// number `docket`, if any), reusing the canonical L9006 fix and its
/// collision checks.
pub(super) fn generated_filing_line(
    mut session: Session,
    original: &CaseText,
    at: &CaseText,
    (date, docket): (&str, Option<u64>),
    line: &str,
) -> Result<(Session, String), Failure> {
    let candidate = CaseText {
        text: filing::splice_docketed(&at.text, date, docket, line),
        ..original.clone()
    };
    added_line(&at.text, &candidate.text, line)?;
    session.cases = vec![original.clone(), at.clone(), candidate];
    let checked = check_session(session, Profile::Dev, &Options::default());
    let candidate_text = &checked.session.cases[2].text;
    let filing_line = filing_line_number(candidate_text, line)?;
    let candidate_case = &checked.cases[2];
    generated_id(&candidate_case.diagnostics, filing_line, date)
        .map(|generated| (checked.session, format!("{generated}: {line}")))
}

/// What the filing changed.
pub(super) struct Changes<'a> {
    pub(super) replay: Vec<&'a Step>,
    pub(super) discharges: Vec<Value>,
    pub(super) triggers: Vec<Value>,
    pub(super) open_before: Vec<Value>,
}

/// The replay steps, obligation endings and new obligations of `filed`
/// that the matter without the filing (`at`) lacks, and what `at` had
/// open, each with its moves under `law`.
pub(super) fn changes<'a>(
    at: &'a CaseCheck,
    filed: &'a CaseCheck,
    law: &[&hir::Package],
) -> Changes<'a> {
    let nodes = at
        .case
        .as_ref()
        .map(|case| posture(law, case, &at.marking))
        .unwrap_or_default();
    let (before, after) = (scheduled(at), scheduled(filed));
    Changes {
        replay: filed
            .replay
            .iter()
            .filter(|s| !at.replay.contains(s))
            .collect(),
        discharges: after
            .iter()
            .filter_map(|s| {
                before
                    .iter()
                    .find(|o| key(o) == key(s) && o.status != s.status)
                    .map(|o| {
                        let mut e = entry(s);
                        e["before"] = Value::from(ics::status_name(o));
                        e
                    })
            })
            .collect(),
        triggers: after
            .iter()
            .filter(|s| !before.iter().any(|o| key(o) == key(s)))
            .map(entry)
            .collect(),
        open_before: before
            .iter()
            .filter(|s| s.status == Status::Open)
            .map(|s| {
                let mut e = entry(s);
                e["moves"] = moves(law, &nodes, s);
                e
            })
            .collect(),
    }
}

/// `litigate lint-filing`.
pub fn lint_filing(args: &Args) -> Outcome {
    let date = Date::parse_iso(&args.date)
        .map_err(|e| Failure::new(format!("--date {}: {e}", args.date)))?
        .to_iso();
    let session = ws::discover(std::slice::from_ref(&args.case))
        .and_then(|targets| Session::load(&targets, &args.lib))?;
    let original = session
        .cases
        .first()
        .cloned()
        .ok_or_else(|| Failure::new(format!("{}: not a .case file", args.case)))?;
    let line = filing::line(&date, &args.event, args.mode, &args.party);
    let at = CaseText {
        text: filing::with_as_of(&original.text, &date),
        ..original.clone()
    };
    let (mut session, line) = generated_filing_line(session, &original, &at, (&date, None), &line)?;
    let filed = CaseText {
        text: filing::splice(&at.text, &date, &line),
        ..original.clone()
    };
    session.cases = vec![original, at, filed];
    let c = check_session(session, Profile::Dev, &Options::default());
    let texts: Vec<&CaseText> = c.session.cases.iter().collect();
    let (before, at, filed) = (&c.cases[0], &c.cases[1], &c.cases[2]);
    let files = c
        .session
        .packages
        .iter()
        .flat_map(|p| p.files.iter().cloned());
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: files
            .clone()
            .chain([(texts[0].path.clone(), texts[0].text.clone())])
            .collect(),
        diagnostics: c
            .verdicts
            .iter()
            .flat_map(|(_, v)| v.diagnostics.iter().cloned())
            .chain(before.diagnostics.iter().cloned())
            .filter(judged)
            .collect(),
        ..Report::default()
    };
    if report.has_errors() {
        return Ok(report);
    }
    // `generated_filing_line` already checked this event without an id;
    // the id added here is the L9006 machine-applicable fix from that same
    // candidate, so it cannot change the event's syntax or move it out of
    // the events block.
    report.sources = files
        .chain([(texts[2].path.clone(), texts[2].text.clone())])
        .collect();
    report.diagnostics = added(at, filed);
    let law = c.session.hir_of(&c.session.closure(at.packages.clone()));
    let changes = changes(at, filed, &law);
    report.result = json!({
        "case": filed.case.as_ref().map(|c| c.id.clone()),
        "filing": {"date": date, "event": args.event, "mode": args.mode, "party": args.party, "line": line},
        "as_of": filed.case.as_ref().map(|c| c.as_of.clone()),
        "accepted": !report.has_errors(),
        "replay": changes.replay, "discharges": changes.discharges,
        "triggers": changes.triggers, "open_before": changes.open_before,
    });
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    #[test]
    fn a_bounded_grant_is_a_move_with_its_ceiling() {
        let bounded = |after_expiry| Extension::Bounded {
            by: "justice".into(),
            until: hir::Bound::Offset {
                from: hir::DateRef::PrescribedDue,
                days: 60,
            },
            after_expiry,
            standard: None,
            motion: None,
            interpretation: None,
        };
        let discretionary = |window| Extension::Discretionary {
            by: "court".into(),
            standard: hir::CauseStandard::GoodCause,
            window,
            motion: None,
            interpretation: None,
        };
        let windows: Vec<Option<Value>> = [
            bounded(false),
            bounded(true),
            discretionary(hir::ExtensionWindow::BeforeExpiry),
            discretionary(hir::ExtensionWindow::AfterExpiryOnMotion),
            Extension::NonExtendable {
                authority: "FRCP 6(b)(2)".into(),
                against: Vec::new(),
            },
        ]
        .iter()
        .map(|e| grant(e).map(|g| g["window"].clone()))
        .collect();
        assert_eq!(
            windows,
            [
                Some(json!("before_expiry")),
                Some(json!("any")),
                Some(json!("before_expiry")),
                Some(json!("after_expiry")),
                None,
            ]
        );
        assert_eq!(
            grant(&bounded(false)),
            Some(json!({
                "by": "justice", "standard": null, "window": "before_expiry", "request": null,
                "until": {"offset": {"from": "prescribed_due", "days": 60}},
            }))
        );
    }

    #[test]
    fn filing_validation_reports_non_events_and_missing_locations() {
        assert!(added_line("events {\n}\n", "events {\n}\n", "2025-01-01 Hearing()").is_err());
        assert_eq!(filing_line_number("events {\n  event\n}\n", "event"), Ok(2));
        assert!(filing_line_number("events {\n}\n", "missing").is_err());
    }

    #[test]
    fn no_prior_diagnostic_is_not_a_duplicate() -> Result<(), &'static str> {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UndeclaredEventId,
            litigate_base::Span {
                file: "case.case".into(),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 2,
            },
            "missing id",
        );
        (!known(&[], &diagnostic))
            .then_some(())
            .ok_or("a diagnostic with no prior entries is not a duplicate")
    }

    #[test]
    fn missing_event_id_fix_is_reported() {
        assert!(generated_id(&[], 1, "2025-01-01").is_err());
        let span = litigate_base::Span {
            file: "case.case".into(),
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 2,
        };
        for replacement in ["id: wrong", "prefix2025-01-01"] {
            let diagnostic = Diagnostic::error(
                DiagnosticCode::UndeclaredEventId,
                span.clone(),
                "missing id",
            )
            .fix(
                span.clone(),
                replacement,
                litigate_base::Applicability::MachineApplicable,
            );
            assert!(generated_id(&[diagnostic], 1, "2025-01-01").is_err());
        }
    }

    #[test]
    fn a_candidate_with_an_explicit_id_has_no_generated_id_fix() {
        let session = testkit::example_locked("", testkit::ACME, None);
        let original = session.cases[0].clone();
        let date = "2025-06-02";
        let at = CaseText {
            text: crate::filing::with_as_of(&original.text, date),
            ..original.clone()
        };
        let proposed = format!("proposal: {date} Hearing() by widget");

        let error = generated_filing_line(session, &original, &at, (date, None), &proposed)
            .map_err(|error| error.0)
            .err()
            .expect("an explicit id has no L9006 fix to reuse");

        assert_eq!(
            error,
            "could not generate a stable id for the proposed filing"
        );
    }

    #[test]
    fn a_proposed_filing_gets_a_generated_id_or_a_failure() {
        let session = || testkit::example_locked("", testkit::ACME, None);
        let original = session().cases[0].clone();
        let date = "2025-06-02";
        let at = CaseText {
            text: crate::filing::with_as_of(&original.text, date),
            ..original.clone()
        };
        let proposed = format!("{date} Hearing() by widget");
        let line = generated_filing_line(session(), &original, &at, (date, None), &proposed)
            .map(|(_, line)| line)
            .ok();
        assert!(
            line.as_deref()
                .is_some_and(|line| line.ends_with(&format!(": {proposed}"))),
            "{line:?}"
        );
        let multiline = format!("{date} Hearing(\n) by widget");
        for (proposed, expected) in [
            ("not an event", "is not one event"),
            (multiline.as_str(), "could not locate the proposed filing"),
        ] {
            let error = generated_filing_line(session(), &original, &at, (date, None), proposed)
                .err()
                .map(|error| error.0)
                .unwrap_or_default();
            assert!(error.contains(expected), "{proposed}: {error}");
        }
    }
}
