// SPDX-License-Identifier: GPL-3.0-or-later
//! The TTAB pack loads alone and composed with the Federal Circuit pack
//! (`cafc@ttab`, `ttab-appeal-to-cafc`), and every terminal carries an
//! authored payoff.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::model::{CompileOptions, NodeKind};

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
