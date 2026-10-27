//! Calibration sets to `calibration` declarations: where each lands, how
//! its values and sources are spelled, and the refs and dates with no
//! lossless spelling (L6002), which fail the import rather than drop.
use litigate_import::{import, import_into, Input};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn source(start: &str, end: Option<&str>) -> Value {
    let mut s =
        json!({"title": "Report", "url": "https://example.org/r.pdf", "vintageStart": start});
    if let Some(end) = end {
        s["vintageEnd"] = json!(end);
        s["retrieved"] = json!("2026-09-28");
    }
    s
}

fn set(entries: &Value) -> Input {
    let text = json!({"id": "demo-fy", "title": "Demo", "description": "D", "entries": entries});
    Input::new("calibration/demo-fy.json", text.to_string())
}

fn messages(found: &[litigate_base::Diagnostic]) -> Vec<String> {
    found
        .iter()
        .map(|d| {
            let at = d.notes.first().map_or("", |n| n.message.as_str());
            format!("{} ({at})", d.message)
        })
        .collect()
}

#[test]
fn every_target_kind_imports_into_the_package_it_calibrates() {
    let input = set(&json!([
        {"target": "edge-probability", "ref": "demo::a", "value": 0.25, "source": source("2023-10-01", Some("2024-09-30")), "n": 40, "method": "10 / 40"},
        {"target": "edge-probability", "ref": "demo::b", "source": source("2023-10-01", Some("2024-09-30")), "n": 40, "method": "cleared"},
        {"target": "edge-duration", "ref": "demo::a", "distribution": {"mode": 3.5}, "source": source("2024-01-01", None), "n": 2},
        {"target": "edge-duration", "ref": "demo::b", "distribution": {"min": 1.0, "mode": 2.0, "max": 5.0}, "source": source("2024-01-01", None), "n": 3},
        {"target": "node-attr", "ref": "demo::n", "attr": "weight", "value": -2.0, "source": source("2024-01-01", None), "n": 1}
    ]));
    let existing = BTreeMap::from([("demo".to_owned(), "demo-dir".to_owned())]);
    let imported = import_into(&[input], &existing).unwrap_or_else(|d| panic!("{d:#?}"));
    let [file] = imported.files.as_slice() else {
        panic!("{imported:#?}")
    };
    assert_eq!(file.path, "lit/demo-dir/calibration-demo-fy.lit");
    insta::assert_snapshot!(file.text);
}

#[test]
fn an_instance_ref_goes_to_the_links_package_kept_qualified() {
    let input = set(&json!([
        {"target": "edge-probability", "ref": "cafc@cofc::panel-to-affirmed", "value": 0.5, "source": source("2023-10-01", None), "n": 2}
    ]));
    let imported = import(&[input]).unwrap_or_else(|d| panic!("{d:#?}"));
    assert_eq!(imported.files[0].path, "lit/links/calibration-demo-fy.lit");
    assert!(imported.files[0]
        .text
        .contains("p cafc@cofc::panel-to-affirmed 0.5 {"));
}

#[test]
fn refs_and_dates_with_no_spelling_fail_the_import() {
    let two_packs = set(&json!([
        {"target": "edge-probability", "ref": "demo::a", "value": 0.5, "source": source("2023-10-01", None), "n": 1},
        {"target": "edge-probability", "ref": "other::a", "value": 0.5, "source": source("2023-10-01", None), "n": 1}
    ]));
    let unqualified = set(&json!([
        {"target": "edge-probability", "ref": "a", "value": 0.5, "source": source("2023-10-01", None), "n": 1}
    ]));
    let bad_dates = set(&json!([
        {"target": "edge-probability", "ref": "demo::a", "value": 0.5, "source": {"title": "R", "url": "u", "vintageStart": "2023-02-30", "vintageEnd": "soon", "retrieved": "2026-9-1"}, "n": 1}
    ]));
    let negative = set(&json!([
        {"target": "edge-duration", "ref": "demo::a", "distribution": {"mode": -1.0}, "source": source("2023-10-01", None), "n": 1}
    ]));
    let empty = set(&json!([]));
    for (input, expected) in [
        (two_packs, vec!["a calibration set calibrates one package: `other::a` is not in `demo` (at JSON pointer `/entries/1/ref`)"]),
        (unqualified, vec!["`a` names no pack (`pack::local`) (at JSON pointer `/entries/0/ref`)"]),
        (bad_dates, vec![
            "`2023-02-30` is not a date `YYYY-MM-DD` (at JSON pointer `/entries/0/source/vintageStart`)",
            "`soon` is not a date `YYYY-MM-DD` (at JSON pointer `/entries/0/source/vintageEnd`)",
            "`2026-9-1` is not a date `YYYY-MM-DD` (at JSON pointer `/entries/0/source/retrieved`)",
        ]),
        (negative, vec!["duration has no exact .lit spelling (at JSON pointer `/entries/0/distribution/mode`)"]),
        (empty, vec!["a calibration set with no entries names no package (at JSON pointer `/entries`)"]),
    ] {
        let found = import(&[input]).expect_err("fails");
        assert_eq!(messages(&found), expected);
    }
}
