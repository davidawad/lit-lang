//! Source-map citations (SPEC §4.3, §8.6): a source's `section_heading`
//! template, bindings that share a heading template (the one whose
//! vendored file holds the heading wins), and shorthands without a source.
use litgraph::cite::SourceCorpus;
use litgraph::model::{CiteCode, CiteRuleForm, CiteShorthand, CiteSyntax};
use litigate_lower::source_map::Citations;
use litigate_sema::hir::SourceBinding;

fn binding(id: &str, path: Option<&str>, heading: Option<&str>) -> SourceBinding {
    SourceBinding {
        id: id.into(),
        title: None,
        url: None,
        path: path.map(str::to_owned),
        sha256: None,
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
}
