//! `litigate run CASE --op OP`: compile the matter and the packages it uses,
//! then hand litgraph a catalog of exactly that output and return its
//! response verbatim as the result.
use super::check::checked_targets;
use crate::out::{Failure, Outcome, Report};
use litgraph::api::{handle, Catalog, Request};
use litgraph::clock::ClockLibrary;
use litigate_base::Profile;
use litigate_lower::{render, Document, Lowerer, Options};
use serde_json::{json, Value};

/// The op object for `--op`: a JSON object, or an op name.
///
/// # Errors
/// Text that is neither.
pub fn op_value(op: &str) -> Result<Value, Failure> {
    if op.trim_start().starts_with('{') {
        serde_json::from_str(op).map_err(|e| Failure::new(format!("--op is not JSON: {e}")))
    } else {
        Ok(json!({ "op": op }))
    }
}

/// The embedded clock library with `fragment`'s rules and calendars replacing same-id entries.
fn merged(mut lib: ClockLibrary, fragment: &ClockLibrary) -> ClockLibrary {
    lib.calendars
        .retain(|c| fragment.calendars.iter().all(|f| f.id != c.id));
    lib.counting_rules
        .retain(|r| fragment.counting_rules.iter().all(|f| f.id != r.id));
    lib.calendars.extend(fragment.calendars.iter().cloned());
    lib.counting_rules
        .extend(fragment.counting_rules.iter().cloned());
    lib
}

/// `litigate run`.
pub fn run(case: &str, op: &str, lib: &[String]) -> Outcome {
    let op = op_value(op)?;
    let mut targets = crate::ws::discover(&[case.to_owned()])?;
    // Links compose between whichever packs are loaded, as `links.json` does.
    targets.packages.extend(crate::ws::links_packages(lib));
    let c = checked_targets(&targets, Profile::Dev, None, lib)?;
    let mut report = Report {
        result: Value::Null,
        provenance: c.provenance(None, None),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        failed: false,
    };
    let matter = c
        .cases
        .first()
        .ok_or_else(|| Failure::new(format!("{case}: not a .case file")))?;
    let Some(lowered) = matter.lowered.as_ref().filter(|_| !report.has_errors()) else {
        return Ok(report);
    };
    let hir = c.session.hir();
    let used: Vec<&litigate_sema::hir::Package> = hir
        .iter()
        .copied()
        .filter(|p| matter.packages.contains(&p.manifest.name) || p.manifest.links)
        .collect();
    let lowerer = Lowerer::new(&hir);
    let mut files = Vec::new();
    let mut clock = ClockLibrary::embedded().0.clone();
    for pkg in used {
        match lowerer.package(pkg, &Options::default()) {
            Ok(out) => {
                let (name, doc) = match &out.document {
                    Document::Pack(pack) => (format!("{}.json", pack.id), render(pack)),
                    Document::Links(links) => ("links.json".to_owned(), render(links)),
                };
                if let Some((fragment, _)) = &out.clock {
                    clock = merged(clock, fragment);
                }
                match doc {
                    Ok(text) => files.push((name, text)),
                    Err(d) => report.diagnostics.push(*d),
                }
            }
            Err(errors) => report.diagnostics.extend(errors),
        }
    }
    if report.has_errors() {
        return Ok(report);
    }
    let mut catalog =
        Catalog::from_files("litigate".into(), files).map_err(|e| Failure::new(e.to_string()))?;
    catalog.clock = clock;
    let scenario_text = serde_json::to_string(&lowered.scenario).unwrap_or_default();
    let fingerprint = litgraph::api::fingerprint(scenario_text.as_bytes());
    catalog.scenarios_origin = "litigate".into();
    catalog.scenarios.push((
        format!("{}.json", lowered.scenario.id),
        lowered.scenario.clone(),
        fingerprint,
    ));
    let request: Request =
        serde_json::from_value(json!({ "scenario": lowered.scenario.id, "op": op }))
            .map_err(|e| Failure::new(format!("invalid litgraph op: {e}")))?;
    let response = handle(&request, &catalog);
    report.failed = !response.ok;
    report.result = serde_json::to_value(&response).unwrap_or_default();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_names_and_objects() {
        assert_eq!(op_value("chain").ok(), Some(json!({"op": "chain"})));
        assert_eq!(
            op_value(r#"{"op":"solve"}"#).ok(),
            Some(json!({"op": "solve"}))
        );
        assert!(op_value("{bad").is_err());
    }
}
