//! Hover locates a cite's L0 text through the lowering sidecar and narrows
//! it to the pinpoint. The worked example binds no corpus file, so this
//! builds a workspace from it whose `[sources.frcp]` binds
//! `sources/frcp.txt` with `section_heading = "Rule {n}"` (SPEC §4.1).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use serde_json::{json, Value};
use std::path::PathBuf;
use support::{at, read, Client};
use tower_lsp::lsp_types::Url;

/// A fresh workspace holding the worked example bound to the corpus.
fn bound(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("litigate-lsp-{}-{name}", std::process::id()));
    let pkg = root.join("lit/ex");
    std::fs::create_dir_all(&pkg).unwrap();
    let manifest = read("lit/examples/Lit.toml").replace(
        "cite_prefixes = [\"FRCP\"]",
        "path = \"sources/frcp.txt\"\n\
         path_sha256 = \"527fb9fbfe1665bdbdc5565ea5614b299e16b07b07edba803aae6eb5f957b990\"\n\
         cite_prefixes = [\"FRCP\"]\nsection_heading = \"Rule {n}\"",
    );
    std::fs::write(pkg.join("Lit.toml"), manifest).unwrap();
    std::fs::write(pkg.join("rule12.lit"), read("lit/examples/rule12.lit")).unwrap();
    root
}

async fn hover(c: &mut Client, root: &std::path::Path, needle: &str, shift: u32) -> String {
    let text = read("lit/examples/rule12.lit");
    let uri = Url::from_file_path(root.join("lit/ex/rule12.lit")).unwrap();
    let h: Value = c
        .request(
            "textDocument/hover",
            json!({"textDocument": {"uri": uri}, "position": at(&text, needle, 0, shift)}),
        )
        .await;
    h["contents"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn cite_hover_shows_the_pinpointed_l0_text() {
    let root = bound("pinpoint");
    let (mut c, _) = Client::start(&root, json!({})).await;
    let value = hover(
        &mut c,
        &root,
        "cite \"FRCP 12(a)(1)(A)(i)\"\n{\n  deadline",
        6,
    )
    .await;
    assert!(
        value.contains("**L0** Rule 12(a)(1)(A)(i) — `sources/frcp.txt:980-981`"),
        "{value}"
    );
    assert!(
        value.contains("(i) within 21 days after being served with the sum-"),
        "{value}"
    );
    // A cite on a lowered node is the sidecar's entry for it.
    let ruling = hover(&mut c, &root, "cite \"FRCP 12(b)(6)\"", 6).await;
    assert!(
        ruling.contains("**L0** Rule 12(b)(6) — `sources/frcp.txt:"),
        "{ruling}"
    );
    assert!(ruling.contains("(6) failure to state a claim"), "{ruling}");
    insta::assert_snapshot!("cite_rule12_a_1_A_i", value);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn declaration_hover_carries_its_cited_text() {
    let root = bound("declaration");
    let (mut c, _) = Client::start(&root, json!({})).await;
    let value = hover(&mut c, &root, "chance time-barred", 8).await;
    assert!(
        value.contains("chance `frcp-rule12-example::time-barred`"),
        "{value}"
    );
    assert!(
        value.contains("**L0** Rule 8(c)(1) — `sources/frcp.txt:"),
        "{value}"
    );
    let _ = std::fs::remove_dir_all(root);
}
