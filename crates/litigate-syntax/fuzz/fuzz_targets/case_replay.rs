//! Fuzz `.case` parsing and the complete semantic replay path.
#![no_main]

use libfuzzer_sys::fuzz_target;
use litigate_base::Profile;
use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::{
    case::{check_source, Env},
    hir::Package,
    pkg::lock::{LockFile, LockedPackage},
    check_workspace, load, LitDatabase, Workspace,
};
use litigate_syntax::{parse, FileKind};
use std::cell::OnceCell;
use std::path::{Path, PathBuf};

struct Fixtures {
    db: LitDatabase,
    workspace: Workspace,
    lock: LockFile,
    clock: EngineClock,
}

thread_local! {
    static FIXTURES: OnceCell<Option<Fixtures>> = const { OnceCell::new() };
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let parsed = parse(text, FileKind::Case);
    assert_eq!(parsed.syntax().to_string(), text);
    // Resolve the worked package once, then pass every input through the
    // real case checker. Corpus seeds preserve enough context to exercise
    // event ordering, posture replay, and fact/analysis traversal.
    if parsed.is_ok() {
        FIXTURES.with(|cell| {
            let fixtures = cell.get_or_init(|| {
                let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
                let example = root.join("lit/examples");
                let (db, workspace) = load(&root, &[PathBuf::from(example)], None).ok()?;
                let checks = check_workspace(&db, workspace);
                let packages: Vec<&Package> = checks
                    .iter()
                    .filter_map(|check| check.package.as_ref())
                    .collect();
                let lock = LockFile::new(
                    packages
                        .iter()
                        .map(|package| LockedPackage {
                            name: package.manifest.name.clone(),
                            version: package.manifest.version.clone(),
                            source: format!("sha256:{}", package.digest),
                            dependencies: Vec::new(),
                            language: 1,
                            ..Default::default()
                        })
                        .collect(),
                );
                let lowerer = Lowerer::new(&packages);
                let clock = EngineClock::lowered(&lowerer, &packages).ok()?;
                Some(Fixtures { db, workspace, lock, clock })
            });
            if let Some(fixtures) = fixtures {
                let checks = check_workspace(&fixtures.db, fixtures.workspace);
                let packages: Vec<_> = checks
                    .iter()
                    .filter_map(|check| check.package.as_ref())
                    .collect();
                let _ = check_source(
                    "fuzz.case",
                    text,
                    Env {
                        packages: &packages,
                        lock: Some(&fixtures.lock),
                        clock: &fixtures.clock,
                        profile: Profile::Dev,
                    },
                );
            }
        });
    }
});
