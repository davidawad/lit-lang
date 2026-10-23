//! Compiler diagnostics as LSP diagnostics and code actions: notes placed
//! in source become related information, fixes become quick fixes
//! (preferred only when machine-applicable), and what cannot be placed is
//! dropped rather than misplaced.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use litigate_base::{
    Applicability, Diagnostic, DiagnosticCode, Fix, Note, Profile, Severity, Span,
};
use litigate_cli::ws::Session;
use litigate_lsp::analysis::Analysis;
use litigate_lsp::diagnostics::{actions, publishable, to_lsp};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{DiagnosticSeverity, Position, Range};

const MODULE: &str = "lit/ex/rule12.lit";

/// Two versions of one deadline whose effective intervals overlap.
const OVERLAP: &str = "
deadline extra-period: 21.days after Served(complaint) counted_by frcp6 effective [2009-12-01, ..)
deadline extra-period: 20.days after Served(complaint) counted_by frcp6 effective [2015-01-01, ..)
";

/// `value` with `file://` URIs under `root` made root-relative.
fn relative(value: &Value, root: &Path) -> Value {
    let prefix = tower_lsp::lsp_types::Url::from_file_path(root)
        .unwrap()
        .to_string();
    serde_json::from_str(&value.to_string().replace(&prefix, "<root>")).unwrap()
}

fn all() -> Range {
    Range::new(Position::new(0, 0), Position::new(9999, 0))
}

#[test]
fn an_overlap_points_at_the_other_version_and_offers_a_reviewed_fix() {
    let text = format!("{}{OVERLAP}", support::read("lit/examples/rule12.lit"));
    let root = support::worked("diagnostics-overlap", &[(MODULE, &text)]);
    let mut host = support::host(&root);
    let a = host.analyze();
    let overlap = a.diagnostics[MODULE]
        .iter()
        .find(|d| d.code == DiagnosticCode::EffectiveOverlap)
        .expect("L3001");
    let lsp = serde_json::to_value(to_lsp(a, overlap, &text)).unwrap();
    insta::assert_json_snapshot!("overlap_lsp", relative(&lsp, &root));
    let line = u32::try_from(text.lines().count() - 1).unwrap();
    let here = Range::new(Position::new(line, 0), Position::new(line, 200));
    let fixes: Vec<Value> = actions(a, MODULE, here)
        .into_iter()
        .filter(|c| c.title.starts_with("L3001"))
        .map(|c| relative(&serde_json::to_value(c).unwrap(), &root))
        .map(|mut c| {
            // The attached diagnostic is the snapshot above.
            c["diagnostics"] = json!("<overlap_lsp>");
            c
        })
        .collect();
    insta::assert_json_snapshot!("overlap_actions", fixes);
}

fn span(file: &str, line: u32) -> Span {
    Span {
        file: file.into(),
        line,
        col: 1,
        end_line: line,
        end_col: 3,
    }
}

/// An analysis of one loose document at `root`, with `diagnostics` on it.
fn loose(root: &Path, diagnostics: Vec<Diagnostic>) -> Analysis {
    let checked = litigate_cli::check_session(
        Session::from_loaded(&[], Vec::new()),
        Profile::Dev,
        &litigate_lower::Options::default(),
    );
    let open: BTreeMap<PathBuf, String> =
        std::iter::once((root.join("loose.lit"), "module loose\nstart s\n".to_owned())).collect();
    let mut analysis = Analysis::new(root.to_path_buf(), checked, open);
    analysis.diagnostics.insert("loose.lit".into(), diagnostics);
    analysis
}

fn hint() -> Diagnostic {
    Diagnostic {
        severity: Severity::Hint,
        notes: vec![
            Note {
                span: Some(span("missing.lit", 1)),
                message: "placed in a file that is gone".into(),
            },
            Note {
                span: None,
                message: "unplaced".into(),
            },
        ],
        fixes: vec![
            Fix {
                span: span("loose.lit", 2),
                replacement: String::new(),
                applicability: Applicability::MachineApplicable,
            },
            Fix {
                span: span("missing.lit", 1),
                replacement: "x".into(),
                applicability: Applicability::MachineApplicable,
            },
        ],
        ..Diagnostic::error(DiagnosticCode::UnexpectedToken, span("loose.lit", 2), "odd")
    }
}

#[test]
fn hints_removals_and_vanished_files() {
    let root = support::workspace("diagnostics-hint", &[]);
    let a = loose(&root, vec![hint()]);
    let lsp = to_lsp(&a, &hint(), "module loose\nstart s\n");
    assert_eq!(lsp.severity, Some(DiagnosticSeverity::HINT));
    assert_eq!(lsp.message, "odd\nnote: unplaced");
    assert_eq!(lsp.related_information, None, "the note's file is gone");
    let titles: Vec<(String, Option<bool>)> = actions(&a, "loose.lit", all())
        .into_iter()
        .map(|c| (c.title, c.is_preferred))
        .collect();
    assert_eq!(titles, [("L0001: remove".to_owned(), Some(true))]);
    assert!(actions(&a, "missing.lit", all()).is_empty());
}

#[test]
fn without_an_absolute_root_nothing_is_placed() {
    // URIs need absolute paths: a relative root places nothing.
    let a = loose(Path::new(""), vec![hint()]);
    assert_eq!(publishable(&a), Vec::new());
    assert!(actions(&a, "loose.lit", all()).is_empty());
    let mut placed = hint();
    placed.notes[0].span = Some(span("loose.lit", 1));
    assert_eq!(
        to_lsp(&a, &placed, "module loose\n").related_information,
        None
    );
}
