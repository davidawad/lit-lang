// SPDX-License-Identifier: GPL-3.0-or-later
//! The repository's packs, `links.json`, clock library and named scenarios,
//! built from `lit/` and `cases/` at compile time (`build.rs`, the
//! arguments in `build.args`) and embedded, so no generated file is
//! committed. The binaries (`litgraph`, `litgraph-mcp`, `litgraph-wasm`)
//! hand these to [`litgraph::api::Catalog`]; tests read them through
//! [`catalog`] or, for a directory, [`PACKS_DIR`] and [`SCENARIOS_DIR`].

use litgraph::api::Catalog;
use litgraph::error::Result;

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

/// `catalog` with the embedded scenario library attached
/// ([`Catalog::with_embedded_scenarios`] over [`SCENARIOS`]).
///
/// # Errors
/// Only if an embedded scenario is malformed (the test suite checks).
pub fn with_scenarios(catalog: Catalog) -> Result<Catalog> {
    catalog.with_embedded_scenarios(SCENARIOS)
}
