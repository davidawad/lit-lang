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
