//! `calibration` declarations (SPEC §5.9): targets resolve to an edge or
//! node of this package, a dependency or a declared instance; values are
//! typed; every entry names a declared source and a sample size. Nothing
//! that does not lower is dropped without a diagnostic.
mod common;
use common::{check_body, package, render, workspace};
use litigate_sema::hir::{CalibratedValue, Declaration};

/// A chance node with named edges after the prelude's `s` and `t`.
const NODE: &str = "chance n: \"N\" {\n  a \"A\" -> t p 1/2\n  b \"B\" -> t p 1/2\n}\n";

#[test]
fn a_well_formed_set_checks_and_types_every_value() {
    let body = format!(
        "{NODE}calibration demo-fy: \"Demo FY\" {{\n  note \"Covers demo.\"\n  \
         source r \"Report\" {{ url \"https://example.org/r.pdf\", vintage [2023-10-01, 2024-10-01), retrieved 2026-09-28 }}\n  \
         source open \"Open\" {{ url \"https://example.org/o\", vintage [2024-01-01, ..) }}\n  \
         p a 0.25 {{ source r, n 40, method \"10 / 40\" }}\n  \
         p b ? {{ source r, n 40, method \"cleared\" }}\n  \
         p demo::a 1/4 {{ source r, n 40 }}\n  \
         duration a 3.5.days {{ source open, n 2 }}\n  \
         duration b {{ min 1.days, mode 2.days, max 5.days }} {{ source open, n 3 }}\n  \
         attr n weight -2 {{ source r, n 1 }}\n}}"
    );
    assert_eq!(check_body(&body), Vec::<String>::new());
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let checked = litigate_sema::check_workspace(&db, ws);
    let pkg = checked
        .iter()
        .find_map(|c| c.package.clone())
        .expect("demo checks");
    let set = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Calibration(c) => Some(c.clone()),
            _ => None,
        })
        .expect("a calibration declaration");
    assert_eq!(set.meta.id, "demo-fy");
    assert_eq!(set.meta.label, "Demo FY");
    assert_eq!(set.description.as_deref(), Some("Covers demo."));
    assert_eq!(set.sources.len(), 2);
    assert_eq!(set.sources[0].to.as_deref(), Some("2024-10-01"));
    assert_eq!(set.sources[0].retrieved.as_deref(), Some("2026-09-28"));
    assert_eq!(set.sources[1].to, None);
    let targets: Vec<&str> = set.entries.iter().map(|e| e.target.as_str()).collect();
    assert_eq!(
        targets,
        ["demo::a", "demo::b", "demo::a", "demo::a", "demo::b", "demo::n"]
    );
    assert!(matches!(
        set.entries[1].value,
        CalibratedValue::Probability { value: None }
    ));
    assert!(matches!(
        &set.entries[5].value,
        CalibratedValue::Attr { name, .. } if name == "weight"
    ));
    assert_eq!(set.entries[0].method.as_deref(), Some("10 / 40"));
    assert!(set.entries.iter().all(|e| e.span.is_some()));
}

#[test]
fn every_entry_that_would_not_lower_is_a_diagnostic() {
    let body = format!(
        "{NODE}calibration bad: \"Bad\" {{\n  note \"one\"\n  note \"two\"\n  \
         source r \"R\" {{ url \"u\", vintage [2023-10-01, 2024-10-01) }}\n  \
         source r \"R2\" {{ url \"u\", vintage [2023-10-01, ..) }}\n  \
         source nourl \"N\" {{ vintage [2023-10-01, ..) }}\n  \
         p a 2 {{ source r, n 1 }}\n  \
         p a (s) {{ source r, n 1 }}\n  \
         p b ? {{ source r, n 1 }}\n  \
         p nope 0.5 {{ source r, n 1 }}\n  \
         p other::a 0.5 {{ source r, n 1 }}\n  \
         p demo::nope 0.5 {{ source r, n 1 }}\n  \
         p ghost@x::a 0.5 {{ source r, n 1 }}\n  \
         p demo.a.b 0.5 {{ source r, n 1 }}\n  \
         p a 0.5 {{ source q, n 1 }}\n  \
         p a 0.5 {{ n 1 }}\n  \
         p a 0.5 {{ source r }}\n  \
         p a 0.5 {{ source r, n 0 }}\n  \
         p a 0.5 {{ source r, n 1.5 }}\n  \
         attr nope w 1 {{ source r, n 1 }}\n  \
         attr n w.v 1 {{ source r, n 1 }}\n  \
         duration a {{ max 1.days }} {{ source r, n 1 }}\n}}"
    );
    insta::assert_debug_snapshot!(check_body(&body));
}

#[test]
fn a_dependency_and_an_instance_are_targets_in_a_links_package() {
    let base = package(
        "base",
        &[],
        &[(
            "main.lit",
            "module base\nstart s\nchance s: \"S\" {\n  a \"A\" -> t p 1/2\n  b \"B\" -> t p 1/2\n}\nterminal t: \"T\"\n",
        )],
    );
    let links = package(
        "links",
        &["base"],
        &[(
            "links.lit",
            "module links\ninstance base@up {\n  pack base\n}\n\
             calibration up-fy: \"Up\" {\n  source r \"R\" { url \"u\", vintage [2023-10-01, ..) }\n  \
             p base@up::a 0.5 { source r, n 2 }\n  p base::b 0.5 { source r, n 2 }\n  \
             attr base::s w 1 { source r, n 1 }\n  p base@up::zz 0.5 { source r, n 2 }\n}\n",
        )],
    );
    let (db, ws) = workspace(&[base, links]);
    insta::assert_debug_snapshot!(render(&db, ws));
}
