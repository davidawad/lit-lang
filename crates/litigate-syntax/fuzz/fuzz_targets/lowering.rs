//! Fuzz the real HIR-to-pack lowering path at varied valid selection dates.
#![no_main]

use libfuzzer_sys::fuzz_target;
use litigate_lower::{render, Document, Lowerer, Options};
use litigate_sema::{check_workspace, load};
use std::path::Path;
use std::sync::OnceLock;

static PACKAGES: OnceLock<Vec<litigate_sema::hir::Package>> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let packages = PACKAGES.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (db, workspace) = load(&root, &[root.join("lit/examples")], None)
            .expect("worked example workspace loads");
        check_workspace(&db, workspace)
            .into_iter()
            .map(|check| check.package.clone().expect("worked example typechecks"))
            .collect()
    });
    let refs: Vec<_> = packages.iter().collect();
    let year = 2000 + u16::from(data.first().copied().unwrap_or_default()) % 40;
    let day = 1 + u16::from(data.get(1).copied().unwrap_or_default()) % 28;
    let date = format!("{year:04}-01-{day:02}");
    for package in packages {
        if let Ok(lowered) = Lowerer::new(&refs).package(
            package,
            &Options {
                as_of: Some(date.clone()),
                known_at: Some(date.clone()),
            },
        ) {
            if let Document::Pack(pack) = lowered.document {
                let text = render(&pack).expect("lowered pack serializes");
                let roundtrip = litigate_engine::model::Pack::from_json(&text)
                    .expect("lowered pack is accepted by the engine");
                assert!(litigate_lower::semantic_diff(&pack, &roundtrip).is_empty());
            }
        }
    }
});
