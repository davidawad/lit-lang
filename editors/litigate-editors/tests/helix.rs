//! Helix configuration: the generated queries are current, every Helix
//! query compiles against the grammar and runs over the corpus, the pinned
//! grammar `rev` carries the grammar, and (when `hx` is installed)
//! `hx --grammar build` plus `hx --health litigate` succeed.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_editors::support::{find_program, run, scratch_dir, skip};
use litigate_editors::{corpus, editors_dir, grammar, helix, repo_root};
use std::path::Path;
use std::process::Command;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator as _};

fn language() -> tree_sitter::Language {
    tree_sitter_litigate::LANGUAGE.into()
}

#[test]
fn generated_queries_are_current() {
    let stale = helix::stale().unwrap();
    assert!(
        stale.is_empty(),
        "stale Helix queries {stale:?}; run `cargo run -p litigate-editors --bin helix-queries`"
    );
}

#[test]
fn every_helix_query_compiles_and_runs_over_the_corpus() {
    let files = corpus::files(&repo_root()).unwrap();
    assert!(files.len() >= 2, "corpus: {files:?}");
    let mut parser = Parser::new();
    parser.set_language(&language()).unwrap();
    let mut names: Vec<_> = std::fs::read_dir(helix::queries_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "highlights.scm",
            "indents.scm",
            "injections.scm",
            "locals.scm",
            "textobjects.scm"
        ]
    );
    for name in names {
        let source = std::fs::read_to_string(helix::queries_dir().join(&name)).unwrap();
        let query =
            Query::new(&language(), &source).unwrap_or_else(|error| panic!("{name}: {error}"));
        for file in &files {
            let text = std::fs::read_to_string(file).unwrap();
            let tree = parser.parse(&text, None).unwrap();
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
            while matches.next().is_some() {}
        }
    }
}

#[test]
fn and_or_are_highlighted_as_keywords_only_in_guard_operations() {
    let source = "module m\nstart and\ninstance base@one {\n  pack base\n  patch go {\n    when and ready\n    when or permitted\n  }\n}\n";
    let mut parser = Parser::new();
    parser.set_language(&language()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let query_source =
        std::fs::read_to_string(helix::queries_dir().join("highlights.scm")).unwrap();
    let query = Query::new(&language(), &query_source).unwrap();
    let keyword = query
        .capture_names()
        .iter()
        .position(|name| *name == "keyword")
        .unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut highlighted = Vec::new();
    while let Some(query_match) = matches.next() {
        highlighted.extend(
            query_match
                .captures()
                .iter()
                .filter(|capture| capture.index as usize == keyword)
                .map(|capture| {
                    capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap()
                        .to_owned()
                }),
        );
    }
    assert_eq!(
        highlighted
            .iter()
            .filter(|word| word.as_str() == "and")
            .count(),
        1
    );
    assert_eq!(
        highlighted
            .iter()
            .filter(|word| word.as_str() == "or")
            .count(),
        1
    );
}

#[test]
fn textobjects_use_helix_object_names() {
    let source = std::fs::read_to_string(helix::queries_dir().join("textobjects.scm")).unwrap();
    let query = Query::new(&language(), &source).unwrap();
    for capture in query.capture_names() {
        let (object, part) = capture.split_once('.').unwrap();
        assert!(
            ["function", "test", "entry", "parameter", "comment"].contains(&object),
            "{capture}"
        );
        assert!(["inside", "around"].contains(&part), "{capture}");
    }
}

fn languages_toml() -> String {
    std::fs::read_to_string(editors_dir().join("helix/languages.toml")).unwrap()
}

#[test]
fn languages_toml_pins_a_rev_that_carries_the_grammar() {
    let toml = languages_toml();
    let rev = toml
        .lines()
        .find(|line| line.starts_with("source = { git ="))
        .and_then(|line| line.split("rev = \"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .unwrap();
    assert_eq!(rev.len(), 40, "rev must be a full commit sha: {rev}");
    let known = Command::new("git")
        .current_dir(repo_root())
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .status()
        .is_ok_and(|status| status.success());
    if !known {
        skip(
            "helix::languages_toml_pins_a_rev",
            &format!("rev {rev} not in this clone"),
        );
        return;
    }
    run(Command::new("git").current_dir(repo_root()).args([
        "cat-file",
        "-e",
        &format!("{rev}:editors/tree-sitter-litigate/src/parser.c"),
    ]))
    .unwrap();
}

#[test]
fn hx_builds_the_grammar_and_reports_health() {
    let Some(hx) = find_program("HX", "hx") else {
        skip(
            "helix::hx_builds_the_grammar_and_reports_health",
            "hx not on PATH",
        );
        return;
    };
    let config = scratch_dir(Path::new(env!("CARGO_TARGET_TMPDIR")), "helix-config").unwrap();
    let helix_dir = config.join("helix");
    std::fs::create_dir_all(helix_dir.join("runtime/grammars")).unwrap();
    let queries = helix_dir.join("runtime/queries/litigate");
    std::fs::create_dir_all(&queries).unwrap();
    for entry in std::fs::read_dir(helix::queries_dir()).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), queries.join(entry.file_name())).unwrap();
    }
    let local_source = format!(
        "source = {{ path = \"{}\" }}",
        grammar::grammar_dir().display()
    );
    let toml: Vec<String> = std::iter::once("use-grammars = { only = [\"litigate\"] }".to_owned())
        .chain(languages_toml().lines().map(|line| {
            if line.starts_with("source = { git =") {
                local_source.clone()
            } else {
                line.to_owned()
            }
        }))
        .collect();
    std::fs::write(helix_dir.join("languages.toml"), toml.join("\n") + "\n").unwrap();

    let hx_command = |args: &[&str]| {
        let mut command = Command::new(&hx);
        command
            // Helix takes its runtime dir from CARGO_MANIFEST_DIR when set.
            .env_remove("CARGO_MANIFEST_DIR")
            .env("XDG_CONFIG_HOME", &config)
            .env("NO_COLOR", "1")
            .args(args);
        command
    };
    run(&mut hx_command(&["--grammar", "build"])).unwrap();
    let built = ["litigate.so", "litigate.dylib", "litigate.dll"]
        .iter()
        .any(|name| helix_dir.join("runtime/grammars").join(name).is_file());
    assert!(built, "hx --grammar build produced no litigate grammar");
    let health = run(&mut hx_command(&["--health", "litigate"])).unwrap();
    let health = strip_ansi(&String::from_utf8_lossy(&health.stdout));
    for line in [
        "Tree-sitter parser: ✓",
        "Highlight queries: ✓",
        "Textobject queries: ✓",
        "Indent queries: ✓",
    ] {
        assert!(health.contains(line), "missing {line:?} in:\n{health}");
    }
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
