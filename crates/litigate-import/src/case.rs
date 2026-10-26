//! A named scenario to a `.case` matter (SPEC §8.3 in reverse).
//!
//! Legacy scenarios assert their posture (`scenario.start`) instead of
//! replaying events, so the import writes an explicit `start` and no
//! `events`; the porter replaces it with the matter's real history. A v2
//! scenario's `parties` come back as a `parties` block and its explicit
//! holes (`lowered.holes`) as `unvalued` stakes and `as_of ?` sources. Its
//! `events` come back as a comment only: replaying them would derive again
//! the `start`, `facts` and `remove_edges` the import already asserts, and
//! which of those they derive is known only to replay.
use crate::issues::Issues;
use crate::json;
use crate::links::role_name;
use crate::manifest::{IN_FORCE_PLACEHOLDER, VERSION};
use crate::text;
use crate::Emitted;
use litigate_engine::model::{Pack, Role};
use litigate_engine::scenario::{DocketEvent, EventRelation, NamedScenario};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Scenario fields with a dedicated `.case` block; everything else goes to
/// `analysis` (SPEC §8.3).
const DEDICATED: [&str; 5] = ["start", "perspective", "facts", "payoffs", "observe"];

/// Imports `named` (whose raw `scenario` object is `raw`) into
/// `cases/<id>.case`. `packs` are the packs known to this import.
pub(crate) fn import_case(
    named: &NamedScenario,
    raw: &Map<String, Value>,
    from: &str,
    packs: &BTreeMap<&str, &Pack>,
    issues: &mut Issues,
) -> Vec<Emitted> {
    let primary = named
        .packs
        .first()
        .and_then(|p| packs.get(p.as_str()))
        .copied();
    let holes = Holes::of(named, issues);
    let mut out = header(named, from, primary, &holes, issues);
    out.extend(posture(named, primary, issues));
    let scenario = &named.scenario;
    if !scenario.payoffs.is_empty() || !holes.unvalued.is_empty() {
        out.push(stakes(raw, &holes, issues));
    }
    if !scenario.observe.is_empty() {
        out.push(beliefs(&scenario.observe, issues));
    }
    let rest: Map<String, Value> = raw
        .iter()
        .filter(|(key, _)| !DEDICATED.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    out.extend(json::analysis(&rest, issues));
    vec![Emitted::case(
        format!("cases/{}.case", named.id),
        out.join("\n\n"),
    )]
}

/// The explicit holes of a v2 scenario (SPEC §8.3): the terminals of its
/// `unvalued` stakes, and each source whose `as_of` is a hole, with the
/// hole's reason.
#[derive(Default)]
struct Holes {
    unvalued: Vec<String>,
    as_of: BTreeMap<usize, Option<String>>,
}

impl Holes {
    /// `named`'s `lowered.holes`; a hole that has no `.case` spelling is
    /// reported.
    fn of(named: &NamedScenario, issues: &mut Issues) -> Self {
        let mut holes = Self::default();
        let listed = named.lowered.iter().flat_map(|l| &l.holes);
        for (i, hole) in listed.enumerate() {
            let at = format!("/lowered/holes/{i}");
            let source = hole
                .pointer
                .strip_prefix("/sources/")
                .and_then(|rest| rest.strip_suffix("/as_of"))
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|n| {
                    named
                        .sources
                        .get(*n)
                        .is_some_and(|s| as_of_matches(s, hole))
                });
            let terminal = hole
                .pointer
                .strip_prefix("/scenario/payoffs/")
                .map(unescape_token)
                .filter(|t| hole.reason.is_none() && !named.scenario.payoffs.contains_key(t));
            match (source, terminal) {
                (Some(n), _) => {
                    holes.as_of.insert(n, hole.reason.clone());
                }
                (None, Some(terminal)) => holes.unvalued.push(terminal),
                (None, None) => issues.report(
                    &at,
                    format!(
                        "the hole at `{}` has no `.case` spelling (SPEC §8.3)",
                        hole.pointer
                    ),
                ),
            }
        }
        holes
    }
}

/// Whether `source`'s `as_of` is what a hole with `hole`'s reason lowers
/// to: the reason, or the empty string for a bare `?`.
fn as_of_matches(
    source: &litigate_engine::scenario::ScenarioSource,
    hole: &litigate_engine::model::Hole,
) -> bool {
    source.as_of.as_deref() == Some(hole.reason.as_deref().unwrap_or_default())
}

/// A JSON pointer reference token unescaped (RFC 6901 §4).
fn unescape_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

fn source_text(
    source: &litigate_engine::scenario::ScenarioSource,
    hole: Option<&Option<String>>,
) -> String {
    let mut fields = Vec::new();
    if let Some(url) = &source.url {
        fields.push(format!("url {}", text::string(url)));
    }
    if let Some(reason) = hole {
        fields.push(match reason {
            Some(reason) => format!("as_of ?({})", text::string(reason)),
            None => "as_of ?".to_owned(),
        });
    } else if let Some(as_of) = &source.as_of {
        // Text that is not a whole date (`2026-06`) stays as a hole's reason.
        if is_date(as_of) {
            fields.push(format!("as_of {as_of}"));
        } else {
            fields.push(format!("as_of ?({})", text::string(as_of)));
        }
    }
    let title = text::string(&source.title);
    if fields.is_empty() {
        format!("source {title}")
    } else {
        format!("source {title} {{ {} }}", fields.join(", "))
    }
}

fn perspective(
    roles: &BTreeMap<String, Role>,
    primary: Option<&Pack>,
    issues: &mut Issues,
) -> String {
    let entries: Vec<String> = roles
        .iter()
        .map(|(actor, role)| {
            let at = format!("/scenario/perspective/{actor}");
            if primary.and_then(|p| p.roles.get(actor)) == Some(role) {
                issues.report(
                    &at,
                    "a perspective entry equal to the package role lowers to nothing (SPEC §8.3)",
                );
            }
            format!("{}: {}", issues.ident(actor, &at, false), role_name(*role))
        })
        .collect();
    format!("perspective {{ {} }}", entries.join(", "))
}

/// `stakes` in the scenario's own key order (the raw JSON object), then
/// each `unvalued` terminal in hole order.
fn stakes(raw: &Map<String, Value>, holes: &Holes, issues: &mut Issues) -> String {
    let payoffs = raw
        .get("payoffs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let valued = payoffs.iter().map(|(node, value)| {
        let money = value.as_f64().and_then(text::money);
        (node, Some(money))
    });
    let unvalued = holes.unvalued.iter().map(|terminal| (terminal, None));
    let entries: Vec<String> = valued
        .chain(unvalued)
        .map(|(node, money)| {
            let at = format!("/scenario/payoffs/{node}");
            let node = issues.ident(node, &at, true);
            let stake = match money {
                Some(money) => issues.spell(money, &at, "payoff"),
                None => "unvalued".to_owned(),
            };
            format!("{node}: {stake}")
        })
        .collect();
    format!("stakes {{ {} }}", entries.join(", "))
}

fn beliefs(observe: &BTreeMap<String, BTreeMap<String, f64>>, issues: &mut Issues) -> String {
    let entries: Vec<String> = observe
        .iter()
        .map(|(node, counts)| {
            let at = format!("/scenario/observe/{node}");
            let counts: Vec<String> = counts
                .iter()
                .map(|(edge, count)| {
                    let edge_at = format!("{at}/{edge}");
                    let count = issues.spell(text::number(*count), &edge_at, "count");
                    format!("{}: {count}", issues.ident(edge, &edge_at, true))
                })
                .collect();
            format!(
                "observe {}: {{ {} }}",
                issues.ident(node, &at, true),
                counts.join(", ")
            )
        })
        .collect();
    format!("beliefs {{ {} }}", entries.join(", "))
}

/// Whether `text` lexes as one `Date` token (`YYYY-MM-DD`, a real day).
fn is_date(text: &str) -> bool {
    let shape = text.len() == 10
        && text.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    shape && litigate_syntax::lexer::valid_date(text)
}

/// `case`, `uses`, `as_of`, `summary`, `notes`, and the `source`s.
fn header(
    named: &NamedScenario,
    from: &str,
    primary: Option<&Pack>,
    holes: &Holes,
    issues: &mut Issues,
) -> Vec<String> {
    let id = issues.ident(&named.id, "/id", false);
    if named.packs.is_empty() {
        issues.report("/packs", "a case `uses` at least one package (SPEC §3.2)");
    }
    let uses: Vec<String> = named
        .packs
        .iter()
        .enumerate()
        .map(|(i, pack)| {
            format!(
                "{} ^{VERSION}",
                issues.ident(pack, &format!("/packs/{i}"), false)
            )
        })
        .collect();
    let as_of = primary.map_or_else(
        || IN_FORCE_PLACEHOLDER.to_owned(),
        crate::manifest::known_at,
    );
    let mut out = vec![
        format!("// Imported by `litigate import` from {from}.\ncase {id}"),
        format!("uses {}", uses.join(", ")),
        format!("as_of {as_of}"),
        format!("summary {}", text::string(&named.summary)),
    ];
    if !named.notes.is_empty() {
        out.push(format!("notes {}", text::string(&named.notes)));
    }
    for (i, source) in named.sources.iter().enumerate() {
        out.push(source_text(source, holes.as_of.get(&i)));
    }
    out
}

/// `parties` in declared order, one per line.
fn parties(named: &NamedScenario, issues: &mut Issues) -> String {
    let entries: Vec<String> = named
        .parties
        .iter()
        .enumerate()
        .map(|(i, party)| {
            let at = format!("/parties/{i}");
            format!(
                "{}: {} {}",
                issues.ident(&party.id, &format!("{at}/id"), false),
                issues.ident(&party.actor, &format!("{at}/actor"), false),
                issues.ident(&party.entity_type, &format!("{at}/entity_type"), false)
            )
        })
        .collect();
    format!("parties {{\n{}\n}}", entries.join("\n"))
}

/// The docket as a commented `events` block: the record of what replay
/// derived the asserted posture from, for the porter to restore. An event
/// whose text would break out of its comment line is reported.
fn docket(events: &[DocketEvent], issues: &mut Issues) -> String {
    let mut out = vec![
        "// The docket this matter was lowered from. Replay derived the posture".to_owned(),
        "// asserted here (`start`, `facts`, `remove_edges`) from it; to replay it".to_owned(),
        "// instead, against packages that declare its events, uncomment it and".to_owned(),
        "// delete what it derives (SPEC §8.7).".to_owned(),
        "// events {".to_owned(),
    ];
    out.extend(events.iter().enumerate().map(|(i, e)| {
        if [&e.date, &e.action, &e.party]
            .iter()
            .any(|t| t.contains(['\n', '\r']))
        {
            issues.report(&format!("/events/{i}"), "a docket event spans lines");
        }
        let docket = e.docket.map(|n| format!(" docket {n}")).unwrap_or_default();
        let relation = match e.relation {
            EventRelation::By => "by",
            EventRelation::On => "on",
        };
        let cite = e
            .authority
            .as_ref()
            .map(|a| format!(" cite {}", text::string(a)))
            .unwrap_or_default();
        format!(
            "//   {}{docket} {} {relation} {}{cite}",
            e.date, e.action, e.party
        )
    }));
    out.push("// }".to_owned());
    out.join("\n")
}

/// `start`, `perspective`, `parties`, the docket comment and `facts`: the
/// asserted posture.
fn posture(named: &NamedScenario, primary: Option<&Pack>, issues: &mut Issues) -> Vec<String> {
    let mut out = Vec::new();
    let scenario = &named.scenario;
    if let Some(start) = &scenario.start {
        let start = issues.ident(start, "/scenario/start", true);
        let implied = primary.map(|p| format!("{}::{}", p.id, p.start_node_id));
        if implied.as_deref() == Some(start.as_str()) {
            issues.report(
                "/scenario/start",
                "an explicit start equal to the package start lowers to no start (SPEC §8.3)",
            );
        }
        out.push(format!("start {start}"));
    }
    if !scenario.perspective.is_empty() {
        out.push(perspective(&scenario.perspective, primary, issues));
    }
    if !named.parties.is_empty() {
        out.push(parties(named, issues));
    }
    if !named.events.is_empty() {
        out.push(docket(&named.events, issues));
    }
    if !scenario.facts.is_empty() {
        let entries: Vec<String> = scenario
            .facts
            .iter()
            .map(|(node, edge)| {
                let at = format!("/scenario/facts/{node}");
                format!(
                    "{}: {}",
                    issues.ident(node, &at, true),
                    issues.ident(edge, &at, true)
                )
            })
            .collect();
        out.push(format!("facts {{ {} }}", entries.join(", ")));
    }
    out
}
