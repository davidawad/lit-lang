//! A named scenario to a `.case` matter (SPEC §8.3 in reverse).
//!
//! Legacy scenarios assert their posture (`scenario.start`) instead of
//! replaying events, so the import writes an explicit `start` and no
//! `events`; the porter replaces it with the matter's real history.
use crate::issues::Issues;
use crate::json;
use crate::links::role_name;
use crate::manifest::{IN_FORCE_PLACEHOLDER, VERSION};
use crate::text;
use crate::Emitted;
use litgraph::model::{Pack, Role};
use litgraph::scenario::NamedScenario;
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
    let mut out = header(named, from, primary, issues);
    out.extend(posture(named, primary, issues));
    let scenario = &named.scenario;
    if !scenario.payoffs.is_empty() {
        out.push(stakes(raw, issues));
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

fn source_text(source: &litgraph::scenario::ScenarioSource) -> String {
    let mut fields = Vec::new();
    if let Some(url) = &source.url {
        fields.push(format!("url {}", text::string(url)));
    }
    if let Some(as_of) = &source.as_of {
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

/// `stakes` in the scenario's own key order (the raw JSON object).
fn stakes(raw: &Map<String, Value>, issues: &mut Issues) -> String {
    let payoffs = raw
        .get("payoffs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let entries: Vec<String> = payoffs
        .iter()
        .map(|(node, value)| {
            let at = format!("/scenario/payoffs/{node}");
            let money = value.as_f64().and_then(text::money);
            format!(
                "{}: {}",
                issues.ident(node, &at, true),
                issues.spell(money, &at, "payoff")
            )
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
    for source in &named.sources {
        out.push(source_text(source));
    }
    out
}

/// `start`, `perspective`, and `facts`: the asserted posture.
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
