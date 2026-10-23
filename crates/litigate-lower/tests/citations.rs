//! Source-map citations (SPEC §4.3, §8.6): a source's `section_heading`
//! template, bindings that share a heading template (the one whose
//! vendored file holds the heading wins), and shorthands without a source.
use litgraph::cite::SourceCorpus;
use litgraph::model::{CiteCode, CiteRuleForm, CiteShorthand, CiteSyntax};
use litigate_lower::source_map::{sha256_hex, Citations, Pin};
use litigate_sema::hir::SourceBinding;

fn binding(id: &str, path: Option<&str>, heading: Option<&str>) -> SourceBinding {
    SourceBinding {
        id: id.into(),
        title: None,
        url: None,
        path: path.map(str::to_owned),
        sha256: None,
        path_sha256: None,
        as_of: None,
        cite_prefixes: vec!["FRCP".into()],
        section_heading: heading.map(str::to_owned),
    }
}

/// `section_heading = "Rule {n}"` is the heading template of the rule the
/// binding adds.
#[test]
fn a_section_heading_is_a_heading_template() {
    let sources = [binding("frcp", Some("sources/frcp.txt"), Some("Rule {n}"))];
    let citations = Citations::new(
        CiteSyntax::default(),
        &sources,
        Some(SourceCorpus::embedded_ref()),
    );
    assert_eq!(citations.syntax().rules.len(), 1);
    assert_eq!(citations.syntax().rules[0].heading, "Rule {}");
    let map = citations.map(Some(&"FRCP 12(b)(6)".to_owned()));
    insta::assert_json_snapshot!(map);
}

/// Two bindings with one template: the pack's own rule keeps it, the
/// binding shares it, and the binding whose file holds the heading is the
/// citation's source. A shorthand without a source binds nothing.
#[test]
fn bindings_sharing_a_template_are_told_apart_by_the_corpus() {
    let syntax = CiteSyntax {
        rules: vec![CiteRuleForm {
            prefixes: vec!["FRCP".into()],
            heading: "Rule {}".into(),
            source: Some("elsewhere".into()),
        }],
        shorthands: vec![CiteShorthand {
            prefix: "U.S.C.".into(),
            code: CiteCode::Usc,
            title: 28,
            section_parts: 1,
            source: None,
        }],
    };
    let elsewhere = SourceBinding {
        cite_prefixes: Vec::new(),
        ..binding("elsewhere", Some("sources/frap.txt"), None)
    };
    let sources = [
        elsewhere,
        binding("frcp", Some("sources/frcp.txt"), Some("Rule {n}")),
    ];
    let citations = Citations::new(syntax, &sources, Some(SourceCorpus::embedded_ref()));
    assert_eq!(citations.syntax().rules.len(), 1);
    let section = citations
        .map(Some(&"FRCP 12".to_owned()))
        .and_then(|m| m.sections.into_iter().next())
        .unwrap_or_else(|| panic!("no section"));
    assert_eq!(section.source.as_deref(), Some("frcp"));
    assert_eq!(section.path.as_deref(), Some("sources/frcp.txt"));
    let missing = citations
        .map(Some(&"FRCP 999".to_owned()))
        .and_then(|m| m.sections.into_iter().next())
        .unwrap_or_else(|| panic!("no section"));
    assert_eq!(
        (missing.path, missing.line, missing.sha256),
        (None, None, None)
    );
}

/// The vendored FRCP text's SHA-256, as `lit/core` pins it.
const FRCP_SHA: &str = "cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44";

fn pinned(path: Option<&str>, pin: Option<&str>) -> SourceBinding {
    SourceBinding {
        path_sha256: pin.map(str::to_owned),
        as_of: Some("2024-12-01".into()),
        ..binding("frcp", path, Some("Rule {n}"))
    }
}

/// A binding whose file is the pinned text resolves to a structured
/// citation: the work's binding, the expression's date, the pinpoint, and
/// the hash of the cited section's text.
#[test]
fn a_pinned_binding_gives_the_expression_and_the_span_hash() {
    let sources = [pinned(Some("sources/frcp.txt"), Some(FRCP_SHA))];
    let corpus = SourceCorpus::embedded_ref();
    let citations = Citations::new(CiteSyntax::default(), &sources, Some(corpus));
    assert_eq!(citations.pin("frcp"), &Pin::Verified);
    let section = citations
        .map(Some(&"FRCP 12(b)(6)".to_owned()))
        .and_then(|m| m.sections.into_iter().next())
        .unwrap_or_else(|| panic!("no section"));
    let (_, body) = &corpus.lookup_all("Rule 12")[0];
    assert_eq!(section.as_of.as_deref(), Some("2024-12-01"));
    assert_eq!(section.pinpoint, ["b", "6"]);
    assert_eq!(section.sha256, Some(sha256_hex(&body.body)));
    assert_eq!(section.path.as_deref(), Some("sources/frcp.txt"));
}

/// Every way a binding's file can fail its pin: unpinned, pinned to other
/// text, not in the corpus, or no file at all. Only the pinned text gets a
/// span hash; an undeclared id has no file.
#[test]
fn a_binding_that_is_not_its_pinned_text_gives_no_span_hash() {
    let corpus = SourceCorpus::embedded_ref();
    let pin = |b: SourceBinding| {
        let sources = [b];
        let citations = Citations::new(CiteSyntax::default(), &sources, Some(corpus));
        let section = citations
            .map(Some(&"FRCP 12".to_owned()))
            .and_then(|m| m.sections.into_iter().next())
            .unwrap_or_else(|| panic!("no section"));
        assert_eq!(section.sha256, None);
        (
            citations.pin("frcp").clone(),
            citations.pin("nowhere").clone(),
        )
    };
    assert_eq!(
        pin(pinned(Some("sources/frcp.txt"), None)),
        (
            Pin::Unpinned {
                actual: FRCP_SHA.into()
            },
            Pin::NoPath
        )
    );
    let other = "0".repeat(64);
    assert_eq!(
        pin(pinned(Some("sources/frcp.txt"), Some(&other))).0,
        Pin::Mismatch {
            pinned: other.clone(),
            actual: FRCP_SHA.into()
        }
    );
    assert_eq!(
        pin(pinned(Some("sources/absent.txt"), Some(&other))).0,
        Pin::Absent
    );
    assert_eq!(pin(pinned(None, None)).0, Pin::NoPath);
}
