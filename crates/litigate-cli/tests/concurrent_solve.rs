//! `litigate solve` on a package with concurrent tracks (SPEC §5.6, §8.1;
//! lg-ww93): `spawn` and `joins` lower to the pack, the engine expands the
//! reachable markings, and a two-track package solves to the value of the
//! same procedure built by hand as one walk. The packages are synthetic.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use std::path::{Path, PathBuf};
use support_cmds::{litigate_in, scratch};

fn manifest(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
language = 1
pack_id = "{name}"
forum = "frcp"
authority = "FederalRule"
title = "Concurrent tracks ({name}, synthetic)"
description = "A motion and discovery at once."
jurisdiction = "United States district courts"
in_force = "2009-12-01"
known_at = "2025-01-01"

[package.roles]
Plaintiff = "self"
Defendant = "opponent"
Court = "nature"
"#
    )
}

/// Filing (100) starts the motion, where we settle or try, and spawns
/// discovery, where the court sanctions the other side a quarter of the
/// time. The value is -100 + max(1,000 - 10, 1,200 - 300)
/// + (0.25 × 4,000 - 50) = 1,840.
const TRACKS: &str = r#"module tracks.main

package tracks { forum frcp, authority FederalRule }

start filing

decision filing: "File" by Plaintiff
{
  file "File the complaint" -> motion { cost 100.usd, spawn [discovery] }
}
decision motion: "Motion" by Plaintiff
{
  settle "Settle" -> settled { cost 10.usd }
  try "Try" -> tried { cost 300.usd }
}
chance discovery: "Discovery" by Court
{
  sanction "Sanction" -> sanctioned p 1/4 { cost 50.usd }
  close "Close" -> closed p 3/4 { cost 50.usd }
}
terminal settled: "Settled" { payoff 1_000.usd }
terminal tried: "Tried" { payoff 1_200.usd }
terminal sanctioned: "Sanctioned" { payoff 4_000.usd }
terminal closed: "Closed" { payoff 0.usd }
"#;

/// `TRACKS` built by hand as one walk: the motion, then discovery after
/// either outcome, each terminal paying both tracks' payoffs.
const PRODUCT: &str = r#"module product.main

package product { forum frcp, authority FederalRule }

start filing

decision filing: "File" by Plaintiff
{
  file "File the complaint" -> motion { cost 100.usd }
}
decision motion: "Motion" by Plaintiff
{
  settle "Settle" -> settled-discovery { cost 10.usd }
  try "Try" -> tried-discovery { cost 300.usd }
}
chance settled-discovery: "Settled; discovery" by Court
{
  settled-sanction "Sanction" -> settled-sanctioned p 1/4 { cost 50.usd }
  settled-close "Close" -> settled-closed p 3/4 { cost 50.usd }
}
chance tried-discovery: "Tried; discovery" by Court
{
  tried-sanction "Sanction" -> tried-sanctioned p 1/4 { cost 50.usd }
  tried-close "Close" -> tried-closed p 3/4 { cost 50.usd }
}
terminal settled-sanctioned: "Settled, sanctioned" { payoff 5_000.usd }
terminal settled-closed: "Settled, closed" { payoff 1_000.usd }
terminal tried-sanctioned: "Tried, sanctioned" { payoff 5_200.usd }
terminal tried-closed: "Tried, closed" { payoff 1_200.usd }
"#;

/// The ruling joins discovery: it waits for discovery to close, then one
/// track remains. The value is -100 - 40 - 60 + 700 = 500.
const JOINED: &str = r#"module joined.main

package joined { forum frcp, authority FederalRule }

start filing

decision filing: "File" by Plaintiff
{
  file "File the complaint" -> motion { cost 100.usd, spawn [discovery] }
}
state motion: "Motion pending"
{
  rule "Rule" -> ruled by Court { cost 60.usd, joins [discovery-closed] }
}
state discovery: "Discovery"
{
  close "Close discovery" -> discovery-closed by Court { cost 40.usd }
}
state discovery-closed: "Discovery closed" { _ "Wait" -> discovery-closed by Court }
terminal ruled: "Ruled" { payoff 700.usd }
"#;

fn packages(name: &str) -> PathBuf {
    let dir = scratch(name);
    for (pkg, text) in [("tracks", TRACKS), ("product", PRODUCT), ("joined", JOINED)] {
        let root = dir.join("lit").join(pkg);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Lit.toml"), manifest(pkg)).unwrap();
        std::fs::write(root.join(format!("{pkg}.lit")), text).unwrap();
    }
    dir
}

fn value(dir: &Path, pkg: &str) -> f64 {
    let run = litigate_in(dir, &["solve", &format!("lit/{pkg}")]);
    assert_eq!(run.code, 0, "{pkg}: {}{}", run.stdout, run.stderr);
    run.json["result"]["result"]["value"].as_f64().unwrap()
}

#[test]
fn a_two_track_package_solves_to_the_value_of_its_hand_built_product() {
    let dir = packages("concurrent-solve");
    let by_hand = value(&dir, "product");
    assert!((by_hand - 1840.0).abs() < 1e-9, "{by_hand}");
    let spawned = value(&dir, "tracks");
    assert!((spawned - by_hand).abs() < 1e-9, "{spawned} vs {by_hand}");
}

#[test]
fn a_join_waits_for_the_track_it_joins() {
    let dir = packages("concurrent-join");
    let joined = value(&dir, "joined");
    assert!((joined - 500.0).abs() < 1e-9, "{joined}");
}

#[test]
fn a_package_with_tracks_checks_clean() {
    let dir = packages("concurrent-check");
    let check = litigate_in(&dir, &["check", "lit/tracks", "lit/joined"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    let codes: Vec<&str> = check.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["code"].as_str())
        .collect();
    assert!(!codes.contains(&"L6001"), "{codes:?}");
    assert!(!codes.contains(&"L5003"), "{codes:?}");
}
