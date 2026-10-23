//! Shared support for the `.case` tests: the SPEC §12 worked example
//! (`lit/examples`), the FRCP 2009 amendment fixture
//! (`fixtures/case-2009`), their `Lit.lock`, the engine clock over their
//! lowered counting rules, and diagnostic rendering.
#![allow(dead_code)]
use litigate_base::{Diagnostic, DiagnosticCode, Profile};
use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::case::{check_source, CheckedCase, Env};
use litigate_sema::hir::Package;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::pkg::lock::{LockFile, LockedPackage};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn checked(db: &litigate_sema::LitDatabase, ws: litigate_sema::Workspace) -> Vec<Package> {
    litigate_sema::check_workspace(db, ws)
        .iter()
        .map(|c| {
            c.package.clone().unwrap_or_else(|| {
                panic!(
                    "{}",
                    serde_json::to_string_pretty(&c.diagnostics).unwrap_or_default()
                )
            })
        })
        .collect()
}

/// `lit/examples` (package `frcp-rule12-example`).
pub fn examples() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let root = repo_root();
        let (db, ws) = litigate_sema::load(&root, &[root.join("lit/examples")], None)
            .unwrap_or_else(|e| panic!("{e}"));
        checked(&db, ws)
    })
}

/// The SPEC §12 matter.
pub fn acme() -> String {
    std::fs::read_to_string(repo_root().join("cases/examples/acme.case")).unwrap_or_default()
}

fn loaded(name: &str, deps: &str, sources: &str, files: &[(&str, &str)]) -> LoadedPackage {
    LoadedPackage {
 texts: std::collections::BTreeMap::new(),
        root: format!("pkgs/{name}"),
        manifest: format!(
            "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\n\
             forum = \"frcp\"\nauthority = \"FederalRule\"\ntitle = \"T\"\nin_force = \"2007-12-01\"\n\
             known_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\n\
             Court = \"nature\"\n{deps}{sources}"
        ),
        lock: None,
        files: files
            .iter()
            .map(|(p, t)| LoadedFile {
                relative: (*p).to_owned(),
                text: (*t).to_owned(),
            })
            .collect(),
    }
}

/// `core` (authority chain) and `frcp` (FRCP 6 and 12 across the
/// December 1, 2009 amendments, with FRCP 86(a) transitions).
pub fn frcp2009() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| frcp2009_with(include_str!("../fixtures/case-2009/rule12.lit")))
}

/// [`frcp2009`] with `rule12` as the text of its `rule12.lit`.
pub fn frcp2009_with(rule12: &str) -> Vec<Package> {
    let core = loaded(
        "core",
        "",
        "",
        &[
            (
                "authority.lit",
                "module core.authority\nauthority Statute > FederalRule > LocalRule\n",
            ),
            ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
        ],
    );
    let frcp = loaded(
        "frcp",
        "\n[dependencies]\ncore = { version = \"^1.0.0\", path = \"../core\" }\n",
        "\n[sources.frcp]\ntitle = \"Federal Rules of Civil Procedure\"\n\
         url = \"https://www.law.cornell.edu/rules/frcp\"\nas_of = \"2024-12-01\"\n\
         cite_prefixes = [\"FRCP\"]\n",
        &[
            ("rule12.lit", rule12),
            ("time.lit", include_str!("../fixtures/frcp-2009/time.lit")),
        ],
    );
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[core, frcp]);
    checked(&db, ws)
}

/// A `Lit.lock` pinning `packages` as loaded.
pub fn lock(packages: &[Package]) -> LockFile {
    LockFile::new(
        packages
            .iter()
            .map(|p| LockedPackage {
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                source: format!("sha256:{}", p.digest),
                dependencies: Vec::new(),
                language: 1,
                ..Default::default()
            })
            .collect(),
    )
}

/// The engine clock over every counting version `packages` declare.
pub fn clock(packages: &[Package]) -> EngineClock {
    let refs: Vec<&Package> = packages.iter().collect();
    EngineClock::lowered(&Lowerer::new(&refs), &refs)
        .unwrap_or_else(|d| panic!("{}", serde_json::to_string_pretty(&d).unwrap_or_default()))
}

/// The engine clock of [`examples`].
pub fn examples_clock() -> &'static EngineClock {
    static C: OnceLock<EngineClock> = OnceLock::new();
    C.get_or_init(|| clock(examples()))
}

/// The engine clock of [`frcp2009`].
pub fn frcp2009_clock() -> &'static EngineClock {
    static C: OnceLock<EngineClock> = OnceLock::new();
    C.get_or_init(|| clock(frcp2009()))
}

/// Checks `text` as `cases/t.case` against `packages` with their lock.
pub fn check_with(
    text: &str,
    packages: &[Package],
    lock: Option<&LockFile>,
    clock: &EngineClock,
    profile: Profile,
) -> CheckedCase {
    let refs: Vec<&Package> = packages.iter().collect();
    let mut checked = check_source(
        "cases/t.case",
        text,
        Env {
            packages: &refs,
            lock,
            clock,
            profile,
        },
    );
    // An event without an id (L9006, info) is on nearly every fixture
    // line; its own tests read it from `check_source`.
    checked
        .diagnostics
        .retain(|d| d.code != DiagnosticCode::UndeclaredEventId);
    checked
}

/// Checks `text` against the worked example.
pub fn check_example(text: &str, profile: Profile) -> CheckedCase {
    let lock = lock(examples());
    check_with(text, examples(), Some(&lock), examples_clock(), profile)
}

/// Checks `text` against the 2009 fixture.
pub fn check_2009(text: &str) -> CheckedCase {
    let lock = lock(frcp2009());
    check_with(
        text,
        frcp2009(),
        Some(&lock),
        frcp2009_clock(),
        Profile::Dev,
    )
}

/// One line per diagnostic, notes and fixes indented below it.
pub fn render(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| {
            let mut s = format!(
                "{}:{} {} {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
            for l in &d.labels {
                let _ = write!(
                    s,
                    "\n    note @{}:{}: {}",
                    l.span.line, l.span.col, l.message
                );
            }
            for n in &d.notes {
                let _ = write!(s, "\n    note: {}", n.message);
            }
            for (f, e) in d
                .fixes
                .iter()
                .flat_map(|f| f.edits.iter().map(move |e| (f, e)))
            {
                let _ = write!(
                    s,
                    "\n    fix @{}:{}-{}:{} {:?}: {:?}",
                    e.span.line,
                    e.span.col,
                    e.span.end_line,
                    e.span.end_col,
                    f.applicability,
                    e.replacement
                );
            }
            s
        })
        .collect()
}

/// A matter over the 2009 fixture with `events` (one per line).
pub fn matter_2009(events: &[&str]) -> String {
    format!(
        "case pending-2009\nuses frcp ^1.0.0\nas_of 2010-06-01\nsummary \"A matter pending across the 2009 amendments.\"\n\n\
         parties {{\n  acme: Plaintiff Corporation\n  widget: Defendant Corporation\n  court: Court Tribunal\n}}\n\n\
         events {{\n{}}}\n\nstakes {{\n  answered: 0.usd\n  dismissed: 0.usd\n  defaulted: -1_000.usd\n}}\n",
        events.iter().fold(String::new(), |mut s, e| {
            let _ = writeln!(s, "  {e}");
            s
        })
    )
}

/// `map` rendered with its compiler version redacted, so a release bump
/// cannot stale a snapshot.
pub fn render_map(map: &litigate_lower::SourceMap) -> String {
    let mut map = map.clone();
    assert_eq!(map.generated.version, env!("CARGO_PKG_VERSION"));
    "[version]".clone_into(&mut map.generated.version);
    litigate_lower::render(&map).unwrap_or_default()
}
