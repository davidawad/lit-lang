//! `litigate lint-filing CASE EVENT --date DATE --by PARTY` (DESIGN §10,
//! the filing linter): replay the matter with a proposed filing added and
//! report what the engine would make of it, before it is filed. The
//! filing is an error when replay rejects it: not a legal move from the
//! posture on that date (L4001), or past the due date of the obligation
//! it discharges (L4002). Otherwise the result lists the obligations it
//! discharges and the deadlines it triggers, with their due dates.
//!
//! The matter itself must check clean first. Diagnostics are then every
//! error of the matter with the filing, and the warnings the filing adds
//! to the matter as of the filing date. The filing's own line is written
//! here, not by the user. Its id is taken from the same L9006 fix that
//! `litigate fix` would apply, keeping its timeline key stable.
use super::check::check_session;
use crate::filing;
use crate::ics;
use crate::matter::{CaseCheck, Scheduled};
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litgraph::clock::Date;
use litigate_base::{Diagnostic, DiagnosticCode, Profile, Severity};
use litigate_lower::Options;
use litigate_sema::case::{Status, Step};
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

/// Whether `known` already has a diagnostic like `d` (spans move with
/// the inserted line, so they are not compared).
fn known(known: &[Diagnostic], d: &Diagnostic) -> bool {
    let like = |x: &Diagnostic| (x.code, x.severity, x.message.clone());
    known.iter().any(|k| like(k) == like(d))
}

/// Every error of the matter with the filing (`filed`), and the warnings
/// it adds to the matter without it (`at`).
fn added(at: &CaseCheck, filed: &CaseCheck) -> Vec<Diagnostic> {
    filed
        .diagnostics
        .iter()
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

/// Add the machine-generated event id to a proposed filing, reusing the
/// canonical L9006 fix and its collision checks.
fn generated_filing_line(
    mut session: Session,
    original: &CaseText,
    at: &CaseText,
    date: &str,
    line: &str,
) -> Result<(Session, String), Failure> {
    let candidate = CaseText {
        text: filing::splice(&at.text, date, line),
        ..original.clone()
    };
    added_line(&at.text, &candidate.text, line)?;
    session.cases = vec![original.clone(), at.clone(), candidate];
    let checked = check_session(session, Profile::Dev, &Options::default());
    let candidate_text = &checked.session.cases[2].text;
    let filing_line = filing_line_number(candidate_text, line)?;
    let candidate_case = &checked.cases[2];
    let generated = generated_id(&candidate_case.diagnostics, filing_line, date)?;
    Ok((checked.session, format!("{generated}: {line}")))
}

/// What the filing changed.
struct Changes<'a> {
    replay: Vec<&'a Step>,
    discharges: Vec<Value>,
    triggers: Vec<Value>,
    open_before: Vec<Value>,
}

/// The replay steps, obligation endings and new obligations of `filed`
/// that the matter without the filing (`at`) lacks, and what `at` had
/// open.
fn changes<'a>(at: &'a CaseCheck, filed: &'a CaseCheck) -> Changes<'a> {
    Changes {
        replay: filed
            .replay
            .iter()
            .filter(|s| !at.replay.contains(s))
            .collect(),
        discharges: filed
            .schedule
            .iter()
            .filter_map(|s| {
                at.schedule
                    .iter()
                    .find(|o| key(o) == key(s) && o.status != s.status)
                    .map(|o| {
                        let mut e = entry(s);
                        e["before"] = Value::from(ics::status_name(o));
                        e
                    })
            })
            .collect(),
        triggers: filed
            .schedule
            .iter()
            .filter(|s| !at.schedule.iter().any(|o| key(o) == key(s)))
            .map(entry)
            .collect(),
        open_before: at
            .schedule
            .iter()
            .filter(|s| s.status == Status::Open)
            .map(entry)
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
    let (mut session, line) = generated_filing_line(session, &original, &at, &date, &line)?;
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
    let changes = changes(at, filed);
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
    fn filing_validation_reports_non_events_and_missing_locations() {
        assert!(added_line("events {\n}\n", "events {\n}\n", "2025-01-01 Hearing()").is_err());
        assert_eq!(filing_line_number("events {\n  event\n}\n", "event"), Ok(2));
        assert!(filing_line_number("events {\n}\n", "missing").is_err());
    }

    #[test]
    fn no_prior_diagnostic_is_not_a_duplicate() {
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
        assert!(!known(&[], &diagnostic));
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

        let error = generated_filing_line(session, &original, &at, date, &proposed)
            .map_err(|error| error.0)
            .err()
            .expect("an explicit id has no L9006 fix to reuse");

        assert_eq!(
            error,
            "could not generate a stable id for the proposed filing"
        );
    }
}
