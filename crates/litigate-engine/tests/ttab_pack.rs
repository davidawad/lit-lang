// SPDX-License-Identifier: GPL-3.0-or-later
//! The TTAB pack loads alone and composed with the Federal Circuit pack
//! (`cafc@ttab`, `ttab-appeal-to-cafc`), and every terminal carries an
//! authored payoff.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::model::{CompileOptions, NodeKind};

const TTAB: &str = "ttab-trademark-trial-and-appeal-board";

#[test]
fn ttab_pack_compiles_alone_and_with_cafc() {
    let catalog = litgraph_packs::catalog().expect("embedded catalog compiles");
    for packs in [
        vec![TTAB.to_string()],
        vec![TTAB.to_string(), "cafc".to_string()],
    ] {
        let g = catalog
            .compile(&packs, true, &CompileOptions::default())
            .unwrap_or_else(|e| panic!("{packs:?} failed to compile: {e}"));
        let terminals = g
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Terminal)
            .count();
        assert!(terminals >= 13, "{packs:?}: only {terminals} terminals");
    }
}

#[test]
fn a_designated_answer_window_is_reported_not_counted() {
    // Pack v5: the answer date is the notice's (37 C.F.R. § 2.105(a)), so
    // the `deadlines` op counts no date from the notice in its place.
    let catalog = litgraph_packs::catalog().expect("embedded catalog compiles");
    let request = format!(
        r#"{{"packs": ["{TTAB}"], "op": {{"op": "deadlines", "trigger": "2026-07-06", "node": "instituted"}}}}"#
    );
    let response = litigate_engine::api::handle_json(&request, &catalog);
    assert!(response.ok, "{:?}", response.error);
    let result = response.result.unwrap();
    assert_eq!(result["count"], 0, "{result}");
    let warning = response
        .warnings
        .iter()
        .find(|w| w.code == "deadline-uncomputable")
        .expect("a deadline-uncomputable warning");
    assert_eq!(warning.count, 3, "{warning:?}");
    assert!(
        warning.example.contains(
            "the deadline is designated by `InstitutionNoticed`'s `due` in each matter (at least 30 calendar days from it), not counted from a trigger"
        ),
        "{warning:?}"
    );
}
