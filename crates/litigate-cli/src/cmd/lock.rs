//! `litigate lock [DIR] [--locked]`: write or verify `Lit.lock` (SPEC §4.2).
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, Session};
use serde_json::json;

/// `litigate lock`.
pub fn lock(dir: Option<&str>, locked: bool) -> Outcome {
    let dir = dir.unwrap_or(".").to_owned();
    let targets = ws::discover(std::slice::from_ref(&dir))?;
    let session = Session::load(&targets, &[])?;
    let pkg = session
        .requested()
        .next()
        .ok_or_else(|| Failure::new(format!("{dir}: no Lit.toml")))?;
    let text = litigate_sema::render_lock(&session.db, session.ws, pkg.input);
    let dir_path = targets.packages.first().cloned().unwrap_or_default();
    let file = dir_path.join("Lit.lock");
    let path = ws::shown(&file);
    let existing = pkg.input.lock_text(&session.db).clone();
    let changed = existing.as_deref() != Some(text.as_str());
    let mut report = Report::of(json!({
        "lockfile": path,
        "changed": changed,
        "packages": session.packages.iter().filter(|p| !p.name.is_empty()).map(|p| json!({
            "name": p.name, "version": p.version, "digest": p.digest,
        })).collect::<Vec<_>>(),
    }));
    report.provenance = crate::prov::session(&session.packages, None, None);
    if locked {
        report.diagnostics = litigate_sema::lock_diagnostics(&session.db, session.ws, pkg.input);
    } else if changed {
        crate::cmd::build::write_atomic(&file, &text)?;
    }
    Ok(report)
}
