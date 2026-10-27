//! Compile -> solve round trip against litgraph-final (tag `litgraph-final`,
//! 5d202da8, the last tree whose engine read JSON inputs): every `lit/`
//! package compiles to a pack whose `solve` is litgraph-final's, every
//! request in `cases/requests/` (the former `examples/*.json`) runs to
//! litgraph-final's response, and every `calibration` declaration lowers to
//! a set semantically equal (SPEC §8.6) to its former `calibration/*.json`.
//! The expected responses are in `tests/litgraph-final/`, recorded from that
//! tree with `elapsed_ms` and `provenance` dropped at every depth (wall
//! clock and byte-level digests); `calibration/` holds that tree's sets.
//!
//! One stated normalisation: a request that removed the Court of Federal
//! Claims limitations and § 1500 edges is a matter that resolves those fact
//! nodes through `facts` instead, so its run has no `fact-unset` warning,
//! and litgraph-final's are dropped before comparing.
//!
//! A package that changes what it means after the tag changes the answers
//! that use it. Its recorded responses are then re-recorded from the
//! current tree (`LITIGATE_RERECORD=1`), so the change shows as a reviewed
//! diff of the recorded file, and listed here: `frcp-civil-procedure`'s
//! solve and `calibration-gaps`, after the FRCP reply-evidence branch
//! (`lit/frcp/02-pleading-to-trial.lit`, 2895596) and again after the
//! post-dismissal remedies (714d7ba), whose four new terminals the FRCP
//! requests stake at 0.usd as `cases/frcp-civil-case.case` does; and
//! `calibration-gaps` once more after those stakes reach every flagged
//! terminal copy (7972eff), which drops its `payoff-not-authored` warning;
//! both again after the fee motion's `fee-motion-supplemented` flag
//! (8030f2f), which doubles FRCP's flag copies (68 to 136) and their
//! residual fills, at the same value and best line. An engine change is
//! re-recorded the same way: `ptab-calibrated-institution` after explain at
//! a base node reports its replayed flag context and the others (4f4fad3).
//! The eighth-circuit bankruptcy appellate panel package was added on main
//! after litgraph-final, so its initial solve has no legacy JSON baseline;
//! its current compiled solve is recorded here to keep every package covered.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_engine::api::{handle, Catalog, Request};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use support::{litigate, repo};

/// Requests whose matter resolves the limitations and § 1500 fact nodes
/// through `facts` where litgraph-final's request removed the barred edges.
const FACTS_FIXED: [&str; 9] = [
    "cofc-1498-chain",
    "cofc-1498-risk-averse-sim",
    "cofc-1498-tornado",
    "cofc-1500-trap-compare",
    "cofc-appeal-threshold-sweep",
    "cofc-cvar-tail-averse",
    "cofc-explain-dispositive-fork",
    "cofc-general-sum-settlement",
    "cofc-prejudgment-interest-sim",
];

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The recorded response `kind/name`. With `LITIGATE_RERECORD` set,
/// `actual` (already [`comparable`]) is recorded in its place first.
fn expected(kind: &str, name: &str, actual: &Value) -> Value {
    let path = repo().join(format!(
        "crates/litigate-cli/tests/litgraph-final/{kind}/{name}.json"
    ));
    if std::env::var_os("LITIGATE_RERECORD").is_some() {
        let text = serde_json::to_string_pretty(actual).unwrap();
        std::fs::write(&path, format!("{text}\n")).unwrap();
    }
    serde_json::from_str(&read(&path)).unwrap()
}

/// `v` with `elapsed_ms` and `provenance` dropped at every depth, and, when
/// `facts_fixed`, its `fact-unset` warnings too.
fn comparable(mut v: Value, facts_fixed: bool) -> Value {
    fn strip(v: &mut Value, facts_fixed: bool) {
        match v {
            Value::Object(map) => {
                map.remove("elapsed_ms");
                map.remove("provenance");
                if let (true, Some(Value::Array(w))) = (facts_fixed, map.get_mut("warnings")) {
                    w.retain(|w| w["code"] != "fact-unset");
                }
                map.values_mut().for_each(|v| strip(v, facts_fixed));
            }
            Value::Array(items) => items.iter_mut().for_each(|v| strip(v, facts_fixed)),
            _ => {}
        }
    }
    strip(&mut v, facts_fixed);
    v
}

/// Whether a `fact-unset` warning appears at any depth.
fn has_fact_unset(v: &Value) -> bool {
    match v {
        Value::Object(map) => {
            map.get("code") == Some(&json!("fact-unset")) || map.values().any(has_fact_unset)
        }
        Value::Array(items) => items.iter().any(has_fact_unset),
        _ => false,
    }
}

/// The `litigate` arguments of a request: its first line,
/// `// litigate run cases/requests/<stem>.case --op '<json>' [--no-links]`.
fn run_line(stem: &str) -> Vec<String> {
    let path = format!("cases/requests/{stem}.case");
    let text = read(&repo().join(&path));
    let line = text.lines().next().unwrap();
    let rest = line
        .strip_prefix(&format!("// litigate run {path} --op '"))
        .unwrap_or_else(|| panic!("{path}: first line is not its `litigate run` line"));
    let (op, flags) = rest.split_once('\'').unwrap();
    let mut args = vec!["run".to_owned(), path, "--op".to_owned(), op.to_owned()];
    args.extend(flags.split_whitespace().map(str::to_owned));
    args
}

fn request(stem: &str) {
    let args = run_line(stem);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = litigate(&args);
    assert_eq!(out.code, 0, "{stem}: {}", out.stdout);
    let fixed = FACTS_FIXED.contains(&stem);
    let ours = comparable(out.json["result"].clone(), false);
    if fixed {
        assert!(!has_fact_unset(&ours), "{stem}: facts left unset");
    }
    let recorded = comparable(expected("requests", stem, &ours), fixed);
    assert_eq!(ours, recorded, "{stem}");
}

/// One test per request, so they run in parallel and fail by name.
macro_rules! requests {
    ($($name:ident: $stem:literal,)*) => {
        const REQUESTS: &[&str] = &[$($stem),*];
        $(#[test] fn $name() { request($stem) })*
    };
}

requests! {
    cafc_cert_preserves_panel_outcome: "cafc-cert-preserves-panel-outcome",
    calibration_gaps: "calibration-gaps",
    cofc_1498_chain: "cofc-1498-chain",
    cofc_1498_named_scenario: "cofc-1498-named-scenario",
    cofc_1498_risk_averse_sim: "cofc-1498-risk-averse-sim",
    cofc_1498_settle: "cofc-1498-settle",
    cofc_1498_tornado: "cofc-1498-tornado",
    cofc_1500_trap_compare: "cofc-1500-trap-compare",
    cofc_appeal_threshold_sweep: "cofc-appeal-threshold-sweep",
    cofc_cvar_tail_averse: "cofc-cvar-tail-averse",
    cofc_explain_dispositive_fork: "cofc-explain-dispositive-fork",
    cofc_general_sum_settlement: "cofc-general-sum-settlement",
    cofc_limitations_fact_known: "cofc-limitations-fact-known",
    cofc_posterior_observed_judge: "cofc-posterior-observed-judge",
    cofc_prejudgment_interest_sim: "cofc-prejudgment-interest-sim",
    cofc_voi_mock_sj_panel: "cofc-voi-mock-sj-panel",
    famous_case_noa_holiday_roll: "famous-case-noa-holiday-roll",
    frcp_answer_deadline: "frcp-answer-deadline",
    ptab_calibrated_institution: "ptab-calibrated-institution",
    ptab_ipr_estoppel_flag: "ptab-ipr-estoppel-flag",
    ptab_robust_institution: "ptab-robust-institution",
}

/// Stems of the `*.<ext>` files in repository directory `dir`.
fn stems(dir: &str, ext: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(repo().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

#[test]
fn every_request_and_every_recorded_response_is_tested() {
    assert_eq!(stems("cases/requests", "case"), REQUESTS);
    assert_eq!(
        stems("crates/litigate-cli/tests/litgraph-final/requests", "json"),
        REQUESTS
    );
}

/// Every file under `dir`, keyed by its repository-relative path.
fn collect(dir: &Path, out: &mut BTreeMap<String, String>) {
    for path in std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()) {
        if path.is_dir() {
            collect(&path, out);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            let rel = path.strip_prefix(repo()).unwrap();
            out.insert(rel.to_string_lossy().replace('\\', "/"), text);
        }
    }
}

/// A catalog over the packs, links, clock fragments and calibration sets
/// `litigate build` compiles now from `lit/` and `cases/` (the
/// `litgraph-packs` build arguments), with the shared clock library
/// (`lit/core` lowered) beside them.
fn compiled() -> Catalog {
    let mut sources = BTreeMap::new();
    for dir in ["lit", "cases"] {
        collect(&repo().join(dir), &mut sources);
    }
    let mut args: Vec<String> = ["--format", "human", "build"].map(str::to_owned).into();
    args.extend(
        read(&repo().join("crates/litgraph-packs/build.args"))
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_owned),
    );
    args.extend(["--out", "out"].map(str::to_owned));
    let run = litigate_cli::execute_in_memory(sources, &args);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    let mut files: Vec<(String, String)> = run
        .files
        .into_iter()
        .filter(|(p, _)| !p.contains(".map."))
        .filter_map(|(p, t)| Some((p.strip_prefix("out/packs/")?.to_owned(), t)))
        .collect();
    let shared = repo().join("crates/litigate-engine/clock");
    for name in stems("crates/litigate-engine/clock", "json") {
        files.push((
            format!("clock/{name}.json"),
            read(&shared.join(format!("{name}.json"))),
        ));
    }
    Catalog::from_files("compiled".into(), files).unwrap()
}

#[test]
fn every_package_compiles_and_solves_to_litgraph_final() {
    let catalog = compiled();
    assert!(catalog.load_errors.is_empty(), "{:?}", catalog.load_errors);
    let ids: Vec<&str> = catalog
        .packs
        .iter()
        .map(|(_, p, _)| p.id.as_str())
        .collect();
    let mut recorded = stems("crates/litigate-cli/tests/litgraph-final/packs", "json");
    recorded.sort();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, recorded, "a package without a recorded solve");
    for id in ids {
        let request: Request =
            serde_json::from_value(json!({"packs": [id], "op": {"op": "solve"}})).unwrap();
        // Through JSON text, as the recorded responses were printed.
        let text = serde_json::to_string(&handle(&request, &catalog)).unwrap();
        let ours = comparable(serde_json::from_str(&text).unwrap(), false);
        let recorded = comparable(expected("packs", id, &ours), false);
        assert_eq!(ours, recorded, "{id}");
    }
}

#[test]
fn every_calibration_set_lowers_to_litgraph_final() {
    let catalog = compiled();
    let ours: Vec<&str> = catalog
        .calibration
        .sets
        .iter()
        .map(|(_, s, _)| s.id.as_str())
        .collect();
    assert_eq!(
        ours,
        stems(
            "crates/litigate-cli/tests/litgraph-final/calibration",
            "json"
        ),
        "a calibration set without its litgraph-final file"
    );
    for (_, set, _) in &catalog.calibration.sets {
        let text = read(&repo().join(format!(
            "crates/litigate-cli/tests/litgraph-final/calibration/{}.json",
            set.id
        )));
        let legacy = litigate_engine::api::CalibrationSet::from_json(&text).unwrap();
        let changes = litigate_lower::semantic_diff_calibration(set, &legacy);
        assert!(changes.is_empty(), "{}: {changes:#?}", set.id);
    }
}
