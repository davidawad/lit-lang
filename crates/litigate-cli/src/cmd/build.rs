//! `litigate build`: check, then write each package's litigatable (IR +
//! verification report + source map + content hash, DESIGN §2 L1b) and its
//! lowered JSON, plus each matter's scenario. The layout mirrors the
//! repository (`packs/<id>.json`, `packs/links.json`, `packs/clock/<id>.json`,
//! `scenarios/<id>.json`), so `diff OUT/packs packs` is the migration check.
//! `--pack-file ID=STEM` renames a pack's files to `packs/STEM.json`, which
//! `crates/litgraph-packs/build.args` uses to keep the legacy file names.
use super::check::{checked_in, Checked};
use crate::cli::Dates;
use crate::out::{Failure, Outcome, Report};
use litigate_base::{Diagnostic, Profile, Severity};
use litigate_lower::{
    canonical, render, render_scenario, Document, LoweredPackage, Lowerer, Options,
};
use litigate_sema::pkg::manifest::is_kebab;
use litigate_sema::pkg::source::{Disk, Source};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// Writes `text` to `path` on disk atomically (temp file, then rename).
///
/// # Errors
/// I/O failures.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), Failure> {
    write_atomic_in(&Disk, path, text)
}

/// [`write_atomic`] in `src`.
///
/// # Errors
/// I/O failures, naming the failing path.
pub fn write_atomic_in(src: &dyn Source, path: &Path, text: &str) -> Result<(), Failure> {
    Ok(src.write_atomic(path, text)?)
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

/// [`render`]'s byte-deterministic JSON of a generated document. These
/// are derive-`Serialize` trees with string map keys, which always
/// serialize; a failure is the empty text, which [`nonempty`] refuses to
/// write.
pub(crate) fn text<T: Serialize>(value: &T) -> String {
    render(value).unwrap_or_default()
}

/// Every generated document of a package, by relative output path.
fn package_outputs(lowered: &LoweredPackage) -> Vec<Output> {
    let map = text(&lowered.source_map);
    match &lowered.document {
        Document::Pack(pack) => {
            let mut out = vec![
                (format!("packs/{}.json", pack.id), text(pack)),
                (format!("packs/{}.map.json", pack.id), map),
            ];
            out.extend(lowered.clock.iter().flat_map(|(clock, clock_map)| {
                [
                    (format!("packs/clock/{}.json", pack.id), text(clock)),
                    (format!("packs/clock/{}.map.json", pack.id), text(clock_map)),
                ]
            }));
            out
        }
        Document::Links(links) => vec![
            ("packs/links.json".to_owned(), text(links)),
            ("packs/links.map.json".to_owned(), map),
        ],
    }
}

/// The litigatable document and its content hash.
fn litigatable(
    c: &Checked,
    index: usize,
    lowered: &LoweredPackage,
    options: &Options,
) -> (Value, String) {
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
    // Over the JCS text (RFC 8785), so the hash names the content, not a
    // layout.
    let hash = sha256(canonical(&doc).unwrap_or_default().as_bytes());
    doc["content_hash"] = Value::String(hash.clone());
    (doc, hash)
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
    let lowerer = Lowerer::new(&hir).with_corpus(crate::verdict::corpus());
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
        let (doc, hash) = litigatable(c, index, &lowered, options);
        let path = format!("litigatables/{}.litigatable.json", pkg.name);
        built.files.push((path.clone(), text(&doc)));
        built.files.extend(package_outputs(&lowered));
        built.litigatables.push((pkg.name.clone(), path, hash));
    }
    for case in &c.cases {
        let Some(lowered) = &case.lowered else {
            continue;
        };
        let id = &lowered.scenario.id;
        built.files.push((
            format!("scenarios/{id}.json"),
            render_scenario(&lowered.scenario).unwrap_or_default(),
        ));
        built.files.push((
            format!("scenarios/{id}.map.json"),
            text(&lowered.source_map),
        ));
    }
    built.files.sort();
    built
}

/// `litigate build` arguments.
pub struct Args {
    /// Package directories or `.case` files.
    pub paths: Vec<String>,
    /// Output directory.
    pub out: String,
    /// `ID=STEM` pack file names (`--pack-file`).
    pub pack_files: Vec<String>,
    /// Checking profile.
    pub profile: Profile,
    /// Build dates.
    pub dates: Dates,
    /// Library directories.
    pub lib: Vec<String>,
}

/// Parses `--pack-file ID=STEM` mappings: both sides kebab-case, each id
/// at most once, and no stem `links` (that file is the links package's).
///
/// # Errors
/// A malformed, repeated, or reserved mapping.
pub fn pack_file_map(specs: &[String]) -> Result<BTreeMap<String, String>, Failure> {
    specs.iter().try_fold(BTreeMap::new(), |mut map, spec| {
        let (id, stem) = spec
            .split_once('=')
            .filter(|(id, stem)| is_kebab(id) && is_kebab(stem))
            .ok_or_else(|| {
                Failure::new(format!(
                    "--pack-file `{spec}`: expected ID=STEM, both kebab-case (e.g. `my-pack=my-pack-file`)"
                ))
            })?;
        if stem == "links" {
            return Err(Failure::new(format!(
                "--pack-file `{spec}`: `packs/links.json` is the links package's file"
            )));
        }
        match map.insert(id.to_owned(), stem.to_owned()) {
            Some(_) => Err(Failure::new(format!("--pack-file: pack `{id}` mapped twice"))),
            None => Ok(map),
        }
    })
}

/// `files` unchanged when every one has contents. A generated document is
/// never empty, so an empty one failed to serialize: writing it would
/// leave a file that reads as a build output and is none.
///
/// # Errors
/// The first empty output's path.
pub fn nonempty(files: Vec<Output>) -> Result<Vec<Output>, Failure> {
    match files.iter().find(|(_, text)| text.is_empty()) {
        Some((path, _)) => Err(Failure::new(format!(
            "`{path}` did not serialize; nothing was written"
        ))),
        None => Ok(files),
    }
}

/// Renames each mapped pack's outputs (`packs/ID.json`, its source map, and
/// its `packs/clock/` fragment) to `STEM`: the files `build` writes.
///
/// # Errors
/// A mapped id that no built pack has, two outputs landing on one path, or
/// an empty output ([`nonempty`]).
pub fn rename_packs(
    files: Vec<Output>,
    map: &BTreeMap<String, String>,
) -> Result<Vec<Output>, Failure> {
    if let Some(id) = map
        .keys()
        .find(|id| !files.iter().any(|(p, _)| *p == format!("packs/{id}.json")))
    {
        return Err(Failure::new(format!(
            "--pack-file: no built package lowers to pack `{id}`"
        )));
    }
    let renamed = |path: String| {
        map.iter()
            .flat_map(|(id, stem)| {
                ["packs/", "packs/clock/"].into_iter().flat_map(move |dir| {
                    [".json", ".map.json"]
                        .into_iter()
                        .map(move |ext| (format!("{dir}{id}{ext}"), format!("{dir}{stem}{ext}")))
                })
            })
            .find(|(from, _)| *from == path)
            .map_or(path, |(_, to)| to)
    };
    let mut out: Vec<Output> = files.into_iter().map(|(p, t)| (renamed(p), t)).collect();
    out.sort();
    match out.windows(2).find(|w| w[0].0 == w[1].0) {
        Some(w) => Err(Failure::new(format!(
            "--pack-file: two outputs would be written to `{}`",
            w[0].0
        ))),
        None => nonempty(out),
    }
}

/// `litigate build` from and into `src`.
pub fn build(src: &dyn Source, args: &Args) -> Outcome {
    let Args {
        paths,
        out,
        pack_files,
        profile,
        dates,
        lib,
    } = args;
    let (out, profile) = (out.as_str(), *profile);
    let stems = pack_file_map(pack_files)?;
    let options = dates.options();
    let c = checked_in(src, paths, profile, &options, lib)?;
    let mut report = Report {
        // A failed build still reports its obligations (SPEC §9.2).
        result: serde_json::json!({"obligations": c.findings}),
        provenance: c.provenance(dates.as_of.as_deref(), dates.known_at.as_deref()),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        check_render: None,
        text: None,
        profile: Some(profile),
        failed: false,
    };
    if report.has_errors() {
        return Ok(report);
    }
    // `check` lowered every package at these same dates, so a build that
    // checks lowers without errors.
    let mut built = lower_all(&c, &options);
    built.files = rename_packs(built.files, &stems)?;
    for (path, text) in &built.files {
        write_atomic_in(src, &Path::new(out).join(path), text)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{self, ACME};
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litigate-build-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn write_atomic_creates_directories_and_names_each_failing_step() {
        let dir = scratch("write");
        let nested = dir.join("a/b/c.json");
        assert_eq!(write_atomic(&nested, "text"), Ok(()));
        assert_eq!(std::fs::read_to_string(&nested).unwrap(), "text");
        let failure = |path: &Path| write_atomic(path, "text").unwrap_err().0;
        std::fs::write(dir.join("file"), "").unwrap();
        let shown = dir.join("file").display().to_string();
        assert!(failure(&dir.join("file/x.json")).starts_with(&format!("{shown}: ")));
        std::fs::create_dir_all(dir.join("t.litigate-tmp")).unwrap();
        let shown = dir.join("t.litigate-tmp").display().to_string();
        assert!(failure(&dir.join("t.json")).starts_with(&format!("{shown}: ")));
        std::fs::create_dir_all(dir.join("d/inner")).unwrap();
        let shown = dir.join("d").display().to_string();
        assert!(failure(&dir.join("d")).starts_with(&format!("{shown}: ")));
        assert!(failure(Path::new("/")).starts_with("/: "));
    }

    fn lowered(extra: &str, text: &str) -> Built {
        let session = testkit::example_locked(extra, text, None);
        lower_all(
            &super::super::check::check_session(session, Profile::Dev, &Options::default()),
            &Options::default(),
        )
    }

    #[test]
    fn a_package_that_does_not_lower_still_lets_its_matters_build() {
        let built = lowered(
            "counting nourl cite \"XYZ 6(a)\"\n{\n  exclude_trigger_day\n  include_last_day\n  holidays federal-holidays\n}",
            ACME,
        );
        let codes: Vec<&str> = built.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, ["L6001"]);
        let files: Vec<&str> = built.files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(
            files,
            [
                "scenarios/acme-v-widget.json",
                "scenarios/acme-v-widget.map.json"
            ]
        );
        assert!(built.litigatables.is_empty());
    }

    #[test]
    fn a_clean_package_writes_its_documents_and_litigatable() {
        for (profile, name) in [(Profile::Dev, "dev"), (Profile::Strict, "strict")] {
            let session = testkit::example_locked("", ACME, None);
            let checked = super::super::check::check_session(session, profile, &Options::default());
            let built = lower_all(&checked, &Options::default());
            let files: Vec<&str> = built.files.iter().map(|(p, _)| p.as_str()).collect();
            insta::assert_debug_snapshot!("clean_build_files", files);
            let (_, litigatable) = &built.files[0];
            let doc: Value = serde_json::from_str(litigatable).unwrap();
            assert_eq!(doc["verification"]["profile"], name);
            assert_eq!(
                doc["content_hash"].as_str(),
                Some(built.litigatables[0].2.as_str())
            );
        }
    }

    #[test]
    fn packages_and_matters_that_failed_their_check_are_skipped() {
        let built = lowered(
            "state broken: \"Broken\" { _ \"Nowhere\" -> nowhere }",
            ACME,
        );
        assert!(built.files.is_empty());
        assert!(built.diagnostics.is_empty());
    }

    #[test]
    fn an_empty_output_is_never_written() {
        let full = vec![("packs/a.json".to_owned(), "{}".to_owned())];
        assert_eq!(nonempty(full.clone()), Ok(full));
        let empty = vec![
            ("packs/a.json".to_owned(), "{}".to_owned()),
            ("packs/b.json".to_owned(), String::new()),
        ];
        assert_eq!(
            nonempty(empty).unwrap_err().0,
            "`packs/b.json` did not serialize; nothing was written"
        );
    }

    fn specs(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    fn files(paths: &[&str]) -> Vec<Output> {
        paths
            .iter()
            .map(|p| ((*p).to_owned(), "{}".to_owned()))
            .collect()
    }

    #[test]
    fn mappings_parse_and_reject_bad_specs() {
        let map = pack_file_map(&specs(&["cafc=cafc-federal-circuit"])).ok();
        assert_eq!(
            map.map(|m| m.into_iter().collect::<Vec<_>>()),
            Some(vec![("cafc".into(), "cafc-federal-circuit".into())])
        );
        for bad in [
            &["cafc"][..],
            &["cafc="],
            &["=x"],
            &["cafc=a/b"],
            &["cafc=x.json"],
            &["cafc=links"],
            &["cafc=a", "cafc=b"],
        ] {
            assert!(pack_file_map(&specs(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn renames_pack_map_and_clock_files_only() {
        let map = pack_file_map(&specs(&["cofc=cofc-court"])).ok();
        let out = map.and_then(|m| {
            rename_packs(
                files(&[
                    "packs/cofc.json",
                    "packs/cofc.map.json",
                    "packs/clock/cofc.json",
                    "packs/cofc-x.json",
                    "litigatables/cofc.litigatable.json",
                    "scenarios/cofc.json",
                ]),
                &m,
            )
            .ok()
        });
        let paths: Option<Vec<String>> = out.map(|o| o.into_iter().map(|(p, _)| p).collect());
        assert_eq!(
            paths,
            Some(specs(&[
                "litigatables/cofc.litigatable.json",
                "packs/clock/cofc-court.json",
                "packs/cofc-court.json",
                "packs/cofc-court.map.json",
                "packs/cofc-x.json",
                "scenarios/cofc.json",
            ]))
        );
    }

    #[test]
    fn unknown_ids_and_collisions_fail() {
        let unknown = pack_file_map(&specs(&["nope=x"]))
            .and_then(|m| rename_packs(files(&["packs/cofc.json"]), &m));
        assert!(unknown.is_err_and(|f| f.0.contains("`nope`")));
        let clash = pack_file_map(&specs(&["a=b"]))
            .and_then(|m| rename_packs(files(&["packs/a.json", "packs/b.json"]), &m));
        assert!(clash.is_err_and(|f| f.0.contains("packs/b.json")));
    }
}
