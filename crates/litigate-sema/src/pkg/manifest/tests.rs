use super::*;

const MINIMAL: &str = r#"[package]
name = "demo"
version = "1.0.0"
language = 1
pack_id = "demo-pack"
forum = "demo"
authority = "FederalRule"
title = "Demo"
in_force = "2020-01-01"
known_at = "2025-01-01"

[package.roles]
applicant = "self"
office = "nature"

[dependencies]
core = { version = "^1.0.0", path = "../core" }

[sources.b]
url = "https://b"
[sources.a]
path = "sources/a.txt"
cite_prefixes = ["A"]
"#;

fn errors(text: &str) -> Vec<String> {
    parse_manifest(text)
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.message)
        .collect()
}

#[test]
fn parses_the_worked_example() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../lit/examples/Lit.toml"
    ))
    .unwrap_or_default();
    let parsed = parse_manifest(&text).map(|m| m.manifest);
    insta::assert_json_snapshot!(parsed.ok());
}

#[test]
fn keeps_source_order_and_dependency_paths() {
    let parsed = parse_manifest(MINIMAL);
    let ids: Vec<String> = parsed
        .iter()
        .flat_map(|m| m.manifest.sources.iter().map(|s| s.id.clone()))
        .collect();
    assert_eq!(ids, ["b", "a"]);
    assert_eq!(
        parsed
            .ok()
            .and_then(|m| m.dependency_paths.get("core").cloned())
            .as_deref(),
        Some("../core")
    );
}

#[test]
fn rejects_unknown_missing_and_invalid_values() {
    assert!(
        errors(&MINIMAL.replace("title = \"Demo\"", "title = \"Demo\"\ncolour = 1"))[0]
            .contains("unknown field `colour`")
    );
    assert!(errors(&MINIMAL.replace("title = \"Demo\"\n", ""))[0].contains("missing field `title`"));
    assert_eq!(
        errors(
            &MINIMAL
                .replace("\"1.0.0\"", "\"1.0\"")
                .replace("= \"nature\"", "= \"judge\"")
                .replace("2020-01-01", "2020-02-30")
        ),
        [
            "`1.0` is not a SemVer version (MAJOR.MINOR.PATCH[-pre])",
            "`in_force` must be a YYYY-MM-DD date, found `2020-02-30`",
            "role of `office` must be \"self\", \"opponent\", or \"nature\"",
        ]
    );
    assert_eq!(
        errors(&MINIMAL.replace("path = \"sources/a.txt\"", "path = \"/abs/a.txt\"")),
        ["source path `/abs/a.txt` must be repository-relative"]
    );
    assert_eq!(
        errors(&MINIMAL.replace("^1.0.0", "1.0.0")),
        ["`1.0.0` is not a requirement (`^`, `~`, `>=`, or `=` plus a version)"]
    );
}

#[test]
fn errors_carry_the_offending_span() {
    let text = MINIMAL.replace("language = 1", "language = 2");
    let reports = parse_manifest(&text).err().unwrap_or_default();
    let start = text
        .find("2\npack_id")
        .map(|n| u32::try_from(n).unwrap_or(0));
    assert_eq!(
        reports.first().map(|r| (r.code, r.range.start)),
        Some((C::InvalidManifest, start.unwrap_or(0)))
    );
}
