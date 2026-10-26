//! Printed names round-trip (lg-d2pi): every move `lint-filing` prints,
//! for a matter over each `lit/` package and for each `cases/` matter, is
//! accepted when fed back, both into `lint-filing` (spliced into the
//! matter as it splices a proposed filing) and into a `.case` that
//! writes one `uses` per package. A move is printed from any posture: the
//! acts out of every node of the used packages that discharge each of
//! their deadlines, each grant's request, and each order, completed with
//! arguments of its parameter types.
use super::{moves, used, Law};
use crate::hirx;
use crate::ws::{self, CaseText, Session};
use crate::{check_session, filing};
use litigate_base::{DiagnosticCode, Profile, Severity};
use litigate_lower::Options;
use litigate_sema::case::names::{self, Resolution};
use litigate_sema::hir::{self, Type};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The date every fed-back move is filed on.
const DATE: &str = "2026-06-01";

/// A matter to print moves for: its text and path, and the packages it
/// uses, primary first.
struct Matter<'p> {
    path: String,
    text: String,
    used: Vec<&'p hir::Package>,
}

/// Every move printed for a matter using `used` among `all`.
fn printed(all: &[&hir::Package], used: &[&hir::Package]) -> BTreeSet<String> {
    let vocabulary = names::vocabulary(used, all);
    let law = Law {
        law: &vocabulary,
        vocabulary: &vocabulary,
    };
    let nodes: Vec<(&str, &hir::Node)> = used
        .iter()
        .flat_map(|p| {
            hirx::nodes(p)
                .into_iter()
                .map(|n| (p.manifest.pack_id.as_str(), n))
        })
        .collect();
    let deadlines = used.iter().flat_map(|p| {
        hirx::declarations(p).filter_map(|d| match d {
            hir::Declaration::Deadline(d) => Some((
                format!("{}::{}", p.manifest.pack_id, d.meta.id),
                format!("{}@{}", d.meta.id, d.meta.effective.from),
            )),
            _ => None,
        })
    });
    let mut out = BTreeSet::new();
    for (deadline, version) in deadlines {
        let m = moves(law, &nodes, &deadline, &version);
        let events = m["file"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| &f["event"]);
        let requests = m["extend"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|g| &g["request"]);
        out.extend(
            events
                .chain(requests)
                .filter_map(Value::as_str)
                .map(str::to_owned),
        );
        let orders = m["orders"].as_array().into_iter().flatten();
        out.extend(
            orders
                .filter_map(Value::as_str)
                .map(|o| order(&vocabulary, o)),
        );
    }
    out
}

/// An order as `lint-filing` names it, completed with an argument of each
/// of its parameter types (none when the name resolves to no event: the
/// matter then reports it).
fn order(vocabulary: &[&hir::Package], name: &str) -> String {
    let args: Vec<&str> = match names::resolve(vocabulary, name) {
        Resolution::Event(_, ty) => ty
            .params
            .iter()
            .map(|(p, t)| match t {
                Type::Date => DATE,
                other => panic!("order `{name}`: no argument for `{p}: {other:?}`"),
            })
            .collect(),
        _ => Vec::new(),
    };
    format!("{name}({})", args.join(", "))
}

/// A `.case` using `used` (one `uses` per package) whose events are
/// `events`, in order.
fn authored(used: &[&hir::Package], events: &BTreeSet<String>) -> String {
    let uses = used.iter().fold(String::new(), |mut s, p| {
        let _ = writeln!(s, "uses {} ={}", p.manifest.name, p.manifest.version);
        s
    });
    let lines = events
        .iter()
        .enumerate()
        .fold(String::new(), |mut s, (i, e)| {
            let _ = writeln!(s, "  m-{i}: {DATE} {e} by party");
            s
        });
    format!(
        "case round-trip\n{uses}as_of {DATE}\nsummary \"printed moves fed back\"\n\n\
         parties {{\n  party: applicant Corporation\n}}\n\nevents {{\n{lines}}}\n"
    )
}

/// The syntax, name and typing errors checking `matters` reports, each
/// with the line it is on.
fn rejected(session: Session, matters: Vec<CaseText>) -> Vec<String> {
    let mut session = session;
    let lock = session.matter_lock(&matters).render();
    session.cases = matters
        .into_iter()
        .map(|m| CaseText {
            lock: Some(lock.clone()),
            ..m
        })
        .collect();
    let checked = check_session(session, Profile::Dev, &Options::default());
    checked
        .cases
        .iter()
        .zip(&checked.session.cases)
        .flat_map(|(c, text)| {
            c.diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .filter(|d| {
                    matches!(
                        d.code,
                        DiagnosticCode::UnexpectedToken
                            | DiagnosticCode::UnresolvedName
                            | DiagnosticCode::TypeMismatch
                    )
                })
                .map(|d| {
                    let line = text.text.lines().nth(d.span.line as usize - 1);
                    format!("{}: {} ({})", c.path, d.message, line.unwrap_or("").trim())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn every_printed_move_is_accepted_back() {
    let targets = ws::discover(&["../../lit".to_owned(), "../../cases".to_owned()]).unwrap();
    let session = Session::load(&targets, &[]).unwrap();
    let cases = session.cases.clone();
    let checked = check_session(session, Profile::Dev, &Options::default());
    let all = checked.session.hir();
    // Each `cases/` matter as written, and one matter over each `lit/`
    // package with a procedure graph.
    let mut matters: Vec<Matter<'_>> = checked
        .cases
        .iter()
        .zip(&cases)
        .map(|(c, text)| Matter {
            path: text.path.clone(),
            text: text.text.clone(),
            used: c
                .case
                .as_ref()
                .map(|case| used(&all, case))
                .unwrap_or_default(),
        })
        .collect();
    matters.extend(
        all.iter()
            .filter(|p| !p.manifest.links && !hirx::nodes(p).is_empty())
            .map(|p| Matter {
                path: format!("cases/round-trip-{}.case", p.manifest.name),
                text: authored(&[p], &BTreeSet::new()),
                used: vec![*p],
            }),
    );
    assert!(matters.iter().all(|m| !m.used.is_empty()));
    let mut fed = Vec::new();
    let mut count = 0;
    for (i, m) in matters.iter().enumerate() {
        let moves = printed(&all, &m.used);
        count += moves.len();
        // `lint-filing`: each move spliced in as it splices a filing.
        let spliced = moves.iter().fold(m.text.clone(), |text, e| {
            filing::splice(
                &filing::with_as_of(&text, DATE),
                DATE,
                &filing::line(DATE, e, "by", "party"),
            )
        });
        fed.push(CaseText {
            path: m.path.clone(),
            text: spliced,
            lock: None,
        });
        // A `.case` authoring them, one `uses` per package.
        fed.push(CaseText {
            path: format!("cases/round-trip-authored-{i}.case"),
            text: authored(&m.used, &moves),
            lock: None,
        });
    }
    assert!(count > 0, "no move was printed");
    let errors = rejected(checked.session, fed);
    assert!(
        errors.is_empty(),
        "{count} moves; rejected:\n{}",
        errors.join("\n")
    );
}
