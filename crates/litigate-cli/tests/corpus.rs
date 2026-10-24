//! The whole legacy corpus imports (round-tripping under SPEC §8.6),
//! checks, and runs: a named scenario run through `litigate run` gives the
//! same answer as litgraph's own embedded scenario.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{litigate, litigate_in, scratch};

/// The built corpus, copied into `dir` in the legacy format the importer
/// reads: a designated window and extension grants (pack v5, links v3)
/// are a `deadline` declaration's, which import reports (L6002) rather
/// than spells, so the window is dropped and the grants left undeclared,
/// as lowering emitted them before v5.
fn corpus(dir: &std::path::Path) -> Vec<String> {
    let mut paths = Vec::new();
    for (kind, built) in [
        ("packs", litgraph_packs::PACKS_DIR),
        ("scenarios", litgraph_packs::SCENARIOS_DIR),
    ] {
        std::fs::create_dir_all(dir.join(kind)).unwrap();
        for entry in std::fs::read_dir(built).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "json") {
                let copy = dir.join(kind).join(path.file_name().unwrap());
                let text = std::fs::read_to_string(&path).unwrap();
                std::fs::write(&copy, legacy(&text)).unwrap();
                paths.push(copy.to_string_lossy().into_owned());
            }
        }
    }
    paths.sort();
    paths
}

fn legacy(text: &str) -> String {
    let mut doc: serde_json::Value = serde_json::from_str(text).unwrap();
    for key in ["edges", "links"] {
        let edges = doc.get_mut(key).and_then(serde_json::Value::as_array_mut);
        for edge in edges
            .into_iter()
            .flatten()
            .filter_map(|e| e.as_object_mut())
        {
            if edge
                .get("deadline")
                .is_some_and(|d| d.get("designated").is_some())
            {
                edge.remove("deadline");
            } else if let Some(d) = edge.get_mut("deadline").and_then(|d| d.as_object_mut()) {
                d.remove("extensions");
            }
        }
    }
    doc.to_string()
}

#[test]
fn import_check_run_matches_litgraph() {
    let out = scratch("corpus");
    let inputs = scratch("corpus-inputs");
    let mut args = vec![
        "import".to_owned(),
        "--out".to_owned(),
        out.to_string_lossy().into_owned(),
    ];
    args.extend(corpus(&inputs));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let import = litigate(&refs);
    assert_eq!(import.code, 0, "{}", import.stdout);
    assert!(import.json["result"]["files"]
        .as_array()
        .is_some_and(|f| f.len() > 20));

    let lock = litigate_in(&out, &["lock", "lit/links"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let stale = litigate_in(&out, &["check", "cases"]);
    assert_eq!(stale.code, 2, "a matter without a Lit.lock is L1007");
    let lock = litigate_in(&out, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let locked = litigate_in(&out, &["lock", "--locked", "cases"]);
    assert_eq!(locked.code, 0, "{}", locked.stdout);
    let check = litigate_in(&out, &["check", "lit", "cases"]);
    assert_eq!(check.code, 0, "{}", check.stdout);

    let ours = litigate_in(
        &out,
        &["run", "cases/cofc-1498-patent-case.case", "--op", "solve"],
    );
    assert_eq!(ours.code, 0, "{}", ours.stdout);
    let catalog = litgraph_packs::catalog().unwrap();
    let request: litgraph::api::Request = serde_json::from_value(
        serde_json::json!({"scenario": "cofc-1498-patent-case", "op": {"op": "solve"}}),
    )
    .unwrap();
    let theirs = litgraph::api::handle(&request, &catalog);
    assert!(theirs.ok);
    assert_eq!(
        ours.json["result"]["result"]["value"],
        theirs.result.unwrap()["value"]
    );
}

#[test]
fn a_pack_with_no_lossless_spelling_is_rejected() {
    let dir = scratch("bad-import");
    std::fs::write(dir.join("x.json"), "{\"nodes\": 3}").unwrap();
    let run = litigate_in(&dir, &["import", "x.json", "--out", "o"]);
    assert_eq!(run.code, 2);
    assert!(!dir.join("o").exists());
}
