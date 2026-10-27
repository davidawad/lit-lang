//! `litigate actions CASE --party PARTY [--date DATE]` (DESIGN §10, agent
//! action sets): the typed marking of a matter projected onto one party,
//! as the moves that party may legally make in that posture.
//!
//! The candidates are the edges a filing may take from the replayed
//! marking (each concurrent track's node, and the `anytime` edges of the
//! used packages, SPEC §5.6) whose actor the party plays (or `either`),
//! each event pattern completed as `lint-filing` prints it. Every
//! candidate is then replayed into the matter on the date, exactly as
//! `lint-filing` replays a proposed filing, and only the ones replay
//! accepts are listed: an illegal move (another party's, a later
//! stage's, a late one) is absent, never offered and refused later. The
//! rules are the packages' and the replay's; nothing here decides
//! legality on its own.
//!
//! `as_of` moves to `DATE` when that is later, as for `lint-filing`. The
//! matter must check clean first; the diagnostics are then its own on
//! that date.
use super::check::check_session;
use super::filing::{added, changes, completions, known, posture, vocabulary, Law};
use crate::filing;
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Profile, Severity};
use litigate_engine::clock::Date;
use litigate_lower::Options;
use litigate_sema::hir;
use litigate_sema::pkg::source::Source;
use serde_json::{json, Value};

/// What `actions` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` file.
    pub case: String,
    /// The party whose moves are listed.
    pub party: String,
    /// The date the moves would be made (default: the matter's `as_of`).
    pub date: Option<String>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// One candidate move: the edge it takes and the event line that takes it.
struct Candidate {
    edge: Value,
    event: String,
    line: String,
}

/// The candidate moves of `party` (who plays `actor`) at the matter
/// `at`'s marking, dated `date`, under the packages of `session`.
fn candidates(
    session: &Session,
    at: &crate::matter::CaseCheck,
    (party, actor, date): (&str, &str, &str),
) -> Vec<Candidate> {
    let Some(case) = at.case.as_ref() else {
        return Vec::new();
    };
    let law = session.hir_of(&session.closure(at.packages.clone()));
    let vocabulary = vocabulary(&session.hir(), at);
    let mut out: Vec<Candidate> = Vec::new();
    for (pack, e) in posture(&law, case, &at.marking) {
        if e.actor != "either" && e.actor != actor {
            continue;
        }
        let Some(pattern) = e.event.as_ref() else {
            continue;
        };
        let to = if e.to.contains("::") {
            e.to.clone()
        } else {
            format!("{pack}::{}", e.to)
        };
        let edge = json!({
            "id": e.id.as_ref().map(|id| format!("{pack}::{id}")),
            "label": e.label, "actor": e.actor, "to": to,
            "authority": e.authority, "anytime": e.anytime,
        });
        for event in completions(&vocabulary, pack, pattern) {
            if out.iter().any(|c| c.event == event && c.edge == edge) {
                continue;
            }
            out.push(Candidate {
                edge: edge.clone(),
                line: filing::line(date, &event, "by", party),
                event,
            });
        }
    }
    out
}

/// The `candidates` that replay accepts, each spliced into the matter
/// `at` (a copy of `original` dated `as_of`) by `party`, with what it
/// discharges and triggers.
fn legal(
    mut session: Session,
    (original, at): (&CaseText, &CaseText),
    candidates: &[Candidate],
    (party, as_of): (&str, &str),
) -> Vec<Value> {
    session.cases = std::iter::once(at.clone())
        .chain(candidates.iter().map(|m| CaseText {
            text: filing::splice(&at.text, as_of, &m.line),
            ..original.clone()
        }))
        .collect();
    let replayed = check_session(session, Profile::Dev, &Options::default());
    let (at, at_text) = (&replayed.cases[0], &replayed.session.cases[0].text);
    let law = replayed
        .session
        .hir_of(&replayed.session.closure(at.packages.clone()));
    let vocabulary = vocabulary(&replayed.session.hir(), at);
    let law = Law {
        law: &law,
        vocabulary: &vocabulary,
    };
    candidates
        .iter()
        .zip(&replayed.cases[1..])
        .zip(&replayed.session.cases[1..])
        .filter(|((m, filed), text)| {
            filing::added(at_text, &text.text, &m.line)
                && !added(at, filed)
                    .iter()
                    .any(|d| d.severity == Severity::Error)
        })
        .map(|((m, filed), _)| {
            let ch = changes(at, filed, law);
            json!({
                "event": m.event, "mode": "by", "party": party, "date": as_of,
                "line": m.line, "edge": m.edge,
                "discharges": ch.discharges, "triggers": ch.triggers,
            })
        })
        .collect()
}

/// What keeps the matter of `c` (as written, then dated) from being
/// judged: the package verdicts' and the matter's diagnostics, and the
/// errors that dating it adds.
fn blocking(c: &super::check::Checked) -> Vec<litigate_base::Diagnostic> {
    let (before, dated) = (&c.cases[0], &c.cases[1]);
    let mut diagnostics: Vec<_> = c
        .verdicts
        .iter()
        .flat_map(|(_, v)| v.diagnostics.iter().cloned())
        .chain(before.diagnostics.iter().cloned())
        .collect();
    let added: Vec<_> = dated
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error && !known(&diagnostics, d))
        .cloned()
        .collect();
    diagnostics.extend(added);
    diagnostics
}

/// The party `id` of `case`, else a usage failure listing its parties.
fn party<'c>(case: &'c hir::Case, id: &str) -> Result<&'c hir::Party, Failure> {
    case.parties.iter().find(|p| p.id == id).ok_or_else(|| {
        let parties: Vec<&str> = case.parties.iter().map(|p| p.id.as_str()).collect();
        Failure::new(format!(
            "`{id}` is not a party of `{}`; parties: {}",
            case.id,
            parties.join(", ")
        ))
    })
}

/// `litigate actions`, reading `src`.
pub fn actions(src: &dyn Source, args: &Args) -> Outcome {
    let date = args
        .date
        .as_deref()
        .map(|d| {
            Date::parse_iso(d)
                .map(Date::to_iso)
                .map_err(|e| Failure::new(format!("--date {d}: {e}")))
        })
        .transpose()?;
    let mut session = ws::discover_in(src, std::slice::from_ref(&args.case))
        .and_then(|targets| Session::load_in(src, &targets, &args.lib))?;
    let original = session
        .cases
        .first()
        .cloned()
        .ok_or_else(|| Failure::new(format!("{}: not a .case file", args.case)))?;
    let at = CaseText {
        text: date.as_deref().map_or_else(
            || original.text.clone(),
            |d| filing::with_as_of(&original.text, d),
        ),
        ..original.clone()
    };
    session.cases = vec![original.clone(), at.clone()];
    let c = check_session(session, Profile::Dev, &Options::default());
    let checked = &c.cases[1];
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: c
            .session
            .packages
            .iter()
            .flat_map(|p| p.files.iter().cloned())
            .chain([(original.path.clone(), original.text.clone())])
            .collect(),
        diagnostics: blocking(&c),
        ..Report::default()
    };
    if report.has_errors() {
        return Ok(report);
    }
    let case = checked
        .case
        .as_ref()
        .ok_or_else(|| Failure::new(format!("{}: the matter did not resolve", args.case)))?;
    let party = party(case, &args.party)?;
    // The package lints were the check's to report; the moves stand
    // beside the matter's own diagnostics on the date.
    report.diagnostics.clone_from(&checked.diagnostics);
    let as_of = case.as_of.clone();
    let (case_id, actor) = (case.id.clone(), party.actor.clone());
    let marking = checked.marking.clone();
    let candidates = candidates(&c.session, checked, (&args.party, &actor, &as_of));
    let legal = legal(
        c.session,
        (&original, &at),
        &candidates,
        (&args.party, &as_of),
    );
    report.result = json!({
        "case": case_id, "party": args.party, "actor": actor, "as_of": as_of,
        "marking": marking, "actions": legal,
    });
    Ok(report)
}
