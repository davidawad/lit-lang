// SPDX-License-Identifier: GPL-3.0-or-later
//! The repository's packs, `links.json`, clock library, calibration sets
//! and named scenarios, built from `lit/` and `cases/` at compile time
//! (`build.rs`, the arguments in `build.args`) and embedded, so no
//! generated file is committed. The binaries (`litgraph`, `litgraph-mcp`, `litgraph-wasm`)
//! hand these to [`litigate_engine::api::Catalog`]; tests read them through
//! [`catalog`] or, for a directory, [`PACKS_DIR`] and [`SCENARIOS_DIR`].

use litigate_engine::api::Catalog;
use litigate_engine::error::Result;

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

/// The embedded packs with the embedded scenario library attached
/// ([`Catalog::embedded`] over [`PACKS`] and [`SCENARIOS`]).
///
/// # Errors
/// Only if an embedded scenario is malformed (the test suite checks).
pub fn catalog() -> Result<Catalog> {
    Catalog::embedded(PACKS, SCENARIOS)
}

/// [`Catalog::default_source`] over [`PACKS`] and [`SCENARIOS`]:
/// `$LITGRAPH_PACKS` and `$LITGRAPH_SCENARIOS` override them independently.
///
/// # Errors
/// As [`Catalog::default_source`].
pub fn default_source() -> Result<Catalog> {
    Catalog::default_source(PACKS, SCENARIOS)
}

/// [`Catalog::resolve`] over [`PACKS`] and [`SCENARIOS`]: each half from
/// its flag (`--packs-dir`, `--scenarios-dir`), else its variable
/// (`$LITGRAPH_PACKS`, `$LITGRAPH_SCENARIOS`), else embedded. The
/// `litgraph` and `litgraph-mcp` binaries load their catalog with this.
///
/// # Errors
/// As [`Catalog::resolve`].
pub fn resolve(
    packs_dir: Option<&std::path::Path>,
    scenarios_dir: Option<&std::path::Path>,
) -> Result<Catalog> {
    Catalog::resolve(packs_dir, scenarios_dir, PACKS, SCENARIOS)
}
