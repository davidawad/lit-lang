//! `litigate lock [DIR] [--locked]`: write or verify `Lit.lock` (SPEC §4.2)
//! for a package, or for each directory of matters under `DIR` (the lock
//! beside a `.case` pins the packages it `uses`, SPEC §5.6); `--merge` is
//! the git merge driver for `Lit.lock`.
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Diagnostic, FileId, LineIndex};
use litigate_sema::pkg::lock::{LockFile, FORMAT};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `litigate lock`.
pub fn lock(dir: Option<&str>, locked: bool) -> Outcome {
    let dir = dir.unwrap_or(".").to_owned();
    let targets = ws::discover(std::slice::from_ref(&dir))?;
    let session = Session::load(&targets, &[])?;
    if !Path::new(&dir).join("Lit.toml").is_file() && !session.cases.is_empty() {
        return matters(&session, &targets.cases, locked);
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
    Ok(report)
}

/// `litigate lock --merge BASE OURS THEIRS`: writes the three-way merge
/// into `OURS`. Packages both sides changed get an unmerged digest, so the
/// lock stays stale (L1007) until `litigate lock` regenerates it. A side
/// in another format has no rows to merge: the result is an empty lock,
/// stale the same way (`stale_format`).
pub fn merge(paths: &[String]) -> Outcome {
    let read = |path: &String| {
        let text =
            std::fs::read_to_string(path).map_err(|e| Failure::new(format!("{path}: {e}")))?;
        let failed =
            |r: litigate_sema::db::span::Report| Failure::new(format!("{path}: {}", r.message));
        let lock = (LockFile::format(&text).map_err(failed)? == FORMAT)
            .then(|| LockFile::parse(&text).map_err(failed))
            .transpose()?;
        Ok::<_, Failure>((text, lock))
    };
    let [base, ours, theirs] = [&paths[0], &paths[1], &paths[2]].map(read);
    let (ours_text, ours) = ours?;
    let (base, theirs) = (base?.1, theirs?.1);
    let sides = base.zip(ours).zip(theirs);
    let stale_format = sides.is_none();
    let merged = sides.map_or_else(
        || litigate_sema::pkg::lock::Merged {
            lock: LockFile::new(Vec::new()),
            unresolved: Vec::new(),
        },
        |((base, ours), theirs)| litigate_sema::pkg::lock::merge(&base, &ours, &theirs),
    );
    let text = merged.lock.render();
    crate::cmd::build::write_atomic(Path::new(&paths[1]), &text)?;
    Ok(Report::of(json!({
        "lockfile": ws::shown(Path::new(&paths[1])),
        "changed": text != ours_text,
        "unresolved": merged.unresolved,
        "stale_format": stale_format,
    })))
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
fn matters(session: &Session, paths: &[PathBuf], locked: bool) -> Outcome {
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
        if locked {
            report
                .diagnostics
                .extend(stale(&shown, existing.as_deref(), &expected));
        } else if changed {
            crate::cmd::build::write_atomic(&file, &text)?;
        }
        files.push(json!({
            "lockfile": shown,
            "changed": changed,
            "cases": cases.iter().map(|c| c.path.clone()).collect::<Vec<_>>(),
            "packages": expected.packages.iter().map(|p| json!({
                "name": p.name, "version": p.version, "digest": p.digest,
            })).collect::<Vec<_>>(),
        }));
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
