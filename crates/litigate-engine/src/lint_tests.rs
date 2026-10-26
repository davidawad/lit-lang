// SPDX-License-Identifier: GPL-3.0-or-later
//! Unit tests for `lint.rs`.

#![allow(clippy::unwrap_used)]
use super::*;
use crate::model::{CompileOptions, LinkFile, RawNode, Source};

fn pack_with_cite(sources: Vec<Source>, cite: &str) -> Pack {
    Pack {
        schema_version: 2,
        id: "test".into(),
        title: "Test".into(),
        description: None,
        jurisdiction: None,
        forum: None,
        counting: None,
        start_node_id: "n1".into(),
        groups: vec![],
        roles: std::collections::BTreeMap::new(),
        sources,
        cite_syntax: serde_json::from_value(serde_json::json!({"rules": [
            {"prefixes": ["FRCP", "Fed. R. Civ. P."], "heading": "Rule {}"}
        ]}))
        .expect("fixture syntax parses"),
        nodes: vec![RawNode {
            id: "n1".into(),
            label: "n1".into(),
            cite: Some(cite.to_string()),
            ..Default::default()
        }],
        edges: vec![],
        filing_limits: vec![],
        lowered: None,
    }
}

fn source(path: Option<&str>) -> Source {
    Source {
        id: "s".into(),
        title: None,
        url: None,
        path: path.map(str::to_string),
        sha256: None,
        as_of: None,
    }
}

fn frcp_corpus() -> SourceCorpus {
    SourceCorpus::from_files([(
        "sources/frcp-fixture.txt".to_string(),
        "## Rule 12\nDefenses and objections.\n".to_string(),
    )])
}

#[test]
fn reports_unverifiable_cite_with_reason() {
    let p = pack_with_cite(vec![source(Some("sources/frcp-fixture.txt"))], "FRCP 99");
    let mut diags = vec![];
    let mut push = |severity, code, at: String, message: String| {
        diags.push(Diagnostic {
            severity,
            code,
            at,
            message,
        });
    };
    lint_pack_cites(&p, &frcp_corpus(), &mut push);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "unverifiable-cite");
    assert_eq!(diags[0].severity, "warn");
    assert!(diags[0].message.contains("Rule 99"));
}

#[test]
fn no_diagnostic_for_cite_that_resolves() {
    let p = pack_with_cite(vec![source(Some("sources/frcp-fixture.txt"))], "FRCP 12");
    let mut diags = vec![];
    let mut push = |severity, code, at: String, message: String| {
        diags.push(Diagnostic {
            severity,
            code,
            at,
            message,
        });
    };
    lint_pack_cites(&p, &frcp_corpus(), &mut push);
    assert!(diags.is_empty());
}

/// A pack with zero `sources` is already covered by `no-sources`
/// (`lint_pack_sources`); `lint_pack_cites` must not pile on a
/// duplicate per-cite diagnostic.
#[test]
fn no_duplicate_diagnostic_when_pack_has_no_sources() {
    let p = pack_with_cite(vec![], "FRCP 12(b)(6)");
    let mut diags = vec![];
    let mut push = |severity, code, at: String, message: String| {
        diags.push(Diagnostic {
            severity,
            code,
            at,
            message,
        });
    };
    lint_pack_cites(&p, &frcp_corpus(), &mut push);
    assert!(diags.is_empty());
}

#[test]
fn case_citation_produces_no_diagnostic() {
    let p = pack_with_cite(
        vec![source(Some("sources/frcp-fixture.txt"))],
        "Bowles v. Russell, 551 U.S. 205 (2007)",
    );
    let mut diags = vec![];
    let mut push = |severity, code, at: String, message: String| {
        diags.push(Diagnostic {
            severity,
            code,
            at,
            message,
        });
    };
    lint_pack_cites(&p, &frcp_corpus(), &mut push);
    assert!(diags.is_empty());
}

/// End-to-end through the public `lint()` entry point (uses the real
/// embedded corpus via `SourceCorpus::default_source()`), not just the
/// unit-level `lint_pack_cites` helper.
#[test]
fn lint_reports_unverifiable_cite_via_public_entry_point() {
    let p = pack_with_cite(vec![source(Some("sources/frcp.txt"))], "FRCP 9999");
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let diags = lint(&g, std::slice::from_ref(&p));
    assert!(diags.iter().any(|d| d.code == "unverifiable-cite"));
}

fn pack(json: &str) -> Pack {
    Pack::from_json(json).unwrap()
}

#[test]
fn fact_node_without_a_full_prior_is_flagged() {
    let p = pack(
        r#"{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
                "sources": [{"id": "s", "title": "t", "url": "https://example.com"}],
                "nodes": [
                    {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
                    {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0, "outcome": ["win"]},
                    {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0, "outcome": ["loss"]}
                ],
                "edges": [
                    {"from": "check", "to": "a", "label": "a", "actor": "office"},
                    {"from": "check", "to": "b", "label": "b", "actor": "office", "probability": 0.1}
                ]
            }"#,
    );
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let d = lint(&g, &[p]);
    assert!(d
        .iter()
        .any(|x| x.code == "fact-no-prior" && x.at == "demo::check"));
    assert!(!d.iter().any(|x| x.code == "chance-unquantified"));
}

#[test]
fn fact_node_with_a_full_prior_is_not_flagged() {
    let p = pack(
        r#"{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
                "sources": [{"id": "s", "title": "t", "url": "https://example.com"}],
                "nodes": [
                    {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
                    {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0, "outcome": ["win"]},
                    {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0, "outcome": ["loss"]}
                ],
                "edges": [
                    {"from": "check", "to": "a", "label": "a", "actor": "office", "probability": 0.9},
                    {"from": "check", "to": "b", "label": "b", "actor": "office", "probability": 0.1}
                ]
            }"#,
    );
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let d = lint(&g, &[p]);
    assert!(!d.iter().any(|x| x.code == "fact-no-prior"));
}

/// The `fact-no-prior` check is gated on "every out-edge is non-applicant"
/// for a pure chance node -- a *mixed* fact node (an applicant choice plus
/// fact-driven interrupts, which the pack schema admits though litigate
/// authors every fact as a `chance` node) never hit that gate at all before
/// this test, so an unauthored interrupt prior went unflagged.
#[test]
fn mixed_fact_node_without_a_full_interrupt_prior_is_flagged() {
    let p = pack(
        r#"{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
                "sources": [{"id": "s", "title": "t", "url": "https://example.com"}],
                "nodes": [
                    {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
                    {"id": "refiled", "label": "Refiled", "kind": "state"},
                    {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0, "outcome": ["win"]},
                    {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0, "outcome": ["loss"]}
                ],
                "edges": [
                    {"from": "check", "to": "refiled", "label": "refile", "actor": "applicant"},
                    {"from": "check", "to": "a", "label": "a", "actor": "office"},
                    {"from": "check", "to": "b", "label": "b", "actor": "office", "probability": 0.1}
                ]
            }"#,
    );
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let d = lint(&g, &[p]);
    assert!(d
        .iter()
        .any(|x| x.code == "fact-no-prior" && x.at == "demo::check"));
    // Still a mixed node -- that info diagnostic is orthogonal and unchanged.
    assert!(d
        .iter()
        .any(|x| x.code == "mixed-node" && x.at == "demo::check"));
}

#[test]
fn mixed_fact_node_with_a_full_interrupt_prior_is_not_flagged() {
    let p = pack(
        r#"{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
                "sources": [{"id": "s", "title": "t", "url": "https://example.com"}],
                "nodes": [
                    {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
                    {"id": "refiled", "label": "Refiled", "kind": "state"},
                    {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0, "outcome": ["win"]},
                    {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0, "outcome": ["loss"]}
                ],
                "edges": [
                    {"from": "check", "to": "refiled", "label": "refile", "actor": "applicant"},
                    {"from": "check", "to": "a", "label": "a", "actor": "office", "probability": 0.9},
                    {"from": "check", "to": "b", "label": "b", "actor": "office", "probability": 0.1}
                ]
            }"#,
    );
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let d = lint(&g, &[p]);
    assert!(!d.iter().any(|x| x.code == "fact-no-prior"));
}

#[test]
fn a_plain_chance_node_without_the_fact_tag_still_gets_chance_unquantified() {
    let p = pack(
        r#"{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
                "sources": [{"id": "s", "title": "t", "url": "https://example.com"}],
                "nodes": [
                    {"id": "check", "label": "Check", "kind": "decision"},
                    {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0, "outcome": ["win"]},
                    {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0, "outcome": ["loss"]}
                ],
                "edges": [
                    {"from": "check", "to": "a", "label": "a", "actor": "office"},
                    {"from": "check", "to": "b", "label": "b", "actor": "office", "probability": 0.1}
                ]
            }"#,
    );
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    let d = lint(&g, &[p]);
    assert!(d.iter().any(|x| x.code == "chance-unquantified"));
    assert!(!d.iter().any(|x| x.code == "fact-no-prior"));
}

/// The sums of the consent-gated `ruling` (`requires`/`forbids` on
/// `consented`) and, when `rr` is not exclusive of `consented`, the context
/// that sees both distributions at once.
fn gated_ruling_sums(rr_forbids: &str) -> Vec<String> {
    let p = pack(&format!(
        r#"{{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "consent",
                "nodes": [
                    {{"id": "consent", "label": "Consent", "kind": "decision"}},
                    {{"id": "ruling", "label": "Ruling", "kind": "state"}},
                    {{"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0}},
                    {{"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0}}
                ],
                "edges": [
                    {{"id": "yes", "from": "consent", "to": "ruling", "label": "yes", "actor": "applicant", "sets": ["consented"]}},
                    {{"id": "no", "from": "consent", "to": "ruling", "label": "no", "actor": "applicant", "sets": ["declined"]}},
                    {{"id": "fa", "from": "ruling", "to": "a", "label": "a", "actor": "office", "probability": 0.5, "requires": ["consented"]}},
                    {{"id": "fb", "from": "ruling", "to": "b", "label": "b", "actor": "office", "probability": 0.5, "requires": ["consented"]}},
                    {{"id": "ra", "from": "ruling", "to": "a", "label": "a", "actor": "office", "probability": 0.7, "requires": ["declined"], "forbids": [{rr_forbids}]}},
                    {{"id": "rb", "from": "ruling", "to": "b", "label": "b", "actor": "office", "probability": 0.3, "requires": ["declined"], "forbids": [{rr_forbids}]}},
                    {{"id": "va", "from": "ruling", "to": "a", "label": "a", "actor": "office", "probability": 0.1, "forbids": ["consented", "declined"]}},
                    {{"id": "vb", "from": "ruling", "to": "b", "label": "b", "actor": "office", "probability": 0.9, "forbids": ["consented", "declined"]}}
                ]
            }}"#
    ));
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    lint(&g, &[p])
        .into_iter()
        .filter(|x| x.code == "probability-sum")
        .map(|x| format!("{}: {}", x.at, x.message))
        .collect()
}

#[test]
fn a_gated_chance_node_sums_to_one_per_flag_context() {
    assert_eq!(gated_ruling_sums(r#""consented""#), Vec::<String>::new());
    assert_eq!(
        gated_ruling_sums(""),
        ["demo::ruling: chance node probabilities sum to 2.000 with flags {consented,declined} set"]
    );
}

#[test]
fn a_node_gated_on_too_many_flags_is_summed_as_if_ungated() {
    let edges: Vec<String> = (0..13)
        .map(|i| {
            format!(
                r#"{{"id": "e{i}", "from": "s", "to": "t", "label": "e", "actor": "office", "probability": 0.5, "requires": ["f{i}"]}}"#
            )
        })
        .collect();
    let p = pack(&format!(
        r#"{{
                "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "s",
                "nodes": [
                    {{"id": "s", "label": "S", "kind": "state"}},
                    {{"id": "t", "label": "T", "kind": "terminal", "payoff": 0.0}}
                ],
                "edges": [{}]
            }}"#,
        edges.join(",")
    ));
    let g = Graph::compile(
        std::slice::from_ref(&p),
        &LinkFile::default(),
        &CompileOptions::default(),
    )
    .unwrap();
    assert!(lint(&g, &[p])
        .iter()
        .any(|x| x.code == "probability-sum"
            && x.message == "chance node probabilities sum to 6.500"));
}
