//! `litigate build`: check, then write each package's litigatable (IR +
//! verification report + source map + content hash, DESIGN §2 L1b) and its
//! lowered JSON, plus each matter's scenario. The layout mirrors the
//! repository (`packs/<id>.json`, `packs/links.json`, `packs/clock/<id>.json`,
//! `scenarios/<id>.json`), so `diff OUT/packs packs` is the migration check.
use super::check::{checked, Checked};
use crate::cli::Dates;
use crate::out::{Failure, Outcome, Report};
use litigate_base::{Diagnostic, Profile, Severity};
use litigate_lower::{render, render_scenario, Document, LoweredPackage, Lowerer, Options};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::Path;

/// Writes `text` to `path` atomically (temp file, then rename).
///
/// # Errors
/// I/O failures.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), Failure> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)
            .map_err(|e| Failure::new(format!("{}: {e}", dir.display())))?;
    }
    let tmp = path.with_extension("litigate-tmp");
    std::fs::write(&tmp, text).map_err(|e| Failure::new(format!("{}: {e}", tmp.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| Failure::new(format!("{}: {e}", path.display())))
}

/// `sha256:<hex>` of `bytes`.
#[must_use]
pub fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().fold(String::from("sha256:"), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// One file to write: `(relative path, contents)`.
pub type Output = (String, String);

/// Every generated document of a package, by relative output path.
fn package_outputs(lowered: &LoweredPackage) -> Result<Vec<Output>, Box<Diagnostic>> {
    let mut out = Vec::new();
    match &lowered.document {
        Document::Pack(pack) => {
            out.push((format!("packs/{}.json", pack.id), render(pack)?));
            out.push((
                format!("packs/{}.map.json", pack.id),
                render(&lowered.source_map)?,
            ));
            if let Some((clock, map)) = &lowered.clock {
                out.push((format!("packs/clock/{}.json", pack.id), render(clock)?));
                out.push((format!("packs/clock/{}.map.json", pack.id), render(map)?));
            }
        }
        Document::Links(links) => {
            out.push(("packs/links.json".to_owned(), render(links)?));
            out.push((
                "packs/links.map.json".to_owned(),
                render(&lowered.source_map)?,
            ));
        }
    }
    Ok(out)
}

/// The litigatable document and its content hash.
fn litigatable(
    c: &Checked,
    index: usize,
    lowered: &LoweredPackage,
    options: &Options,
) -> Result<(Value, String), Box<Diagnostic>> {
    let pkg = &c.session.packages[index];
    let (_, verdict) = &c.verdicts[index];
    let mut doc = json!({
        "litigatable": 1,
        "package": {"name": pkg.name, "version": pkg.version, "digest": pkg.digest},
        "as_of": options.as_of,
        "known_at": options.known_at,
        "ir": pkg.check.package,
        "verification": {
            "profile": match c.profile { Profile::Dev => "dev", Profile::Strict => "strict" },
            "obligations": verdict.findings,
            "warnings": verdict.diagnostics.iter().filter(|d| d.severity != Severity::Error).collect::<Vec<_>>(),
            "tests": verdict.tests,
        },
        "source_map": lowered.source_map,
    });
    let hash = sha256(render(&doc)?.as_bytes());
    doc["content_hash"] = Value::String(hash.clone());
    Ok((doc, hash))
}

/// Everything a build writes, and the litigatable hashes.
pub struct Built {
    /// Files, by relative path.
    pub files: Vec<Output>,
    /// `(package, litigatable path, content hash)`.
    pub litigatables: Vec<(String, String, String)>,
    /// Lowering diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Lowers every requested package and every matter of a clean check.
#[must_use]
pub fn lower_all(c: &Checked, options: &Options) -> Built {
    let hir = c.session.hir();
    let lowerer = Lowerer::new(&hir).with_corpus(litgraph::cite::SourceCorpus::embedded_ref());
    let mut built = Built {
        files: Vec::new(),
        litigatables: Vec::new(),
        diagnostics: Vec::new(),
    };
    for (index, pkg) in c
        .session
        .packages
        .iter()
        .enumerate()
        .filter(|(_, p)| p.requested)
    {
        let Some(package) = &pkg.check.package else {
            continue;
        };
        let lowered = match lowerer.package(package, options) {
            Ok(lowered) => lowered,
            Err(errors) => {
                built.diagnostics.extend(errors);
                continue;
            }
        };
        let written = package_outputs(&lowered).and_then(|files| {
            let (doc, hash) = litigatable(c, index, &lowered, options)?;
            Ok((files, doc, hash))
        });
        match written {
            Ok((files, doc, hash)) => {
                let path = format!("litigatables/{}.litigatable.json", pkg.name);
                match render(&doc) {
                    Ok(text) => built.files.push((path.clone(), text)),
                    Err(d) => built.diagnostics.push(*d),
                }
                built.files.extend(files);
                built.litigatables.push((pkg.name.clone(), path, hash));
            }
            Err(d) => built.diagnostics.push(*d),
        }
    }
    for case in &c.cases {
        let Some(lowered) = &case.lowered else {
            continue;
        };
        let id = &lowered.scenario.id;
        match (
            render_scenario(&lowered.scenario),
            render(&lowered.source_map),
        ) {
            (Ok(doc), Ok(map)) => {
                built.files.push((format!("scenarios/{id}.json"), doc));
                built.files.push((format!("scenarios/{id}.map.json"), map));
            }
            (Err(d), _) | (_, Err(d)) => built.diagnostics.push(*d),
        }
    }
    built.files.sort();
    built
}

/// `litigate build`.
pub fn build(
    paths: &[String],
    out: &str,
    profile: Profile,
    dates: &Dates,
    lib: &[String],
) -> Outcome {
    let c = checked(paths, profile, dates.as_of.as_deref(), lib)?;
    let options = Options {
        as_of: dates.as_of.clone(),
        known_at: dates.known_at.clone(),
    };
    let mut report = Report {
        result: Value::Null,
        provenance: c.provenance(dates.as_of.as_deref(), dates.known_at.as_deref()),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        failed: false,
    };
    if report.has_errors() {
        return Ok(report);
    }
    let built = lower_all(&c, &options);
    report.diagnostics.extend(built.diagnostics);
    if report.has_errors() {
        return Ok(report);
    }
    for (path, text) in &built.files {
        write_atomic(&Path::new(out).join(path), text)?;
    }
    let shown = |p: &str| format!("{}/{p}", out.trim_end_matches('/'));
    let kind = |f: &dyn Fn(&str) -> bool| -> Vec<String> {
        built
            .files
            .iter()
            .map(|(p, _)| p.as_str())
            .filter(|p| f(p))
            .map(shown)
            .collect()
    };
    let maps = |p: &str| p.ends_with(".map.json");
    let mut hashes: Vec<String> = built
        .litigatables
        .iter()
        .map(|(n, _, h)| format!("{n} {h}\n"))
        .collect();
    hashes.sort();
    let combined = match built.litigatables.as_slice() {
        [(_, _, one)] => one.clone(),
        _ => sha256(hashes.concat().as_bytes()),
    };
    report.result = json!({
        "outputs": {
            "packs": kind(&|p| p.starts_with("packs/") && !p.starts_with("packs/clock/") && p != "packs/links.json" && !maps(p)),
            "links": kind(&|p| p == "packs/links.json"),
            "clock": kind(&|p| p.starts_with("packs/clock/") && !maps(p)),
            "scenarios": kind(&|p| p.starts_with("scenarios/") && !maps(p)),
            "source_maps": kind(&maps),
            "litigatables": kind(&|p| p.starts_with("litigatables/")),
        },
        "litigatable_hash": combined,
        "litigatables": built.litigatables.iter().map(|(n, p, h)| json!({"package": n, "path": shown(p), "content_hash": h})).collect::<Vec<_>>(),
        "obligations": c.findings,
    });
    Ok(report)
}
