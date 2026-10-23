//! Worlds for the temporal and precedence tables built from every module
//! that type-checks, so package-level legal errors (which withhold the
//! package HIR) do not hide the HIR the checks and queries run over; and
//! the same HIR without source spans, as a consumer of serialized HIR has.
#![allow(dead_code)]
use litigate_base::Diagnostic;
use litigate_sema::db::queries::{check_module, manifest};
use litigate_sema::hir::{Declaration, Manifest, Module};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::temporal::world::{PackInfo, World};
use serde_json::Value;
use std::fmt::Write as _;

/// One package: its manifest and the modules that type-check.
pub type Pack = (Manifest, Vec<Module>);

/// Every package of `packages` (in load order) with its checked modules.
pub fn packs(packages: &[LoadedPackage]) -> Vec<Pack> {
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, packages);
    ws.packages(&db)
        .iter()
        .filter_map(|p| {
            let m = manifest(&db, *p).as_ref().ok()?.manifest.clone();
            let modules = p
                .files(&db)
                .iter()
                .filter_map(|f| check_module(&db, *f).module.clone())
                .collect();
            Some((m, modules))
        })
        .collect()
}

/// The single package `demo` (lifetime from 2009-12-01) of
/// `legal::PRELUDE` followed by `body`.
pub fn demo(body: &str) -> Vec<Pack> {
    let text = format!("{}{body}\n", crate::legal::PRELUDE);
    packs(&[crate::legal::pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )])
}

/// The world of `packs`, in order.
pub fn world(packs: &[Pack]) -> World<'_> {
    let mut w = World::default();
    for (m, modules) in packs {
        w.add(PackInfo::of(m), modules);
    }
    w
}

fn strip_value(v: &mut Value) {
    match v {
        Value::Object(map) => {
            for (k, x) in map.iter_mut() {
                if k == "span" {
                    *x = Value::Null;
                } else {
                    strip_value(x);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(strip_value),
        _ => {}
    }
}

/// `packs` with every source span removed (a JSON round trip).
pub fn strip(packs: &[Pack]) -> Vec<Pack> {
    packs
        .iter()
        .map(|(m, modules)| {
            let stripped = serde_json::to_value(modules).and_then(|mut v| {
                strip_value(&mut v);
                serde_json::from_value(v)
            });
            let Ok(modules) = stripped else {
                panic!("HIR round-trips through JSON: {stripped:?}");
            };
            (m.clone(), modules)
        })
        .collect()
}

/// Applies `f` to every declaration of `packs`.
pub fn edit(packs: &mut [Pack], mut f: impl FnMut(&mut Declaration)) {
    for (_, modules) in packs {
        for m in modules {
            m.declarations.iter_mut().for_each(&mut f);
        }
    }
}

/// `file:line:col CODE message`, then notes and fixes, one per line.
pub fn lines(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| {
            let mut s = format!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
            for l in &d.labels {
                let at = format!(" @{}:{}:{}", l.span.file.0, l.span.line, l.span.col);
                let _ = write!(s, "\n    note{at}: {}", l.message);
            }
            for n in &d.notes {
                let _ = write!(s, "\n    note: {}", n.message);
            }
            for e in d.fixes.iter().flat_map(|f| &f.edits) {
                let _ = write!(
                    s,
                    "\n    fix @{}:{}: {:?}",
                    e.span.line, e.span.col, e.replacement
                );
            }
            s
        })
        .collect()
}
