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

#[test]
fn accepts_pack_schema_v3_keys() {
    let text = MINIMAL
        .replace(
            "known_at = \"2025-01-01\"\n",
            "known_at = \"2025-01-01\"\ncounting = \"frcp6\"\n",
        )
        .replace("office = \"nature\"", "office = \"nature\"\neither = \"nature\"")
        .replace("cite_prefixes = [\"A\"]", "as_of = \"2025-06\"")
        + "\n[[cite_syntax.rules]]\nprefixes = [\"FRCP\"]\nheading = \"Rule {}\"\nsource = \"a\"\n\n[[cite_syntax.shorthands]]\nprefix = \"cfr\"\ncode = \"cfr\"\ntitle = 19\nsection_parts = 2\n";
    let m = parse_manifest(&text).map(|p| p.manifest);
    let m = m.expect("the manifest is valid");
    assert_eq!(m.counting.as_deref(), Some("frcp6"));
    assert_eq!(m.roles.get("either"), Some(&ActorRole::Nature));
    assert_eq!(m.sources[1].as_of.as_deref(), Some("2025-06"));
    assert_eq!(m.cite_syntax.rules[0].source.as_deref(), Some("a"));
    assert_eq!(m.cite_syntax.shorthands[0].section_parts, 2);
    assert!(!m.links);
    assert_eq!(
        errors(
            &text
                .replace("heading = \"Rule {}\"", "heading = \"Rule\"")
                .replace("source = \"a\"\n\n", "source = \"z\"\n\n")
        ),
        [
            "`heading` must contain exactly one `{}`",
            "`source = \"z\"` names no `[sources.z]` table",
        ]
    );
    assert!(
        errors(&text.replace("as_of = \"2025-06\"", "as_of = \"2025-13\""))[0]
            .contains("YYYY-MM-DD or YYYY-MM")
    );
}

#[test]
fn links_manifests_have_no_pack_identity() {
    let links = "[package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\nin_force = \"0001-01-01\"\nknown_at = \"2026-10-19\"\n\n[dependencies]\ncafc = { version = \"^0.1.0\", path = \"../cafc\" }\n";
    let m = parse_manifest(links).map(|p| p.manifest);
    let m = m.expect("the manifest is valid");
    assert!(m.links);
    assert!(m.pack_id.is_empty());
    assert!(m.roles.is_empty());
    assert_eq!(
        errors(&links.replace("language = 1\n", "language = 1\ncounting = \"frcp6\"\n"))[0]
            .split(':')
            .next(),
        Some("`counting` needs a pack identity")
    );
    assert_eq!(
        errors(&MINIMAL.replace("pack_id = \"demo-pack\"\n", "")),
        ["invalid Lit.toml: missing field `pack_id`"]
    );
}
