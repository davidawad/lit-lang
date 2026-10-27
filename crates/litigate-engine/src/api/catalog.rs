// SPDX-License-Identifier: GPL-3.0-or-later
//! Where packs come from: a set embedded at build time, or a directory.
//! The repository's own packs and scenarios are built from `lit/` and
//! `cases/` by the `litgraph-packs` crate, which hands them to
//! [`Catalog::embedded`]; this crate embeds none.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Serialize;

use crate::api::calibration::{CalibrationCatalog, CalibrationSet};
use crate::clock::ClockLibrary;
use crate::error::{Error, Result};
use crate::model::{CompileOptions, Graph, LinkFile, Pack};
use crate::scenario::NamedScenario;

/// `(file name, contents)` of files compiled into a binary: packs (with
/// `links.json`, `clock/<fragment>.json` and `calibration/<set>.json`) or
/// named scenarios, as
/// `litgraph-packs` embeds them.
pub type EmbeddedFiles = &'static [(&'static str, &'static str)];

fn owned(files: EmbeddedFiles) -> impl Iterator<Item = (String, String)> {
    files
        .iter()
        .map(|(n, t)| ((*n).to_string(), (*t).to_string()))
}

/// A pack or `links.json` file that failed to parse. Recorded, not fatal: a
/// single malformed file must not take every op down for every user of the
/// binary, so [`Catalog::from_files`] skips it and keeps loading the rest
/// (the `packs` and `describe` ops surface these; a request that names the
/// broken pack gets an ordinary `NotFound` — it was simply never loaded —
/// not a global failure).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LoadError {
    /// The file that failed to parse.
    pub file: String,
    /// The parse error.
    pub message: String,
}

/// A set of packs plus their `links.json`, and a named-scenario library.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// `embedded` or the directory the packs were read from.
    pub origin: String,
    /// `(file name, pack, content fingerprint)`, sorted by file name.
    pub packs: Vec<(String, Pack, String)>,
    /// Cross-pack links and instances. Empty (not composed) if `links.json`
    /// itself failed to parse -- see `load_errors`.
    pub links: LinkFile,
    /// Counting rules and holiday calendars (`clock/*.json` files), which
    /// a pack's `counting` field resolves against. A source with no
    /// `clock/` files gets the embedded library.
    pub clock: ClockLibrary,
    /// Calibration sets (`calibration/<set>.json` files, built from `lit/`
    /// `calibration` declarations) a scenario opts into
    /// by id (`scenario.calibration`); `$LITGRAPH_CALIBRATION` overrides
    /// them per request.
    pub calibration: CalibrationCatalog,
    /// Pack, `links.json`, `clock/` and `calibration/` files that failed to parse and were
    /// skipped, in the order encountered. Empty in the ordinary case.
    pub load_errors: Vec<LoadError>,
    /// `embedded` or the directory the scenario library was read from.
    pub scenarios_origin: String,
    /// `(file name, scenario, content fingerprint)`, sorted by file name.
    pub scenarios: Vec<(String, NamedScenario, String)>,
}

/// FNV-1a 64 content fingerprint (not cryptographic): enough to tell that a
/// pack changed between two analyses.
#[must_use]
pub fn fingerprint(bytes: &[u8]) -> String {
    format!("fnv1a64:{:016x}", fnv1a64(bytes))
}

/// FNV-1a, 64-bit.
#[must_use]
pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// `(prefix + file name, contents)` of every `*.json` directly in `dir`.
fn json_files(dir: &Path, prefix: &str) -> Result<Vec<(String, String)>> {
    let read = std::fs::read_dir(dir).map_err(|e| Error::Io(format!("{}: {e}", dir.display())))?;
    let mut files = vec![];
    for entry in read.filter_map(std::result::Result::ok) {
        let p = entry.path();
        if p.extension().is_some_and(|x| x == "json") {
            let text = std::fs::read_to_string(&p)
                .map_err(|e| Error::Io(format!("{}: {e}", p.display())))?;
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            files.push((format!("{prefix}{name}"), text));
        }
    }
    Ok(files)
}

/// The directory one half of a catalog comes from, with what chose it:
/// `flag`'s value, else `$var` (empty counts as unset), else `None`.
/// A chosen directory that does not exist is an error naming its chooser.
fn source_dir(flag: &str, value: Option<&Path>, var: &str) -> Result<Option<(PathBuf, String)>> {
    let chosen = match value {
        Some(dir) => Some((dir.to_path_buf(), flag.to_string())),
        None => std::env::var_os(var)
            .filter(|d| !d.is_empty())
            .map(|d| (PathBuf::from(d), format!("${var}"))),
    };
    match chosen {
        Some((dir, by)) if !dir.is_dir() => Err(Error::Io(format!(
            "{by}: {} is not a directory",
            dir.display()
        ))),
        chosen => Ok(chosen),
    }
}

impl Catalog {
    /// Build a catalog from `(file name, JSON text)` pairs; `links.json` is
    /// the link file, `clock/*` the clock library and `calibration/*` the
    /// calibration sets. A file that fails to parse is skipped and recorded in
    /// `load_errors` rather than failing the whole catalog -- one bad pack
    /// (hand-edited, a bad merge, truncated on disk) shouldn't cost every
    /// other pack, every op, for every user of the binary. This never
    /// returns `Err`; its `Result` is `Ok` in every case reachable from
    /// well-formed `(name, text)` pairs, kept for API stability and to
    /// match `with_scenario_files`'s signature.
    ///
    /// # Errors
    /// Never, currently; kept `Result` for forward compatibility.
    pub fn from_files(
        origin: String,
        files: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Catalog> {
        let mut packs = vec![];
        let mut links = LinkFile::default();
        let mut load_errors = vec![];
        let mut clock_files = vec![];
        let mut calibration = vec![];
        for (name, text) in files {
            if name.starts_with("clock/") {
                clock_files.push((name, text));
            } else if let Some(file) = name.strip_prefix("calibration/") {
                match CalibrationSet::from_json(&text) {
                    Ok(set) => {
                        calibration.push((file.to_owned(), set, fingerprint(text.as_bytes())));
                    }
                    Err(e) => load_errors.push(LoadError {
                        file: name.clone(),
                        message: format!("{name}: {e}"),
                    }),
                }
            } else if name == "links.json" {
                match LinkFile::from_json(&text) {
                    Ok(lf) => links = lf,
                    Err(e) => load_errors.push(LoadError {
                        file: name,
                        message: format!("links.json: {e}"),
                    }),
                }
            } else {
                match Pack::from_json(&text) {
                    Ok(pack) => packs.push((name, pack, fingerprint(text.as_bytes()))),
                    Err(e) => load_errors.push(LoadError {
                        file: name.clone(),
                        message: format!("{name}: {e}"),
                    }),
                }
            }
        }
        packs.sort_by(|a, b| a.0.cmp(&b.0));
        calibration.sort_by(|a, b| a.0.cmp(&b.0));
        let (clock, clock_errors) = if clock_files.is_empty() {
            ClockLibrary::embedded().clone()
        } else {
            ClockLibrary::merge(clock_files)
        };
        load_errors.extend(
            clock_errors
                .into_iter()
                .map(|(file, message)| LoadError { file, message }),
        );
        Ok(Catalog {
            calibration: CalibrationCatalog {
                origin: origin.clone(),
                sets: calibration,
            },
            origin,
            packs,
            links,
            clock,
            load_errors,
            scenarios_origin: String::new(),
            scenarios: vec![],
        })
    }

    /// The embedded `packs` (`litgraph-packs`' `PACKS`), with the embedded
    /// scenario library `scenarios` attached. A malformed embedded pack is
    /// skipped and recorded in `load_errors`, not fatal -- see
    /// [`Catalog::from_files`].
    ///
    /// # Errors
    /// Only if an embedded scenario is malformed (caught by the test suite;
    /// packs themselves never fail this call, see `from_files`).
    pub fn embedded(packs: EmbeddedFiles, scenarios: EmbeddedFiles) -> Result<Catalog> {
        Catalog::from_files("embedded".into(), owned(packs))?.with_embedded_scenarios(scenarios)
    }

    /// Attach the embedded scenario library `scenarios`, replacing
    /// whatever scenarios (if any) this catalog already had.
    ///
    /// # Errors
    /// Only if an embedded scenario is malformed (caught by the test suite).
    pub fn with_embedded_scenarios(self, scenarios: EmbeddedFiles) -> Result<Catalog> {
        self.with_scenario_files("embedded".into(), owned(scenarios))
    }

    /// Attach every `*.json` scenario file in `dir`, replacing whatever
    /// scenarios (if any) this catalog already had.
    ///
    /// # Errors
    /// Unreadable directory or a malformed scenario file.
    pub fn with_scenarios_dir(self, dir: &Path) -> Result<Catalog> {
        let files = json_files(dir, "")?;
        self.with_scenario_files(dir.display().to_string(), files)
    }

    fn with_scenario_files(
        mut self,
        origin: String,
        files: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Catalog> {
        let mut scenarios = vec![];
        for (name, text) in files {
            let def = NamedScenario::from_json(&text)
                .map_err(|e| Error::Parse(format!("{name}: {e}")))?;
            if let Some((dup, ..)) = scenarios
                .iter()
                .find(|(_, d, _): &&(String, NamedScenario, String)| d.id == def.id)
            {
                return Err(Error::Parse(format!(
                    "scenario {name}: id `{}` duplicates {dup}",
                    def.id
                )));
            }
            scenarios.push((name, def, fingerprint(text.as_bytes())));
        }
        scenarios.sort_by(|a, b| a.0.cmp(&b.0));
        self.scenarios_origin = origin;
        self.scenarios = scenarios;
        Ok(self)
    }

    /// Every scenario in the library.
    pub fn scenarios(&self) -> impl Iterator<Item = &NamedScenario> {
        self.scenarios.iter().map(|(_, s, _)| s)
    }

    /// Resolve a scenario ref: its `id`, or the file stem it was loaded from.
    ///
    /// # Errors
    /// `NotFound` listing the available names.
    pub fn scenario(&self, name: &str) -> Result<&NamedScenario> {
        self.scenarios
            .iter()
            .find(|(file, s, _)| s.id == name || file.strip_suffix(".json") == Some(name))
            .map(|(_, s, _)| s)
            .ok_or_else(|| {
                let ids: Vec<&str> = self
                    .scenarios
                    .iter()
                    .map(|(_, s, _)| s.id.as_str())
                    .collect();
                Error::NotFound(format!("scenario {name}; available: {}", ids.join(", ")))
            })
    }

    /// Every `*.json` in `dir` (and `links.json` if present), plus the
    /// clock library in `dir/clock/*.json` and the calibration sets in
    /// `dir/calibration/*.json` where those directories exist.
    ///
    /// # Errors
    /// Unreadable directory or malformed pack.
    pub fn load(dir: &Path) -> Result<Catalog> {
        let mut files = json_files(dir, "")?;
        let clock = dir.join("clock");
        if clock.is_dir() {
            files.extend(json_files(&clock, "clock/")?);
        }
        let calibration = dir.join("calibration");
        if calibration.is_dir() {
            files.extend(json_files(&calibration, "calibration/")?);
        }
        Catalog::from_files(dir.display().to_string(), files)
    }

    /// `$LITGRAPH_PACKS` if set, else the embedded `packs`; independently,
    /// `$LITGRAPH_SCENARIOS` if set, else the embedded `scenarios`
    /// ([`Catalog::resolve`] with no flags).
    ///
    /// # Errors
    /// As [`Catalog::resolve`].
    pub fn default_source(packs: EmbeddedFiles, scenarios: EmbeddedFiles) -> Result<Catalog> {
        Catalog::resolve(None, None, packs, scenarios)
    }

    /// The packs and the scenario library a binary serves, each half
    /// resolved on its own: the flag (`--packs-dir`, `--scenarios-dir`),
    /// else its variable (`$LITGRAPH_PACKS`, `$LITGRAPH_SCENARIOS`), else
    /// the embedded `packs`/`scenarios`. A variable set to the empty string
    /// counts as unset.
    ///
    /// # Errors
    /// `Io` naming the flag or variable if its directory does not exist;
    /// `Invalid` naming it if a packs directory holds no pack file (a
    /// stale `LITGRAPH_PACKS=packs/`: the packs are built into
    /// `target/lit-build/packs`); else as [`Catalog::load`] /
    /// [`Catalog::with_scenarios_dir`].
    pub fn resolve(
        packs_dir: Option<&Path>,
        scenarios_dir: Option<&Path>,
        packs: EmbeddedFiles,
        scenarios: EmbeddedFiles,
    ) -> Result<Catalog> {
        let catalog = match source_dir("--packs-dir", packs_dir, "LITGRAPH_PACKS")? {
            Some((dir, by)) => {
                let catalog = Catalog::load(&dir)?;
                if catalog.packs.is_empty() && catalog.load_errors.is_empty() {
                    return Err(Error::Invalid(format!(
                        "{by}: {} holds no pack file",
                        dir.display()
                    )));
                }
                catalog
            }
            None => Catalog::from_files("embedded".into(), owned(packs))?,
        };
        match source_dir("--scenarios-dir", scenarios_dir, "LITGRAPH_SCENARIOS")? {
            Some((dir, _)) => catalog.with_scenarios_dir(&dir),
            None => catalog.with_embedded_scenarios(scenarios),
        }
    }

    /// Resolve pack refs: id, forum key, file stem, or a path to a pack file.
    /// Empty (or `all`) selects every pack.
    ///
    /// # Errors
    /// `NotFound` listing the available ids.
    pub fn select(&self, refs: &[String]) -> Result<Vec<(Pack, String)>> {
        if refs.is_empty() || refs.iter().any(|r| r == "all") {
            return Ok(self
                .packs
                .iter()
                .map(|(_, p, h)| (p.clone(), h.clone()))
                .collect());
        }
        refs.iter().map(|r| self.select_one(r)).collect()
    }

    fn select_one(&self, r: &str) -> Result<(Pack, String)> {
        if Path::new(r).extension().is_some_and(|x| x == "json") && Path::new(r).exists() {
            let text = std::fs::read_to_string(r).map_err(|e| Error::Io(e.to_string()))?;
            return Ok((Pack::from_json(&text)?, fingerprint(text.as_bytes())));
        }
        self.packs
            .iter()
            .find(|(file, p, _)| {
                p.id == r || p.forum.as_deref() == Some(r) || file.strip_suffix(".json") == Some(r)
            })
            .map(|(_, p, h)| (p.clone(), h.clone()))
            .ok_or_else(|| {
                let ids: Vec<&str> = self.packs.iter().map(|(_, p, _)| p.id.as_str()).collect();
                Error::NotFound(format!("pack {r}; available: {}", ids.join(", ")))
            })
    }

    /// Compile the referenced packs (with links unless `links` is false).
    ///
    /// # Errors
    /// Selection or compilation errors.
    pub fn compile(&self, refs: &[String], links: bool, opts: &CompileOptions) -> Result<Graph> {
        let packs: Vec<Pack> = self.select(refs)?.into_iter().map(|(p, _)| p).collect();
        let lf = if links {
            self.links.clone()
        } else {
            LinkFile::default()
        };
        Graph::compile(&packs, &lf, opts)
    }
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
