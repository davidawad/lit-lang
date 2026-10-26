//! `litigate run CASE --op OP`: compile the matter and the packages it uses,
//! then hand litgraph a catalog of exactly that output and return its
//! response verbatim as the result.
use super::build::text;
use super::check::checked_targets;
use crate::out::{Failure, Outcome, Report};
use litigate_base::Profile;
use litigate_engine::api::{handle, Catalog, Request};
use litigate_engine::clock::ClockLibrary;
use litigate_lower::{Document, Options};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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

/// `litigate run`; `links` composes the links packages between the used
/// ones. The run sees the closure of what the matter `uses` and the links
/// packages it composes: no other package is loaded, and a dependency of a
/// links package outside the matter's closure is checked (the links
/// package needs it) but never reported, so it cannot set `ok: false`.
pub fn run(case: &str, op: &str, links: bool, lib: &[String]) -> Outcome {
    let op = op_value(op)?;
    let mut targets = crate::ws::discover(&[case.to_owned()])?;
    let used: BTreeSet<String> = targets
        .cases
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .flat_map(|t| crate::ws::uses_of(&t))
        .collect();
    if links {
        // Links compose between whichever packs are loaded, as `links.json` does.
        targets
            .packages
            .extend(crate::ws::links_packages(lib, &targets.cases, &used));
    }
    let c = checked_targets(&targets, Profile::Dev, &Options::default(), lib)?;
    let session = &c.session;
    let mut scope = session.closure(used);
    scope.extend(session.requested().map(|p| p.name.clone()));
    let mut report = Report {
        result: Value::Null,
        provenance: crate::prov::session(
            session.packages.iter().filter(|p| scope.contains(&p.name)),
            None,
            None,
        ),
        sources: session.sources(),
        diagnostics: session.scoped(&scope, c.diagnostics.clone()),
        check_render: None,
        profile: Some(Profile::Dev),
        failed: false,
    };
    let matter = c
        .cases
        .first()
        .ok_or_else(|| Failure::new(format!("{case}: not a .case file")))?;
    let Some(lowered) = matter.lowered.as_ref().filter(|_| !report.has_errors()) else {
        return Ok(report);
    };
    let mut files = Vec::new();
    let mut clock = ClockLibrary::embedded().0.clone();
    // Verification lowered every package at the default dates already, and
    // a clean check means each used one lowered.
    let used = c
        .session
        .packages
        .iter()
        .zip(&c.verdicts)
        .filter(|(p, _)| {
            matter.packages.contains(&p.name)
                || p.check.package.as_ref().is_some_and(|h| h.manifest.links)
        })
        .filter_map(|(_, (_, v))| v.lowered.as_ref());
    for out in used {
        files.push(match &out.document {
            Document::Pack(pack) => (format!("{}.json", pack.id), text(pack)),
            Document::Links(links) => ("links.json".to_owned(), text(links)),
        });
        if let Some((fragment, _)) = &out.clock {
            clock = merged(clock, fragment);
        }
    }
    let request = serde_json::from_value::<Request>(
        json!({ "scenario": lowered.scenario.id, "op": op, "links": links }),
    )
    .map_err(|e| Failure::new(format!("invalid litgraph op: {e}")));
    // `from_files` records unreadable files in the catalog rather than failing.
    let (mut catalog, request) = Catalog::from_files("litigate".into(), files)
        .map_err(Failure::from)
        .and_then(|catalog| Ok((catalog, request?)))?;
    catalog.clock = clock;
    let scenario_text = serde_json::to_string(&lowered.scenario).unwrap_or_default();
    let fingerprint = litigate_engine::api::fingerprint(scenario_text.as_bytes());
    catalog.scenarios_origin = "litigate".into();
    catalog.scenarios.push((
        format!("{}.json", lowered.scenario.id),
        lowered.scenario.clone(),
        fingerprint,
    ));
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
