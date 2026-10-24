//! iCalendar (RFC 5545) rendering of a matter's deadline schedule
//! (DESIGN §10): one all-day `VEVENT` per deadline whose due date the
//! engine computed. A deadline whose due date is unknown never gets a
//! date; the caller reports it instead.
//!
//! The output is a pure function of its input: the `UID` hashes what
//! caused the deadline (case, deadline, trigger, trigger date), so a
//! re-export updates the same calendar entry; `DTSTAMP` is the matter's
//! `as_of`, never the wall clock.
use crate::matter::Scheduled;
use litgraph::clock::Date;
use sha2::{Digest, Sha256};
use std::fmt::Write;

/// RFC 5545 §3.1: content lines fold at 75 octets.
const LINE_OCTETS: usize = 75;

/// What one calendar is rendered from.
#[derive(Debug, Clone)]
pub struct Calendar<'a> {
    /// The matter's id (`case <id>`).
    pub case: &'a str,
    /// Its repository-relative path.
    pub path: &'a str,
    /// The date statuses are judged at, `YYYY-MM-DD` (a checked matter's).
    pub as_of: &'a str,
    /// Days before the due date to remind (`VALARM`), in order.
    pub remind: &'a [u32],
}

/// One deadline to put on the calendar.
#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    /// The date it is due ([`due_of`]).
    pub due: Date,
    /// The obligation instance.
    pub scheduled: &'a Scheduled,
    /// The cite of the deadline itself (else of the rule declaring it).
    pub authority: Option<&'a str>,
}

/// One rendered event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Event {
    /// Its `UID`.
    pub uid: String,
    /// The qualified deadline id.
    pub deadline: String,
    /// The date it is due (an extension's date when one moved it).
    pub due: String,
    /// Its status at `as_of`.
    pub status: String,
    /// The triggering event and its date.
    pub trigger: String,
    /// The trigger's date.
    pub trigger_date: String,
    /// The cite of the deadline (else of the rule declaring it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// The counting rule.
    pub counting: String,
    /// The cite of the counting rule's version that counted it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counting_cite: Option<String>,
}

/// The status name `s` serializes to (`open`, `lapsed`, ...).
#[must_use]
pub fn status_name(s: &Scheduled) -> String {
    serde_json::to_value(s.status)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// The date `s` is due on: the date an extension moved it to, else the
/// counted one; else why it has none (never a guess in its place).
///
/// # Errors
/// The due date is unknown, or not a date.
pub fn due_of(s: &Scheduled) -> Result<Date, String> {
    let Some(due) = s.extended_to.as_deref().or(s.due.as_deref()) else {
        return Err(s
            .unknown
            .clone()
            .unwrap_or_else(|| "the due date is unknown".to_owned()));
    };
    Date::parse_iso(due).map_err(|e| format!("due date `{due}` is not a date: {e}"))
}

/// RFC 5545 §3.3.11 `TEXT` escaping.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// `line` folded at 75 octets (RFC 5545 §3.1), never inside a UTF-8
/// character; every physical line ends in CRLF.
#[must_use]
pub fn fold(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 8);
    let mut width = 0;
    for c in line.chars() {
        if width + c.len_utf8() > LINE_OCTETS {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(c);
        width += c.len_utf8();
    }
    out.push_str("\r\n");
    out
}

/// `day` as an iCalendar `DATE` (`YYYYMMDD`).
fn compact(day: Date) -> String {
    day.to_iso().replace('-', "")
}

/// The stable `UID` of the deadline `s` creates in matter `case`.
#[must_use]
pub fn uid(case: &str, s: &Scheduled) -> String {
    let key = format!("{case}\n{}\n{}\n{}", s.deadline, s.trigger, s.trigger_date);
    let digest = Sha256::digest(key.as_bytes());
    let mut uid = digest[..16].iter().fold(String::new(), |mut hex, b| {
        let _ = write!(hex, "{b:02x}");
        hex
    });
    uid.push_str("@litigate");
    uid
}

/// The `DESCRIPTION` of `s`: what it is, where it comes from, how it was
/// counted, and that the cited text governs.
fn description(cal: &Calendar<'_>, row: &Row<'_>, status: &str) -> String {
    let s = row.scheduled;
    let mut lines = vec![format!("Deadline: {}", s.deadline)];
    if let Some(authority) = row.authority {
        lines.push(format!("Authority: {authority}"));
    }
    lines.push(format!("Trigger: {} on {}", s.trigger, s.trigger_date));
    if let Some(bearer) = &s.bearer {
        lines.push(format!("Bearer: {bearer}"));
    }
    if let (Some(to), Some(counted)) = (&s.extended_to, &s.due) {
        lines.push(format!("Extended to {to} (counted due date {counted})"));
    }
    if let Some(floor) = &s.floor {
        lines.push(format!("Not before: {floor}"));
    }
    lines.push(match &s.cite {
        Some(cite) => format!("Counting: {} ({cite})", s.rule),
        None => format!("Counting: {}", s.rule),
    });
    lines.extend(s.steps.iter().map(|step| format!("- {step}")));
    lines.push(format!("Status as of {}: {status}", cal.as_of));
    lines.push(format!(
        "Computed by litigate from {}; the cited rule text governs.",
        cal.path
    ));
    lines.join("\n")
}

/// The content lines of one `VEVENT` for `row`: an all-day event, so
/// `DTEND` is the exclusive day after the due date.
fn vevent(cal: &Calendar<'_>, row: &Row<'_>) -> Vec<String> {
    let (s, due) = (row.scheduled, row.due);
    let status = status_name(s);
    let local = s.deadline.rsplit("::").next().unwrap_or(&s.deadline);
    let summary = match status.as_str() {
        "open" => format!("{local} due ({})", cal.case),
        other => format!("[{other}] {local} due ({})", cal.case),
    };
    let mut lines = vec![
        "BEGIN:VEVENT".to_owned(),
        format!("UID:{}", uid(cal.case, s)),
        format!("DTSTAMP:{}T000000Z", cal.as_of.replace('-', "")),
        format!("DTSTART;VALUE=DATE:{}", compact(due)),
        format!("DTEND;VALUE=DATE:{}", compact(due.add_days(1))),
        format!("SUMMARY:{}", escape(&summary)),
        format!("DESCRIPTION:{}", escape(&description(cal, row, &status))),
        format!("CATEGORIES:litigate,{}", escape(&status)),
        "TRANSP:TRANSPARENT".to_owned(),
    ];
    for days in cal.remind {
        lines.extend([
            "BEGIN:VALARM".to_owned(),
            "ACTION:DISPLAY".to_owned(),
            format!("TRIGGER;RELATED=START:-P{days}D"),
            format!(
                "DESCRIPTION:{}",
                escape(&format!("{local} due in {days} day(s)"))
            ),
            "END:VALARM".to_owned(),
        ]);
    }
    lines.push("END:VEVENT".to_owned());
    lines
}

/// The `VCALENDAR` of `rows`, in due-date order, and the events it holds.
#[must_use]
pub fn render(cal: &Calendar<'_>, rows: &[Row<'_>]) -> (String, Vec<Event>) {
    let mut rows = rows.to_vec();
    rows.sort_by(|a, b| (a.due, &a.scheduled.deadline).cmp(&(b.due, &b.scheduled.deadline)));
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_owned(),
        "VERSION:2.0".to_owned(),
        format!(
            "PRODID:-//litgraph//litigate {}//EN",
            env!("CARGO_PKG_VERSION")
        ),
        "CALSCALE:GREGORIAN".to_owned(),
        "METHOD:PUBLISH".to_owned(),
        format!("X-WR-CALNAME:{}", escape(cal.case)),
    ];
    let events = rows
        .iter()
        .map(|row| {
            lines.extend(vevent(cal, row));
            let s = row.scheduled;
            Event {
                uid: uid(cal.case, s),
                deadline: s.deadline.clone(),
                due: row.due.to_iso(),
                status: status_name(s),
                trigger: s.trigger.clone(),
                trigger_date: s.trigger_date.clone(),
                authority: row.authority.map(str::to_owned),
                counting: s.rule.clone(),
                counting_cite: s.cite.clone(),
            }
        })
        .collect();
    lines.push("END:VCALENDAR".to_owned());
    (lines.iter().map(|l| fold(l)).collect(), events)
}

#[cfg(test)]
mod tests;
