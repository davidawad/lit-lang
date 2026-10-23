// SPDX-License-Identifier: GPL-3.0-or-later
//! `validate`: check any litgraph document (pack, links, clock library,
//! scenario, request)
//! without running an analysis. Parsing is strict (unknown fields are
//! errors); semantic checks compile the graph and resolve the scenario so
//! bad node/edge refs and expressions surface too.

use serde::Serialize;
use serde_json::Value;

use super::calibration::CalibrationSet;
use super::catalog::Catalog;
use super::{scenario_ref, Request};
use crate::clock::ClockLibrary;
use crate::error::{Error, Result};
use crate::lint::{self, Diagnostic};
use crate::model::{CompileOptions, Graph, LinkFile, Pack};
use crate::scenario::{NamedScenario, Scenario, View};

/// Outcome of validating one document.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Validation {
    /// Detected (or requested) kind.
    pub kind: &'static str,
    /// No errors.
    pub valid: bool,
    /// Errors (parse, reference, expression).
    pub errors: Vec<String>,
    /// Lint diagnostics (packs only).
    pub diagnostics: Vec<Diagnostic>,
    /// Scenario warnings (fallbacks the engine would take).
    pub warnings: Vec<crate::scenario::Warning>,
}

/// Guess a document's kind from its top-level keys.
#[must_use]
pub fn detect(doc: &Value) -> &'static str {
    let has = |k: &str| doc.get(k).is_some();
    if has("countingRules") || has("calendars") {
        // A clock-library fragment (`packs/clock/*.json`) also carries a
        // `schemaVersion`, so it must be checked before "pack".
        "clock"
    } else if has("links") || has("instances") {
        // A v2 link file also carries a `schemaVersion`.
        "links"
    } else if has("schemaVersion") {
        "pack"
    } else if has("summary") && has("scenario") {
        // A scenario-library file (`scenarios/*.json`) also has a `scenario`
        // (and often a `packs`) key, so it must be checked before "request"
        // below; `summary` is unique to that shape.
        "named-scenario"
    } else if has("op") || has("scenario") || has("packs") {
        "request"
    } else if has("entries") {
        "calibration"
    } else {
        "scenario"
    }
}

fn parse<T: serde::de::DeserializeOwned>(doc: &Value) -> Result<T> {
    serde_json::from_value(doc.clone()).map_err(|e| Error::Parse(e.to_string()))
}

fn scenario_against(
    catalog: &Catalog,
    packs: &[String],
    links: bool,
    sc: &Scenario,
    out: &mut Validation,
) {
    match catalog
        .compile(packs, links, &CompileOptions::default())
        .and_then(|g| View::new(&g, sc).map(|v| v.warnings))
    {
        Ok(w) => out.warnings = w,
        Err(e) => out.errors.push(e.to_string()),
    }
}

/// A pack's `citeSyntax` is well formed, it compiles on its own, lints,
/// and names a counting rule the catalog's clock library has.
fn validate_pack(doc: &Value, catalog: &Catalog, out: &mut Validation) {
    let compiled = parse::<Pack>(doc).and_then(|p| {
        p.check_cite_syntax()?;
        p.check_lowered()?;
        Graph::compile(
            std::slice::from_ref(&p),
            &LinkFile::default(),
            &CompileOptions::default(),
        )
        .map(|g| (p, g))
    });
    match compiled {
        Ok((p, g)) => {
            if let Some(Err(e)) = p.counting.as_deref().map(|id| catalog.clock.clock(id)) {
                out.errors.push(format!("pack {}: `counting`: {e}", p.id));
            }
            out.diagnostics = lint::lint(&g, &[p]);
        }
        Err(e) => out.errors.push(e.to_string()),
    }
}

/// A clock-library fragment parses and validates, and every calendar its
/// rules name exists in it or in the catalog's library.
fn validate_clock(doc: &Value, catalog: &Catalog, out: &mut Validation) {
    match ClockLibrary::from_json(&doc.to_string()) {
        Ok(lib) => {
            for r in &lib.counting_rules {
                for h in &r.holidays {
                    let mut known = lib.calendars.iter().chain(&catalog.clock.calendars);
                    if !known.any(|c| &c.id == h) {
                        out.errors.push(format!(
                            "counting rule {}: unknown holiday calendar {h}",
                            r.id
                        ));
                    }
                }
            }
        }
        Err(e) => out.errors.push(e.to_string()),
    }
}

/// Validate `doc` as `kind` (or the detected kind).
#[must_use]
pub fn validate(doc: &Value, kind: Option<&str>, catalog: &Catalog) -> Validation {
    let kind = match kind.unwrap_or_else(|| detect(doc)) {
        "pack" => "pack",
        "links" => "links",
        "request" => "request",
        "calibration" => "calibration",
        "named-scenario" => "named-scenario",
        "clock" => "clock",
        _ => "scenario",
    };
    let mut out = Validation {
        kind,
        valid: false,
        errors: vec![],
        diagnostics: vec![],
        warnings: vec![],
    };
    match kind {
        "pack" => validate_pack(doc, catalog, &mut out),
        "clock" => validate_clock(doc, catalog, &mut out),
        "links" => match parse::<LinkFile>(doc).and_then(|lf| lf.check().map(|()| lf)) {
            Ok(lf) => {
                let packs: Vec<Pack> = catalog.packs.iter().map(|(_, p, _)| p.clone()).collect();
                if let Err(e) = Graph::compile(&packs, &lf, &CompileOptions::default()) {
                    out.errors.push(e.to_string());
                }
            }
            Err(e) => out.errors.push(e.to_string()),
        },
        "calibration" => match parse::<CalibrationSet>(doc) {
            Ok(set) => {
                let packs: Vec<Pack> = catalog.packs.iter().map(|(_, p, _)| p.clone()).collect();
                match Graph::compile(&packs, &catalog.links, &CompileOptions::default()) {
                    Ok(g) => {
                        for e in &set.entries {
                            let found = match e.target {
                                super::calibration::CalibrationTarget::NodeAttr => {
                                    g.node(&e.target_ref).is_ok()
                                }
                                _ => g.edge(&e.target_ref).is_ok(),
                            };
                            if !found {
                                out.errors.push(format!(
                                    "calibration {}: ref {} not found against {} (not every ref needs to resolve against every pack selection at request time, but a validated calibration file should resolve against its author's packs)",
                                    set.id, e.target_ref, catalog.origin
                                ));
                            }
                        }
                    }
                    Err(e) => out.errors.push(e.to_string()),
                }
            }
            Err(e) => out.errors.push(e.to_string()),
        },
        "request" => match parse::<Request>(doc) {
            Ok(req) => match scenario_ref::resolve(&req.scenario, &req.packs, catalog) {
                Ok((sc, packs)) => scenario_against(catalog, &packs, req.links, &sc, &mut out),
                Err(e) => out.errors.push(e.to_string()),
            },
            Err(e) => out.errors.push(e.to_string()),
        },
        "named-scenario" => match parse::<NamedScenario>(doc).and_then(|d| d.check().map(|()| d)) {
            Ok(def) => scenario_against(catalog, &def.packs, true, &def.scenario, &mut out),
            Err(e) => out.errors.push(e.to_string()),
        },
        _ => match parse::<Scenario>(doc) {
            Ok(sc) => scenario_against(catalog, &[], true, &sc, &mut out),
            Err(e) => out.errors.push(e.to_string()),
        },
    }
    out.valid = out.errors.is_empty() && !out.diagnostics.iter().any(|d| d.severity == "error");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_and_validates_each_kind() -> Result<()> {
        let c = Catalog::embedded()?;
        let ok = |d: Value| validate(&d, None, &c);
        assert_eq!(ok(json!({"op": {"op": "solve"}})).kind, "request");
        assert!(ok(json!({"packs": ["cofc"], "op": {"op": "solve"}})).valid);
        assert!(!ok(json!({"op": {"op": "solve", "bogus": 1}})).valid);
        assert!(!ok(json!({"params": {"rate": 1}, "remove_edges": ["no-such-edge"]})).valid);
        assert!(ok(json!({"params": {"rate": 1}})).valid);
        assert!(!ok(json!({"cost": "hours * nope"})).valid);
        assert_eq!(ok(json!({"links": []})).kind, "links");
        assert!(ok(json!({"links": []})).valid);
        let pack = json!({"schemaVersion": 2, "id": "t", "title": "t", "startNodeId": "a",
            "nodes": [{"id": "a", "label": "a"}, {"id": "b", "kind": "terminal", "label": "b", "payoff": 1}],
            "edges": [{"from": "a", "to": "b", "label": "go", "actor": "applicant"}]});
        let v = ok(pack.clone());
        assert!(v.kind == "pack" && v.valid, "{:?}", v.errors);
        let mut bad = pack;
        bad["nodes"][0]["colour"] = json!("red");
        assert!(!ok(bad).valid);
        Ok(())
    }

    /// A structurally valid `links.json` (parses fine) whose `replaces`
    /// points at a non-existent edge fails at `Graph::compile`, not parse.
    #[test]
    fn links_compile_error_is_reported_without_panicking() -> Result<()> {
        let c = Catalog::embedded()?;
        let doc = json!({
            "links": [
                {
                    "from": "cofc::claim-accrues",
                    "to": "cofc::dismissed-time-barred",
                    "label": "bogus link",
                    "replaces": ["cofc::no-such-edge"]
                }
            ]
        });
        let v = validate(&doc, Some("links"), &c);
        assert_eq!(v.kind, "links");
        assert!(!v.valid);
        assert!(
            v.errors.iter().any(|e| e.contains("no-such-edge")),
            "{:?}",
            v.errors
        );
        Ok(())
    }

    /// A `links.json` that fails to parse at all (missing required fields)
    /// is reported as a parse error, distinct from the compile-error case
    /// above.
    #[test]
    fn links_parse_error_is_reported() -> Result<()> {
        let c = Catalog::embedded()?;
        // `RawEdge` requires `to` and `label`; only `from` is given.
        let doc = json!({ "links": [ { "from": "cofc::claim-accrues" } ] });
        let v = validate(&doc, Some("links"), &c);
        assert_eq!(v.kind, "links");
        assert!(!v.valid);
        assert_eq!(v.errors.len(), 1);
        Ok(())
    }

    /// A calibration set is detected from its `entries` key, and a bad ref
    /// (a typo'd edge id) is reported as a validation error — unlike
    /// `calibration::apply`, which silently skips an unresolved ref at
    /// request time (the two have different jobs: validate catches authoring
    /// mistakes, apply tolerates a multi-forum set used with a pack subset).
    #[test]
    fn calibration_set_detects_and_validates() -> Result<()> {
        let c = Catalog::embedded()?;
        let good = json!({
            "id": "t", "title": "t",
            "entries": [{
                "target": "edge-probability", "ref": "cofc::claim-accrues->claim-time-barred#0",
                "value": 0.1,
                "source": {"title": "t", "url": "https://example.com", "vintageStart": "2024-01-01"},
                "n": 1
            }]
        });
        assert_eq!(detect(&good), "calibration");
        let bad = json!({
            "id": "t", "title": "t",
            "entries": [{
                "target": "edge-probability", "ref": "cofc::no-such-edge",
                "value": 0.1,
                "source": {"title": "t", "url": "https://example.com", "vintageStart": "2024-01-01"},
                "n": 1
            }]
        });
        let v = validate(&bad, Some("calibration"), &c);
        assert_eq!(v.kind, "calibration");
        assert!(!v.valid);
        assert!(
            v.errors.iter().any(|e| e.contains("no-such-edge")),
            "{:?}",
            v.errors
        );
        Ok(())
    }

    /// A scenario document that fails to parse (unknown field, `deny_unknown_fields`)
    /// is reported as a parse error on the default ("scenario") kind.
    #[test]
    fn scenario_parse_error_is_reported() -> Result<()> {
        let c = Catalog::embedded()?;
        let doc = json!({ "not_a_real_scenario_field": 1 });
        let v = validate(&doc, None, &c);
        assert_eq!(v.kind, "scenario");
        assert!(!v.valid);
        assert_eq!(v.errors.len(), 1);
        Ok(())
    }

    #[test]
    fn validates_clock_fragments_and_pack_counting_references() -> Result<()> {
        let c = Catalog::embedded()?;
        let frcp6 = include_str!("../../../../packs/clock/frcp6.json");
        let doc: Value = serde_json::from_str(frcp6).map_err(|e| Error::Parse(e.to_string()))?;
        let v = validate(&doc, None, &c);
        assert!(v.kind == "clock" && v.valid, "{:?}", v.errors);
        let mut dangling = doc.clone();
        dangling["countingRules"][0]["holidays"] = json!(["no-such-calendar"]);
        let v = validate(&dangling, None, &c);
        assert!(
            !v.valid && v.errors[0].contains("no-such-calendar"),
            "{:?}",
            v.errors
        );
        let mut typo = doc;
        typo["countingRules"][0]["steps"][0]["text"] = json!("exclude {trigger_date}");
        assert!(!validate(&typo, None, &c).valid);

        let pack = |counting: &str| {
            json!({"schemaVersion": 3, "id": "t", "title": "t", "startNodeId": "a", "counting": counting,
                "nodes": [{"id": "a", "label": "a"}, {"id": "b", "kind": "terminal", "label": "b", "payoff": 1}],
                "edges": [{"from": "a", "to": "b", "label": "go", "actor": "applicant"}]})
        };
        assert!(validate(&pack("frcp6"), None, &c).valid);
        let v = validate(&pack("frcp7"), None, &c);
        assert!(!v.valid && v.errors[0].contains("frcp7"), "{:?}", v.errors);
        Ok(())
    }

    #[test]
    fn rejects_a_pack_whose_cite_syntax_names_an_unknown_source() -> Result<()> {
        let c = Catalog::embedded()?;
        let pack = |source: &str| {
            json!({"schemaVersion": 3, "id": "t", "title": "t", "startNodeId": "a",
                "sources": [{"id": "s"}],
                "citeSyntax": {"rules": [{"prefixes": ["XR"], "heading": "XR {}", "source": source}]},
                "nodes": [{"id": "a", "label": "a"}, {"id": "b", "kind": "terminal", "label": "b", "payoff": 1}],
                "edges": [{"from": "a", "to": "b", "label": "go", "actor": "applicant"}]})
        };
        assert!(validate(&pack("s"), None, &c).valid);
        let v = validate(&pack("nope"), None, &c);
        assert_eq!(
            v.errors,
            vec!["parse: pack t: citeSyntax source `nope` is not a `sources[].id` (valid: s)"]
        );
        Ok(())
    }
}
