//! Content-addressed cache for completed CLI compilations.
use super::Compiled;
use crate::cache::CompiledCache;
use crate::out::{Failure, Report};
use litigate_engine::api::{fingerprint, Catalog};
use litigate_engine::scenario::NamedScenario;
use litigate_sema::pkg::source::Source;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Entry {
    files: Vec<(String, String)>,
    pack_fingerprints: Vec<(String, String)>,
    calibration_fingerprints: Vec<(String, String)>,
    scenarios: Vec<(String, String)>,
    report: Value,
    packs: Vec<String>,
    scenario_ids: Vec<String>,
}

/// Returns a successful cached compilation, if the entry can be restored.
pub(super) fn load(cache: &CompiledCache, key: &str) -> Option<Compiled> {
    cache
        .get::<Entry>(key)
        .and_then(|entry| restore(entry).ok())
}

/// Stores a successful compiled result.
pub(super) fn store(cache: &CompiledCache, key: &str, compiled: &Compiled) {
    if let Some(entry) = encode(compiled) {
        cache.put(key, &entry);
    }
}

/// Content identity for all source inputs that can affect this compilation.
pub(super) fn key(
    src: &dyn Source,
    paths: &[String],
    links: bool,
    lib: &[String],
) -> Result<String, Failure> {
    let version = env!("CARGO_PKG_VERSION");
    key_with_identity(src, paths, links, lib, version)
}

fn encode(compiled: &Compiled) -> Option<Entry> {
    let mut files = compiled
        .catalog
        .packs
        .iter()
        .map(|(n, p, _)| Some((n.clone(), serde_json::to_string(p).ok()?)))
        .collect::<Option<Vec<_>>>()?;
    if !compiled.catalog.links.instances.is_empty() || !compiled.catalog.links.links.is_empty() {
        files.push((
            "links.json".into(),
            serde_json::to_string(&compiled.catalog.links).ok()?,
        ));
    }
    for (n, set, _) in &compiled.catalog.calibration.sets {
        files.push((format!("calibration/{n}"), serde_json::to_string(set).ok()?));
    }
    files.push((
        "clock/cache.json".into(),
        serde_json::to_string(&compiled.catalog.clock).ok()?,
    ));
    let scenarios = compiled
        .catalog
        .scenarios
        .iter()
        .map(|(n, s, _)| Some((n.clone(), serde_json::to_string(s).ok()?)))
        .collect::<Option<Vec<_>>>()?;
    let report = json!({
        "diagnostics": compiled.report.diagnostics,
        "provenance": compiled.report.provenance,
        "sources": compiled.report.sources,
        "profile": compiled.report.profile,
    });
    Some(Entry {
        files,
        pack_fingerprints: compiled
            .catalog
            .packs
            .iter()
            .map(|(name, _, fingerprint)| (name.clone(), fingerprint.clone()))
            .collect(),
        calibration_fingerprints: compiled
            .catalog
            .calibration
            .sets
            .iter()
            .map(|(name, _, fingerprint)| (name.clone(), fingerprint.clone()))
            .collect(),
        scenarios,
        report,
        packs: compiled.packs.clone(),
        scenario_ids: compiled.scenarios.clone(),
    })
}

fn restore(entry: Entry) -> Result<Compiled, Failure> {
    let mut catalog = Catalog::from_files("litigate".into(), entry.files).map_err(Failure::from)?;
    for (name, _, fingerprint) in &mut catalog.packs {
        if let Some((_, cached)) = entry
            .pack_fingerprints
            .iter()
            .find(|(cached_name, _)| cached_name == name)
        {
            fingerprint.clone_from(cached);
        }
    }
    for (name, _, fingerprint) in &mut catalog.calibration.sets {
        if let Some((_, cached)) = entry
            .calibration_fingerprints
            .iter()
            .find(|(cached_name, _)| cached_name == name)
        {
            fingerprint.clone_from(cached);
        }
    }
    let mut scenarios = Vec::new();
    for (name, text) in entry.scenarios {
        let scenario = NamedScenario::from_json(&text).map_err(Failure::from)?;
        scenarios.push((name, scenario, fingerprint(text.as_bytes())));
    }
    catalog.scenarios_origin = "litigate".into();
    catalog.scenarios = scenarios;
    let diagnostics = serde_json::from_value(entry.report["diagnostics"].clone())
        .map_err(|e| Failure::new(e.to_string()))?;
    let sources = serde_json::from_value(entry.report["sources"].clone())
        .map_err(|e| Failure::new(e.to_string()))?;
    let profile = serde_json::from_value(entry.report["profile"].clone())
        .map_err(|e| Failure::new(e.to_string()))?;
    let report = Report {
        diagnostics,
        provenance: entry.report["provenance"].clone(),
        sources,
        profile,
        ..Report::default()
    };
    Ok(Compiled {
        report,
        catalog,
        scenarios: entry.scenario_ids,
        packs: entry.packs,
    })
}

fn key_with_identity(
    src: &dyn Source,
    paths: &[String],
    links: bool,
    lib: &[String],
    compiler: &str,
) -> Result<String, Failure> {
    let mut hash = Sha256::new();
    hash.update(compiler.as_bytes());
    hash.update([u8::from(links)]);
    for part in paths.iter().chain(lib) {
        framed(&mut hash, part.as_bytes());
    }
    let root = src.root()?;
    let mut roots = vec![root.join("lit"), root.join("cases"), root.join("sources")];
    roots.extend(lib.iter().map(|p| root.join(p)));
    roots.extend(paths.iter().map(|p| root.join(p)));
    roots.sort();
    roots.dedup();
    let mut files = Vec::new();
    for path in roots {
        collect_inputs(src, &root, &path, &mut files)?;
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, bytes) in files {
        framed(&mut hash, name.as_bytes());
        framed(&mut hash, &bytes);
    }
    Ok(litigate_sema::pkg::lock::hex(&hash.finalize()))
}

fn collect_inputs(
    src: &dyn Source,
    root: &Path,
    path: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), Failure> {
    if src.is_file(path) {
        let relevant = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e, "lit" | "case" | "toml" | "lock" | "txt" | "world"));
        if !relevant {
            return Ok(());
        }
        let bytes = src.read(path).map_err(|e| Failure::new(e.to_string()))?;
        let name = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push((name, bytes));
    } else if src.is_dir(path) {
        for child in src.entries(path).map_err(|e| Failure::new(e.to_string()))? {
            collect_inputs(src, root, &child, files)?;
        }
    }
    Ok(())
}

fn framed(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_sema::pkg::source::MemoryTree;

    fn tree(rule: &str, lock: &str) -> MemoryTree {
        MemoryTree::new([
            ("lit/p/Lit.toml", "[package]\nid = \"p\"\n".to_owned()),
            ("lit/p/rule.lit", rule.to_owned()),
            ("cases/Lit.lock", lock.to_owned()),
            ("cases/a.case", "case a {}\n".to_owned()),
            ("sources/law.txt", "public text".to_owned()),
        ])
    }

    #[test]
    fn compiled_cache_hits_and_misses_after_source_lock_or_compiler_change() {
        let paths = ["cases/a.case".to_owned()];
        let lib = ["lit".to_owned()];
        let first = key_with_identity(
            &tree("rule one", "p 1.0.0\n"),
            &paths,
            true,
            &lib,
            "0.6.0 build-a",
        )
        .unwrap();
        let edited = key_with_identity(
            &tree("rule two", "p 1.0.0\n"),
            &paths,
            true,
            &lib,
            "0.6.0 build-a",
        )
        .unwrap();
        let relocked = key_with_identity(
            &tree("rule one", "p 1.0.1\n"),
            &paths,
            true,
            &lib,
            "0.6.0 build-a",
        )
        .unwrap();
        let compiler = key_with_identity(
            &tree("rule one", "p 1.0.0\n"),
            &paths,
            true,
            &lib,
            "0.6.1 build-b",
        )
        .unwrap();
        let dir =
            std::env::temp_dir().join(format!("litigate-compiled-key-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cache = CompiledCache::in_dir(dir.clone());
        cache.put(&first, &"compiled output");
        assert_eq!(
            cache.get::<String>(&first).as_deref(),
            Some("compiled output")
        );
        assert_eq!(cache.get::<String>(&edited), None);
        assert_eq!(cache.get::<String>(&relocked), None);
        assert_eq!(cache.get::<String>(&compiler), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
