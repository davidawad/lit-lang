//! A links package's manifest has no pack identity (SPEC §4.1, §8.4); its
//! links' actors are declared by the pack of each link's `from` node.
mod common;

use common::{package, render, workspace};
use litigate_sema::pkg::load::LoadedPackage;

const LINKS_TOML: &str = "[package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\nin_force = \"0001-01-01\"\nknown_at = \"2026-10-19\"\n\n[dependencies]\nbase = { version = \"^1.0.0\", path = \"../base\" }\n";

fn links(text: &str) -> LoadedPackage {
    LoadedPackage {
        texts: std::collections::BTreeMap::new(),
        root: "pkgs/links".to_owned(),
        manifest: LINKS_TOML.to_owned(),
        lock: None,
        files: vec![litigate_sema::pkg::load::LoadedFile {
            relative: "links.lit".to_owned(),
            text: text.to_owned(),
        }],
    }
}

fn base() -> LoadedPackage {
    package(
        "base",
        &[],
        &[("base.lit", "module base\nstart s\nstate s: \"S\" by Plaintiff { go \"Go\" -> t }\nterminal t: \"T\"\n")],
    )
}

#[test]
fn links_package_without_pack_identity_checks_clean() {
    let text = "module links\nlink up \"Up\": base::t -> base::s by Defendant\nlink odd \"Odd\": base::t -> base::s by Clerk\n";
    let (db, ws) = workspace(&[links(text), base()]);
    assert_eq!(
        render(&db, ws),
        [
            "pkgs/links/links.lit:3:39 L1009 actor `Clerk` is not declared in `[package.roles]` of pack `base`"
        ]
    );
}

#[test]
fn a_links_manifest_cannot_hold_nodes() {
    let text = "module links\nstart s\nterminal s: \"S\"\n";
    let (db, ws) = workspace(&[links(text), base()]);
    let out = render(&db, ws);
    assert!(
        out.iter()
            .any(|d| d.starts_with("pkgs/links/Lit.toml:1:1 L1004")),
        "{out:#?}"
    );
}
