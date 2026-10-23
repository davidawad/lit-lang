// SPDX-License-Identifier: GPL-3.0-or-later
//! Where packs come from: the set embedded at build time, or a directory.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Serialize;

use crate::clock::ClockLibrary;
use crate::error::{Error, Result};
use crate::model::{CompileOptions, Graph, LinkFile, Pack};
use crate::scenario::NamedScenario;

include!(concat!(env!("OUT_DIR"), "/embedded_packs.rs"));
include!(concat!(env!("OUT_DIR"), "/embedded_scenarios.rs"));

/// A pack or `links.json` file that failed to parse. Recorded, not fatal: a
/// single malformed file must not take every op down for every user of the
/// binary, so [`Catalog::from_files`] skips it and keeps loading the rest
/// (`litgraph packs`/`describe` surface these; a request that names the
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
    /// Pack, `links.json` and `clock/` files that failed to parse and were
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

impl Catalog {
    /// Build a catalog from `(file name, JSON text)` pairs; `links.json` is
    /// the link file. A file that fails to parse is skipped and recorded in
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
        for (name, text) in files {
            if name.starts_with("clock/") {
                clock_files.push((name, text));
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
            origin,
            packs,
            links,
            clock,
            load_errors,
            scenarios_origin: String::new(),
            scenarios: vec![],
        })
    }

    /// The packs compiled into this build, with the embedded scenario
    /// library attached. A malformed embedded pack is skipped and recorded
    /// in `load_errors`, not fatal -- see [`Catalog::from_files`].
    ///
    /// # Errors
    /// Only if an embedded scenario is malformed (caught by the test suite;
    /// packs themselves never fail this call, see `from_files`).
    pub fn embedded() -> Result<Catalog> {
        Catalog::from_files(
            "embedded".into(),
            EMBEDDED
                .iter()
                .map(|(n, t)| ((*n).to_string(), (*t).to_string())),
        )?
        .with_embedded_scenarios()
    }

    /// Attach the scenario library embedded in this binary, replacing
    /// whatever scenarios (if any) this catalog already had.
    ///
    /// # Errors
    /// Only if an embedded scenario is malformed (caught by the test suite).
    pub fn with_embedded_scenarios(self) -> Result<Catalog> {
        self.with_scenario_files(
            "embedded".into(),
            EMBEDDED_SCENARIOS
                .iter()
                .map(|(n, t)| ((*n).to_string(), (*t).to_string())),
        )
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
    /// clock library in `dir/clock/*.json` if that directory exists.
    ///
    /// # Errors
    /// Unreadable directory or malformed pack.
    pub fn load(dir: &Path) -> Result<Catalog> {
        let mut files = json_files(dir, "")?;
        let clock = dir.join("clock");
        if clock.is_dir() {
            files.extend(json_files(&clock, "clock/")?);
        }
        Catalog::from_files(dir.display().to_string(), files)
    }

    /// `$LITGRAPH_PACKS` if set, else the embedded packs; independently,
    /// `$LITGRAPH_SCENARIOS` if set, else the embedded scenario library.
    ///
    /// # Errors
    /// As [`Catalog::load`] / [`Catalog::embedded`] / [`Catalog::with_scenarios_dir`].
    pub fn default_source() -> Result<Catalog> {
        let catalog = match std::env::var_os("LITGRAPH_PACKS") {
            Some(d) => Catalog::load(&PathBuf::from(d))?,
            None => Catalog::from_files(
                "embedded".into(),
                EMBEDDED
                    .iter()
                    .map(|(n, t)| ((*n).to_string(), (*t).to_string())),
            )?,
        };
        match std::env::var_os("LITGRAPH_SCENARIOS") {
            Some(d) => catalog.with_scenarios_dir(&PathBuf::from(d)),
            None => catalog.with_embedded_scenarios(),
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
