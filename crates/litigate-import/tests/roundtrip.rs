//! Bead acceptance over the whole repository corpus: every pack,
//! `links.json` and named scenario imports to canonical source that parses
//! with zero diagnostics, and reading that source back (the CST oracle in
//! `support::reader`, following SPEC §8) gives JSON semantically equal to
//! the original (SPEC §8.6).
mod support;

use litigate_engine::model::{LinkFile, Pack};
use litigate_engine::scenario::NamedScenario;
use litigate_import::Input;
use litigate_syntax::{format, parse, FileKind};
use support::{
    corpus, first_difference, import_ok, normalize_links, normalize_pack, package, read_case,
    read_links, read_pack,
};

#[test]
fn the_corpus_is_found() {
    let inputs = corpus();
    let packs = inputs
        .iter()
        .filter(|i| i.path.starts_with("packs/"))
        .count();
    let scenarios = inputs.len() - packs;
    assert_eq!(
        (packs, scenarios),
        (10, 5),
        "nine packs + links.json, five scenarios"
    );
}

#[test]
fn every_output_parses_cleanly_and_is_canonical() {
    let files = import_ok(&corpus());
    for file in &files {
        let kind = match file.path.rsplit('.').next() {
            Some("lit") => FileKind::Lit,
            Some("case") => FileKind::Case,
            _ => continue,
        };
        let parsed = parse(&file.text, kind);
        assert!(parsed.is_ok(), "{}: {:#?}", file.path, parsed.errors());
        assert_eq!(
            format(&file.text, kind).as_deref(),
            Ok(file.text.as_str()),
            "{} is not canonical",
            file.path
        );
    }
}

#[test]
fn every_pack_round_trips() {
    let inputs = corpus();
    let files = import_ok(&inputs);
    for input in inputs
        .iter()
        .filter(|i| i.path.starts_with("packs/") && !i.path.ends_with("links.json"))
    {
        let original =
            Pack::from_json(&input.text).unwrap_or_else(|e| panic!("{}: {e}", input.path));
        let dir = files
            .iter()
            .find(|f| f.from == input.path && f.path.ends_with("/Lit.toml"))
            .and_then(|f| f.path.strip_suffix("/Lit.toml"))
            .unwrap_or_else(|| panic!("{}: no package", input.path));
        let back = read_pack(&package(&files, dir));
        let (expected, actual) = (normalize_pack(original), normalize_pack(back));
        assert!(
            expected == actual,
            "{}: {:?}",
            input.path,
            first_difference(&expected, &actual, "")
        );
    }
}

#[test]
fn links_round_trip() {
    let inputs = corpus();
    let files = import_ok(&inputs);
    let input = inputs.iter().find(|i| i.path == "packs/links.json");
    let original: LinkFile = input
        .and_then(|i| serde_json::from_str(&i.text).ok())
        .unwrap_or_default();
    let back = read_links(&package(&files, "lit/links"));
    let (expected, actual) = (normalize_links(original), normalize_links(back));
    assert!(
        expected == actual,
        "{:?}",
        first_difference(&expected, &actual, "")
    );
}

#[test]
fn every_scenario_round_trips() {
    let inputs = corpus();
    let files = import_ok(&inputs);
    for input in inputs.iter().filter(|i| i.path.starts_with("scenarios/")) {
        let original =
            NamedScenario::from_json(&input.text).unwrap_or_else(|e| panic!("{}: {e}", input.path));
        let file = files
            .iter()
            .find(|f| f.from == input.path)
            .unwrap_or_else(|| panic!("{}: no case", input.path));
        let back = read_case(file);
        // Import writes the replayed posture, with the docket only as a
        // comment (SPEC §8.7), so a v2 file's events and `lowered` header
        // do not come back; its parties and explicit holes do.
        let holes: Vec<(String, Option<String>)> = original
            .lowered
            .iter()
            .flat_map(|l| &l.holes)
            .map(|h| (h.pointer.clone(), h.reason.clone()))
            .collect();
        let mut back_holes = support::case_holes(file);
        back_holes.sort();
        assert_eq!(holes, back_holes, "{}: holes", input.path);
        let as_json = |s: &NamedScenario| {
            let mut s = s.clone();
            s.schema_version = None;
            s.events.clear();
            s.lowered = None;
            support::numbers_as_f64(serde_json::to_value(&s).unwrap_or_default())
        };
        let (expected, actual) = (as_json(&original), as_json(&back));
        assert!(
            expected == actual,
            "{}: {:?}",
            input.path,
            first_difference(&expected, &actual, "")
        );
    }
}

#[test]
fn import_is_deterministic_and_independent_of_input_order() {
    let inputs = corpus();
    let mut reversed: Vec<Input> = inputs.clone();
    reversed.reverse();
    assert_eq!(import_ok(&inputs), import_ok(&inputs));
    assert_eq!(import_ok(&inputs), import_ok(&reversed));
}

#[test]
fn outputs_carry_no_absolute_paths() {
    let files = import_ok(&corpus());
    let root = support::repo_root().canonicalize().unwrap_or_default();
    let root = root.to_string_lossy();
    for file in &files {
        assert!(!file.text.contains(root.as_ref()), "{}", file.path);
    }
}

/// Every `ERROR` or `MISSING` node of the editor grammar's parse.
fn tree_sitter_errors(text: &str) -> Vec<String> {
    fn walk(node: tree_sitter::Node<'_>, out: &mut Vec<String>) {
        if node.is_error() || node.is_missing() {
            let at = node.start_position();
            out.push(format!("{}:{} {}", at.row + 1, at.column + 1, node.kind()));
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(child, out);
        }
    }
    let mut parser = tree_sitter::Parser::new();
    let mut out = Vec::new();
    if parser
        .set_language(&tree_sitter_litigate::LANGUAGE.into())
        .is_err()
    {
        return vec!["grammar failed to load".to_owned()];
    }
    match parser.parse(text, None) {
        Some(tree) => walk(tree.root_node(), &mut out),
        None => out.push("no tree".to_owned()),
    }
    out
}

/// Porters commit this output under `lit/` and `cases/`, where the
/// tree-sitter conformance test parses it too; it must already agree.
#[test]
fn the_editor_grammar_parses_every_output() {
    for file in import_ok(&corpus()) {
        if support::is_ext(&file.path, "lit") || support::is_ext(&file.path, "case") {
            let errors = tree_sitter_errors(&file.text);
            assert!(errors.is_empty(), "{}: {errors:?}", file.path);
        }
    }
}
