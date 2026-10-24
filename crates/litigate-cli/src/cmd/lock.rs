//! `litigate lock [DIR] [--locked] [--root]`: write or verify `Lit.lock`
//! (SPEC §4.2) for a package, or for each directory of matters under `DIR`
//! (the lock beside a `.case` pins the packages it `uses`, SPEC §5.6);
//! `--root` verifies and prints the Merkle root computed from the lock.
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Diagnostic, FileId, LineIndex};
use litigate_sema::pkg::lock::LockFile;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The Merkle root of the lock `text`, when it verified (`diagnostics`
/// empty): computed on read, never stored.
fn root_of(text: Option<&str>, diagnostics: &[Diagnostic]) -> Value {
    text.filter(|_| diagnostics.is_empty())
        .and_then(|t| LockFile::parse(t).ok())
        .map_or(Value::Null, |l| Value::String(l.root()))
}

/// `litigate lock`. `root` verifies as `locked` does, and adds the lock's
/// Merkle root to the result.
pub fn lock(dir: Option<&str>, locked: bool, root: bool) -> Outcome {
    let locked = locked || root;
    let dir = dir.unwrap_or(".").to_owned();
    let targets = ws::discover(std::slice::from_ref(&dir))?;
    let session = Session::load(&targets, &[])?;
    if !Path::new(&dir).join("Lit.toml").is_file() && !session.cases.is_empty() {
        return matters(&session, &targets.cases, locked, root);
    }
    let pkg = session
        .requested()
        .next()
        .ok_or_else(|| Failure::new(format!("{dir}: no Lit.toml and no .case files")))?;
    let text = litigate_sema::render_lock(&session.db, session.ws, pkg.input);
    let dir_path = targets.packages.first().cloned().unwrap_or_default();
    let file = dir_path.join("Lit.lock");
    let path = ws::shown(&file);
    let existing = pkg.input.lock_text(&session.db).clone();
    let changed = existing.as_deref() != Some(text.as_str());
    let mut report = Report::of(json!({
        "lockfile": path,
        "changed": changed,
        "packages": pinned(&session),
    }));
    report.provenance = crate::prov::session(&session.packages, None, None);
    if locked {
        report.diagnostics = litigate_sema::lock_diagnostics(&session.db, session.ws, pkg.input);
    } else if changed {
        crate::cmd::build::write_atomic(&file, &text)?;
    }
    if root {
        report.result["root"] = root_of(existing.as_deref(), &report.diagnostics);
    }
    Ok(report)
}

fn pinned(session: &Session) -> Vec<Value> {
    session
        .packages
        .iter()
        .filter(|p| !p.name.is_empty())
        .map(|p| json!({"name": p.name, "version": p.version, "digest": p.digest}))
        .collect()
}

/// One `Lit.lock` per directory holding matters.
fn matters(session: &Session, paths: &[PathBuf], locked: bool, root: bool) -> Outcome {
    let mut dirs: BTreeMap<PathBuf, Vec<&CaseText>> = BTreeMap::new();
    for (path, case) in paths.iter().zip(&session.cases) {
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
        dirs.entry(parent).or_default().push(case);
    }
    let mut report = Report::of(Value::Null);
    let mut files = Vec::new();
    for (dir, cases) in dirs {
        let expected = session.matter_lock(cases.iter().copied());
        let text = expected.render();
        let file = dir.join("Lit.lock");
        let shown = ws::shown(&file);
        let existing = cases.first().and_then(|c| c.lock.clone());
        let changed = existing.as_deref() != Some(text.as_str());
        let diagnostics: Vec<Diagnostic> = if locked {
            stale(&shown, existing.as_deref(), &expected)
                .into_iter()
                .collect()
        } else {
            if changed {
                crate::cmd::build::write_atomic(&file, &text)?;
            }
            Vec::new()
        };
        let merkle = expected.merkle();
        let mut entry = json!({
            "lockfile": shown,
            "changed": changed,
            "cases": cases.iter().map(|c| c.path.clone()).collect::<Vec<_>>(),
            "packages": expected.packages.iter().map(|p| json!({
                "name": p.name,
                "version": p.version,
                "digest": merkle[&format!("{} {}", p.name, p.version)],
            })).collect::<Vec<_>>(),
        });
        if root {
            entry["root"] = root_of(existing.as_deref(), &diagnostics);
        }
        report.diagnostics.extend(diagnostics);
        files.push(entry);
    }
    report.result = json!({"lockfiles": files});
    report.provenance = crate::prov::session(&session.packages, None, None);
    Ok(report)
}

/// L1007 when the lock at `file` is missing or differs from `expected`.
fn stale(file: &str, existing: Option<&str>, expected: &LockFile) -> Option<Diagnostic> {
    litigate_sema::pkg::lock::verify(existing, expected)
        .err()
        .map(|r| {
            r.to_diagnostic(
                &FileId(file.to_owned()),
                &LineIndex::new(existing.unwrap_or_default()),
            )
        })
}
