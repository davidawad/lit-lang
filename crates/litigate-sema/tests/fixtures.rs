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
    let reasons: Vec<Option<&str>> = check.holes.iter().map(|h| h.reason.as_deref()).collect();
    assert_eq!(reasons, [Some("no calibration yet"), None, None]);
    insta::assert_json_snapshot!("rule12_hir", check.package);
    Ok(())
}

#[test]
fn worked_example_lock_is_reported_missing_then_round_trips() -> std::io::Result<()> {
    let repo = repo();
    let (db, ws) = litigate_sema::load(&repo, &[repo.join("lit/examples")], None)?;
    let pkg = ws.packages(&db)[0];
    let missing = litigate_sema::lock_diagnostics(&db, ws, pkg);
    assert_eq!(
        missing
            .iter()
            .map(|d| (d.code, d.span.file.0.as_str()))
            .collect::<Vec<_>>(),
        [(C::StaleLock, "lit/examples/Lit.lock")]
    );
    let text = litigate_sema::render_lock(&db, ws, pkg);
    assert!(text.starts_with("version = 2\n\n[[package]]\nname = \"frcp-rule12-example\"\nversion = \"0.1.0\"\nlanguage = 1\nsource = \"sha256:"));
    let expected = litigate_sema::pkg::lock::LockFile::parse(&text)
        .map_err(|r| std::io::Error::other(r.message))?;
    assert_eq!(
        litigate_sema::pkg::lock::verify(Some(&text), &expected),
        Ok(())
    );
    Ok(())
}
