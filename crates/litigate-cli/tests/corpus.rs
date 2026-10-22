//! The whole legacy corpus imports (round-tripping under SPEC §8.6),
//! checks, and runs: a named scenario run through `litigate run` gives the
//! same answer as litgraph's own embedded scenario.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{litigate, litigate_in, repo, scratch};

fn corpus() -> Vec<String> {
    let mut paths = Vec::new();
    for dir in ["packs", "scenarios"] {
        for entry in std::fs::read_dir(repo().join(dir)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "json") {
                paths.push(path.to_string_lossy().into_owned());
            }
        }
    }
    paths.sort();
    paths
}

#[test]
fn import_check_run_matches_litgraph() {
    let out = scratch("corpus");
    let mut args = vec![
        "import".to_owned(),
        "--out".to_owned(),
        out.to_string_lossy().into_owned(),
    ];
    args.extend(corpus());
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
    let catalog = litgraph::api::Catalog::embedded().unwrap();
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
