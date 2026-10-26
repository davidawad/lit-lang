//! A proposed filing spliced into a matter's source (DESIGN §10, the
//! filing linter): the event goes into the `events` block in docket
//! order, and `as_of` moves to the filing date when that is later, so
//! replay judges the filing on the day it would be made.
//!
//! The functions are total over any text, including one that does not
//! parse: the caller checks the original matter first and re-parses the
//! result ([`added`]) before trusting it.
use litigate_syntax::ast::{CaseEvent, CaseFile};
use litigate_syntax::{SyntaxKind, TextSize};

fn offset(at: TextSize) -> usize {
    u32::from(at) as usize
}

fn insert(text: &str, at: usize, what: &str) -> String {
    format!("{}{what}{}", &text[..at], &text[at..])
}

fn case_file(text: &str) -> Option<CaseFile> {
    litigate_syntax::parse_case(text).case_file()
}

/// The event line for a filing: `DATE EVENT by|on PARTY`.
#[must_use]
pub fn line(date: &str, event: &str, mode: &str, party: &str) -> String {
    format!("{date} {event} {mode} {party}")
}

/// `text` with `as_of` set to `date` when `date` is later; else unchanged.
#[must_use]
pub fn with_as_of(text: &str, date: &str) -> String {
    match case_file(text).and_then(|f| f.as_of()) {
        Some(token) if token.text() < date => {
            let range = token.text_range();
            format!(
                "{}{date}{}",
                &text[..offset(range.start())],
                &text[offset(range.end())..]
            )
        }
        _ => text.to_owned(),
    }
}

/// `text` with event `line` (dated `date`) in its `events` block: before
/// the first event dated after it, else after the last event, else in an
/// empty block, else in a new block before `facts`/`stakes`/`beliefs`/
/// `analysis` (or at the end).
#[must_use]
pub fn splice(text: &str, date: &str, line: &str) -> String {
    splice_docketed(text, date, None, line)
}

/// A docket number as written (digit separators allowed).
fn docket_number(text: &str) -> Option<u64> {
    text.replace('_', "").parse().ok()
}

/// [`splice`] for an event with docket number `docket`: it also goes
/// before an event of its own date with a greater docket number, so the
/// block stays in docket order (SPEC §5.6).
#[must_use]
pub fn splice_docketed(text: &str, date: &str, docket: Option<u64>, line: &str) -> String {
    let file = case_file(text);
    let events = file.as_ref().map(CaseFile::events).unwrap_or_default();
    let later = |e: &CaseEvent| {
        e.date().is_some_and(|d| {
            d.text() > date
                || (d.text() == date
                    && docket.is_some_and(|n| {
                        e.docket()
                            .and_then(|t| docket_number(t.text()))
                            .is_some_and(|m| m > n)
                    }))
        })
    };
    if let Some(later) = events.iter().find(|e| later(e)).and_then(CaseEvent::date) {
        let start = offset(later.text_range().start());
        let at = text[..start].rfind('\n').map_or(0, |i| i + 1);
        return insert(text, at, &format!("  {line}\n"));
    }
    if let Some(last) = events.last() {
        let at = offset(last.syntax().text_range().end());
        return insert(text, at, &format!("\n  {line}"));
    }
    let block = file.as_ref().and_then(|f| {
        f.syntax()
            .children()
            .find(|n| n.kind() == SyntaxKind::EventsBlock)
    });
    if let Some(close) = block
        .and_then(|b| b.last_token())
        .filter(|t| t.kind() == SyntaxKind::RBrace)
    {
        let at = offset(close.text_range().start());
        return insert(text, at, &format!("\n  {line}\n"));
    }
    let blocks = file.as_ref().map(CaseFile::blocks).unwrap_or_default();
    match blocks.first() {
        Some(b) => insert(
            text,
            offset(b.syntax().text_range().start()),
            &format!("events {{\n  {line}\n}}\n\n"),
        ),
        None if text.ends_with('\n') => format!("{text}\nevents {{\n  {line}\n}}\n"),
        None => format!("{text}\n\nevents {{\n  {line}\n}}\n"),
    }
}

/// Whether `after` is `before` with exactly one event added, written as
/// `line`: anything else means the filing's text was not one event.
#[must_use]
pub fn added(before: &str, after: &str, line: &str) -> bool {
    let texts = |text: &str| -> Vec<String> {
        case_file(text)
            .map(|f| f.events())
            .unwrap_or_default()
            .iter()
            .map(|e| e.syntax().text().to_string().trim().to_owned())
            .collect()
    };
    let (old, new) = (texts(before), texts(after));
    new.len() == old.len() + 1 && new.iter().any(|e| e == line)
}

#[cfg(test)]
mod tests;
