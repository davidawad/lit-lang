//! `render` (SPEC §8.6): a value `serde_json` cannot serialize is L6005.
mod support;

use std::collections::BTreeMap;

#[test]
fn an_unserializable_value_is_an_output_error() {
    let map = BTreeMap::from([((1, 2), "pair-keyed")]);
    let found = litigate_lower::render(&map)
        .err()
        .map(|d| vec![*d])
        .unwrap_or_default();
    insta::assert_snapshot!(support::render_diagnostics(&found));
}

/// `canonical` is RFC 8785: `render` without its insignificant whitespace.
#[test]
fn canonical_is_render_without_whitespace() {
    let value = serde_json::json!({"b": [1.0, {"d": "x y"}], "a": 1e21});
    let canonical = litigate_lower::canonical(&value).unwrap_or_default();
    assert_eq!(canonical, r#"{"a":1e+21,"b":[1,{"d":"x y"}]}"#);
    let pretty = litigate_lower::render(&value).unwrap_or_default();
    let squeezed: String = pretty
        .lines()
        .map(|l| l.trim_start().replacen("\": ", "\":", 1))
        .collect();
    assert_eq!(squeezed, canonical);
    let map = BTreeMap::from([((1, 2), "pair-keyed")]);
    assert!(litigate_lower::canonical(&map).is_err());
}
