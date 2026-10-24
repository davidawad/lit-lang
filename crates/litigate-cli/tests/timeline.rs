//! `litigate timeline` (lg-z2m.1): golden timelines of a TTAB opposition
//! and an FRCP matter (each with a fork and an unknown deadline), the
//! options, and every output validated against the frozen schema
//! `docs/litigate/timeline.v1.schema.json` together with its example.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate, litigate_in, repo, scratch};

/// The frozen schema.
fn schema() -> Value {
    let text = std::fs::read_to_string(repo().join("docs/litigate/timeline.v1.schema.json"));
    serde_json::from_str(&text.unwrap()).unwrap()
}

/// The JSON type names `v` has (`integer` is also a `number`).
fn types_of(v: &Value) -> Vec<&'static str> {
    match v {
        Value::Null => vec!["null"],
        Value::Bool(_) => vec!["boolean"],
        Value::Number(n) if n.is_u64() || n.is_i64() => vec!["integer", "number"],
        Value::Number(_) => vec!["number"],
        Value::String(_) => vec!["string"],
        Value::Array(_) => vec!["array"],
        Value::Object(_) => vec!["object"],
    }
}

/// True for the frozen schema's one `pattern`, a `YYYY-MM-DD` date.
fn is_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

/// The error of leaf keyword `key` (with value `rule`) on `v`, if any.
fn leaf(key: &str, rule: &Value, v: &Value) -> Option<String> {
    let number = |x: &Value| x.as_f64();
    match key {
        "$schema" | "$id" | "title" | "description" | "$defs" => None,
        "type" => {
            let allowed: Vec<&str> = match rule {
                Value::String(t) => vec![t.as_str()],
                other => other
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|t| t.as_str().unwrap())
                    .collect(),
            };
            (!types_of(v).iter().any(|t| allowed.contains(t)))
                .then(|| format!("{v} is not of type {rule}"))
        }
        "const" => (v != rule).then(|| format!("{v} is not {rule}")),
        "enum" => {
            (!rule.as_array().unwrap().contains(v)).then(|| format!("{v} is not one of {rule}"))
        }
        "pattern" => {
            assert_eq!(rule, "^[0-9]{4}-[0-9]{2}-[0-9]{2}$", "unsupported pattern");
            v.as_str()
                .is_some_and(|t| !is_date(t))
                .then(|| format!("{v} does not match {rule}"))
        }
        "minLength" => {
            assert_eq!(rule, 1, "unsupported minLength");
            v.as_str()
                .is_some_and(str::is_empty)
                .then(|| format!("{v} is empty"))
        }
        "minimum" => number(v)
            .is_some_and(|n| n < number(rule).unwrap())
            .then(|| format!("{v} < {rule}")),
        "maximum" => number(v)
            .is_some_and(|n| n > number(rule).unwrap())
            .then(|| format!("{v} > {rule}")),
        "minItems" => v
            .as_array()
            .is_some_and(|a| (a.len() as u64) < rule.as_u64().unwrap())
            .then(|| format!("fewer than {rule} items")),
        other => panic!("unsupported schema keyword `{other}`"),
    }
}

/// The errors validating `v` at `at` against `s`, for the subset of JSON
/// Schema draft 2020-12 the frozen schema uses (an unknown keyword fails,
/// so the validator cannot silently ignore a constraint).
fn validate(root: &Value, s: &Value, v: &Value, at: &str, errors: &mut Vec<String>) {
    for (key, rule) in s.as_object().unwrap() {
        match key.as_str() {
            "$ref" => {
                let name = rule.as_str().unwrap().strip_prefix("#/$defs/").unwrap();
                validate(root, &root["$defs"][name], v, at, errors);
            }
            "required" => {
                for name in rule.as_array().unwrap() {
                    if v.as_object()
                        .is_some_and(|o| !o.contains_key(name.as_str().unwrap()))
                    {
                        errors.push(format!("{at}: missing {name}"));
                    }
                }
            }
            "additionalProperties" => {
                assert_eq!(rule, false, "unsupported additionalProperties");
                let known = s["properties"].as_object().unwrap();
                for name in v.as_object().into_iter().flat_map(|o| o.keys()) {
                    if !known.contains_key(name) {
                        errors.push(format!("{at}: unknown property {name}"));
                    }
                }
            }
            "properties" => {
                for (name, sub) in rule.as_object().unwrap() {
                    if let Some(x) = v.get(name) {
                        validate(root, sub, x, &format!("{at}/{name}"), errors);
                    }
                }
            }
            "items" => {
                for (i, x) in v.as_array().into_iter().flatten().enumerate() {
                    validate(root, rule, x, &format!("{at}/{i}"), errors);
                }
            }
            "oneOf" => {
                let passing = rule
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|sub| {
                        let mut e = Vec::new();
                        validate(root, sub, v, at, &mut e);
                        e.is_empty()
                    })
                    .count();
                if passing != 1 {
                    errors.push(format!(
                        "{at}: {v} matches {passing} of oneOf, not exactly 1"
                    ));
                }
            }
            leaf_key => errors.extend(leaf(leaf_key, rule, v).map(|m| format!("{at}: {m}"))),
        }
    }
}

/// The schema errors of document `v`.
fn schema_errors(v: &Value) -> Vec<String> {
    let root = schema();
    let mut errors = Vec::new();
    validate(&root, &root, v, "", &mut errors);
    errors
}

/// Checks the document's internal references: ids are unique, every row,
/// edge and world names a world that exists, every trigger, fork row and
/// edge end names a row, every provenance entry a layer.
fn check_references(t: &Value) {
    let ids = |key: &str| -> Vec<String> {
        t[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["id"].as_str().unwrap().to_owned())
            .collect()
    };
    let (rows, worlds, layers) = (ids("rows"), ids("worlds"), ids("layers"));
    let mut unique = rows.clone();
    unique.dedup();
    assert_eq!(unique.len(), rows.len());
    let named =
        |v: &Value, set: &[String]| v.is_null() || set.contains(&v.as_str().unwrap().to_owned());
    for r in t["rows"].as_array().unwrap() {
        assert!(named(&r["world"], &worlds), "{r}");
        assert!(named(&r["trigger"], &rows), "{r}");
        let provenance = r["provenance"].as_array().unwrap();
        assert!(provenance.iter().all(|p| named(p, &layers)), "{r}");
        let last = provenance.last().unwrap();
        let class = t["layers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| &l["id"] == last)
            .map(|l| l["class"].clone());
        assert_eq!(class.as_ref(), Some(&r["layer"]), "{r}");
        let state = r["status"]["state"].as_str().unwrap();
        if state == "unknown" || state == "superseded" {
            assert!(r["status"]["reason"].is_string(), "{r}");
        }
    }
    // Keys pair rows across worlds, so none repeats within a world.
    let mut keys: Vec<(String, String)> = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["world"].to_string(), r["key"].to_string()))
        .collect();
    let count = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), count, "{t:#}");
    for w in t["worlds"].as_array().unwrap() {
        assert!(named(&w["parent"], &worlds), "{w}");
        assert!(named(&w["fork_row"], &rows), "{w}");
    }
    for e in t["edges"].as_array().unwrap() {
        assert!(named(&e["from"], &rows) && named(&e["to"], &rows), "{e}");
        assert!(named(&e["world"], &worlds), "{e}");
    }
}

/// The timeline document of `litigate timeline ARGS` run in `dir`,
/// validated against the frozen schema.
fn timeline_in(dir: &Path, args: &[&str]) -> Value {
    let mut argv = vec!["timeline"];
    argv.extend_from_slice(args);
    let run = litigate_in(dir, &argv);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    let t = run.json["result"].clone();
    assert_eq!(schema_errors(&t), Vec::<String>::new(), "{t:#}");
    check_references(&t);
    t
}

fn timeline(args: &[&str]) -> Value {
    timeline_in(&repo(), args)
}

/// A scratch tree with the TTAB, FRCP and core packages and the timeline
/// fixture matters under `cases/`, locked.
fn fixtures() -> PathBuf {
    let dir = scratch("timeline-fixtures");
    for pkg in ["ttab", "frcp", "core"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    copy_dir(
        &repo().join("crates/litigate-cli/tests/fixtures/timeline"),
        &dir.join("cases"),
    );
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

#[test]
fn the_frozen_example_is_valid_against_the_frozen_schema() {
    let text = std::fs::read_to_string(
        repo().join("docs/litigate/examples/timeline-ttab-opposition.json"),
    );
    let example: Value = serde_json::from_str(&text.unwrap()).unwrap();
    assert_eq!(schema_errors(&example), Vec::<String>::new());
    check_references(&example);
}

#[test]
fn the_validator_rejects_what_the_schema_rejects() {
    let mut bad = json!({
        "schema": "litigate.timeline/2",
        "source": {"file": "a.case", "kind": "matter", "extra": 1},
        "as_of": "2026-4-8",
        "known_at": 7,
        "layers": [],
        "worlds": [],
        "rows": [{"id": ""}],
        "edges": [],
        "diagnostics": [{"code": "L1", "severity": "error", "message": "m",
            "source": {"file": "f", "start": -1, "end": 0, "line": 0, "column": 1}}]
    });
    let errors = schema_errors(&bad).join("\n");
    for expected in [
        "/schema: \"litigate.timeline/2\" is not",
        "/source: unknown property extra",
        "/source/kind: \"matter\" is not one of",
        "/as_of: \"2026-4-8\" does not match",
        "/known_at: 7 matches 0 of oneOf",
        "/worlds: fewer than 1 items",
        "/rows/0: missing \"world\"",
        "/rows/0/id: \"\" is empty",
        "/diagnostics/0/source: {",
    ] {
        assert!(errors.contains(expected), "{expected} not in\n{errors}");
    }
    bad["worlds"] = json!([{"id": "w0", "label": "docket", "parent": null, "fork_row": null,
        "choice": {"kind": "chance", "label": "x", "cite": null, "probability": 1.5}}]);
    let errors = schema_errors(&bad).join("\n");
    assert!(errors.contains("/worlds/0/choice"), "{errors}");
}

#[test]
fn ttab_opposition_matter() {
    let t = timeline(&["cases/ttab-opposition-applicant-defense.case"]);
    insta::assert_json_snapshot!("ttab_opposition_matter", t);
}

#[test]
fn frcp_civil_matter() {
    let t = timeline(&["cases/frcp-civil-case.case"]);
    insta::assert_json_snapshot!("frcp_civil_matter", t);
}

#[test]
fn ttab_published_fixture_forks_with_unknown_deadlines() {
    let dir = fixtures();
    let t = timeline_in(&dir, &["cases/ttab-published.case"]);
    let unknown: Vec<&Value> = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["status"]["state"] == "unknown" && r["kind"] == "deadline")
        .collect();
    assert!(unknown.iter().any(|r| r["world"] == "w0"));
    assert!(unknown.iter().any(|r| r["world"] == "w1"));
    assert!(unknown.iter().all(|r| r["due"].is_null()));
    insta::assert_json_snapshot!("ttab_published_fixture", t);
}

#[test]
fn frcp_filed_fixture_forks_with_unknown_deadlines() {
    let dir = fixtures();
    let t = timeline_in(&dir, &["cases/frcp-filed.case"]);
    let fork = &t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "fork");
    assert_eq!(fork.unwrap()["status"]["state"], "unknown");
    // 90 days after 2024-12-02 is Sunday 2025-03-02: FRCP 6(a)(1)(C) rolls it to Monday.
    let service = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["label"].as_str().unwrap().starts_with("service-period"));
    assert_eq!(service.unwrap()["due"], "2025-03-03");
    insta::assert_json_snapshot!("frcp_filed_fixture", t);
}

/// The keys of world `id`'s view, in document order: each ancestor's rows
/// up to the fork row the path branches at, then its own.
fn view_keys(t: &Value, id: &str) -> Vec<String> {
    let worlds = t["worlds"].as_array().unwrap();
    let world = |w: &str| worlds.iter().find(|x| x["id"] == w).unwrap();
    let mut path = vec![(id.to_owned(), None)];
    let mut at = world(id);
    while let Some(parent) = at["parent"].as_str() {
        path.push((
            parent.to_owned(),
            at["fork_row"].as_str().map(str::to_owned),
        ));
        at = world(parent);
    }
    let mut keys = Vec::new();
    for (w, cut) in path.iter().rev() {
        for r in t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["world"] == *w)
        {
            keys.push(r["key"].as_str().unwrap().to_owned());
            if cut.as_deref() == r["id"].as_str() {
                break;
            }
        }
    }
    keys
}

/// Every `.case` file under `dir`, repository-relative, sorted.
fn cases_under(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(cases_under(&path));
        } else if path.extension().is_some_and(|e| e == "case") {
            let shown = path.strip_prefix(repo()).unwrap();
            found.push(shown.to_string_lossy().replace('\\', "/"));
        }
    }
    found.sort();
    found
}

#[test]
fn keys_are_unique_in_every_world_view_of_every_case() {
    let cases = cases_under(&repo().join("cases"));
    assert!(cases.len() >= 20, "{cases:?}");
    // The end-to-end matter rejected by design (L4001, `e2e_ttab.rs`)
    // exits 2; its timeline is golden-tested there.
    for case in cases.iter().filter(|c| !c.ends_with("e2e-ttab-short.case")) {
        let t = timeline(&[case]);
        // The op re-emits no row yet (lg-z2m.3), so no key may repeat in
        // a view, inherited or not.
        for w in t["worlds"].as_array().unwrap() {
            let id = w["id"].as_str().unwrap();
            let mut keys = view_keys(&t, id);
            let count = keys.len();
            keys.sort();
            keys.dedup();
            assert_eq!(keys.len(), count, "{case} {id}: {:?}", view_keys(&t, id));
        }
        assert!(
            !t["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "L1001"),
            "{case}"
        );
    }
}

#[test]
fn an_election_is_keyed_by_the_option_it_elects() {
    let t = timeline(&["cases/cofc-1498-patent-case.case"]);
    let events: Vec<(&str, &str)> = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "event" && r["world"] == "w0")
        .map(|r| (r["key"].as_str().unwrap(), r["label"].as_str().unwrap()))
        .collect();
    assert_eq!(
        events,
        [
            (
                "event:Elected@2026-09-14/e-filingfork-standard",
                "Elected(e-filingfork-standard) by owner"
            ),
            (
                "event:Elected@2026-09-14/e-casetype-patent",
                "Elected(e-casetype-patent) by owner"
            ),
        ]
    );
}

#[test]
fn a_child_world_inserting_an_earlier_trigger_keeps_every_inherited_key() {
    // `frcp-served` is `frcp-filed`'s first world ("Serve defendant via
    // formal process") with its service recorded on 2024-12-10: an event
    // that triggers the answer period, ahead of the parent's service-period
    // row (due 2025-03-03) in date order.
    let dir = fixtures();
    let parent = timeline_in(&dir, &["cases/frcp-filed.case"]);
    let child = timeline_in(&dir, &["cases/frcp-served.case"]);
    let keys = |t: &Value| -> Vec<(String, String)> {
        let worlds = t["worlds"].as_array().unwrap();
        t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let world = worlds.iter().find(|w| w["id"] == r["world"]).unwrap();
                (
                    world["label"].as_str().unwrap().to_owned(),
                    r["key"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    };
    let (before, after) = (keys(&parent), keys(&child));
    // The docket and every world the parent forks into: each key the
    // parent has, the child has unchanged, except the answer period its
    // first world assumed, which the child's docket now records.
    let assumed = "frcp.periods.answer-period@event:service-completed@fork:frcp.pleading-to-trial.service-window@start";
    let inherited: Vec<&(String, String)> = before.iter().filter(|(_, k)| k != assumed).collect();
    for row in &inherited {
        assert!(after.contains(row), "{row:?} not in {after:#?}");
    }
    assert!(after
        .iter()
        .any(|(w, k)| w == "docket"
            && k == "frcp.periods.answer-period@event:ProcessServed@2024-12-10"));
    let child_only: Vec<&(String, String)> =
        after.iter().filter(|row| !before.contains(row)).collect();
    // Stable across runs.
    assert_eq!(keys(&timeline_in(&dir, &["cases/frcp-served.case"])), after);
    insta::assert_json_snapshot!(
        "child_world_inherited_keys",
        json!({"parent": before, "child": after, "child_only": child_only})
    );
}

#[test]
fn an_answer_to_a_notice_without_its_due_date_reads_unknown_with_why() {
    let dir = fixtures();
    let t = timeline_in(&dir, &["cases/ttab-unjudged.case"]);
    let answer = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "ttab.answer-period@event:InstitutionNoticed@2026-07-06")
        .unwrap();
    assert_eq!(answer["due"], Value::Null);
    assert_eq!(
        answer["status"],
        json!({"state": "unknown", "reason": "timeliness unknown: InstitutionNoticed due date not recorded; acted by AnswerFiled(Plain) on 2026-08-10"})
    );
    let codes: Vec<&Value> = t["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| &d["code"])
        .collect();
    assert!(codes.contains(&&json!("L4010")), "{codes:?}");
}

#[test]
fn a_rule_module_lists_its_deadlines_undated() {
    let t = timeline(&["lit/ttab/ttab.lit", "--as-of", "2026-10-19"]);
    assert_eq!(
        t["source"],
        json!({"file": "lit/ttab/ttab.lit", "kind": "lit"})
    );
    let rows = t["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| r["kind"] == "deadline"
        && r["status"]["state"] == "unknown"
        && r["due"].is_null()));
    // The answer period is the date the institution notice designates.
    assert!(rows.iter().any(|r| r["label"]
        == "answer-period: the date InstitutionNoticed() designates, at least 30 days after it"));
    let default = timeline(&["lit/ttab/ttab.lit"]);
    assert_eq!(default["as_of"], "2026-10-19");
}

#[test]
fn as_of_judges_statuses_at_another_date() {
    let t = timeline(&[
        "cases/ttab-opposition-applicant-defense.case",
        "--as-of",
        "2026-12-20",
        "--known-at",
        "2026-10-19",
    ]);
    assert_eq!(
        (t["as_of"].as_str(), t["known_at"].as_str()),
        (Some("2026-12-20"), Some("2026-10-19"))
    );
    // The 180-day outer limit, due 2026-12-14, is open on the matter's own
    // `as_of` (2026-10-19) and has lapsed by 2026-12-20.
    let outer = |t: &Value| {
        t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["label"].as_str().unwrap().starts_with("outer-limit-180"))
            .map(|r| r["status"]["state"].clone())
    };
    assert_eq!(outer(&t), Some(json!("lapsed")));
    let known = timeline(&[
        "cases/ttab-opposition-applicant-defense.case",
        "--known-at",
        "2026-10-19",
    ]);
    assert_eq!(known["as_of"], "2026-10-19");
    assert_eq!(known["known_at"], "2026-10-19");
    assert_eq!(
        outer(&timeline(&["cases/ttab-opposition-applicant-defense.case"])),
        Some(json!("open"))
    );
}

#[test]
fn layers_switch_rows_off_but_stay_listed() {
    let file = "cases/ptab-ipr-defense.case";
    let all = timeline(&[file]);
    let off = timeline(&[file, "--layers=-ptab-patent-trial-appeal-board"]);
    let layer = off["layers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == "ptab-patent-trial-appeal-board")
        .cloned();
    assert_eq!(layer.unwrap()["enabled"], false);
    assert!(off["rows"].as_array().unwrap().is_empty());
    assert_eq!(off["worlds"].as_array().unwrap().len(), 1);
    assert!(off["edges"].as_array().unwrap().is_empty());
    // Naming only `cafc` switches the rest off; naming both keeps all.
    let only = timeline(&[file, "--layers", "cafc"]);
    assert_eq!(only["rows"], off["rows"]);
    let both = timeline(&[file, "--layers", "+cafc,ptab-patent-trial-appeal-board"]);
    assert_eq!(both["rows"], all["rows"]);
    let bad = litigate(&["timeline", file, "--layers", "nope"]);
    assert_eq!(bad.code, 1);
    assert!(
        bad.stderr
            .contains("no layer `nope`; layers: ptab-patent-trial-appeal-board, cafc"),
        "{}",
        bad.stderr
    );
}

#[test]
fn world_keeps_one_branch_and_its_ancestors_up_to_the_fork() {
    let file = "cases/ptab-ipr-defense.case";
    let all = timeline(&[file]);
    let w2 = timeline(&[file, "--world", "w2"]);
    let worlds: Vec<&str> = w2["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["id"].as_str().unwrap())
        .collect();
    assert_eq!(worlds, ["w0", "w2"]);
    let fork = w2["worlds"][1]["fork_row"].as_str().unwrap().to_owned();
    let rows: Vec<(String, String)> = w2["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap().to_owned(),
                r["world"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let docket: Vec<&str> = rows
        .iter()
        .filter(|(_, w)| w == "w0")
        .map(|(r, _)| r.as_str())
        .collect();
    assert_eq!(docket.last(), Some(&fork.as_str()));
    assert!(rows.iter().all(|(_, w)| w != "w1"));
    // Row ids are stable across the filter.
    for (id, _) in &rows {
        assert!(all["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == id.as_str()));
    }
    let docket_only = timeline(&[file, "--world", "w0"]);
    assert_eq!(docket_only["worlds"].as_array().unwrap().len(), 1);
    let bad = litigate(&["timeline", file, "--world", "w9"]);
    assert_eq!(bad.code, 1);
    assert!(
        bad.stderr.contains("no world `w9`; worlds: w0, w1, w2"),
        "{}",
        bad.stderr
    );
}

#[test]
fn usage_failures_exit_1() {
    for (args, expected) in [
        (
            vec![
                "timeline",
                "cases/frcp-civil-case.case",
                "--as-of",
                "2024-1-01",
            ],
            "is not a YYYY-MM-DD date",
        ),
        (
            vec![
                "timeline",
                "cases/frcp-civil-case.case",
                "--known-at",
                "soon",
            ],
            "is not a YYYY-MM-DD date",
        ),
        (
            vec!["timeline", "lit/ttab/Lit.toml"],
            "not a .case or .lit file",
        ),
        (vec!["timeline", "README"], "not a .case or .lit file"),
        (vec!["timeline", "cases/no-such.case"], "no such file"),
    ] {
        let run = litigate(&args);
        assert_eq!(run.code, 1, "{args:?}");
        assert!(run.stderr.contains(expected), "{args:?}: {}", run.stderr);
    }
}

#[test]
fn a_matter_that_does_not_resolve_has_no_timeline() {
    let dir = scratch("timeline-unresolved");
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(
        dir.join("cases/broken.case"),
        "case broken\nuses nowhere ^1.0.0\n",
    )
    .unwrap();
    let run = litigate_in(&dir, &["timeline", "cases/broken.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(run.json["ok"], false);
    assert!(run.json.get("result").is_none(), "{}", run.json);
}

#[test]
fn a_module_of_a_package_that_does_not_check_has_no_timeline() {
    let dir = scratch("timeline-broken-lit");
    copy_dir(&repo().join("lit/examples"), &dir.join("lit/examples"));
    std::fs::write(
        dir.join("lit/examples/rule12.lit"),
        "module frcp.rule12\nthis is not litigate\n",
    )
    .unwrap();
    let run = litigate_in(&dir, &["timeline", "lit/examples/rule12.lit"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none(), "{}", run.json);
}

#[test]
fn a_layer_authority_outside_the_lattice_is_an_error_not_passed_through() {
    let dir = scratch("timeline-authority");
    copy_dir(&repo().join("lit/examples"), &dir.join("lit/examples"));
    copy_dir(&repo().join("cases/examples"), &dir.join("cases/examples"));
    let manifest = dir.join("lit/examples/Lit.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        text.replace(
            "authority = \"FederalRule\"",
            "authority = \"Constitution\"",
        ),
    )
    .unwrap();
    let module = dir.join("lit/examples/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    std::fs::write(
        &module,
        text.replace("authority FederalRule }", "authority Constitution }"),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases/examples"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let run = litigate_in(&dir, &["timeline", "cases/examples/acme.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(run.json["ok"], false);
    assert!(run.json.get("result").is_none(), "{}", run.json);
    let invalid: Vec<&Value> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "L1004")
        .collect();
    assert_eq!(invalid.len(), 1, "{}", run.stdout);
    assert_eq!(invalid[0]["severity"], "error");
    let message = invalid[0]["message"].as_str().unwrap();
    assert!(
        message.starts_with(
            "layer `frcp-rule12-example`: authority `Constitution` is not in the authority lattice"
        ),
        "{message}"
    );
    assert!(message.contains("FederalRule"), "{message}");
    assert!(
        !run.stdout.contains("\"authority\":\"Constitution\""),
        "{}",
        run.stdout
    );
}

#[test]
fn a_matter_with_errors_reports_them_in_the_timeline() {
    let dir = scratch("timeline-errors");
    copy_dir(&repo().join("lit/examples"), &dir.join("lit/examples"));
    copy_dir(&repo().join("cases/examples"), &dir.join("cases/examples"));
    let path = dir.join("cases/examples/acme.case");
    let text = std::fs::read_to_string(&path).unwrap();
    // An event after `as_of` is an error located in the matter.
    std::fs::write(
        &path,
        text.replace("2025-03-20 MovedToDismiss", "2025-09-20 MovedToDismiss"),
    )
    .unwrap();
    let run = litigate_in(&dir, &["timeline", "cases/examples/acme.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let t = &run.json["result"];
    assert_eq!(schema_errors(t), Vec::<String>::new());
    let diagnostics = t["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d["severity"] == "error" && d["source"]["file"] == "cases/examples/acme.case"),
        "{t:#}"
    );
    // Judged at a date after the event, it is no longer in the future
    // (L4001), but it still missed the answer period: a `late` row (L4002).
    let later = litigate_in(
        &dir,
        &[
            "timeline",
            "cases/examples/acme.case",
            "--as-of",
            "2025-10-01",
        ],
    );
    let codes: Vec<&Value> = later.json["result"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| &d["code"])
        .collect();
    assert_eq!(codes, [&json!("L4002")], "{}", later.stdout);
    let late = later.json["result"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "deadline")
        .map(|r| r["status"].clone());
    assert_eq!(
        late,
        Some(
            json!({"state": "late", "reason": "acted by MovedToDismiss(complaint) on 2025-09-20, after the due date"})
        )
    );
}

#[test]
fn human_output_renders_the_envelope() {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(repo())
        .args(["--format", "human", "timeline", "cases/examples/acme.case"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout.contains("litigate timeline: ok"), "{stdout}");
}
