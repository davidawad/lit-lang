//! A `.case` matter to a litgraph `NamedScenario` (SPEC §8.3).
use crate::diag::Sink;
use crate::source_map::{Entry, SourceMap};
use crate::Build;
use litgraph::model::Role;
use litgraph::scenario::{NamedScenario, ScenarioSource};
use litgraph::Scenario;
use litigate_base::DiagnosticCode as C;
use litigate_sema::hir::{self, Perspective, Stake};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// `Scenario` fields with a dedicated `.case` block (L4007 in `analysis`).
const DEDICATED: [&str; 5] = ["start", "perspective", "facts", "payoffs", "observe"];

/// `perspective`: a party makes its actor `self` and every other party's
/// actor `opponent`, listing only the actors whose role changes from the
/// primary package's; an explicit actor map is emitted as written.
fn perspective(build: &Build<'_, '_>, sink: &mut Sink, case: &hir::Case) -> BTreeMap<String, Role> {
    match &case.perspective {
        None => BTreeMap::new(),
        Some(Perspective::Actors(map)) => map
            .iter()
            .map(|(a, r)| (a.clone(), crate::pack::role(*r)))
            .collect(),
        Some(Perspective::Party(party)) => {
            let Some(me) = case.parties.iter().find(|p| p.id == *party) else {
                sink.error(
                    C::InvalidParty,
                    None,
                    format!("perspective names `{party}`, which is not a party"),
                );
                return BTreeMap::new();
            };
            let roles = case
                .uses
                .first()
                .and_then(|pin| build.packages.iter().find(|p| p.manifest.name == pin.name))
                .map(|p| p.manifest.roles.clone());
            let Some(roles) = roles else {
                sink.unsupported(
                    None,
                    "`perspective <party>` needs the primary package loaded to read its roles",
                );
                return BTreeMap::new();
            };
            case.parties
                .iter()
                .map(|p| {
                    let role = if p.actor == me.actor {
                        Role::Me
                    } else {
                        Role::Opponent
                    };
                    (p.actor.clone(), role)
                })
                .filter(|(actor, role)| {
                    roles.get(actor).map(|r| crate::pack::role(*r)) != Some(*role)
                })
                .collect()
        }
    }
}

/// Builds the `Scenario` object from the dedicated blocks and `analysis`,
/// then lets litgraph's own serde (`deny_unknown_fields`) judge it.
fn scenario(build: &Build<'_, '_>, sink: &mut Sink, case: &hir::Case) -> Option<Scenario> {
    let mut object = Map::new();
    for (key, value) in &case.analysis.fields {
        if DEDICATED.contains(&key.as_str()) {
            sink.error(
                C::InvalidAnalysis,
                None,
                format!("`{key}` has its own `.case` block; it cannot appear in `analysis`"),
            );
        } else {
            object.insert(key.clone(), value.clone());
        }
    }
    let mut put = |key: &str, value: Value| {
        object.insert(key.to_owned(), value);
    };
    if let Some(start) = &case.start {
        put("start", Value::String(start.clone()));
    }
    let roles = perspective(build, sink, case);
    if !roles.is_empty() {
        put(
            "perspective",
            serde_json::to_value(roles).unwrap_or_default(),
        );
    }
    if !case.facts.is_empty() {
        put(
            "facts",
            serde_json::to_value(&case.facts).unwrap_or_default(),
        );
    }
    let payoffs: Map<String, Value> = case
        .stakes
        .iter()
        .filter_map(|(terminal, stake)| match stake {
            Stake::Value(m) => Some((terminal.clone(), Value::from(sink.money(m, None)?))),
            Stake::Unvalued => None,
        })
        .collect();
    if !payoffs.is_empty() {
        put("payoffs", Value::Object(payoffs));
    }
    let observe: Map<String, Value> = case
        .observations
        .iter()
        .map(|o| {
            let counts: Map<String, Value> = o
                .outcomes
                .iter()
                .map(|(edge, n)| (edge.clone(), Value::from(*n)))
                .collect();
            (o.node.clone(), Value::Object(counts))
        })
        .collect();
    if !observe.is_empty() {
        put("observe", Value::Object(observe));
    }
    match serde_json::from_value::<Scenario>(Value::Object(object)) {
        Ok(s) => {
            expressions(sink, &s);
            Some(s)
        }
        Err(e) => {
            sink.error_with(
                C::InvalidAnalysis,
                None,
                "the matter is not a valid litgraph scenario",
                e.to_string(),
            );
            None
        }
    }
}

/// Expression-valued fields must parse as litgraph expressions (L6003).
fn expressions(sink: &mut Sink, s: &Scenario) {
    let single = [
        ("mask", &s.mask),
        ("probability_fn", &s.probability_fn),
        ("opponent_objective", &s.opponent_objective),
        ("cost", &s.cost),
        ("utility", &s.utility),
    ];
    let named = single
        .iter()
        .filter_map(|(field, v)| Some(((*field).to_owned(), v.as_ref()?)))
        .chain(s.metrics.iter().map(|(k, v)| (format!("metrics.{k}"), v)))
        .chain(
            s.utilities
                .iter()
                .map(|(k, v)| (format!("utilities.{k}"), v)),
        );
    for (field, text) in named {
        if let Err(e) = litgraph::expr::parse(text) {
            sink.error_with(
                C::InvalidEngineExpression,
                None,
                format!("`analysis.{field}` is not a litgraph expression"),
                e.to_string(),
            );
        }
    }
}

/// `case` as a named scenario and its source map.
pub(crate) fn case(
    build: &Build<'_, '_>,
    case: &hir::Case,
    sink: &mut Sink,
) -> Option<(NamedScenario, SourceMap)> {
    let scenario = scenario(build, sink, case)?;
    let named = NamedScenario {
        id: case.id.clone(),
        summary: case.summary.clone(),
        notes: case.notes.clone().unwrap_or_default(),
        packs: case.uses.iter().map(|u| u.pack_id.clone()).collect(),
        sources: case
            .sources
            .iter()
            .map(|s| ScenarioSource {
                title: s.title.clone(),
                url: s.url.clone(),
                as_of: s.as_of.clone(),
            })
            .collect(),
        scenario,
    };
    let checked = serde_json::to_string(&named)
        .map_err(|e| e.to_string())
        .and_then(|t| NamedScenario::from_json(&t).map_err(|e| e.to_string()));
    if let Err(why) = checked {
        sink.error_with(
            C::InvalidAnalysis,
            None,
            "the matter is not a valid litgraph named scenario",
            why,
        );
        return None;
    }
    let map = SourceMap {
        kind: "scenario".to_owned(),
        id: case.id.clone(),
        package: None,
        as_of: case.as_of.clone(),
        known_at: case.known_at.clone(),
        entries: vec![Entry {
            pointer: String::new(),
            id: case.id.clone(),
            span: Some(sink.fallback.clone()),
            decl: None,
            cite: None,
        }],
        generated: crate::source_map::Generated::default(),
    };
    Some((named, map))
}
