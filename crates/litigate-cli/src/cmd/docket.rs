//! `litigate match-docket CASE DOCKET --out FILE` (DESIGN §10, the docket
//! matcher): match PACER/RECAP docket entries to the event forms the
//! matter's packages declare, and advance the matter only by the entries
//! the text supports and replay accepts.
//!
//! Entries are taken in docket order (date, then entry number). Each is
//! reported with its source verbatim, its candidates and their
//! confidence, and a status: `advanced` (spliced in, `docket N` keeping
//! its entry number), `rejected` (supported, but replay refuses it, with
//! the diagnostics), `ambiguous`, `incomplete` or `unmatched` (never
//! spliced: posture does not change on a guess), or `recorded` (the
//! matter already has an event with that docket number). Each advanced
//! entry is replayed on the matter as advanced by the entries before it,
//! exactly as `lint-filing` replays a proposed filing. The matter itself
//! must check clean first; `--out` writes the advanced matter.
use super::check::check_session;
use super::filing::{added, changes, generated_filing_line};
use crate::docket::{self, Entry, Form, Param, Party, Verdict};
use crate::filing;
use crate::hirx;
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Profile, Severity};
use litigate_engine::clock::Date;
use litigate_lower::Options;
use litigate_sema::hir::{self, Declaration, Type};
use serde_json::{json, Value};

/// What `match-docket` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` file.
    pub case: String,
    /// The docket JSON file.
    pub docket: String,
    /// Where to write the advanced matter, if anywhere.
    pub out: Option<String>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// The variants of enum `name` declared in pack `pack`.
fn variants(all: &[&hir::Package], pack: &str, name: &str) -> Option<Vec<String>> {
    all.iter()
        .filter(|p| p.manifest.pack_id == pack)
        .flat_map(|p| hirx::declarations(p))
        .find_map(|d| match d {
            Declaration::Enum(e) if e.name == name => Some(e.variants.clone()),
            _ => None,
        })
}

/// The event forms the matter may record: those of the packages it uses
/// and their dependencies, spelled as the matter writes them (bare in the
/// primary package, else `pack::Event`).
fn forms(session: &Session, used: &[String]) -> Vec<Form> {
    let all = session.hir();
    let closure = session.closure(used.iter().cloned());
    let primary = used
        .first()
        .and_then(|n| all.iter().find(|p| &p.manifest.name == n))
        .map(|p| p.manifest.pack_id.as_str());
    let param = |pack: &str, t: &Type| match t {
        Type::Named(q) => {
            let (ep, en) = q.split_once("::").unwrap_or((pack, q));
            variants(&all, ep, en).map_or_else(|| Param::Other(q.clone()), Param::Enum)
        }
        other => Param::Other(format!("{other:?}")),
    };
    let mut out = Vec::new();
    for p in all.iter().filter(|p| closure.contains(&p.manifest.name)) {
        let pack = p.manifest.pack_id.as_str();
        for d in hirx::declarations(p) {
            if let Declaration::Event(e) = d {
                out.push(Form {
                    name: if primary == Some(pack) {
                        e.name.clone()
                    } else {
                        format!("{pack}::{}", e.name)
                    },
                    params: e
                        .params
                        .iter()
                        .map(|(n, t)| (n.clone(), param(pack, t)))
                        .collect(),
                });
            }
        }
    }
    out
}

/// What replaying one supported entry did.
struct Replayed {
    /// The matter with the entry, when replay accepted it.
    text: Option<String>,
    /// The entry's report fields.
    fields: Value,
}

/// Replays the matter `current` with the event `(call, mode, party)` of
/// `entry` dated `date` added.
fn replay(
    session: Session,
    current: &CaseText,
    (entry, date): (&Entry, &str),
    (call, mode, party): (&str, &str, &str),
) -> Result<(Session, Replayed), Failure> {
    let docket = entry
        .number
        .map(|n| format!(" docket {n}"))
        .unwrap_or_default();
    let line = format!("{date}{docket} {call} {mode} {party}");
    let at = CaseText {
        text: filing::with_as_of(&current.text, date),
        ..current.clone()
    };
    let (mut session, line) =
        generated_filing_line(session, current, &at, (date, entry.number), &line)?;
    let filed = CaseText {
        text: filing::splice_docketed(&at.text, date, entry.number, &line),
        ..current.clone()
    };
    session.cases = vec![current.clone(), at, filed];
    let c = check_session(session, Profile::Dev, &Options::default());
    let (at, filed) = (&c.cases[1], &c.cases[2]);
    let diagnostics = added(at, filed);
    let accepted = !diagnostics.iter().any(|d| d.severity == Severity::Error);
    let law = c.session.hir_of(&c.session.closure(at.packages.clone()));
    let ch = changes(at, filed, &law);
    let fields = json!({
        "event": {"call": call, "mode": mode, "party": party, "line": line},
        "diagnostics": diagnostics,
        "replay": ch.replay, "discharges": ch.discharges, "triggers": ch.triggers,
    });
    let text = accepted.then(|| c.session.cases[2].text.clone());
    Ok((c.session, Replayed { text, fields }))
}

/// `entry`'s report: its source, number, date and text, with `status`.
fn row(entry: &Entry, status: &str) -> Value {
    json!({
        "entry_number": entry.number, "date_filed": entry.date,
        "description": entry.text, "source": entry.source, "status": status,
    })
}

/// Sets `to[key]` for each `(key, value)` of object `from`.
fn merge(to: &mut Value, from: Value) {
    if let (Some(to), Value::Object(from)) = (to.as_object_mut(), from) {
        to.extend(from);
    }
}

/// The docket file's entries, in docket order (date, then entry number).
fn load(path: &str) -> Result<Vec<Entry>, Failure> {
    let fail = |e: &dyn std::fmt::Display| Failure::new(format!("{path}: {e}"));
    let raw = std::fs::read_to_string(path).map_err(|e| fail(&e))?;
    let doc: Value = serde_json::from_str(&raw).map_err(|e| fail(&e))?;
    let mut entries = docket::entries(&doc).map_err(|e| fail(&e))?;
    entries.sort_by(|a, b| (&a.date, a.number).cmp(&(&b.date, b.number)));
    Ok(entries)
}

/// What every entry is matched against: the matter's event forms and
/// parties, and the docket numbers it already records.
struct Matter {
    forms: Vec<Form>,
    parties: Vec<Party>,
    recorded: Vec<u64>,
}

/// A supported event: `(call, mode, party, date)`.
type Event<'v> = (&'v str, &'v str, &'v str, String);

/// An entry that does not advance the matter: its status and why.
type Unsettled = (&'static str, Option<String>);

/// The event `entry` supports, else its status and the reason.
fn settled<'v>(
    matter: &Matter,
    entry: &Entry,
    verdict: &'v Verdict,
) -> Result<Event<'v>, Unsettled> {
    if entry.number.is_some_and(|n| matter.recorded.contains(&n)) {
        return Err(("recorded", None));
    }
    let date = entry.date.as_deref().map(Date::parse_iso);
    match (verdict, date) {
        (Verdict::Unmatched, _) => Err(("unmatched", None)),
        (Verdict::Ambiguous(why), _) => Err(("ambiguous", Some(why.clone()))),
        (Verdict::Incomplete(why), _) => Err(("incomplete", Some(why.clone()))),
        (Verdict::Supported(..), None) => Err(("incomplete", Some("no `date_filed`".to_owned()))),
        (Verdict::Supported(..), Some(Err(e))) => {
            Err(("incomplete", Some(format!("`date_filed`: {e}"))))
        }
        (Verdict::Supported(call, mode, party), Some(Ok(day))) => {
            Ok((call, mode, party, day.to_iso()))
        }
    }
}

/// `entry`'s report row, advancing `current` when replay accepts it.
fn match_entry(
    session: Session,
    matter: &Matter,
    current: &mut CaseText,
    entry: &Entry,
) -> Result<(Session, Value), Failure> {
    let (candidates, verdict) = docket::judge(&entry.text, &matter.forms, &matter.parties);
    let (session, mut r) = match settled(matter, entry, &verdict) {
        Err((status, reason)) => {
            let mut r = row(entry, status);
            if let Some(reason) = reason {
                r["reason"] = Value::from(reason);
            }
            (session, r)
        }
        Ok((call, mode, party, day)) => {
            let (session, replayed) = replay(session, current, (entry, &day), (call, mode, party))?;
            let status = if replayed.text.is_some() {
                "advanced"
            } else {
                "rejected"
            };
            let mut r = row(entry, status);
            merge(&mut r, replayed.fields);
            if let Some(text) = replayed.text {
                current.text = text;
            }
            (session, r)
        }
    };
    r["candidates"] = json!(candidates);
    Ok((session, r))
}

/// `litigate match-docket`.
pub fn match_docket(args: &Args) -> Outcome {
    let entries = load(&args.docket)?;
    let mut session = ws::discover(std::slice::from_ref(&args.case))
        .and_then(|targets| Session::load(&targets, &args.lib))?;
    let original = session
        .cases
        .first()
        .cloned()
        .ok_or_else(|| Failure::new(format!("{}: not a .case file", args.case)))?;
    session.cases = vec![original.clone()];
    let c = check_session(session, Profile::Dev, &Options::default());
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: c.session.sources(),
        diagnostics: c
            .verdicts
            .iter()
            .flat_map(|(_, v)| v.diagnostics.iter().cloned())
            .chain(c.cases[0].diagnostics.iter().cloned())
            .collect(),
        ..Report::default()
    };
    let Some(hcase) = c.cases[0].case.clone().filter(|_| !report.has_errors()) else {
        return Ok(report);
    };
    // Each entry carries the diagnostics its own replay adds.
    report.diagnostics.clear();
    let matter = Matter {
        forms: forms(&c.session, &c.cases[0].packages),
        parties: hcase
            .parties
            .iter()
            .map(|p| Party {
                id: p.id.clone(),
                actor: p.actor.clone(),
            })
            .collect(),
        recorded: hcase.events.iter().filter_map(|e| e.docket).collect(),
    };
    let mut session = c.session;
    let mut current = original;
    let mut rows = Vec::new();
    for entry in &entries {
        let (s, r) = match_entry(session, &matter, &mut current, entry)?;
        session = s;
        rows.push(r);
    }
    if let Some(out) = &args.out {
        std::fs::write(out, &current.text).map_err(|e| Failure::new(format!("{out}: {e}")))?;
    }
    let count = |s: &str| rows.iter().filter(|r| r["status"] == s).count();
    let as_of = litigate_syntax::parse_case(&current.text)
        .case_file()
        .and_then(|f| f.as_of())
        .map(|t| t.text().to_owned());
    report.result = json!({
        "case": hcase.id, "docket": args.docket, "as_of": as_of,
        "advanced": count("advanced"), "unresolved": rows.len() - count("advanced") - count("recorded"),
        "written": args.out, "entries": rows, "text": current.text,
    });
    Ok(report)
}
