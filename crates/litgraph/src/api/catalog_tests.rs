// SPDX-License-Identifier: GPL-3.0-or-later
//! Unit tests for `api::catalog` (split out to keep `catalog.rs` under the
//! 500-line ceiling).

use super::*;
use crate::clock::ClockLibrary;

/// The repository's packs and scenarios, as `litgraph-packs` builds them.
fn repo_catalog() -> Result<Catalog> {
    Catalog::embedded(litgraph_packs::PACKS, litgraph_packs::SCENARIOS)
}

#[test]
fn embedded_catalog_loads_and_compiles_everything() -> Result<()> {
    let c = repo_catalog()?;
    assert!(c.packs.len() >= 8, "embedded packs: {}", c.packs.len());
    assert!(!c.links.links.is_empty());
    assert!(
        c.load_errors.is_empty(),
        "embedded packs should all parse cleanly: {:?}",
        c.load_errors
    );
    let g = c.compile(&[], true, &CompileOptions::default())?;
    assert!(g.nodes.len() > 400);
    assert!(c.select(&["nope".into()]).is_err());
    assert_eq!(c.select(&["cofc".into()])?.len(), 1);
    Ok(())
}

/// The failure shape a single malformed pack must have: skipped and
/// recorded, not a hard failure that takes every other pack (and every
/// op for every user of the binary) down with it. Three files: one
/// good pack, one with invalid JSON, and a `links.json` that also fails
/// to parse -- all three loaded together, in one `from_files` call.
#[test]
fn a_malformed_pack_or_links_file_is_skipped_and_recorded_not_a_hard_failure() -> Result<()> {
    let good = r#"{
        "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "start",
        "nodes": [{"id": "start", "label": "Start", "kind": "terminal", "payoff": 1.0}],
        "edges": []
    }"#;
    let bad_pack = r#"{ "schemaVersion": 2, "id": "broken", not valid json"#;
    let bad_links = r#"{ "links": [ this is not valid json either"#;
    let c = Catalog::from_files(
        "test".into(),
        [
            ("good.json".to_string(), good.to_string()),
            ("broken.json".to_string(), bad_pack.to_string()),
            ("links.json".to_string(), bad_links.to_string()),
        ],
    )?;

    // The good pack loaded and is usable...
    assert_eq!(c.packs.len(), 1);
    assert_eq!(c.select(&["demo".into()])?.len(), 1);
    assert!(c
        .compile(&["demo".into()], true, &CompileOptions::default())
        .is_ok());

    // ...while both bad files are named, not silently dropped, and
    // don't appear as if they'd loaded (links stays the empty default).
    assert_eq!(c.load_errors.len(), 2);
    assert!(c
        .load_errors
        .iter()
        .any(|e| e.file == "broken.json" && e.message.contains("broken.json")));
    assert!(c
        .load_errors
        .iter()
        .any(|e| e.file == "links.json" && e.message.contains("links.json")));
    assert!(c.links.links.is_empty() && c.links.instances.is_empty());

    // Asking for the broken pack by name is an ordinary NotFound (it
    // was never loaded), not a panic or a global outage.
    assert!(c.select(&["broken".into()]).is_err());
    Ok(())
}

/// Every embedded pack file, checked *individually* against the strict
/// `Pack` struct, with a per-file assertion message naming exactly which
/// one failed and why.
///
/// A prior incident: two agents each added a top-level `"sources"` key
/// to the same pack, at different enough line positions that git's
/// line-based merge combined them with no conflict marker. The result
/// was syntactically valid JSON (a generic parser or `serde_json::Value`
/// tolerates a duplicate object key, keeping the last one silently), but
/// serde's derived `Deserialize` for the `Pack` struct rejects a
/// duplicate field -- so every embedded pack failed to compile at
/// startup, and because dozens of unrelated test files each
/// independently call `Catalog::embedded()`/`Pack::from_json`, the
/// *entire* suite failed at once with one root cause smeared across
/// hundreds of lines of near-identical panics, not a single clear
/// signal naming the file.
///
/// This test is the single, first, clearly-named place that class of
/// failure surfaces: it parses each embedded pack file on its own, in a
/// loop with an assertion message giving the exact file name and parse
/// error, so a regression reads as one focused failure instead of a
/// suite-wide cascade.
#[test]
fn every_embedded_pack_file_parses_individually_against_the_strict_pack_struct() {
    let mut checked = 0;
    for (name, text) in litgraph_packs::PACKS {
        if *name == "links.json" || name.starts_with("clock/") {
            continue;
        }
        if let Err(e) = Pack::from_json(text) {
            panic!("embedded pack {name} failed to parse against the Pack struct: {e}");
        }
        checked += 1;
    }
    assert!(
        checked >= 8,
        "expected at least 8 embedded packs, checked {checked}"
    );
}

#[test]
fn embedded_catalog_loads_the_scenario_library() -> Result<()> {
    let c = repo_catalog()?;
    assert_eq!(c.scenarios_origin, "embedded");
    assert!(
        c.scenarios().count() >= 4,
        "embedded scenarios: {}",
        c.scenarios().count()
    );
    assert!(c.scenario("nope").is_err());
    Ok(())
}

/// `with_scenarios_dir` reads every `*.json` in a directory and reports a
/// clear error naming the unknown id, matching pack selection.
#[test]
fn with_scenarios_dir_reads_a_directory_and_reports_unknown_names() -> Result<()> {
    let dir = std::path::Path::new(litgraph_packs::SCENARIOS_DIR);
    assert!(
        dir.is_dir(),
        "expected a scenarios/ dir at {}",
        dir.display()
    );
    let c = repo_catalog()?.with_scenarios_dir(dir)?;
    assert_eq!(c.scenarios_origin, dir.display().to_string());
    assert!(!c.scenarios().collect::<Vec<_>>().is_empty());
    let err = c.scenario("definitely-not-a-scenario").unwrap_err();
    assert!(err.to_string().contains("available:"), "{err}");
    Ok(())
}

/// `default_source` reads `$LITGRAPH_SCENARIOS` independently of
/// `$LITGRAPH_PACKS` (nextest gives every test its own process).
#[test]
fn default_source_honors_litgraph_scenarios_env_var() -> Result<()> {
    let dir = std::path::Path::new(litgraph_packs::SCENARIOS_DIR);
    std::env::set_var("LITGRAPH_SCENARIOS", dir);
    let c = Catalog::default_source(litgraph_packs::PACKS, litgraph_packs::SCENARIOS)?;
    std::env::remove_var("LITGRAPH_SCENARIOS");
    assert_eq!(c.scenarios_origin, dir.display().to_string());
    assert_eq!(c.origin, "embedded");
    Ok(())
}

#[test]
fn fingerprint_is_stable() {
    assert_eq!(fingerprint(b""), "fnv1a64:cbf29ce484222325");
    assert_ne!(fingerprint(b"a"), fingerprint(b"b"));
}

/// `default_source` reads `$LITGRAPH_PACKS` when set (nextest gives every
/// test its own process, so mutating the env var here is isolated).
#[test]
fn default_source_honors_litgraph_packs_env_var() -> Result<()> {
    let dir = std::path::Path::new(litgraph_packs::PACKS_DIR);
    assert!(dir.is_dir(), "expected a packs/ dir at {}", dir.display());
    std::env::set_var("LITGRAPH_PACKS", dir);
    let c = Catalog::default_source(litgraph_packs::PACKS, litgraph_packs::SCENARIOS)?;
    std::env::remove_var("LITGRAPH_PACKS");
    assert_eq!(c.origin, dir.display().to_string());
    assert!(!c.packs.is_empty());
    Ok(())
}

/// `compile(.., links=false, ..)` drops the link file entirely: a graph
/// compiled without links has strictly fewer (or equal) nodes than one
/// compiled with them, since cross-forum link edges add `#end` twins.
#[test]
fn compile_without_links_uses_an_empty_link_file() -> Result<()> {
    let c = repo_catalog()?;
    let with_links = c.compile(&[], true, &CompileOptions::default())?;
    let without_links = c.compile(&[], false, &CompileOptions::default())?;
    assert!(without_links.nodes.len() <= with_links.nodes.len());
    assert!(with_links.nodes.len() > without_links.nodes.len());
    Ok(())
}

/// Every shipped pack that names a counting rule resolves it against the
/// clock library, whether loaded embedded or from `packs/` + `packs/clock/`;
/// that library is the hand-authored one plus the generated fragments.
#[test]
fn every_pack_counting_reference_resolves_embedded_and_from_disk() -> Result<()> {
    let dir = std::path::Path::new(litgraph_packs::PACKS_DIR);
    let hand = &ClockLibrary::embedded().0;
    let embedded = repo_catalog()?;
    assert!(hand
        .counting_rules
        .iter()
        .all(|r| embedded.clock.counting_rules.contains(r)));
    for c in [embedded.clone(), Catalog::load(dir)?] {
        assert!(c.load_errors.is_empty(), "{:?}", c.load_errors);
        assert_eq!(c.clock, embedded.clock);
        let mut named = 0;
        for (file, p, _) in &c.packs {
            if let Some(id) = &p.counting {
                c.clock
                    .clock(id)
                    .map_err(|e| Error::Invalid(format!("{file}: {e}")))?;
                named += 1;
            }
        }
        assert!(named >= 6, "packs naming a counting rule: {named}");
    }
    Ok(())
}

/// `clock/` files replace the embedded library; a bad one is recorded.
#[test]
fn clock_files_replace_the_embedded_library_and_bad_ones_are_recorded() -> Result<()> {
    let only_cal =
        r#"{"schemaVersion": 3, "calendars": [{"id": "none", "title": "None", "holidays": []}]}"#;
    let c = Catalog::from_files(
        "t".into(),
        [
            ("clock/a.json".to_string(), only_cal.to_string()),
            ("clock/b.json".to_string(), "{ nope".to_string()),
        ],
    )?;
    assert_eq!(c.clock.calendars.len(), 1);
    assert!(c.clock.counting_rules.is_empty());
    assert_eq!(c.load_errors.len(), 1);
    assert_eq!(c.load_errors[0].file, "clock/b.json");
    let c = Catalog::from_files("t".into(), std::iter::empty())?;
    assert!(
        c.clock.clock("frcp6").is_ok(),
        "no clock/ files: embedded library"
    );
    Ok(())
}
