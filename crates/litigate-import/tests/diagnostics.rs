//! Every diagnostic the importer emits that the snapshots don't pin:
//! structural errors at their JSON pointer, malformed documents at serde's
//! position with serde's message, two inputs claiming one output path, and
//! a generated file the formatter rejects, v2 holes and docket events with
//! no `.case` spelling. All are L6002 errors at the
//! input file.
use litigate_base::{DiagnosticCode, Severity};
use litigate_engine::model::{Hole, LinkFile, Lowered, Pack, Pin};
use litigate_engine::scenario::NamedScenario;
use litigate_import::{import, Input};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

/// One reported diagnostic: input file, line, column, message, and the
/// JSON pointer its note names (if any).
type Reported = (String, u32, u32, String, Option<String>);

/// Every diagnostic of a failed import, asserting each is an L6002 error.
fn reported(inputs: &[Input]) -> Vec<Reported> {
    let diagnostics = import(inputs)
        .err()
        .unwrap_or_else(|| panic!("import succeeded"));
    diagnostics
        .into_iter()
        .map(|d| {
            assert_eq!(d.code, DiagnosticCode::SemanticDiff, "{d:?}");
            assert_eq!(d.severity, Severity::Error, "{d:?}");
            let pointer = d.notes.iter().find_map(|n| {
                n.message
                    .strip_prefix("at JSON pointer `")
                    .and_then(|p| p.strip_suffix('`'))
                    .map(str::to_owned)
            });
            (d.span.file.0, d.span.line, d.span.col, d.message, pointer)
        })
        .collect()
}

/// An error at the file start under `pointer`.
fn at(file: &str, message: &str, pointer: &str) -> Reported {
    (
        file.to_owned(),
        1,
        1,
        message.to_owned(),
        Some(pointer.to_owned()),
    )
}

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

#[test]
fn package_structure_errors_name_their_json_pointer() {
    let terminal_only =
        "`outcome`, `payoff`, `payoffByFlag` and `feeShift` are valid only on terminals (SPEC §5.1)";
    let inputs = [
        pack("noforum", |p| {
            p.as_object_mut().map(|o| o.remove("forum"));
        }),
        pack("dangling", |p| p["edges"][0]["to"] = json!("nowhere")),
        pack("outcome", |p| p["nodes"][0]["outcome"] = json!(["win"])),
        pack("flagged", |p| {
            p["nodes"][0]["payoffByFlag"] = json!({"late": 5});
        }),
        pack("shifted", |p| p["nodes"][0]["feeShift"] = json!(0.5)),
    ];
    assert_eq!(
        reported(&inputs),
        [
            at(
                "packs/dangling.json",
                "`nowhere` is not a node of this pack",
                "/edges/0/to"
            ),
            at("packs/flagged.json", terminal_only, "/nodes/0"),
            at(
                "packs/noforum.json",
                "a package needs a `forum` key (SPEC §4.1)",
                "/forum"
            ),
            at("packs/outcome.json", terminal_only, "/nodes/0"),
            at("packs/shifted.json", terminal_only, "/nodes/0"),
        ]
    );
}

#[test]
fn replaces_on_a_pack_edge_is_rejected_by_pack_validation() {
    let input = pack("p", |p| p["edges"][0]["replaces"] = json!(["q::x"]));
    let message = "parse: pack p: edge a->b uses `replaces`, which is only valid in links.json";
    assert_eq!(
        reported(&[input]),
        [("packs/p.json".to_owned(), 1, 1, message.to_owned(), None)]
    );
}

#[test]
fn links_structure_errors_name_their_json_pointer() {
    let links = json!({"instances": {
        "nobase": {"pack": "p"},
        "p@q": {"pack": "p", "patch_edges": {
            "go": [1],
            "ok": {"hours": 2},
            "typed": {"label": "L", "note": null, "deadline": {"note": "part"}}
        }}
    }, "links": []});
    let inputs = [Input::new("packs/links.json", links.to_string())];
    assert_eq!(
        reported(&inputs),
        [
            at(
                "packs/links.json",
                "an instance name is `base@origin` (SPEC §3.1)",
                "/instances/nobase"
            ),
            at(
                "packs/links.json",
                "an edge patch must be a JSON object (SPEC §8.4)",
                "/instances/p@q/patch_edges/go"
            ),
            at(
                "packs/links.json",
                "a `patch` writes edge fields only; `label` has no `.lit` spelling",
                "/instances/p@q/patch_edges/typed/label"
            ),
            at(
                "packs/links.json",
                "a `patch` cannot unset `note`; it replaces a field with a value",
                "/instances/p@q/patch_edges/typed/note"
            ),
            at(
                "packs/links.json",
                "a `patch` replaces whole edge fields; this one is not: parse: the patch: `deadline`: needs `length` (or, a designated window, `designated`)",
                "/instances/p@q/patch_edges/typed"
            ),
        ]
    );
}

#[test]
fn a_case_must_use_a_package() {
    let named = json!({"id": "s", "summary": "S", "packs": [], "scenario": {}});
    let inputs = [Input::new("scenarios/s.json", named.to_string())];
    assert_eq!(
        reported(&inputs),
        [at(
            "scenarios/s.json",
            "a case `uses` at least one package (SPEC §3.2)",
            "/packs"
        )]
    );
}

#[test]
fn two_inputs_importing_to_one_path_are_reported_at_the_second() {
    let links = |path: &str| Input::new(path, json!({"links": []}).to_string());
    let inputs = [links("a/links.json"), links("b/links.json")];
    let message = |path: &str| format!("two inputs both import to `{path}`");
    assert_eq!(
        reported(&inputs),
        [
            (
                "b/links.json".to_owned(),
                1,
                1,
                message("lit/links/Lit.toml"),
                None
            ),
            (
                "b/links.json".to_owned(),
                1,
                1,
                message("lit/links/links.lit"),
                None
            ),
        ]
    );
}

/// What serde reports for a document: message, line, column.
type SerdeError = (String, u32, u32);

/// A malformed document, what its error must say, and how serde reads it.
type Malformed = (String, &'static str, fn(&str) -> SerdeError);

/// A parser note: output file, line, message.
type ParserNote = (String, u32, String);

/// The error serde reports for `text` as a `T`: message, line, column.
fn serde_error<T: DeserializeOwned>(text: &str) -> SerdeError {
    let error = serde_json::from_str::<T>(text)
        .err()
        .unwrap_or_else(|| panic!("{text} deserializes"));
    let line = u32::try_from(error.line()).unwrap_or_default();
    let col = u32::try_from(error.column()).unwrap_or_default();
    (error.to_string(), line, col)
}

#[test]
fn malformed_documents_are_reported_at_serdes_position() {
    let pretty = |value: Value| serde_json::to_string_pretty(&value).unwrap_or_default();
    let base = |edit: fn(&mut Value)| {
        let input = pack("p", edit);
        let value: Value = serde_json::from_str(&input.text).unwrap_or_default();
        pretty(value)
    };
    let cases: Vec<Malformed> = vec![
        (
            base(|p| p["nodes"] = json!(5)),
            "invalid type: integer `5`, expected a sequence",
            serde_error::<Pack>,
        ),
        (
            base(|p| {
                p.as_object_mut().map(|o| o.remove("startNodeId"));
            }),
            "missing field `startNodeId`",
            serde_error::<Pack>,
        ),
        (
            base(|p| p["edges"][0]["colour"] = json!("red")),
            "unknown field `colour`",
            serde_error::<Pack>,
        ),
        (
            pretty(json!({"links": {}})),
            "invalid type: map, expected a sequence",
            serde_error::<LinkFile>,
        ),
        (
            pretty(json!({"instances": {"p@q": {"note": "no pack"}}})),
            "missing field `pack`",
            serde_error::<LinkFile>,
        ),
        (
            pretty(json!({"links": [], "extra": 1})),
            "unknown field `extra`",
            serde_error::<LinkFile>,
        ),
        (
            pretty(json!({"id": "s", "summary": "S", "packs": "p"})),
            "invalid type: string \"p\", expected a sequence",
            serde_error::<NamedScenario>,
        ),
        (
            pretty(json!({"id": "s", "scenario": {}})),
            "missing field `summary`",
            serde_error::<NamedScenario>,
        ),
        (
            pretty(json!({"id": "s", "summary": "S", "scenario": {"bogus": 1}})),
            "unknown field `bogus`",
            serde_error::<NamedScenario>,
        ),
    ];
    for (index, (text, says, serde)) in cases.into_iter().enumerate() {
        let path = format!("inputs/{index}.json");
        let (message, line, col) = serde(&text);
        assert!(message.contains(says), "{index}: {message}");
        assert!(line > 1, "{index}: a pretty document fails past line 1");
        assert_eq!(
            reported(&[Input::new(path.clone(), text)]),
            [(path, line, col, message, None)],
            "{index}"
        );
    }
}

#[test]
fn documents_litgraph_rejects_and_non_documents_are_reported_at_the_start() {
    let one = |path: &str, text: String, message: &str| {
        assert_eq!(
            reported(&[Input::new(path, text)]),
            [(path.to_owned(), 1, 1, message.to_owned(), None)]
        );
    };
    one(
        "packs/v9.json",
        pack("v9", |p| p["schemaVersion"] = json!(9)).text,
        "parse: pack v9: unsupported schemaVersion 9",
    );
    one(
        "scenarios/blank.json",
        json!({"id": " ", "summary": "S"}).to_string(),
        "parse: scenario: `id` must not be empty",
    );
    for text in ["[1, 2]", "\"text\"", "{}"] {
        one(
            "inputs/other.json",
            text.to_owned(),
            "not a pack, links file, named scenario, or calibration set",
        );
    }
}

#[test]
fn a_generated_file_that_does_not_parse_is_reported_with_the_parser_errors() {
    // A newline in the input path ends the header comment early, so the
    // generated modules don't parse: an importer limitation that must be
    // reported, never written out.
    let input = Input::new("odd\npath.json", pack("p", |_| {}).text);
    let diagnostics = import(&[input]).err().unwrap_or_default();
    let summary: Vec<(String, String, Vec<ParserNote>)> = diagnostics
        .into_iter()
        .map(|d| {
            assert_eq!(d.code, DiagnosticCode::SemanticDiff);
            let notes = d
                .labels
                .into_iter()
                .map(|l| (l.span.file.0, l.span.line, l.message))
                .collect();
            (d.span.file.0, d.message, notes)
        })
        .collect();
    let rejected = |path: &str| {
        (
            "odd\npath.json".to_owned(),
            format!("generated `{path}` does not parse"),
            vec![(
                path.to_owned(),
                2,
                "expected `module`, found `path`".to_owned(),
            )],
        )
    };
    assert_eq!(
        summary,
        [rejected("lit/p/p.lit"), rejected("lit/p/01-ungrouped.lit")]
    );
}

/// A transform with no typed spelling is reported, not written.
#[test]
fn an_instance_transform_without_a_typed_spelling_is_reported() {
    let links = json!({"instances": {
        "base@broken": {"pack": "base", "payoff_transform": "stake *"},
        "base@ternary": {"pack": "base", "payoff_transform": "payoff > 0 ? payoff : 0"},
        "base@rate": {"pack": "base", "payoff_transform": "payoff * rate"},
        "base@lhs": {"pack": "base", "payoff_transform": "rate + payoff"},
        "base@cents": {"pack": "base", "payoff_transform": "payoff + 0.001"},
        "base@huge": {"pack": "base", "payoff_transform": "payoff * 1e400"},
        "base@negated": {"pack": "base", "payoff_transform": "-rate"}
    }});
    let inputs = [Input::new("packs/links.json", links.to_string())];
    let messages: Vec<(String, Option<String>)> =
        reported(&inputs).into_iter().map(|r| (r.3, r.4)).collect();
    insta::assert_debug_snapshot!(messages);
}

/// A scenario field `analysis` does not type, or a `null`, has no
/// `.case` spelling.
#[test]
fn untyped_analysis_fields_are_reported() {
    let named = json!({"id": "s", "summary": "S", "packs": ["p"],
                       "scenario": {"probabilities": {"p::e": 0.5}, "policy": {"p::n": "p::e"},
                                    "discount_annual": null, "params": {"rate": 0.001}}});
    let inputs = [Input::new("scenarios/s.json", named.to_string())];
    let messages: Vec<(String, Option<String>)> =
        reported(&inputs).into_iter().map(|r| (r.3, r.4)).collect();
    insta::assert_debug_snapshot!(messages);
}

#[test]
fn v2_holes_and_events_with_no_case_spelling_are_reported() {
    let hole = |pointer: &str, reason: Option<&str>| Hole {
        pointer: pointer.into(),
        reason: reason.map(str::to_owned),
    };
    let pin = Pin {
        name: "p".into(),
        version: "0.1.0".into(),
        digest: format!("sha256:{}", "0".repeat(64)),
    };
    // Sorted by pointer, as `Lowered::new` keeps them.
    let holes = vec![
        // A hole no `.case` block writes.
        hole("/scenario/params/rate", None),
        // A stake hole whose terminal also has a payoff, or with a reason.
        hole("/scenario/payoffs/p::b", None),
        hole("/scenario/payoffs/p::c", Some("why")),
        // A source `as_of` hole its `as_of` text contradicts.
        hole("/sources/0/as_of", Some("2026-06")),
    ];
    let lowered = Lowered::new(vec![pin], "2026-02-01".into(), None, holes);
    let named = json!({
        "schema_version": 2, "id": "s", "summary": "S", "packs": ["p"],
        "sources": [{"title": "T", "as_of": "2026-07"}],
        "scenario": {"payoffs": {"p::b": 5}},
        "parties": [{"id": "me", "actor": "applicant", "entity_type": "Corporation"}],
        "events": [{"date": "2026-01-02", "action": "A(\n)", "relation": "by", "party": "me"}],
        "lowered": lowered,
    });
    let inputs = [Input::new("scenarios/s.json", named.to_string())];
    let file = "scenarios/s.json";
    let no_spelling =
        |pointer: &str| format!("the hole at `{pointer}` has no `.case` spelling (SPEC §8.3)");
    assert_eq!(
        reported(&inputs),
        [
            at(
                file,
                &no_spelling("/scenario/params/rate"),
                "/lowered/holes/0"
            ),
            at(
                file,
                &no_spelling("/scenario/payoffs/p::b"),
                "/lowered/holes/1"
            ),
            at(
                file,
                &no_spelling("/scenario/payoffs/p::c"),
                "/lowered/holes/2"
            ),
            at(file, &no_spelling("/sources/0/as_of"), "/lowered/holes/3"),
            at(file, "a docket event spans lines", "/events/0"),
        ]
    );
}

#[test]
fn declaration_only_windows_and_grants_are_reported_not_dropped() {
    // A designated window and extension grants (pack v5) are a `deadline`
    // declaration's, whose trigger event and counting rule an edge window
    // does not carry: each is reported at its pointer (lg-flu.33).
    let input = pack("v5", |p| {
        p["schemaVersion"] = json!(5);
        p["edges"][0]["deadline"] = json!({
            "designated": {"minPeriod": 30, "sourceEvent": "Noticed", "sourceEventArg": "due"},
            "extensions": [{"bound": "nonExtendable", "authority": "FRCP 6(b)(2)"}]
        });
    });
    assert_eq!(
        reported(&[input]),
        [
            at(
                "packs/v5.json",
                "extension grants have no edge-window .lit spelling: they are a `deadline` declaration's `extend { ... }` (SPEC §5.3)",
                "/edges/0/deadline/extensions"
            ),
            at(
                "packs/v5.json",
                "a designated window has no edge-window .lit spelling: it is a `deadline` declaration's `designated p`, its trigger event's `Date` argument (SPEC §5.3)",
                "/edges/0/deadline/designated"
            ),
        ]
    );
}

#[test]
fn a_patch_field_of_the_wrong_type_is_not_a_whole_field() {
    let links = json!({"instances": {
        "p@q": {"pack": "p", "patch_edges": {"typed": {"hours": "many"}}}
    }, "links": []});
    let found = reported(&[Input::new("packs/links.json", links.to_string())]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0]
            .3
            .starts_with("a `patch` replaces whole edge fields; this one is not: invalid type"),
        "{found:?}"
    );
    assert_eq!(
        found[0].4.as_deref(),
        Some("/instances/p@q/patch_edges/typed")
    );
}
