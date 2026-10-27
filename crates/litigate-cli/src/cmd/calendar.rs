//! `litigate calendar CASE` (DESIGN §10): the matter's computed deadlines
//! as an iCalendar file. Only a matter that checks clean has a calendar;
//! a deadline whose due date is unknown is listed under `unscheduled`
//! with the reason, never given a date.
use super::check::checked_in;
use crate::hirx;
use crate::hirx::rule_of;
use crate::ics::{self, Calendar, Row};
use crate::matter::Scheduled;
use crate::out::{Failure, Outcome, Report};
use litigate_base::Profile;
use litigate_lower::Options;
use litigate_sema::case::Status;
use litigate_sema::hir;
use litigate_sema::pkg::source::Source;
use serde_json::json;
use std::path::Path;

/// What `calendar` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` file.
    pub case: String,
    /// Where to write the `.ics` file, if anywhere.
    pub out: Option<String>,
    /// Every deadline, not only the open ones.
    pub all: bool,
    /// Days before each due date to remind.
    pub remind: Vec<u32>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// The cite of the deadline version that counted `s` (which may be a
/// claimant displacing its deadline), else of the rule declaring it.
fn authority<'p>(all: &[&'p hir::Package], s: &Scheduled) -> Option<&'p str> {
    all.iter()
        .find_map(|p| {
            hirx::declarations(p).find_map(|d| match d {
                hir::Declaration::Deadline(v)
                    if format!("{}@{}", v.meta.id, v.meta.effective.from) == s.version =>
                {
                    Some((*p, v))
                }
                _ => None,
            })
        })
        .and_then(|(pkg, v)| {
            v.meta
                .cite
                .as_deref()
                .or_else(|| rule_of(pkg, &v.meta.id).and_then(|r| r.meta.cite.as_deref()))
        })
}

/// `litigate calendar`, reading (and writing `--out` into) `src`.
pub fn calendar(src: &dyn Source, args: &Args) -> Outcome {
    let c = checked_in(
        src,
        std::slice::from_ref(&args.case),
        Profile::Dev,
        &Options::default(),
        &args.lib,
    )?;
    let matter = c
        .cases
        .first()
        .ok_or_else(|| Failure::new(format!("{}: not a .case file", args.case)))?;
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        ..Report::default()
    };
    let Some(hcase) = matter.case.as_ref().filter(|_| !report.has_errors()) else {
        return Ok(report);
    };
    let wanted: Vec<&Scheduled> = matter
        .schedule
        .iter()
        .filter(|s| args.all || s.status == Status::Open)
        .collect();
    let (rows, unscheduled): (Vec<_>, Vec<_>) = wanted
        .iter()
        .map(|s| ics::due_of(s).map(|due| (due, *s)).map_err(|why| (*s, why)))
        .partition(Result::is_ok);
    let all = c.session.hir();
    let rows: Vec<Row> = rows
        .into_iter()
        .filter_map(Result::ok)
        .map(|(due, s)| Row {
            due,
            scheduled: s,
            authority: authority(&all, s),
        })
        .collect();
    let unscheduled: Vec<_> = unscheduled
        .into_iter()
        .filter_map(Result::err)
        .map(|(s, why)| {
            json!({
                "deadline": s.deadline, "trigger": s.trigger,
                "trigger_date": s.trigger_date, "status": ics::status_name(s),
                "reason": why,
            })
        })
        .collect();
    let (text, events) = ics::render(
        &Calendar {
            case: &hcase.id,
            path: &matter.path,
            as_of: &hcase.as_of,
            remind: &args.remind,
        },
        &rows,
    );
    if let Some(out) = &args.out {
        src.write_atomic(Path::new(out), &text)?;
    }
    report.result = json!({
        "case": hcase.id, "as_of": hcase.as_of, "written": args.out,
        "events": events, "unscheduled": unscheduled, "ics": text,
    });
    Ok(report)
}
