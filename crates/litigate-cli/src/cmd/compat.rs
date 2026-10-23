//! `litigate compat OLD NEW` (SPEC §4.4): the computed compatibility class
//! of a package release against an earlier one, and whether its version
//! bump is enough (L1012 when it is not).
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, Session};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, FileId, LineIndex};
use litigate_sema::hir;
use litigate_sema::pkg::compat::{classify, Bump, Compat};
use litigate_sema::pkg::lock::semantic_digest;
use litigate_sema::pkg::semver::Version;
use serde_json::{json, Value};

/// One side: its session (for sources and diagnostics) and the HIR of the
/// package named, when it checks.
struct Side {
    session: Session,
    hir: Option<hir::Package>,
    manifest: (String, String),
}

fn side(dir: &str) -> Result<Side, Failure> {
    let targets = ws::discover(&[dir.to_owned()])?;
    let session = Session::load(&targets, &[])?;
    let pkg = session
        .requested()
        .next()
        .ok_or_else(|| Failure::new(format!("{dir}: no Lit.toml")))?;
    let hir = pkg.check.package.clone();
    let manifest = pkg.files[0].clone();
    Ok(Side {
        session,
        hir,
        manifest,
    })
}

/// The least version after `old` with `bump`.
fn next(old: &Version, bump: Bump) -> String {
    match bump {
        Bump::Major => format!("{}.0.0", old.major + 1),
        Bump::Minor => format!("{}.{}.0", old.major, old.minor + 1),
        Bump::Patch | Bump::Pre => format!("{}.{}.{}", old.major, old.minor, old.patch + 1),
    }
}

fn name(bump: Bump) -> &'static str {
    match bump {
        Bump::Major => "major",
        Bump::Minor => "minor",
        Bump::Patch => "patch",
        Bump::Pre => "pre-release",
    }
}

/// L1012 at the new manifest's `version`, with the least sufficient
/// version as a fix.
fn too_small(
    c: &Compat,
    old: &hir::Manifest,
    new: &hir::Manifest,
    manifest: &(String, String),
) -> Diagnostic {
    let (path, text) = manifest;
    let index = LineIndex::new(text);
    let quoted = format!("\"{}\"", new.version);
    let at = text
        .find("version")
        .and_then(|v| text[v..].find(&quoted).map(|q| v + q))
        .unwrap_or_default();
    let range = litigate_sema::db::span::TextRange::new(
        u32::try_from(at).unwrap_or_default(),
        u32::try_from(at + quoted.len()).unwrap_or_default(),
    );
    // Both manifests are valid (the packages checked), so both parse.
    let base = Version::parse(&old.version).map_or_else(String::new, |o| next(&o, c.required));
    let class = serde_json::to_value(c.class).unwrap_or(Value::Null);
    let message = match c.declared {
        Some(d) => format!(
            "`{}` {} bumps the {} version of {}, but its changes are `{}`: they need a {} bump ({base} or later)",
            new.name,
            new.version,
            name(d),
            old.version,
            class.as_str().unwrap_or_default(),
            name(c.required)
        ),
        None => format!(
            "`{}` {} is not later than {}: a release needs a later version ({base} or later)",
            new.name, new.version, old.version
        ),
    };
    litigate_sema::db::span::Report::new(C::IncompatibleRelease, range, message)
        .note("`^` requirements admit only releases without interface changes, `~` only releases without value changes (SPEC §4.4)")
        .fix(range, format!("\"{base}\""), Applicability::MaybeIncorrect)
        .to_diagnostic(&FileId(path.clone()), &index)
}

/// `litigate compat`.
pub fn compat(old_dir: &str, new_dir: &str) -> Outcome {
    let (old, new) = (side(old_dir)?, side(new_dir)?);
    let mut report = Report::of(Value::Null);
    let (Some(a), Some(b)) = (&old.hir, &new.hir) else {
        for s in [&old, &new] {
            report.diagnostics.extend(
                s.session
                    .packages
                    .iter()
                    .filter(|p| p.requested)
                    .flat_map(|p| p.check.diagnostics.clone()),
            );
            report.sources.extend(s.session.sources());
        }
        return Ok(report);
    };
    if a.manifest.name != b.manifest.name {
        return Err(Failure::new(format!(
            "{old_dir} is `{}` and {new_dir} is `{}`: compat compares two releases of one package",
            a.manifest.name, b.manifest.name
        )));
    }
    let c = classify(a, b);
    let summary = |p: &hir::Package| json!({"version": p.manifest.version, "semantic": format!("sha256:{}", semantic_digest(p))});
    report.result = json!({
        "package": b.manifest.name,
        "old": summary(a),
        "new": summary(b),
        "class": c.class,
        "required": c.required,
        "declared": c.declared,
        "ok": c.ok(a.manifest.version == b.manifest.version),
        "changes": c.changes,
    });
    if !c.ok(a.manifest.version == b.manifest.version) {
        report
            .diagnostics
            .push(too_small(&c, &a.manifest, &b.manifest, &new.manifest));
        report.sources.push(new.manifest.clone());
    }
    Ok(report)
}
