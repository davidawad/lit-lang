//! What the host analyses and how the snapshot reads it back: the root a
//! client gives or the one guessed from a document, open documents laid
//! over the disk, rebuilds when the shape of the workspace changes, and
//! the analysis lookups the features use.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use litigate_lsp::host::{Host, Open};
use std::path::Path;

fn open(host: &mut Host, path: &Path, text: &str) {
    host.open.insert(
        path.to_path_buf(),
        Open {
            text: text.into(),
            version: 1,
        },
    );
}

fn files(host: &mut Host) -> Vec<String> {
    host.analyze().files.keys().cloned().collect()
}

#[test]
fn without_a_client_root_the_root_holds_lit() {
    let root = support::worked("ws-guess", &[]);
    let mut host = Host::default();
    let module = root.join("lit/ex/rule12.lit");
    open(
        &mut host,
        &module,
        &support::read("lit/examples/rule12.lit"),
    );
    let analysis = host.analyze();
    assert_eq!(analysis.root, root);
    assert_eq!(
        analysis.files.keys().collect::<Vec<_>>(),
        ["cases/acme.case", "lit/ex/Lit.toml", "lit/ex/rule12.lit"]
    );
    // An open document inside a checked package carries the check's
    // diagnostics, not just its parser's.
    let codes: Vec<&str> = analysis.diagnostics["lit/ex/rule12.lit"]
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["L9002", "L9002", "L9002"]);
}

#[test]
fn without_any_root_only_open_documents_are_analyzed() {
    // A guessed root is not walked whole: the package beside the open
    // document is left alone.
    let manifest = support::read("lit/examples/Lit.toml");
    let root = support::workspace("ws-loose", &[("pkg/Lit.toml", &manifest)]);
    let mut host = Host::default();
    open(&mut host, &root.join("notes/x.lit"), "module x\nstart s\n");
    let analysis = host.analyze();
    assert_eq!(analysis.root, root.join("notes"));
    assert_eq!(analysis.files.keys().collect::<Vec<_>>(), ["x.lit"]);
}

#[test]
fn a_client_root_without_lit_or_cases_is_checked_whole() {
    let manifest = support::read("lit/examples/Lit.toml");
    let module = support::read("lit/examples/rule12.lit");
    let root = support::workspace(
        "ws-whole",
        &[
            ("pkgs/ex/Lit.toml", &manifest),
            ("pkgs/ex/rule12.lit", &module),
        ],
    );
    let mut host = support::host(&root);
    assert_eq!(files(&mut host), ["pkgs/ex/Lit.toml", "pkgs/ex/rule12.lit"]);
}

#[test]
fn a_new_module_joins_its_package_and_rebuilds() {
    let root = support::worked("ws-new", &[]);
    let mut host = support::host(&root);
    let _ = host.analyze();
    assert_eq!(host.rebuilds, 1);
    let _ = host.analyze();
    assert_eq!(host.rebuilds, 1, "nothing changed");
    open(&mut host, &root.join("lit/ex/new.lit"), "module frcp.new\n");
    let all = files(&mut host);
    assert!(all.contains(&"lit/ex/new.lit".to_owned()), "{all:?}");
    assert_eq!(host.rebuilds, 2, "a new file is a new session");
}

#[test]
fn an_unreadable_matter_is_left_out() {
    let root = support::worked("ws-unreadable", &[]);
    std::fs::write(root.join("cases/bad.case"), [0xff, 0xfe, b'\n']).unwrap();
    let mut host = support::host(&root);
    let a = host.analyze();
    let cases: Vec<&str> = a.checked.cases.iter().map(|c| c.path.as_str()).collect();
    assert_eq!(cases, ["cases/acme.case"]);
}

#[test]
fn the_analysis_reads_any_file_under_the_root() {
    let root = support::worked("ws-text", &[("NOTES", "not checked\n")]);
    let mut host = support::host(&root);
    let a = host.analyze();
    assert_eq!(a.text("NOTES").as_deref(), Some("not checked\n"));
    assert_eq!(a.text("nowhere"), None);
}

#[test]
fn packages_are_found_by_pack_id_or_name() {
    // `lit/core` is the package whose pack id is not its name.
    let core: Vec<(String, String)> = std::fs::read_dir(support::repo().join("lit/core"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (
                format!("lit/core/{name}"),
                std::fs::read_to_string(p).unwrap(),
            )
        })
        .collect();
    let files: Vec<(&str, &str)> = core.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    let mut host = support::host(&support::workspace("ws-core", &files));
    let a = host.analyze();
    let by_id = a
        .hir_by_key("litigate-core")
        .map(|p| p.manifest.name.clone());
    let by_name = a.hir_by_key("core").map(|p| p.manifest.pack_id.clone());
    assert_eq!(by_id.as_deref(), Some("core"));
    assert_eq!(by_name.as_deref(), Some("litigate-core"));
    assert!(a.hir_by_key("no-such-package").is_none());
}

#[test]
fn a_cancelled_check_keeps_the_last_complete_analysis() {
    let root = support::worked("ws-cancel", &[]);
    let mut host = support::host(&root);
    let module = root.join("lit/ex/rule12.lit");
    let text = support::read("lit/examples/rule12.lit");
    // Cancelled with nothing before it: its partial analysis is current.
    let (complete, partial) = host.run(&|| true);
    assert!(!complete);
    assert!(partial.checked.verdicts.is_empty() && !partial.files.is_empty());

    // Complete, then an edit cancelled: the complete analysis answers, its
    // texts the ones it checked.
    let checked = host.analyze().checked.verdicts.len();
    assert_eq!(checked, 1);
    let edited = text.replace("\"Pleadings\"", "\"Pleading papers\"");
    open(&mut host, &module, &edited);
    let (complete, kept) = host.run(&|| true);
    assert!(!complete);
    assert_eq!(kept.files["lit/ex/rule12.lit"], text);
    assert_eq!(kept.checked.verdicts.len(), checked);
    assert_eq!(kept.checked.session.packages[0].files[1].1, text);
    // The next check reuses the database the cancelled one updated.
    let fresh = host.analyze();
    assert_eq!(fresh.files["lit/ex/rule12.lit"], edited);
    assert_eq!((host.rebuilds, host.cancelled, host.checks), (1, 2, 4));

    // A new file changes the workspace's shape: a cancelled rebuild leaves
    // the last analysis as it was.
    open(
        &mut host,
        &root.join("lit/ex/extra.lit"),
        "module frcp.extra\n",
    );
    let (complete, kept) = host.run(&|| true);
    assert!(!complete);
    assert!(!kept.files.contains_key("lit/ex/extra.lit"));
    assert_eq!(host.rebuilds, 2);
    assert!(files(&mut host).contains(&"lit/ex/extra.lit".to_owned()));
}
