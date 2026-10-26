//! The legal-change examples (docs/litigate/CHANGE.md §4): one package per
//! instrument class for the deadline domain, each on its vendored
//! instrument text, and the worlds that layer them. `litigate check` finds
//! the `.world` files with the packages and reports each world's layers.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use support::{codes, litigate, scratch};

const EXAMPLES: &str = "docs/litigate/examples/change";

#[test]
fn every_example_layer_and_world_checks() {
    let run = litigate(&["check", EXAMPLES]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    // Only the corpus's own history notes: no error, no other warning.
    let found = codes(&run.json);
    assert!(
        found.iter().all(|(code, severity)| severity != "error"
            && (severity == "info" || code == "L9002" || code == "L1010")),
        "{found:?}"
    );
    let classes: Vec<(String, String)> = run.json["result"]["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|w| w["layers"].as_array().unwrap().iter())
        .map(|l| {
            (
                l["name"].as_str().unwrap().to_owned(),
                l["class"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    // Every class has its example in some world.
    for class in [
        "base",
        "amendment",
        "local-rule",
        "general-order",
        "standing-order",
        "case-order",
        "emergency-declaration",
        "guidance",
        "binding precedent",
        "overruling",
    ] {
        assert!(
            classes.iter().any(|(_, c)| c == class),
            "{class}: {classes:?}"
        );
    }
    let worlds: Vec<&Value> = run.json["result"]["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .collect();
    insta::assert_json_snapshot!("change_worlds", worlds);
}

#[test]
fn a_counterfactual_world_fails_the_strict_profile() {
    let world = format!("{EXAMPLES}/worlds/frcp87-what-if.world");
    let run = litigate(&["check", "--profile", "strict", &world, EXAMPLES]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let found = codes(&run.json);
    assert!(
        found.iter().any(|(c, s)| c == "L1014" && s == "error"),
        "{found:?}"
    );
}

#[test]
fn a_world_alone_finds_its_layers_in_the_library() {
    // Layers resolve among the loaded packages, else in the library
    // (`--lib`, default `lit`).
    let dir = scratch("world-alone");
    let world = dir.join("w.world");
    std::fs::write(
        &world,
        "layers = [\"frcp-civil-procedure\"]\n\n[location]\nfamily = \"district-civil\"\ncircuit = \"ca9\"\n",
    )
    .unwrap();
    let run = litigate(&["check", world.to_str().unwrap()]);
    let layers = &run.json["result"]["worlds"][0]["layers"];
    assert_eq!(layers[0]["name"], "frcp-civil-procedure", "{}", run.stdout);
    assert_eq!(layers[0]["class"], "base");
    let missing = dir.join("missing.world");
    std::fs::write(
        &missing,
        "layers = [\"no-such-package\"]\n\n[location]\nfamily = \"ptab\"\n",
    )
    .unwrap();
    let run = litigate(&["check", missing.to_str().unwrap()]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(codes(&run.json).iter().any(|(c, _)| c == "L1013"));
    let unreadable = litigate(&["check", dir.join("absent.world").to_str().unwrap()]);
    assert_eq!(unreadable.code, 1, "{}", unreadable.stdout);
}
