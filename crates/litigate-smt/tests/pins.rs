//! The pinned Z3 release is declared twice: `build.rs` downloads and
//! verifies it, and `flake.nix` fetches the same zip for the offline Nix
//! sandbox. These tests fail when the version, an asset name, a SHA-256 or
//! the download URL differs between the two.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::BTreeMap;

const BUILD_RS: &str = include_str!("../build.rs");
const FLAKE_NIX: &str = include_str!("../../../flake.nix");

/// One side's pin: the version, and per `(os, arch)` the download URL and
/// SHA-256 of its asset.
#[derive(Debug, PartialEq, Eq)]
struct Pin {
    version: String,
    assets: BTreeMap<(String, String), (String, String)>,
}

/// The text between the first `start` and the next `end` after it.
fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let from = text.find(start).expect(start) + start.len();
    let len = text[from..].find(end).expect(end);
    &text[from..from + len]
}

/// Every double-quoted string in `text`, in order.
fn quoted(text: &str) -> Vec<String> {
    text.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

fn url(template: &str, version_var: &str, asset_var: &str, version: &str, asset: &str) -> String {
    template
        .replace(version_var, version)
        .replace(asset_var, asset)
}

fn build_rs_pin(text: &str) -> Pin {
    let version = between(text, "const Z3_VERSION: &str = \"", "\"").to_owned();
    let template = between(text, "let url = format!(\"", "\"");
    let strings = quoted(between(text, "const ASSETS", "];"));
    assert_eq!(
        strings.len() % 4,
        0,
        "ASSETS rows are (os, arch, asset, sha256)"
    );
    let assets = strings
        .chunks(4)
        .map(|row| {
            let url = url(template, "{Z3_VERSION}", "{asset}", &version, &row[2]);
            ((row[0].clone(), row[1].clone()), (url, row[3].clone()))
        })
        .collect();
    Pin { version, assets }
}

fn flake_pin(text: &str) -> Pin {
    let version = between(text, "z3Version = \"", "\"").to_owned();
    let template = between(
        between(text, "z3Archive = pkgs.fetchurl", "};"),
        "url = \"",
        "\"",
    );
    let mut assets = BTreeMap::new();
    let mut system = None;
    let mut name = None;
    for line in between(text, "z3Asset =", ".${system};").lines() {
        let line = line.trim();
        if let Some(key) = line.strip_suffix(" = {") {
            let (arch, os) = key.split_once('-').expect(key);
            let os = if os == "darwin" { "macos" } else { os };
            system = Some((os.to_owned(), arch.to_owned()));
        } else if line.starts_with("name = ") {
            name = quoted(line).pop();
        } else if line.starts_with("sha256 = ") {
            let asset = name.take().expect("name before sha256");
            let url = url(
                template,
                "${z3Version}",
                "${z3Asset.name}",
                &version,
                &asset,
            );
            let sha = quoted(line).pop().unwrap();
            assets.insert(system.take().expect("system before sha256"), (url, sha));
        }
    }
    Pin { version, assets }
}

#[test]
fn build_rs_and_flake_pin_the_same_z3_assets() {
    let build = build_rs_pin(BUILD_RS);
    assert_eq!(build.assets.len(), 4, "{build:?}");
    assert_eq!(build, flake_pin(FLAKE_NIX));
}

#[test]
fn a_drifted_flake_is_caught() {
    let build = build_rs_pin(BUILD_RS);
    let (url, sha) = build.assets.values().next().unwrap().clone();
    let asset = url.rsplit('/').next().unwrap();
    for (from, to) in [
        (sha.as_str(), "0".repeat(64)),
        (asset, "z3-other.zip".to_owned()),
        (
            &format!("z3Version = \"{}\"", build.version),
            "z3Version = \"0.0.0\"".to_owned(),
        ),
        (
            "releases/download/z3-${z3Version}",
            "releases/download/${z3Version}".to_owned(),
        ),
        ("x86_64-linux = {", "i686-linux = {".to_owned()),
    ] {
        let drifted = FLAKE_NIX.replacen(from, &to, 1);
        assert_ne!(drifted, FLAKE_NIX, "{from}");
        assert_ne!(build, flake_pin(&drifted), "{from} -> {to}");
    }
}
