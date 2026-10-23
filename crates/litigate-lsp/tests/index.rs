//! The symbol index over an in-memory package and matter: resolution
//! rules, and (proptest) that every declared name resolves to itself in
//! any edit of the worked example, however broken.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_base::Profile;
use litigate_cli::ws::{CaseText, Session};
use litigate_lsp::index::{Index, Space};
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_syntax::TextSize;
use proptest::prelude::*;
use std::collections::BTreeMap;

const MANIFEST: &str = "[package]\nname = \"p\"\nversion = \"0.1.0\"\nlanguage = 1\npack_id = \"p\"\nforum = \"f\"\nauthority = \"FederalRule\"\ntitle = \"P\"\nin_force = \"2020-01-01\"\nknown_at = \"2020-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nCourt = \"nature\"\n";

fn index(modules: &[(&str, &str)], case: Option<&str>) -> (Index, BTreeMap<String, String>) {
    let loaded = LoadedPackage {
        root: "lit/p".into(),
        manifest: MANIFEST.into(),
        lock: None,
        files: modules
            .iter()
            .map(|(relative, text)| LoadedFile {
                relative: (*relative).into(),
                text: (*text).into(),
            })
            .collect(),
    };
    let cases = case
        .map(|text| CaseText {
            path: "cases/m.case".into(),
            text: text.into(),
            lock: None,
        })
        .into_iter()
        .collect();
    let checked = litigate_cli::check_session(
        Session::from_loaded(&[loaded], cases),
        Profile::Dev,
        &litigate_lower::Options::default(),
    );
    let files: BTreeMap<String, String> = checked.session.sources().into_iter().collect();
    (Index::build(&files, &checked), files)
}

fn target<'i>(
    index: &'i Index,
    file: &str,
    text: &str,
    needle: &str,
) -> Option<&'i litigate_lsp::index::Def> {
    let offset = text.find(needle).unwrap();
    let i = index.at(file, TextSize::try_from(offset).unwrap())?;
    Some(&index.defs[i])
}

const A: &str =
    "module p.a\nimport p.b.{shared}\n\nstart s\n\nstate s: \"S\" { go \"Go\" -> shared }\n";
const B: &str = "module p.b\n\nterminal shared: \"Shared\" outcome [win] { payoff 0.usd }\n";

#[test]
fn names_resolve_across_modules() {
    let (index, files) = index(&[("a.lit", A), ("b.lit", B)], None);
    let a = &files["lit/p/a.lit"];
    let def = target(&index, "lit/p/a.lit", a, "shared }").unwrap();
    assert_eq!(
        (def.file.as_str(), def.space.clone()),
        ("lit/p/b.lit", Space::Decl)
    );
    let edge = target(&index, "lit/p/a.lit", a, "go ").unwrap();
    assert_eq!(edge.space, Space::Edge("s".into()));
    let refs: Vec<&str> = index
        .references(index.defs.iter().position(|d| d.name == "shared").unwrap())
        .map(|u| u.file.as_str())
        .collect();
    assert_eq!(
        refs,
        ["lit/p/a.lit", "lit/p/a.lit"],
        "the import and the edge target"
    );
}

#[test]
fn a_matter_sees_its_parties_and_used_packages() {
    let case = "case m\nuses p ^0.1.0\nas_of 2025-01-01\n\nperspective acme\n\nparties {\n  acme: Plaintiff Corporation\n}\n";
    let (index, files) = index(&[("a.lit", A), ("b.lit", B)], Some(case));
    let text = &files["cases/m.case"];
    let party = target(&index, "cases/m.case", text, "acme\n\nparties").unwrap();
    assert_eq!(
        (party.space.clone(), party.file.as_str()),
        (Space::Party, "cases/m.case")
    );
    let role = target(&index, "cases/m.case", text, "Plaintiff").unwrap();
    assert_eq!(
        (role.space.clone(), role.file.as_str()),
        (Space::Role, "lit/p/Lit.toml")
    );
}

#[test]
fn unknown_names_resolve_to_nothing() {
    let broken = A.replace("-> shared", "-> nowhere");
    let (index, files) = index(&[("a.lit", &broken), ("b.lit", B)], None);
    let a = &files["lit/p/a.lit"];
    assert!(target(&index, "lit/p/a.lit", a, "nowhere").is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    #[test]
    fn prop_every_declared_name_resolves_to_itself(cut in 0_usize..4000, len in 0_usize..40) {
        let source = include_str!("../../../lit/examples/rule12.lit");
        let start = (0..=cut.min(source.len())).rev().find(|i| source.is_char_boundary(*i)).unwrap_or(0);
        let end = (start + len..=source.len()).find(|i| source.is_char_boundary(*i)).unwrap_or(source.len());
        let edited = format!("{}{}", &source[..start], &source[end..]);
        let (index, _) = index(&[("rule12.lit", &edited)], None);
        for (i, def) in index.defs.iter().enumerate() {
            prop_assert_eq!(index.at(&def.file, def.name_range.start()), Some(i));
        }
        for u in &index.uses {
            if let Some(t) = u.target {
                prop_assert_eq!(&index.defs[t].name, &edited[std::ops::Range::<usize>::from(u.range)]);
            }
        }
    }
}

#[test]
fn a_fact_names_its_node_and_edge_unless_its_value_is_not_a_name() {
    let case =
        "case m\nuses p ^0.1.0\nas_of 2025-01-01\n\nfacts { nowhere::s: go, p::s: go, s: \"x\" }\n";
    let (index, files) = index(&[("a.lit", A), ("b.lit", B)], Some(case));
    let text = &files["cases/m.case"];
    let at = |needle: &str| target(&index, "cases/m.case", text, needle).cloned();
    // An unknown qualifier names no package.
    assert_eq!(at("s: go, p"), None);
    let edge = at("go, p").unwrap();
    assert_eq!(edge.space, Space::Edge("s".into()));
    assert_eq!(at("s: go, s").unwrap().space, Space::Decl);
    // Not a `node: edge` pair: the key resolves as a plain name.
    let node = at("s: \"x\"").unwrap();
    assert_eq!((node.name.as_str(), node.space), ("s", Space::Decl));
}

#[test]
fn imports_name_their_module() {
    let (index, files) = index(&[("a.lit", A), ("b.lit", B)], None);
    let a = &files["lit/p/a.lit"];
    let module = target(&index, "lit/p/a.lit", a, "p.b").unwrap();
    assert_eq!(
        (module.space.clone(), module.file.as_str()),
        (Space::Module, "lit/p/b.lit")
    );
}
