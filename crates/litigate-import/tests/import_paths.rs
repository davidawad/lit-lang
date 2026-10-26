//! Import paths the corpus and the demo fixture don't reach: package
//! directory fallbacks, cite holes for text with no citation, partial
//! durations, a links file or case imported without its packs, and
//! absolute input paths. Each output is checked verbatim where the
//! spelling is the point, and read back through the CST oracle (SPEC §8.6)
//! where losslessness is.
mod support;

use litigate_engine::model::{Hole, LinkFile, Lowered, Pack, Pin};
use litigate_engine::scenario::NamedScenario;
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
            "go": {
                "note": "x",
                "tags": [],
                "attrs": {},
                "sets": ["f"],
                "cost": null,
                "deadline": {"length": 45, "extendable": null, "note": "n"}
            },
            "stay": {"attrs": {"w": 1}}
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
        module.contains(
            "patch go { deadline 45.days { note \"n\" }, note \"x\", sets [f], tags [], attrs {}, cost ? }\n  patch stay { attrs { w: 1 } }"
        ),
        "{module}"
    );
    // A patch replaces whole fields: the null `extendable` only cleared
    // the base deadline's, which no longer exists after the import.
    let mut original: LinkFile = serde_json::from_str(&input.text).unwrap_or_default();
    for instance in original.instances.values_mut() {
        for patch in instance.patch_edges.values_mut() {
            if let Some(deadline) = patch.get_mut("deadline").and_then(Value::as_object_mut) {
                deadline.remove("extendable");
            }
        }
    }
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

#[test]
fn an_instance_stake_imports_as_money_or_as_a_hole_its_transform_needs() {
    let links = json!({"instances": {
        "base@known": {"pack": "base", "payoff_transform": "stake - payoff", "stake": 1_500_000.5},
        "base@unknown": {"pack": "base", "payoff_transform": "stake - payoff"},
        "base@plain": {"pack": "base", "payoff_transform": "payoff * 2"}
    }});
    let input = Input::new("packs/links.json", links.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    let module = text(&files, "lit/links/links.lit");
    assert!(
        module.contains("payoff_transform (stake - payoff), stake 1_500_000.5.usd }"),
        "{module}"
    );
    assert!(
        module.contains("payoff_transform (stake - payoff), stake ? }"),
        "{module}"
    );
    assert!(
        module.contains("payoff_transform (payoff * 2) }"),
        "{module}"
    );
    assert_eq!(module.matches(", stake ").count(), 2, "{module}");
}

/// A legacy transform keeps its meaning as a typed expression: numbers
/// beside money are money, a factor stays a number, and parentheses
/// appear only where precedence needs them.
#[test]
fn a_transform_imports_as_a_typed_expression() {
    let links = json!({"instances": {
        "base@neg": {"pack": "base", "payoff_transform": "-(payoff - 1000000)"},
        "base@left": {"pack": "base", "payoff_transform": "2 * payoff / 4"},
        "base@sum": {"pack": "base", "payoff_transform": "(payoff + 1.5) * 2"},
        "base@nested": {"pack": "base", "payoff_transform": "payoff - (payoff - 1)"},
        "base@factor": {"pack": "base", "payoff_transform": "payoff * (2 * 3)"}
    }});
    let input = Input::new("packs/links.json", links.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    let module = text(&files, "lit/links/links.lit");
    for spelled in [
        "payoff_transform (-(payoff - 1_000_000.usd))",
        "payoff_transform (2 * payoff / 4)",
        "payoff_transform ((payoff + 1.5.usd) * 2)",
        "payoff_transform (payoff - (payoff - 1.usd))",
        "payoff_transform (payoff * (2 * 3))",
    ] {
        assert!(module.contains(spelled), "{spelled}\n{module}");
    }
}

/// `analysis` is written as expressions: lists and records, an empty
/// record, and edges as unquoted refs.
#[test]
fn analysis_imports_as_expressions() {
    let named = json!({"id": "typed", "summary": "Typed.", "packs": ["elsewhere"],
                       "scenario": {"calibration": ["a"], "uncertainty": {},
                                    "objective": {"type": "cvar", "alpha": 0.1},
                                    "remove_edges": ["elsewhere::e1"]}});
    let input = Input::new("scenarios/typed.json", named.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    let case = text(&files, "cases/typed.case");
    assert!(
        case.contains(
            "analysis {\n  calibration: [\"a\"]\n  uncertainty: {}\n  \
             objective: { type: \"cvar\", alpha: 0.1 }\n  remove_edges: [elsewhere::e1]\n}"
        ),
        "{case}"
    );
}

/// `params` spells each engine rate as money per its time unit (SPEC
/// §8.3 `Params`); every other entry as written, and an empty record as
/// `{}`.
#[test]
fn params_import_with_units() {
    let named = json!({"id": "rates", "summary": "Rates.", "packs": ["elsewhere"],
                       "scenario": {"params": {"rate": 650, "opp_rate": 512.5,
                                               "carry_per_day": 30, "stakes": 2}}});
    let input = Input::new("scenarios/rates.json", named.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    let case = text(&files, "cases/rates.case");
    assert!(
        case.contains(
            "params: {\n    rate: 650.usd / 1.hours\n    opp_rate: 512.5.usd / 1.hours\n    \
             carry_per_day: 30.usd / 1.days\n    stakes: 2\n  }"
        ),
        "{case}"
    );
    let named = json!({"id": "none", "summary": "None.", "packs": ["elsewhere"],
                       "scenario": {"params": {}}});
    let input = Input::new("scenarios/none.json", named.to_string());
    let files = import_ok(std::slice::from_ref(&input));
    let case = text(&files, "cases/none.case");
    assert!(case.contains("analysis { params: {} }"), "{case}");
}

/// A v2 named scenario `id` using package `p`, its `lowered` header holding
/// `holes`, with `edit` applied to its JSON.
fn v2_scenario(id: &str, holes: Vec<Hole>, edit: impl FnOnce(&mut Value)) -> Input {
    let pin = Pin {
        name: "p".into(),
        version: "0.1.0".into(),
        digest: format!("sha256:{}", "0".repeat(64)),
    };
    let lowered = Lowered::new(vec![pin], "2026-02-01".into(), None, holes);
    let mut named = json!({
        "schema_version": 2, "id": id, "summary": "S", "packs": ["p"],
        "scenario": {}, "lowered": lowered,
    });
    edit(&mut named);
    Input::new(format!("scenarios/{id}.json"), named.to_string())
}

#[test]
fn a_v2_scenario_restores_its_parties_and_holes_and_keeps_its_docket_as_a_comment() {
    let hole = |pointer: &str, reason: Option<&str>| Hole {
        pointer: pointer.into(),
        reason: reason.map(str::to_owned),
    };
    let holes = vec![
        hole("/scenario/payoffs/p::c", None),
        hole("/sources/0/as_of", Some("2026-06-01")),
        hole("/sources/1/as_of", None),
    ];
    let input = v2_scenario("docket", holes, |named| {
        named["sources"] = json!([
            {"title": "Reasoned", "as_of": "2026-06-01"},
            {"title": "Bare", "as_of": ""},
            {"title": "Dated", "as_of": "2026-01-05"},
        ]);
        named["scenario"] = json!({"start": "p::b", "payoffs": {"p::b": 5}});
        named["parties"] = json!([
            {"id": "me", "actor": "applicant", "entity_type": "Corporation"},
            {"id": "them", "actor": "either", "entity_type": "Individual"},
        ]);
        named["events"] = json!([
            {"date": "2026-01-02", "docket": 3, "action": "Filed(complaint, Mail)",
             "relation": "by", "party": "me", "authority": "Rule \"4\""},
            {"date": "2026-01-03", "action": "Noticed()", "relation": "on", "party": "them"},
        ]);
    });
    let files = import_ok(std::slice::from_ref(&input));
    let case = text(&files, "cases/docket.case");
    // The reasoned hole stays a hole although its reason is a date.
    for expected in [
        "source \"Reasoned\" { as_of ?(\"2026-06-01\") }\n",
        "source \"Bare\" { as_of ? }\n",
        "source \"Dated\" { as_of 2026-01-05 }\n",
        "parties {\n  me: applicant Corporation\n  them: either Individual\n}\n",
        "// events {\n\
         //   2026-01-02 docket 3 Filed(complaint, Mail) by me cite \"Rule \\\"4\\\"\"\n\
         //   2026-01-03 Noticed() on them\n\
         // }\n",
        "stakes { p::b: 5.usd, p::c: unvalued }\n",
    ] {
        assert!(case.contains(expected), "{expected} not in\n{case}");
    }
    let original = NamedScenario::from_json(&input.text).unwrap_or_else(|e| panic!("{e}"));
    let file = &package(&files, "cases")[0];
    let back = read_case(file);
    assert_eq!(back.parties, original.parties);
    let as_json = |s: &NamedScenario| {
        let mut s = s.clone();
        s.schema_version = None;
        s.events.clear();
        s.lowered = None;
        numbers_as_f64(serde_json::to_value(&s).unwrap_or_default())
    };
    let (expected, actual) = (as_json(&original), as_json(&back));
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
    let holes: Vec<(String, Option<String>)> = original
        .lowered
        .iter()
        .flat_map(|l| &l.holes)
        .map(|h| (h.pointer.clone(), h.reason.clone()))
        .collect();
    assert_eq!(support::case_holes(file), holes);
}

#[test]
fn a_v2_scenario_with_only_unvalued_stakes_writes_a_stakes_block() {
    let hole = Hole {
        pointer: "/scenario/payoffs/p::c".into(),
        reason: None,
    };
    let input = v2_scenario("open", vec![hole], |_| {});
    let files = import_ok(std::slice::from_ref(&input));
    let case = text(&files, "cases/open.case");
    assert!(case.ends_with("stakes { p::c: unvalued }\n"), "{case}");
    assert_eq!(
        support::case_holes(package(&files, "cases")[0]),
        [("/scenario/payoffs/p::c".to_owned(), None)]
    );
}
