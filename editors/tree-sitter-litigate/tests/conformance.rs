//! Conformance between the tree-sitter grammar and the litigate corpus
//! (DESIGN §6, SPEC §3): every `.lit`/`.case` file in the repository parses
//! with zero `ERROR`/`MISSING` nodes, the grammar's contextual keywords are
//! exactly the compiler's (`litigate_syntax::SyntaxKind`), and juxtaposed
//! items and citations behave as SPEC §2-§3 say.
//!
//! The compiler's parser is the source of truth: when this test fails on a
//! file the compiler accepts, fix the grammar, not the file. A deliberately
//! malformed fixture must not use the `.lit`/`.case` extension.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_syntax::syntax_kind::SyntaxKind;
use proptest::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Tree};

/// Directories that hold build output or third-party code, never sources.
const SKIPPED_DIRS: [&str; 4] = [".git", "target", "node_modules", ".worktrees"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn parse(text: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_litigate::LANGUAGE.into())
        .expect("grammar loads");
    parser.parse(text, None).expect("parse returns a tree")
}

/// Every `ERROR` or `MISSING` node as `line:col kind`, one-based.
fn syntax_errors(tree: &Tree) -> Vec<String> {
    fn walk(node: Node<'_>, out: &mut Vec<String>) {
        if node.is_error() || node.is_missing() {
            let at = node.start_position();
            let what = if node.is_missing() {
                format!("MISSING {}", node.kind())
            } else {
                "ERROR".to_owned()
            };
            out.push(format!("{}:{} {what}", at.row + 1, at.column + 1));
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(tree.root_node(), &mut out);
    out
}

fn corpus_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            if !SKIPPED_DIRS.contains(&name) {
                corpus_files(&path, out);
            }
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("lit" | "case")
        ) {
            out.push(path);
        }
    }
}

#[test]
fn every_repository_source_parses_without_errors() {
    let root = repo_root();
    let mut files = Vec::new();
    corpus_files(&root, &mut files);
    let relative: BTreeSet<String> = files
        .iter()
        .filter_map(|f| f.strip_prefix(&root).ok())
        .map(|f| f.to_string_lossy().replace('\\', "/"))
        .collect();
    for required in ["lit/examples/rule12.lit", "cases/examples/acme.case"] {
        assert!(relative.contains(required), "corpus walk missed {required}");
    }
    let failures: Vec<String> = files
        .iter()
        .flat_map(|file| {
            let text = std::fs::read_to_string(file).unwrap_or_default();
            let shown = file
                .strip_prefix(&root)
                .unwrap_or(file)
                .display()
                .to_string();
            syntax_errors(&parse(&text))
                .into_iter()
                .map(move |error| format!("{shown}:{error}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "tree-sitter rejects:\n{}",
        failures.join("\n")
    );
}

#[test]
fn worked_examples_have_the_expected_roots() {
    let root = repo_root();
    for (file, kind) in [
        ("lit/examples/rule12.lit", "lit_file"),
        ("cases/examples/acme.case", "case_file"),
    ] {
        let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
        let tree = parse(&text);
        let top = tree.root_node().child(0).map(|n| n.kind().to_owned());
        assert_eq!(top.as_deref(), Some(kind), "{file}");
    }
}

/// Word-shaped anonymous nodes plus keywords the grammar aliases to named
/// nodes (`unit`, `unvalued`, `discretion`).
fn grammar_keywords() -> BTreeSet<String> {
    let language: tree_sitter::Language = tree_sitter_litigate::LANGUAGE.into();
    let aliased = ["unit", "unvalued", "discretion"];
    (0..language.node_kind_count())
        .filter_map(|id| u16::try_from(id).ok())
        .filter(|&id| !language.node_kind_is_named(id) && language.node_kind_is_visible(id))
        .filter_map(|id| language.node_kind_for_id(id))
        .chain(aliased)
        .filter(|kind| kind.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        .filter(|kind| *kind != "_")
        .map(str::to_owned)
        .collect()
}

#[test]
fn keywords_match_the_compiler() {
    let compiler: BTreeSet<String> = SyntaxKind::ALL
        .iter()
        .filter(|kind| kind.is_keyword())
        .filter_map(|kind| kind.text())
        .map(str::to_owned)
        .collect();
    let grammar = grammar_keywords();
    let only_grammar: Vec<_> = grammar.difference(&compiler).collect();
    let only_compiler: Vec<_> = compiler.difference(&grammar).collect();
    assert!(
        only_grammar.is_empty() && only_compiler.is_empty(),
        "grammar-only keywords {only_grammar:?}; compiler-only keywords {only_compiler:?}"
    );
}

/// The reserved refinement forms (SPEC §3.5): the grammar gives each a
/// node so editors highlight it, and the compiler rejects each with L0009
/// only, so the slot never shadows another error.
#[test]
fn reserved_refinements_parse_here_and_are_l0009_in_the_compiler() {
    for (source, node) in [
        (
            "module m\ntype T = {d: Days | d > 0.days}\n",
            "refined_type",
        ),
        (
            "module m\ntype T = Map<Date, {d: Days | d > 0.days}>\n",
            "refined_type",
        ),
        (
            "module m\nevent E(on: {t: Date | t >= 2026-01-01})\n",
            "refined_type",
        ),
        ("module m\ntype T = Days where T > 0.days\n", "where_clause"),
        (
            "module m\nevent E(on: Date where on >= 2026-01-01, x: E)\n",
            "where_clause",
        ),
    ] {
        let tree = parse(source);
        assert!(syntax_errors(&tree).is_empty(), "{source}");
        assert!(find(tree.root_node(), node).is_some(), "{source}");
        let compiled = litigate_syntax::parse_lit(source);
        let codes: Vec<&str> = compiled.errors().iter().map(|e| e.code.as_str()).collect();
        assert_eq!(codes, ["L0009"], "{source}");
        assert!(compiled.errors()[0]
            .message
            .starts_with(litigate_syntax::RESERVED_REFINEMENT));
    }
}

/// The guard operations of a patch (SPEC §3.6): the grammar gives each a
/// `guard_op` node, and the compiler's parser accepts each (sema checks
/// the authority it needs).
#[test]
fn guard_operations_parse_here_and_in_the_compiler() {
    for op in [
        "when and has_event(Served()) by CaseOrder",
        "when or date_of(Served()) < 2026-01-01 effective [2026-11-01, ..)",
        "supersedes when true known_at 2026-10-01 cite \"Fed. Cir. R. 1\"",
        "supersedes when false by FederalRule cite ?(\"Order, Dkt. 12\")",
    ] {
        let source = format!("module links\ninstance a@b {{\n  patch go {{\n    {op}\n  }}\n}}\n");
        let tree = parse(&source);
        assert!(syntax_errors(&tree).is_empty(), "{source}");
        assert!(find(tree.root_node(), "guard_op").is_some(), "{source}");
        let compiled = litigate_syntax::parse_lit(&source);
        assert!(compiled.is_ok(), "{source}: {:?}", compiled.errors());
    }
}

/// The reserved filing requirements (SPEC §3.7): the grammar gives a
/// `limit` declaration and each body entry a node so editors highlight
/// them, and the compiler rejects them with L0011 only (once per reserved
/// word), so the slot never shadows another error.
#[test]
fn reserved_filing_limits_parse_here_and_are_l0011_in_the_compiler() {
    for (entry, node, reserved) in [
        (
            "words <= 13_000 cite \"FRAP 32(a)(7)(B)(i)\"",
            "limit_bound",
            1,
        ),
        ("pages <= ?(\"local rule\")", "limit_bound", 1),
        ("lines <= 1300", "limit_bound", 1),
        (
            "requires certificate compliance cite \"FRAP 32(g)(1)\"",
            "certificate_req",
            2,
        ),
        (
            "excludes [cover-page, ?(\"local rule\")] cite \"FRAP 32(f)\"",
            "exclusion_list",
            2,
        ),
        ("ecf \"code\"", "ecf_code", 1),
        ("ecf ?(\"district ECF event code\")", "ecf_code", 1),
    ] {
        let source = format!(
            "module m\nlimit brief effective [1998-12-01, ..) {{\n  {entry}\n}} cite \"FRAP 32\"\nstart s\n"
        );
        let tree = parse(&source);
        assert!(syntax_errors(&tree).is_empty(), "{source}");
        assert!(find(tree.root_node(), "limit_decl").is_some(), "{source}");
        assert!(find(tree.root_node(), node).is_some(), "{source}");
        let compiled = litigate_syntax::parse_lit(&source);
        let codes: Vec<&str> = compiled.errors().iter().map(|e| e.code.as_str()).collect();
        assert_eq!(codes, vec!["L0011"; reserved], "{source}");
        for error in compiled.errors() {
            assert!(error.message.starts_with(litigate_syntax::RESERVED_FILING));
        }
    }
    // A local rule or order displaces a limit through the relations (§5.5).
    let source = "module m\nlimit local::brief override frap::brief {\n  words <= ?\n} \
                  cite ?(\"local rule\")\n";
    let tree = parse(source);
    assert!(syntax_errors(&tree).is_empty(), "{source}");
    assert!(find(tree.root_node(), "relation").is_some(), "{source}");
    let codes: Vec<&str> = litigate_syntax::parse_lit(source)
        .errors()
        .iter()
        .map(|e| e.code.as_str())
        .collect();
    assert_eq!(codes, ["L0011"], "{source}");
}

/// Lexical rejection parity (SPEC §2): input the compiler rejects as an
/// invalid character, date or number is an `ERROR`, `MISSING` or
/// `invalid_date` node here too, never a clean tree.
fn rejected_here(text: &str) -> bool {
    let tree = parse(text);
    !syntax_errors(&tree).is_empty() || find(tree.root_node(), "invalid_date").is_some()
}

#[test]
fn what_the_compiler_rejects_lexically_is_rejected_here() {
    for (text, code) in [
        ("default d when x == 2026-02-30 => 1", "L0004"),
        ("default d when x == 2025-02-29 => 1", "L0004"),
        ("default d when x == 1900-02-29 => 1", "L0004"),
        ("default d when x == 2026-13-01 => 1", "L0004"),
        ("default d when x == 2026-04-31 => 1", "L0004"),
        ("test \"t\" { given Ev(a) on 2026-02-30 expect true }", "L0004"),
        ("default d when x == 1__0 => 1", "L0005"),
        ("default d when x == 1_ => 1", "L0005"),
        ("default d when x == 2__0.usd => 1", "L0005"),
        ("default d when x == 3.0__1.days => 1", "L0005"),
        ("default d when\u{a0}true => 1", "L0003"),
        ("default d when\u{2003}true => 1", "L0003"),
        ("default d when\u{c}true => 1", "L0003"),
    ] {
        let source = format!("module m\n{text}\n");
        let codes: Vec<&str> = litigate_syntax::parse_lit(&source)
            .errors()
            .iter()
            .map(|e| e.code.as_str())
            .collect();
        assert!(codes.contains(&code), "{source:?}: {codes:?}");
        assert!(rejected_here(&source), "tree-sitter accepts {source:?}");
    }
    for date in ["2024-02-29", "2000-02-29", "0000-02-29", "1900-02-28", "2026-12-31"] {
        let source = format!("module m\ndefault d when x == {date} => 1\n");
        assert!(litigate_syntax::parse_lit(&source).is_ok(), "{source}");
        assert!(!rejected_here(&source), "{source}");
    }
}

/// Self-contained top-level items; any sequence of them is a valid module
/// body because items are juxtaposed (SPEC §3.1).
const ITEMS: [&str; 12] = [
    "start s",
    "group g: \"G\"",
    "enum E { a, b }",
    "event Ev(x: E)",
    "authority A > B",
    "state s: \"S\" by Court cite \"FRCP 1\"\n{ note \"n\", go \"Go\" -> t p 1/2 }",
    "decision d: \"D\" { p \"P\" -> t by X\n by \"B\" -> t { cost ? } }",
    "terminal t: \"T\" outcome [win] { payoff -1.usd }",
    "deadline dl: 21.days from Ev(a) counted_by c",
    "counting c { exclude_trigger_day, anchor trigger_day }",
    "default df when has_event(Ev(a)) => match x { _ => 1 }",
    "test \"t\" { given Ev(a) on 2025-01-01 expect deadline(dl) == 2025-01-22 }",
];

/// Kebab-case identifier (SPEC §2): hyphens only between word characters.
fn identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,6}(-[a-z0-9]{1,4}){0,3}"
}

proptest! {
    #[test]
    fn prop_juxtaposed_items_parse(picks in prop::collection::vec(0..ITEMS.len(), 0..16)) {
        let body: Vec<&str> = picks.iter().map(|&i| ITEMS[i]).collect();
        let text = format!("module m\n{}\n", body.join("\n"));
        let errors = syntax_errors(&parse(&text));
        prop_assert!(errors.is_empty(), "{errors:?} in\n{text}");
    }

    #[test]
    fn prop_cite_text_is_a_string(
        text in "[A-Za-z0-9.()§ ,{}/;]{0,20}[A-Za-z0-9.()§]",
        historical in prop::bool::ANY,
    ) {
        let mark = if historical { " historical" } else { "" };
        let source = format!("module m\nstate s: \"S\" cite \"{text}\"{mark} // note\n");
        let tree = parse(&source);
        prop_assert!(syntax_errors(&tree).is_empty());
        let cite = find(tree.root_node(), "cite").expect("a cite node");
        let string = cite.child_by_field_name("text").expect("its text");
        prop_assert_eq!(&source[string.byte_range()], format!("\"{text}\""));
        prop_assert_eq!(cite.child_by_field_name("historical").is_some(), historical);
    }

    #[test]
    fn prop_kebab_identifiers_are_single_tokens(name in identifier()) {
        let source = format!("module m\nstate {name}: \"S\"\n");
        let tree = parse(&source);
        prop_assert!(syntax_errors(&tree).is_empty(), "{}", source);
        let node = find(tree.root_node(), "node_decl")
            .and_then(|n| n.child_by_field_name("name"))
            .expect("a node name");
        prop_assert_eq!(&source[node.byte_range()], name.as_str());
    }

    /// Every date-shaped literal: accepted here exactly when the compiler
    /// accepts it (SPEC §2, L0004).
    #[test]
    fn prop_dates_agree_with_the_compiler(y in 0u32..10_000, m in 0u32..100, d in 0u32..100) {
        let source = format!("module m\ndefault d when x == {y:04}-{m:02}-{d:02} => 1\n");
        let valid = litigate_syntax::parse_lit(&source).is_ok();
        prop_assert_eq!(rejected_here(&source), !valid, "{}", source);
    }

    /// Digit grouping: a number is accepted here exactly when the compiler
    /// accepts it (SPEC §2, L0005).
    #[test]
    fn prop_digit_groups_agree_with_the_compiler(n in "[0-9][0-9_]{0,5}(\\.[0-9][0-9_]{0,3})?") {
        let source = format!("module m\ndefault d when x == {n} => 1\n");
        let valid = litigate_syntax::parse_lit(&source).is_ok();
        prop_assert_eq!(rejected_here(&source), !valid, "{}", source);
    }

    #[test]
    fn prop_any_input_yields_a_total_tree(text in "\\PC{0,200}") {
        let tree = parse(&text);
        prop_assert!(tree.root_node().end_byte() <= text.len());
    }
}

fn find<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    let children: Vec<Node<'t>> = node.children(&mut cursor).collect();
    children.into_iter().find_map(|child| find(child, kind))
}
