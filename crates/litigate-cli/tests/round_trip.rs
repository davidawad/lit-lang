//! Compile -> solve round trip (lg-9lrj): `litigate solve` on every MVP
//! package under `lit/` and every matter under `cases/` compiles it and
//! the engine solves the result, so the compiler and the engine cannot
//! disagree on a shape they exchange.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::{collections::BTreeMap, path::Path};
use support::repo;

/// The MVP package directories (or every package when requested) and every
/// `.case` under `cases/`, repository-relative.
fn targets() -> Vec<String> {
    fn cases(dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                cases(&path, out);
            } else if path.extension().is_some_and(|e| e == "case") {
                out.push(rel(&path));
            }
        }
    }
    fn rel(path: &Path) -> String {
        path.strip_prefix(repo())
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
    let all_packages =
        std::env::var_os("LITIGATE_ROUNDTRIP_ALL").as_deref() == Some(std::ffi::OsStr::new("1"));
    // Full package census is the post-MVP ratchet; default coverage stays
    // bounded to the 16 core packages so this integration test fits its CI limit.
    let mvp = [
        "lit/core",
        "lit/links",
        "lit/examples",
        "lit/frcp",
        "lit/frap",
        "lit/frcrimp",
        "lit/bap8",
        "lit/ca1",
        "lit/ca8",
        "lit/cadc",
        "lit/cafc",
        "lit/cofc",
        "lit/itc",
        "lit/mpep",
        "lit/ptab",
        "lit/ttab",
    ];
    let mut out: Vec<String> = if all_packages {
        std::fs::read_dir(repo().join("lit"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.join("Lit.toml").is_file())
            .map(|p| rel(&p))
            .collect()
    } else {
        mvp.into_iter().map(str::to_owned).collect()
    };
    cases(&repo().join("cases"), &mut out);
    out.sort();
    out
}

/// Compile the repository sources once, then return each target's source
/// tree with the generated pack outputs overlaid. This keeps solving every
/// package and matter covered without recompiling the whole library for
/// every target.
fn compiled_catalog(targets: &[String]) -> litigate_engine::api::Catalog {
    fn collect(dir: &Path, root: &Path, out: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, root, out);
            } else if let Ok(text) = std::fs::read_to_string(&path) {
                out.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    text,
                );
            }
        }
    }

    let root = repo();
    let mut sources = BTreeMap::new();
    for dir in ["lit", "cases"] {
        collect(&root.join(dir), &root, &mut sources);
    }
    let mut args = vec!["--format".into(), "human".into(), "build".into()];
    args.extend(
        std::fs::read_to_string(root.join("crates/litgraph-packs/build.args"))
            .unwrap()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned),
    );
    // Add the packages each matter uses, alongside the 16 MVP packages.
    // This makes every matter resolvable without compiling the full post-MVP
    // package census on every default run.
    let mut build_targets: std::collections::BTreeSet<String> = targets
        .iter()
        .filter(|t| !t.ends_with(".case"))
        .cloned()
        .collect();
    for target in targets.iter().filter(|t| t.ends_with(".case")) {
        let text = std::fs::read_to_string(root.join(target)).unwrap();
        for line in text
            .lines()
            .filter(|line| line.trim_start().starts_with("uses "))
        {
            let id = line.split_whitespace().nth(1).unwrap();
            let package = std::fs::read_dir(root.join("lit"))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .find(|path| {
                    path.join("Lit.toml").is_file()
                        && std::fs::read_to_string(path.join("Lit.toml"))
                            .is_ok_and(|manifest| manifest.contains(&format!("pack_id = \"{id}\"")))
                })
                .unwrap_or_else(|| panic!("{target} uses unknown package {id}"));
            build_targets.insert(
                package
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    args.extend(build_targets);
    args.extend(targets.iter().filter(|t| t.ends_with(".case")).cloned());
    args.extend(["--out".into(), "out".into()]);
    let build = litigate_cli::execute_in_memory(sources, &args, None);
    assert_eq!(build.code, 0, "{}{}", build.stdout, build.stderr);
    let mut scenario_files = Vec::new();
    let mut files: Vec<(String, String)> = build
        .files
        .into_iter()
        .filter(|(path, _)| !path.contains(".map."))
        .filter_map(|(path, text)| {
            if let Some(path) = path.strip_prefix("out/scenarios/") {
                scenario_files.push((path.to_owned(), text));
                None
            } else {
                path.strip_prefix("out/packs/")
                    .map(|path| (path.to_owned(), text))
            }
        })
        .collect();
    for entry in std::fs::read_dir(root.join("crates/litigate-engine/clock")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push((
                format!("clock/{}", path.file_name().unwrap().to_string_lossy()),
                std::fs::read_to_string(path).unwrap(),
            ));
        }
    }
    let scenario_dir =
        std::env::temp_dir().join(format!("litigate-round-trip-{}", std::process::id()));
    std::fs::create_dir_all(&scenario_dir).unwrap();
    for (name, text) in scenario_files {
        std::fs::write(scenario_dir.join(name), text).unwrap();
    }
    let catalog = litigate_engine::api::Catalog::from_files("compiled".into(), files).unwrap();
    catalog.with_scenarios_dir(&scenario_dir).unwrap()
}

#[test]
fn every_package_and_matter_compiles_and_solves() {
    let targets = targets();
    assert!(targets.iter().any(|t| t == "lit/links"), "{targets:?}");
    assert!(
        targets.iter().any(|t| t == "cases/examples/acme.case"),
        "{targets:?}"
    );
    let catalog = compiled_catalog(&targets);
    // Solve each target independently and drop the response before the next
    // one, preserving full target coverage without parallel result retention.
    for target in targets {
        let request = if target.ends_with(".case") {
            let case = std::fs::read_to_string(repo().join(&target)).unwrap();
            let scenario = case
                .lines()
                .find(|line| line.trim_start().starts_with("case "))
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            serde_json::from_value(serde_json::json!({"scenario": scenario, "op": {"op": "solve"}}))
                .unwrap()
        } else {
            if target == "lit/links" {
                let request =
                    serde_json::from_value(serde_json::json!({"op": {"op": "solve"}})).unwrap();
                let solved =
                    serde_json::to_value(litigate_engine::api::handle(&request, &catalog)).unwrap();
                assert_eq!(solved["ok"], true, "{target}: {solved}");
                assert_eq!(solved["result"]["converged"], true, "{target}: {solved}");
                assert!(solved["result"]["value"].is_number(), "{target}: {solved}");
                continue;
            }
            let dir = Path::new(&target);
            let manifest = std::fs::read_to_string(repo().join(&dir).join("Lit.toml")).unwrap();
            let id = toml::from_str::<toml::Value>(&manifest).unwrap()["package"]["pack_id"]
                .as_str()
                .unwrap()
                .to_owned();
            serde_json::from_value(serde_json::json!({"packs": [id], "op": {"op": "solve"}}))
                .unwrap()
        };
        let solved =
            serde_json::to_value(litigate_engine::api::handle(&request, &catalog)).unwrap();
        assert_eq!(solved["ok"], true, "{target}: {solved}");
        assert_eq!(solved["result"]["converged"], true, "{target}: {solved}");
        assert!(solved["result"]["value"].is_number(), "{target}: {solved}");
    }
}
