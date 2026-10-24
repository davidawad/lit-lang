// SPDX-License-Identifier: GPL-3.0-or-later
//! Pack schema v5 (links v3): a deadline window is a counted `length` or a
//! `designated` due date the triggering notice sets, never both or
//! neither, and carries its extension grants as a sum of bounds.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::model::{ExtensionBound, LinkFile, Pack};
use serde_json::{json, Value};

/// A one-edge pack of `version` whose edge has `deadline`.
fn pack(version: u32, deadline: &Value) -> Result<Pack, String> {
    let pack = json!({
        "schemaVersion": version, "id": "p", "title": "P", "startNodeId": "a",
        "nodes": [{"id": "a", "label": "A"}, {"id": "b", "kind": "terminal", "label": "B"}],
        "edges": [{"from": "a", "to": "b", "label": "Go", "deadline": deadline}]
    });
    Pack::from_json(&pack.to_string()).map_err(|e| e.to_string())
}

fn designated() -> Value {
    json!({"minPeriod": 30, "sourceEvent": "Noticed", "sourceEventArg": "due"})
}

#[test]
fn a_window_is_counted_or_designated() {
    assert!(pack(5, &json!({"length": 21})).is_ok());
    let found = pack(5, &json!({"designated": designated()})).unwrap();
    let d = found.edges[0].deadline.as_ref().unwrap();
    assert_eq!((d.length, d.days()), (None, None));
    assert_eq!(
        pack(5, &json!({"length": 21, "designated": designated()})).unwrap_err(),
        "parse: pack p: edge a->b#0: `deadline`: give `length` or `designated`, not both"
    );
    assert!(pack(5, &json!({"note": "n"}))
        .unwrap_err()
        .ends_with("`deadline`: needs `length` (or, a designated window, `designated`)"));
    assert!(pack(5, &json!({"designated": designated(), "unit": "court"}))
        .unwrap_err()
        .ends_with("a designated window's least time is in calendar days"));
    assert!(pack(5, &json!({"designated": designated(), "unit": "calendar"})).is_ok());
    for bad in [json!(-1), json!(2.5)] {
        let mut d = designated();
        d["minPeriod"] = bad;
        assert!(pack(5, &json!({ "designated": d }))
            .unwrap_err()
            .contains("is not a non-negative integer"));
    }
}

#[test]
fn designated_windows_and_extensions_need_v5() {
    let grant = json!([{"bound": "nonExtendable", "authority": "FRCP 6(b)(2)"}]);
    for deadline in [
        json!({"designated": designated()}),
        json!({"length": 21, "extensions": grant}),
    ] {
        assert!(pack(4, &deadline)
            .unwrap_err()
            .ends_with("`designated` and `extensions` need schemaVersion 5, found 4"));
        assert!(pack(5, &deadline).is_ok());
    }
    let links = |version: u32| {
        let file = json!({"schemaVersion": version, "links": [
            {"from": "p::a", "to": "q::b", "label": "L", "deadline": {"designated": designated()}}
        ]});
        LinkFile::from_json(&file.to_string()).map_err(|e| e.to_string())
    };
    assert!(links(2)
        .unwrap_err()
        .contains("links.json: link p::a->q::b#0: `deadline`: `designated` and `extensions` need schemaVersion 3, found 2"));
    assert!(links(3).is_ok());
}

#[test]
fn extension_bounds_read_back_as_written() {
    let extensions = json!([
        {"bound": "bounded", "by": "court",
         "until": {"max": [{"offset": {"from": "prescribedDue", "days": 30}},
                           {"min": [{"offset": {"from": "grantOrderEntered", "days": 14}}]}]},
         "afterExpiry": true, "standard": "GoodCauseOrExcusableNeglect",
         "motion": {"event": "AppealExtensionMoved()",
                    "within": {"offset": {"from": "prescribedDue", "days": 30}}}},
        {"bound": "discretionary", "by": "court", "standard": "GoodCause", "window": "beforeExpiry",
         "interpretation": "37 CFR 2.116(a) reaches the period"},
        {"bound": "discretionary", "by": "court", "standard": "ExcusableNeglect",
         "window": "afterExpiryOnMotion", "motion": {"event": "ExtensionMoved(AnswerPeriod)"}},
        {"bound": "nonExtendable", "authority": "FRCP 6(b)(2)"}
    ]);
    let deadline = json!({"length": 21.0, "extensions": extensions});
    let found = pack(5, &deadline).unwrap();
    let d = found.edges[0].deadline.as_ref().unwrap();
    assert!(matches!(d.extensions[3], ExtensionBound::NonExtendable { .. }));
    assert_eq!(serde_json::to_value(d).unwrap(), deadline);
    // A misspelled field of a bound is an error, as everywhere in a pack.
    let mut typo = deadline.clone();
    typo["extensions"][1]["windw"] = json!("beforeExpiry");
    assert!(pack(5, &typo).unwrap_err().contains("windw"));
}
