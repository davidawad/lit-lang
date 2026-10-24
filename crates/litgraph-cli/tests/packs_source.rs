// SPDX-License-Identifier: GPL-3.0-or-later
//! Where the `litgraph` binary reads its packs from, run as a child
//! process: `--packs-dir`, else `$LITGRAPH_PACKS`, else the packs built
//! into it; a directory that does not exist is an error naming its source.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::{Command, Output};

use serde_json::Value;

const MISSING: &str = "/no/such/litgraph-packs-dir";

/// `litgraph describe --compact` with `configure` applied, and
/// `LITGRAPH_PACKS` cleared unless `configure` sets it.
fn describe(configure: impl FnOnce(&mut Command)) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_litgraph"));
    cmd.args(["describe", "--compact"])
        .env_remove("LITGRAPH_PACKS");
    configure(&mut cmd);
    cmd.output().unwrap()
}

fn packs_source(out: &Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["result"]["packs_source"].as_str().unwrap().to_string()
}

#[test]
fn neither_flag_nor_variable_reads_the_embedded_packs() {
    assert_eq!(packs_source(&describe(|_| {})), "embedded");
}

#[test]
fn an_empty_variable_counts_as_unset() {
    assert_eq!(
        packs_source(&describe(|c| {
            c.env("LITGRAPH_PACKS", "");
        })),
        "embedded"
    );
}

#[test]
fn the_variable_overrides_the_embedded_packs() {
    let out = describe(|c| {
        c.env("LITGRAPH_PACKS", litgraph_packs::PACKS_DIR);
    });
    assert_eq!(packs_source(&out), litgraph_packs::PACKS_DIR);
}

#[test]
fn the_flag_overrides_the_variable() {
    let out = describe(|c| {
        c.env("LITGRAPH_PACKS", MISSING)
            .args(["--packs-dir", litgraph_packs::PACKS_DIR]);
    });
    assert_eq!(packs_source(&out), litgraph_packs::PACKS_DIR);
}

#[test]
fn a_variable_naming_a_missing_directory_is_an_error_naming_it() {
    let out = describe(|c| {
        c.env("LITGRAPH_PACKS", MISSING);
    });
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(&format!("$LITGRAPH_PACKS: {MISSING}")),
        "{err}"
    );
}

#[test]
fn a_flag_naming_a_missing_directory_is_an_error_naming_it() {
    let out = describe(|c| {
        c.args(["--packs-dir", MISSING]);
    });
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(&format!("--packs-dir: {MISSING}")), "{err}");
}
