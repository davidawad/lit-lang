//! `Lit.toml` validation (SPEC §4.1): every L1004 check, each driven by one
//! edit of a valid manifest, plus the identifier rules the checks share.
use litigate_base::{FileId, LineIndex};
use litigate_sema::pkg::manifest::{is_ident, is_kebab, parse_manifest};

const BASE: &str = r#"[package]
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

[sources.a]
path = "sources/a.txt"
cite_prefixes = ["A"]
"#;

const LINKS: &str = "[package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\nin_force = \"0001-01-01\"\nknown_at = \"2026-10-19\"\n";

/// `line:col CODE message` for every report of `text`.
fn errors(text: &str) -> Vec<String> {
    let index = LineIndex::new(text);
    let file = FileId("Lit.toml".to_owned());
    parse_manifest(text)
        .err()
        .unwrap_or_default()
        .iter()
        .map(|r| {
            let d = r.to_diagnostic(&file, &index);
            format!(
                "{}:{} {} {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .collect()
}

fn edit(from: &str, to: &str) -> String {
    assert!(BASE.contains(from), "{from}");
    BASE.replacen(from, to, 1)
}

#[test]
fn the_base_manifest_is_valid() {
    assert_eq!(errors(BASE), Vec::<String>::new());
}

#[test]
fn package_table_values_are_checked() {
    let cases = [
        edit("\"demo\"\nversion", "\"Demo_Pkg\"\nversion"),
        edit("\"demo-pack\"", "\"demo--pack\""),
        edit("forum = \"demo\"", "forum = \"1demo\""),
        edit("\"FederalRule\"", "\"Federal.Rule\""),
        edit(
            "known_at = \"2025-01-01\"\n",
            "known_at = \"2025-01-01\"\ncounting = \"frcp-\"\n",
        ),
        edit(
            "known_at = \"2025-01-01\"\n",
            "known_at = \"2025-01-01\"\ncounting = \"a--b\"\n",
        ),
        edit(
            "in_force = \"2020-01-01\"\n",
            "in_force = \"2020-01-01\"\nin_force_until = \"2019-12-31\"\n",
        ),
        edit(
            "in_force = \"2020-01-01\"\n",
            "in_force = \"2020-01-01\"\nin_force_until = \"2020-02-30\"\n",
        ),
        edit("applicant = \"self\"", "1applicant = \"self\""),
        edit("core = {", "Core = {"),
        format!(
            "dependencies = 1\n{}",
            edit(
                "[dependencies]\ncore = { version = \"^1.0.0\", path = \"../core\" }\n",
                ""
            )
        ),
        edit(
            "core = { version = \"^1.0.0\", path = \"../core\" }",
            "core = 1",
        ),
    ];
    let out: Vec<Vec<String>> = cases.iter().map(|text| errors(text)).collect();
    insta::assert_debug_snapshot!(out);
}

#[test]
fn source_tables_are_checked() {
    let cases = [
        edit("[sources.a]", "[sources.1a]"),
        edit("path = \"sources/a.txt\"\n", "title = \"No location\"\n"),
        edit("sources/a.txt", "/abs/a.txt"),
        edit("sources/a.txt", "sources\\\\a.txt"),
        edit("sources/a.txt", "sources/../a.txt"),
        edit("sources/a.txt", "sources//a.txt"),
        edit("cite_prefixes = [\"A\"]", "cite_prefixes = [\"A\", \" \"]"),
        edit("cite_prefixes = [\"A\"]", "sha256 = \"abc\""),
        edit(
            "cite_prefixes = [\"A\"]",
            &format!("sha256 = \"{}\"", "AB".repeat(32)),
        ),
        edit("cite_prefixes = [\"A\"]", "section_heading = \"Section\""),
        edit("cite_prefixes = [\"A\"]", "as_of = \"2025-13\""),
        edit("cite_prefixes = [\"A\"]", "as_of = \"June 2025\""),
        format!("{LINKS}\n[sources.a]\nurl = \"https://a\"\n"),
    ];
    let out: Vec<Vec<String>> = cases.iter().map(|text| errors(text)).collect();
    insta::assert_debug_snapshot!(out);
}

#[test]
fn cite_syntax_tables_are_checked() {
    let rule = "\n[[cite_syntax.rules]]\nprefixes = [\"FRCP\"]\nheading = \"Rule {}\"\n";
    let shorthand =
        "\n[[cite_syntax.shorthands]]\nprefix = \"usc\"\ncode = \"usc\"\ntitle = 35\nsection_parts = 1\n";
    let valid = format!("{BASE}{rule}{shorthand}");
    assert_eq!(errors(&valid), Vec::<String>::new());
    let cases = [
        valid.replace("[\"FRCP\"]", "[]"),
        valid.replace("[\"FRCP\"]", "[\"FRCP\", \"  \"]"),
        valid.replace("prefix = \"usc\"", "prefix = \" \""),
        valid.replace("code = \"usc\"", "code = \"ustc\""),
    ];
    let out: Vec<Vec<String>> = cases.iter().map(|text| errors(text)).collect();
    insta::assert_debug_snapshot!(out);
}

#[test]
fn optional_values_reach_the_manifest() {
    let sha = "0123456789abcdef".repeat(4);
    let text = edit(
        "in_force = \"2020-01-01\"\n",
        "in_force = \"2020-01-01\"\nin_force_until = \"2030-01-01\"\n",
    )
    .replacen(
        "[\"A\"]",
        &format!(
            "[\"A\"]\nsha256 = \"{sha}\"\nsection_heading = \"Section {{n}}\"\nas_of = \"2025-06-30\""
        ),
        1,
    )
    .replacen(", path = \"../core\"", "", 1)
    .replacen("applicant", "_applicant", 1);
    let parsed = parse_manifest(&text).unwrap_or_else(|e| panic!("{e:?}"));
    let m = &parsed.manifest;
    assert_eq!(m.in_force.to.as_deref(), Some("2030-01-01"));
    assert_eq!(m.sources[0].sha256.as_deref(), Some(sha.as_str()));
    assert_eq!(m.sources[0].section_heading.as_deref(), Some("Section {n}"));
    assert_eq!(m.sources[0].as_of.as_deref(), Some("2025-06-30"));
    // A month-only date is not a date (SPEC §4.1).
    let month = text.replacen("as_of = \"2025-06-30\"", "as_of = \"2025-06\"", 1);
    assert_eq!(
        errors(&month),
        ["25:9 L1004 `as_of` must be a YYYY-MM-DD date, found `2025-06`"]
    );
    assert!(m.roles.contains_key("_applicant"));
    assert_eq!(
        m.dependencies.get("core").map(String::as_str),
        Some("^1.0.0")
    );
    assert!(parsed.dependency_paths.is_empty());
}

#[test]
fn identifiers_and_kebab_names() {
    for (text, ident) in [
        ("a", true),
        ("_a", true),
        ("a_b-c9", true),
        ("é", true),
        ("", false),
        ("1a", false),
        ("-a", false),
        ("a-", false),
        ("a--b", false),
        ("a.b", false),
    ] {
        assert_eq!(is_ident(text), ident, "is_ident({text:?})");
    }
    for (text, kebab) in [
        ("a", true),
        ("a1-b2", true),
        ("", false),
        ("1a", false),
        ("-a", false),
        ("a-", false),
        ("a--b", false),
        ("a-B", false),
        ("a_b", false),
    ] {
        assert_eq!(is_kebab(text), kebab, "is_kebab({text:?})");
    }
}
