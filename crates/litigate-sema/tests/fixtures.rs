//! The SPEC §12 worked example, checked end to end from real `.lit` source.
use litigate_base::DiagnosticCode as C;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn worked_example_checks_and_lowers_to_hir() -> std::io::Result<()> {
    let repo = repo();
    let (db, ws) = litigate_sema::load(&repo, &[repo.join("lit/examples")], None)?;
    let checks = litigate_sema::check_workspace(&db, ws);
    assert_eq!(checks.len(), 1);
    let check = checks[0];
    let codes: Vec<(C, String)> = check
        .diagnostics
        .iter()
        .map(|d| (d.code, d.message.clone()))
        .collect();
    assert_eq!(codes, vec![]);
    assert!(check.holes.is_empty());
    insta::assert_json_snapshot!("rule12_hir", check.package);
    Ok(())
}

#[test]
fn worked_example_hole_reasons_survive_the_sema_check() {
    use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};

    let source = include_str!("../../../lit/examples/rule12.lit")
        .replace(
            "granted \"Motion granted\" -> dismissed p 1/2",
            "granted \"Motion granted\" -> dismissed p ?(\"no calibration yet\")",
        )
        .replace(
            "denied \"Motion denied\" -> answered p 1/2",
            "denied \"Motion denied\" -> answered p ?",
        )
        .replacen("payoff 0.usd", "payoff ?", 1);
    let package = LoadedPackage {
        texts: std::collections::BTreeMap::default(),
        root: "lit/examples".to_owned(),
        manifest: include_str!("../../../lit/examples/Lit.toml").to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: "rule12.lit".to_owned(),
            text: source,
        }],
    };
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[package]);
    let checks = litigate_sema::check_workspace(&db, ws);
    assert_eq!(checks.len(), 1);
    let reasons: Vec<Option<&str>> = checks[0]
        .holes
        .iter()
        .map(|hole| hole.reason.as_deref())
        .collect();
    assert_eq!(reasons, [Some("no calibration yet"), None, None]);
}

#[test]
fn omitted_edge_ids_lower_stably_and_disambiguate_parallel_targets() {
    use litigate_sema::hir::Declaration;
    use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};

    let source = include_str!("../../../lit/examples/rule12.lit")
        .replace("barred \"Barred\" -> dismissed", "\"Barred\" -> dismissed")
        .replace(
            "not-barred \"Timely\" -> respond",
            "\"Timely\" -> dismissed",
        );
    let package = LoadedPackage {
        texts: std::collections::BTreeMap::default(),
        root: "lit/examples".to_owned(),
        manifest: include_str!("../../../lit/examples/Lit.toml").to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: "rule12.lit".to_owned(),
            text: source,
        }],
    };
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[package]);
    let checks = litigate_sema::check_workspace(&db, ws);
    assert!(
        checks[0].diagnostics.is_empty(),
        "{:#?}",
        checks[0].diagnostics
    );
    let node = checks[0]
        .package
        .as_ref()
        .unwrap()
        .modules
        .iter()
        .flat_map(|module| &module.declarations)
        .find_map(|declaration| match declaration {
            Declaration::Node(node) if node.meta.id == "time-barred" => Some(node),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        node.edges
            .iter()
            .map(|edge| edge.id.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("edge-time-barred-dismissed-1"),
            Some("edge-time-barred-dismissed-2")
        ]
    );
}

#[test]
fn derived_edge_id_collision_names_both_edges() {
    use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};

    let source = include_str!("../../../lit/examples/rule12.lit")
        .replace("barred \"Barred\" -> dismissed", "\"Barred\" -> dismissed")
        .replace(
            "not-barred \"Timely\" -> respond",
            "edge-time-barred-dismissed \"Timely\" -> respond",
        );
    let package = LoadedPackage {
        texts: std::collections::BTreeMap::default(),
        root: "lit/examples".to_owned(),
        manifest: include_str!("../../../lit/examples/Lit.toml").to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: "rule12.lit".to_owned(),
            text: source,
        }],
    };
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[package]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let duplicate = checks[0]
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == C::DuplicateName)
        .expect("the explicit edge id collides with the derived id");
    assert!(duplicate.message.contains("dismissed"), "{duplicate:#?}");
    assert!(duplicate.message.contains("respond"), "{duplicate:#?}");
}

#[test]
fn module_forum_is_inherited_by_deadlines_and_declaration_options_override_it() {
    use litigate_sema::db::parse::tree::{Atom, ExprKind, ItemKind};

    let parsed = litigate_sema::db::parse::parse(
        "module m\nforum frap\ndeadline own: 2.days from Served() counted_by c { forum frcp }\ndeadline inherited: 3.days from Served() counted_by c\n",
    );
    assert!(parsed.reports.is_empty(), "{:#?}", parsed.reports);
    let forums: Vec<_> = parsed
        .tree
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ItemKind::Deadline { opts, .. } => opts
                .iter()
                .find(|field| field.key.text == "forum")
                .and_then(|field| field.values.first())
                .and_then(|(atom, _)| match atom {
                    Atom::Expr(expr) => match &expr.kind {
                        ExprKind::Ref(path) => Some(path.head.text.as_str()),
                        _ => None,
                    },
                    _ => None,
                }),
            _ => None,
        })
        .collect();
    assert_eq!(forums, ["frcp", "frap"], "{:#?}", parsed.tree.items);
}

#[test]
fn old_edge_extension_options_have_machine_applicable_removal_fixes() {
    use litigate_base::{Applicability, DiagnosticCode};
    use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};

    let source = include_str!("../../../lit/examples/rule12.lit").replace(
        "within answer-period\n",
        "within answer-period { extendable false, extension_authority \"FRCP 6(b)\" }\n",
    );
    let package = LoadedPackage {
        texts: std::collections::BTreeMap::default(),
        root: "lit/examples".to_owned(),
        manifest: include_str!("../../../lit/examples/Lit.toml").to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: "rule12.lit".to_owned(),
            text: source,
        }],
    };
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[package]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let diagnostics: Vec<_> = checks[0]
        .diagnostics
        .iter()
        .filter(|d| {
            d.code == DiagnosticCode::InvalidField
                && d.message.starts_with("edge deadline option `")
        })
        .collect();
    assert_eq!(diagnostics.len(), 4, "{:#?}", checks[0].diagnostics);
    assert!(diagnostics.iter().all(|d| {
        d.fixes.len() == 1
            && d.fixes[0].applicability == Applicability::MachineApplicable
            && d.fixes[0].edits[0].replacement.is_empty()
    }));
}

#[test]
fn worked_example_lock_matches_and_round_trips() -> std::io::Result<()> {
    let repo = repo();
    let (db, ws) = litigate_sema::load(&repo, &[repo.join("lit/examples")], None)?;
    let pkg = ws.packages(&db)[0];
    let missing = litigate_sema::lock_diagnostics(&db, ws, pkg);
    assert!(missing.is_empty(), "{missing:#?}");
    let text = litigate_sema::render_lock(&db, ws, pkg);
    assert!(text.starts_with("version = 3\n\n[[package]]\nname = \"frcp-rule12-example\"\nversion = \"0.1.0\"\nlanguage = 1\nmodules = ["));
    let expected = litigate_sema::pkg::lock::LockFile::parse(&text)
        .map_err(|r| std::io::Error::other(r.message))?;
    assert_eq!(
        litigate_sema::pkg::lock::verify(Some(&text), &expected),
        Ok(())
    );
    Ok(())
}
