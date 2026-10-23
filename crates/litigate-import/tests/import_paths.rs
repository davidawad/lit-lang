//! Import paths the corpus and the demo fixture don't reach: package
//! directory fallbacks, cite holes for text with no citation, partial
//! durations, a links file or case imported without its packs, and
//! absolute input paths. Each output is checked verbatim where the
//! spelling is the point, and read back through the CST oracle (SPEC §8.6)
//! where losslessness is.
mod support;

use litgraph::model::{LinkFile, Pack};
use litgraph::scenario::NamedScenario;
use litigate_import::{ImportedFile, Input};
use serde_json::{json, Value};
use support::{
    first_difference, import_ok, normalize_links, normalize_pack, numbers_as_f64, package,
    read_case, read_links, read_pack,
};

/// A minimal valid pack `id` with `edit` applied to its JSON.
fn pack(id: &str, edit: impl FnOnce(&mut Value)) -> Input {
    let mut pack = json!({
        "schemaVersion": 2, "id": id, "title": "P", "forum": id, "startNodeId": "a",
        "roles": {"applicant": "self", "either": "nature"},
        "nodes": [{"id": "a", "label": "A"}, {"id": "b", "kind": "terminal", "label": "B"}],
        "edges": [{"id": "go", "from": "a", "to": "b", "label": "Go", "actor": "applicant"}]
    });
    edit(&mut pack);
    Input::new(format!("packs/{id}.json"), pack.to_string())
}

/// The text of the imported file at `path`.
fn text<'a>(files: &'a [ImportedFile], path: &str) -> &'a str {
    files
        .iter()
        .find(|f| f.path == path)
        .map_or_else(|| panic!("no {path} among {files:#?}"), |f| f.text.as_str())
}

/// Asserts that package `dir` of `files` reads back as the pack `input`.
fn assert_pack_round_trips(input: &Input, files: &[ImportedFile], dir: &str) {
    let original = Pack::from_json(&input.text).unwrap_or_else(|e| panic!("{e}"));
    let expected = normalize_pack(original);
    let actual = normalize_pack(read_pack(&package(files, dir)));
    assert!(
        expected == actual,
        "{dir}: {:?}",
        first_difference(&expected, &actual, "")
    );
}

#[test]
fn a_forum_that_is_not_an_identifier_or_is_shared_falls_back_to_the_pack_id() {
    let inputs = [
        pack("spaced", |p| p["forum"] = json!("Two Words")),
        pack("first", |p| p["forum"] = json!("shared")),
        pack("second", |p| p["forum"] = json!("shared")),
        pack("own", |p| p["forum"] = json!("solo")),
    ];
    let files = import_ok(&inputs);
    let roots: Vec<&str> = files
        .iter()
        .filter(|f| f.path.ends_with("Lit.toml"))
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(
        roots,
        [
            "lit/first/Lit.toml",
            "lit/second/Lit.toml",
            "lit/solo/Lit.toml",
            "lit/spaced/Lit.toml",
        ]
    );
    // The directory is only a location: the manifest keeps the authored
    // forum, and the root module is named after the directory.
    assert!(text(&files, "lit/spaced/Lit.toml").contains("forum = \"Two Words\"\n"));
    assert!(text(&files, "lit/first/Lit.toml").contains("forum = \"shared\"\n"));
    assert!(text(&files, "lit/solo/solo.lit").contains("\nmodule solo\n"));
    for (input, dir) in inputs
        .iter()
        .zip(["lit/spaced", "lit/first", "lit/second", "lit/solo"])
    {
        assert_pack_round_trips(input, &files, dir);
    }
}

#[test]
fn text_with_no_citation_is_a_cite_hole_and_partial_durations_keep_their_bounds() {
    let input = pack("holes", |p| {
        p["nodes"][0]["cite"] = json!(";");
        p["edges"][0]["authority"] = json!(" ; ");
        p["edges"][0]["duration"] = json!({"min": 1, "mode": 2});
        if let Some(edges) = p["edges"].as_array_mut() {
            let slow = json!({"from": "a", "to": "b", "label": "Slow", "actor": "applicant",
                              "duration": {"mode": 3, "max": 9.5}});
            edges.push(slow);
        }
        p["schemaVersion"] = json!(3);
        p["counting"] = json!("frcp6");
        p["citeSyntax"] = json!({"shorthands": [
            {"prefix": "usc", "code": "usc", "title": 28, "sectionParts": 1}
        ]});
    });
    let files = import_ok(std::slice::from_ref(&input));
    let module = text(&files, "lit/holes/01-ungrouped.lit");
    assert!(
        module.contains("state a: \"A\" by applicant cite ?(\";\") {"),
        "{module}"
    );
    assert!(module.contains("authority cite ?(\" ; \")"), "{module}");
    assert!(
        module.contains("duration { min 1.days, mode 2.days }"),
        "{module}"
    );
    assert!(
        module.contains("duration { mode 3.days, max 9.5.days }"),
        "{module}"
    );
    let manifest = text(&files, "lit/holes/Lit.toml");
    assert!(
        manifest.ends_with(
            "[[cite_syntax.shorthands]]\nprefix = \"usc\"\ncode = \"usc\"\ntitle = 28\nsection_parts = 1\n"
        ),
        "a shorthand with no source has no `source` key:\n{manifest}"
    );
    assert!(manifest.contains("counting = \"frcp6\"\n"), "{manifest}");
    assert_pack_round_trips(&input, &files, "lit/holes");
}

#[test]
fn a_links_file_imported_without_its_packs_depends_on_them_by_id() {
    // Only `instances`: still a links file.
    let links = json!({"instances": {
        "base@copy": {"pack": "base"},
        "base@patched": {"pack": "base", "patch_edges": {
            "go": {"flag": true, "off": false, "empty": {}, "big": 18_446_744_073_709_551_615_u64}
        }}
    }});
    let input = Input::new("packs/links.json", links.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    assert_eq!(
        text(&files, "lit/links/Lit.toml"),
        "# Imported by `litigate import` from packs/links.json.\n\
         # in_force is a placeholder: legacy JSON records no effective date.\n\
         [package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\n\
         in_force = \"0001-01-01\"\nknown_at = \"0001-01-01\"\n\n\
         [dependencies]\nbase = { version = \"^0.1.0\", path = \"../base\" }\n"
    );
    let module = text(&files, "lit/links/links.lit");
    assert!(
        module.contains("instance base@copy { pack base }"),
        "{module}"
    );
    assert!(
        module
            .contains("patch go { flag: true, off: false, empty: {}, big: 18446744073709551615 }"),
        "{module}"
    );
    let original: LinkFile = serde_json::from_str(&input.text).unwrap_or_default();
    let (expected, actual) = (
        normalize_links(original),
        normalize_links(read_links(&package(&files, "lit/links"))),
    );
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
}

#[test]
fn a_case_without_its_package_or_scenario_asserts_only_its_header() {
    // Only `summary` marks it a named scenario; it has no `scenario` object.
    let named = json!({"id": "bare", "summary": "Bare.", "packs": ["elsewhere"],
                       "sources": [{"title": "Only a title"}]});
    let input = Input::new("scenarios/bare.json", named.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    // The package is not imported, so `as_of` is the placeholder date.
    assert_eq!(
        text(&files, "cases/bare.case"),
        "// Imported by `litigate import` from scenarios/bare.json.\n\
         case bare\nuses elsewhere ^0.1.0\nas_of 0001-01-01\nsummary \"Bare.\"\n\
         source \"Only a title\"\n"
    );
    let original = NamedScenario::from_json(&input.text).unwrap_or_else(|e| panic!("{e}"));
    let as_json = |s: &NamedScenario| numbers_as_f64(serde_json::to_value(s).unwrap_or_default());
    let case = &package(&files, "cases")[0];
    let (expected, actual) = (as_json(&original), as_json(&read_case(case)));
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
}

#[test]
fn absolute_input_paths_are_shown_by_file_name_only() {
    let case = |id: &str, path: &str| {
        Input::new(
            path,
            json!({"id": id, "summary": "S", "packs": ["p"]}).to_string(),
        )
    };
    let inputs = [
        case("unix", "/home/someone/scenarios/unix.json"),
        case("windows", "C:\\Users\\someone\\windows.json"),
        case("relative", "scenarios\\relative.json"),
    ];
    let files = import_ok(&inputs);
    let shown: Vec<(&str, &str, &str)> = files
        .iter()
        .map(|f| {
            let first = f.text.lines().next().unwrap_or_default();
            (f.path.as_str(), f.from.as_str(), first)
        })
        .collect();
    assert_eq!(
        shown,
        [
            (
                "cases/relative.case",
                "scenarios\\relative.json",
                "// Imported by `litigate import` from scenarios/relative.json."
            ),
            (
                "cases/unix.case",
                "/home/someone/scenarios/unix.json",
                "// Imported by `litigate import` from unix.json."
            ),
            (
                "cases/windows.case",
                "C:\\Users\\someone\\windows.json",
                "// Imported by `litigate import` from windows.json."
            ),
        ]
    );
}

#[test]
fn a_state_whose_every_edge_carries_p_is_written_as_chance() {
    let input = pack("odds", |p| {
        p["nodes"] = json!([
            {"id": "a", "kind": "state", "label": "Explicit state"},
            {"id": "b", "label": "Implicit state"},
            {"id": "c", "kind": "state", "label": "Retry"},
            {"id": "z", "kind": "terminal", "label": "End"}
        ]);
        p["edges"] = json!([
            {"from": "a", "to": "b", "label": "Up", "probability": 0.5},
            {"from": "a", "to": "z", "label": "Down", "probability": 0.5},
            {"from": "b", "to": "c", "label": "On", "probability": 1},
            {"from": "c", "to": "c", "label": "Again", "probability": 0.25},
            {"from": "c", "to": "z", "label": "Done", "probability": 0.75}
        ]);
    });
    let files = import_ok(std::slice::from_ref(&input));
    let module = text(&files, "lit/odds/01-ungrouped.lit");
    let heads: Vec<&str> = module
        .lines()
        .filter_map(|l| l.split_once(':').map(|(head, _)| head))
        .filter(|head| {
            ["chance ", "state ", "terminal "]
                .iter()
                .any(|k| head.starts_with(k))
        })
        .collect();
    // A node with a self-loop stays `state`: the sugar forbids loops.
    assert_eq!(
        heads,
        ["chance a", "chance b", "state c", "terminal z"],
        "{module}"
    );
    assert_pack_round_trips(&input, &files, "lit/odds");
}
