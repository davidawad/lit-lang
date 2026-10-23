use super::{links_manifest, toml_key, toml_string};

#[test]
fn strings_and_keys_escape_exactly() {
    assert_eq!(
        toml_string("a \"b\" \\ \n\r\t\u{1}\u{7f}"),
        "\"a \\\"b\\\" \\\\ \\n\\r\\t\\u0001\\u007F\""
    );
    assert_eq!(toml_key("frcp-rules"), "frcp-rules");
    assert_eq!(toml_key("a_b9"), "a_b9");
    assert_eq!(toml_key("a.b"), "\"a.b\"");
    // An empty key cannot be bare in TOML.
    assert_eq!(toml_key(""), "\"\"");
}

#[test]
fn every_string_parses_back_as_toml() {
    for value in [
        "",
        "plain",
        "q\"uote",
        "back\\slash",
        "\r\n\t",
        "\u{0}\u{1b}\u{9f}",
        "é§",
    ] {
        let line = format!("{} = {}", toml_key(value), toml_string(value));
        let table: toml::Table = line
            .parse()
            .unwrap_or_else(|e| panic!("{line:?} does not parse: {e}"));
        assert_eq!(table.get(value).and_then(toml::Value::as_str), Some(value));
    }
}

#[test]
fn a_links_manifest_lists_each_dependency_by_relative_path() {
    let dependencies = [
        ("frcp".to_owned(), "frcp".to_owned()),
        ("cafc.x".to_owned(), "cafc".to_owned()),
    ];
    let text = links_manifest("packs/links.json", "2024-01-01", &dependencies);
    let table: toml::Table = text
        .parse()
        .unwrap_or_else(|e| panic!("manifest does not parse: {e}"));
    assert_eq!(table["package"]["name"].as_str(), Some("links"));
    assert_eq!(table["package"]["known_at"].as_str(), Some("2024-01-01"));
    assert_eq!(
        table["dependencies"]["cafc.x"]["path"].as_str(),
        Some("../cafc")
    );
    assert_eq!(
        table["dependencies"]["frcp"]["version"].as_str(),
        Some("^0.1.0")
    );
    let empty = links_manifest("l.json", "0001-01-01", &[]);
    assert!(empty.ends_with("[dependencies]\n"), "{empty}");
}
