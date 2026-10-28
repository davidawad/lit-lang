// SPDX-License-Identifier: GPL-3.0-or-later
//! The repository's packs, `links.json`, clock library, calibration sets
//! and named scenarios, built from `lit/` and `cases/` at compile time
//! (`build.rs`, the arguments in `build.args`) and embedded, so no
//! generated file is committed. Tests hand these to
//! [`litigate_engine::api::Catalog`] through [`catalog`] or, for a
//! directory, [`PACKS_DIR`] and [`SCENARIOS_DIR`].

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
