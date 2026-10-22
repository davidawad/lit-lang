//! Every `.lit` / `.case` file in the repository (globbed at test time)
//! parses without diagnostics, losslessly, and formats idempotently
//! without changing its structure.
mod support;

use litigate_syntax::{format, parse};
use support::{corpus, repo_root, shape};

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn corpus_is_found() {
    let files = corpus();
    assert!(
        files.len() >= 2,
        "expected at least the SPEC §12 fixtures, found {files:?}"
    );
}

#[test]
fn every_corpus_file_parses_cleanly_and_losslessly() {
    for (path, kind) in corpus() {
        let text = read(&path);
        let parsed = parse(&text, kind);
        assert!(parsed.is_ok(), "{}: {:#?}", path.display(), parsed.errors());
        assert_eq!(parsed.syntax().to_string(), text, "{}", path.display());
    }
}

#[test]
fn formatting_the_corpus_is_idempotent_and_structure_preserving() {
    for (path, kind) in corpus() {
        let text = read(&path);
        let once = format(&text, kind).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let twice = format(&once, kind).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(once, twice, "{} is not idempotent", path.display());
        assert_eq!(
            shape(&parse(&text, kind).syntax()),
            shape(&parse(&once, kind).syntax()),
            "{}: formatting changed the tree",
            path.display()
        );
    }
}

#[test]
fn worked_examples_are_canonical() {
    for relative in ["lit/examples/rule12.lit", "cases/examples/acme.case"] {
        let path = repo_root().join(relative);
        let text = read(&path);
        let kind = litigate_syntax::FileKind::from_path(&path).expect("kind");
        assert_eq!(
            format(&text, kind).as_deref(),
            Ok(text.as_str()),
            "{relative}"
        );
        let file = litigate_base::FileId::from(relative);
        assert_eq!(litigate_syntax::check_format(&text, kind, &file), None);
    }
}
