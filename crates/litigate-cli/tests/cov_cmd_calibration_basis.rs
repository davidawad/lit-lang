//! Typed provenance is exposed by the CLI calibration operation.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use support_cmds::{litigate_in, repo};

#[test]
fn calibration_rows_include_basis_provenance() {
    let result = litigate_in(&repo(), &["calibration", "lit/frcp"]);
    assert_eq!(result.code, 0, "{}", result.stderr);
    let rows = result.json["result"]["result"]["probabilities"]
        .as_array()
        .unwrap();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|row| {
        row["pointer"].is_string()
            && row["basis"].is_string()
            && row["input"].is_string()
            && row["swing"].is_number()
    }));
}
