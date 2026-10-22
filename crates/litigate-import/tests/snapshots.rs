//! insta snapshots of the importer's output for a fixture that uses every
//! construct it spells, and of its diagnostics for values that have no
//! lossless spelling.
mod support;

use litgraph::model::{LinkFile, Pack};
use litgraph::scenario::NamedScenario;
use litigate_import::{import, Input};
use serde_json::{json, Value};
use std::fmt::Write as _;
use support::{
    first_difference, import_ok, normalize_links, normalize_pack, package, read_case, read_links,
    read_pack,
};

fn fixture(name: &str) -> Input {
    let path = format!("tests/fixtures/{name}");
    let text =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&path))
            .unwrap_or_default();
    Input::new(path, text)
}

fn fixtures() -> Vec<Input> {
    ["demo-pack.json", "demo-links.json", "demo-scenario.json"]
        .map(fixture)
        .to_vec()
}

#[test]
fn fixture_output() {
    let files = import_ok(&fixtures());
    let listing = files.iter().fold(String::new(), |mut out, f| {
        let _ = write!(out, "==> {} <- {}\n{}", f.path, f.from, f.text);
        out
    });
    insta::assert_snapshot!(listing);
}

#[test]
fn fixture_round_trips() {
    let inputs = fixtures();
    let files = import_ok(&inputs);
    let pack = Pack::from_json(&inputs[0].text).unwrap_or_default();
    let (expected, actual) = (
        normalize_pack(pack),
        normalize_pack(read_pack(&package(&files, "lit/demo"))),
    );
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
    let links: LinkFile = serde_json::from_str(&inputs[1].text).unwrap_or_default();
    let (expected, actual) = (
        normalize_links(links),
        normalize_links(read_links(&package(&files, "lit/links"))),
    );
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
    let named = NamedScenario::from_json(&inputs[2].text).unwrap_or_else(|e| panic!("{e}"));
    let case = files
        .iter()
        .find(|f| support::is_ext(&f.path, "case"))
        .unwrap_or_else(|| panic!("no case"));
    let as_json =
        |s: &NamedScenario| support::numbers_as_f64(serde_json::to_value(s).unwrap_or_default());
    let (expected, actual) = (as_json(&named), as_json(&read_case(case)));
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
}

#[test]
fn the_oracle_notices_a_changed_value() {
    let inputs = fixtures();
    let mut files = import_ok(&inputs);
    let pack = Pack::from_json(&inputs[0].text).unwrap_or_default();
    for file in &mut files {
        file.text = file.text.replace("effort 30.hours", "effort 31.hours");
    }
    let actual = normalize_pack(read_pack(&package(&files, "lit/demo")));
    let difference = first_difference(&normalize_pack(pack), &actual, "");
    assert_eq!(difference.as_deref(), Some("/edges/0/hours: 30.0 != 31.0"));
}

/// A minimal valid pack with `edit` applied to its JSON.
fn pack_with(edit: impl FnOnce(&mut Value)) -> Input {
    let mut pack = json!({
        "schemaVersion": 2, "id": "p", "title": "P", "forum": "p", "startNodeId": "a",
        "roles": {"applicant": "self", "either": "nature"},
        "nodes": [{"id": "a", "label": "A"}, {"id": "b", "kind": "terminal", "label": "B"}],
        "edges": [{"id": "go", "from": "a", "to": "b", "label": "Go", "actor": "applicant"}]
    });
    edit(&mut pack);
    Input::new("packs/p.json", pack.to_string())
}

#[test]
fn unrepresentable_values_are_l6002_with_their_json_pointer() {
    let inputs = vec![
        pack_with(|p| p["edges"][0]["cost"] = json!(0.125)),
        pack_with(|p| p["edges"][0]["deadline"] = json!({"length": 1.5})),
        pack_with(|p| p["edges"][0]["deadline"] = json!({"length": 3, "unit": "business"})),
        pack_with(|p| p["nodes"][0]["id"] = json!("a b")),
        pack_with(|p| p["nodes"][0]["payoff"] = json!(5)),
        pack_with(|p| p["schemaVersion"] = json!(3)),
        pack_with(|p| p["edges"][0]["duration"] = json!({"mode": -2})),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, input)| {
        Input::new(
            format!("packs/p{i}.json"),
            input
                .text
                .replace("\"id\":\"p\"", &format!("\"id\":\"p{i}\"")),
        )
    })
    .collect::<Vec<_>>();
    let diagnostics = import(&inputs).err().unwrap_or_default();
    insta::assert_snapshot!(serde_json::to_string_pretty(&diagnostics).unwrap_or_default());
}

#[test]
fn invalid_documents_are_reported_at_their_position() {
    let inputs = [
        Input::new("packs/broken.json", "{\"nodes\": [}"),
        Input::new("packs/other.json", "{\"hello\": 1}"),
        pack_with(|p| p["edges"][0]["replaces"] = json!(["x::y"])),
        Input::new("packs/q.json", pack_with(|p| p["id"] = json!("q")).text),
        Input::new(
            "scenarios/s.json",
            json!({"id": "s", "summary": "S", "packs": ["q"], "sources": [{"title": "T", "as_of": "x"}],
                   "scenario": {"start": "q::a", "perspective": {"applicant": "self"}}})
            .to_string(),
        ),
    ];
    let diagnostics = import(&inputs).err().unwrap_or_default();
    insta::assert_snapshot!(serde_json::to_string_pretty(&diagnostics).unwrap_or_default());
}
