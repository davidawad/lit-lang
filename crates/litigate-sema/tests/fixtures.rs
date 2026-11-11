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
