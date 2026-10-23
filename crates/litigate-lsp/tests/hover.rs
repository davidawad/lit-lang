//! Hover over in-memory workspaces built from the worked example: every
//! kind of declaration and edge, cites in packages that lower, packages
//! that do not, links packages, and loose modules, and the places hover
//! has nothing to say.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use litigate_lsp::analysis::Analysis;
use litigate_lsp::hover;
use litigate_syntax::TextSize;

const MODULE: &str = "lit/ex/rule12.lit";

/// Declarations the worked example does not have.
const EXTRA: &str = "
type Dates = List<Date>

deadline reply-period: 14.court_days before Served(complaint) counted_by frcp6
deadline brief-period: 3.business_days after Answered(complaint) counted_by frcp6

state appeal: \"Appeal\" { group pleadings, onward \"On\" -> answered { authority cite \"FRCP 55(a)\" } }
";

fn extended() -> String {
    format!("{}{EXTRA}", support::read("lit/examples/rule12.lit"))
}

/// The byte offset `shift` into the first `needle` of `text`.
fn offset(text: &str, needle: &str, shift: usize) -> TextSize {
    TextSize::try_from(text.find(needle).expect(needle) + shift).unwrap()
}

/// The hover markdown at `needle` (shifted) in `file`.
fn value(
    analysis: &Analysis,
    file: &str,
    text: &str,
    needle: &str,
    shift: usize,
) -> Option<String> {
    hover::at(analysis, file, text, offset(text, needle, shift)).map(|(v, _)| v)
}

#[test]
fn declarations_show_their_type_line() {
    let text = extended();
    let root = support::worked("hover-types", &[(MODULE, &text)]);
    let mut host = support::host(&root);
    let a = host.analyze();
    let alias = value(a, MODULE, &text, "type Dates", 5).unwrap();
    insta::assert_snapshot!("hover_type_alias", alias);
    let court = value(a, MODULE, &text, "deadline reply-period", 9).unwrap();
    insta::assert_snapshot!("hover_backward_court_days", court);
    let business = value(a, MODULE, &text, "deadline brief-period", 9).unwrap();
    assert!(
        business.contains("`Days<BusinessDays>`: 3 after Answered(complaint)"),
        "{business}"
    );
}

#[test]
fn an_edge_shows_its_authority_and_cited_text() {
    let text = extended();
    let root = support::worked("hover-edge", &[(MODULE, &text)]);
    let mut host = support::host(&root);
    let a = host.analyze();
    let edge = value(a, MODULE, &text, "onward", 0).unwrap();
    insta::assert_snapshot!("hover_edge_authority", edge);
    // The cite token itself, in a package whose lowering has no entry for
    // it (an unbound corpus), resolves through the manifest's sources.
    let cite = value(a, MODULE, &text, "FRCP 55(a)", 1).unwrap();
    assert!(cite.starts_with("**cite** FRCP 55(a)"), "{cite}");
    assert!(cite.contains("not in the vendored corpus"), "{cite}");
}

#[test]
fn a_shadowed_package_shows_the_first_of_its_name() {
    // Two packages share a name: hover reads the first, which lacks the
    // second's node, so the edge has no target line. The duplicate
    // versions do not lower, so the cite resolves without a sidecar.
    let twin = support::read("lit/examples/rule12.lit").replace(
        "state answered:",
        "state extra: \"Extra\" { onward \"On\" -> answered }\nstate answered:",
    );
    let manifest = support::read("lit/examples/Lit.toml");
    let root = support::worked(
        "hover-twin",
        &[
            ("lit/twin/Lit.toml", &manifest),
            ("lit/twin/rule12.lit", &twin),
        ],
    );
    let mut host = support::host(&root);
    let a = host.analyze();
    let file = "lit/twin/rule12.lit";
    let edge = value(a, file, &twin, "onward", 0).unwrap();
    assert!(
        edge.ends_with("edge `frcp-rule12-example::extra.onward`\n"),
        "{edge}"
    );
    let cite = value(a, file, &twin, "FRCP 8(c)(1)", 1).unwrap();
    assert!(
        cite.starts_with("**cite** FRCP 8(c)(1)\n\n**L0** 8(c)(1)"),
        "{cite}"
    );
}

#[test]
fn a_package_sema_rejects_has_no_cite_hover() {
    let broken = support::read("lit/examples/rule12.lit")
        .replace("-> dismissed p 1/10", "-> nowhere p 1/10");
    let root = support::worked("hover-rejected", &[(MODULE, &broken)]);
    let mut host = support::host(&root);
    let a = host.analyze();
    assert!(a.hir("frcp-rule12-example").is_none());
    assert_eq!(value(a, MODULE, &broken, "FRCP 8(c)(1)", 1), None);
    // Names still hover, from the source alone.
    let node = value(a, MODULE, &broken, "chance time-barred", 8).unwrap();
    assert_eq!(node, "```litigate\nchance time-barred: \"Is the claim time-barred?\" by Court cite \"FRCP 8(c)(1)\"\n```\n");
}

#[test]
fn a_loose_module_has_no_cite_hover() {
    let root = support::workspace("hover-loose", &[]);
    let file = root.join("loose.lit");
    let text = "module loose\n\ncounting c cite \"FRCP 6(a)\"\n{\n  exclude_trigger_day\n}\n";
    let mut host = support::host(&root);
    host.open.insert(
        file,
        litigate_lsp::host::Open {
            text: text.into(),
            version: 1,
        },
    );
    let a = host.analyze();
    assert_eq!(value(a, "loose.lit", text, "FRCP", 1), None);
    let counting = value(a, "loose.lit", text, "counting c", 9).unwrap();
    assert_eq!(
        counting,
        "```litigate\ncounting c cite \"FRCP 6(a)\"\n```\n"
    );
}

#[test]
fn a_links_package_cite_resolves_through_its_composition() {
    let mut host = support::host(&support::repo());
    let a = host.analyze();
    let file = "lit/links/links.lit";
    let text = support::read(file);
    let cite = value(a, file, &text, "\"Fed. Cir. R. 40(f)(1)\"\n{", 1).unwrap();
    assert!(cite.starts_with("**cite** Fed. Cir. R. 40(f)(1)"), "{cite}");
}

#[test]
fn nothing_to_say_is_no_hover() {
    let text = support::read("lit/examples/rule12.lit")
        .replace("-> dismissed p 1/10", "-> nowhere p 1/10");
    let root = support::worked("hover-nothing", &[(MODULE, &text)]);
    let mut host = support::host(&root);
    let a = host.analyze();
    // Whitespace between tokens.
    assert_eq!(value(a, MODULE, &text, "\n\nstart", 1), None);
    // A name that resolves to nothing.
    assert_eq!(value(a, MODULE, &text, "nowhere", 2), None);
    // A manifest key that is not a role.
    let manifest = support::read("lit/examples/Lit.toml");
    assert_eq!(value(a, "lit/ex/Lit.toml", &manifest, "version", 2), None);
    // A role of the package sema rejected: its key, without the actor line.
    let role = value(a, "lit/ex/Lit.toml", &manifest, "Defendant", 2).unwrap();
    assert_eq!(role, "```litigate\nDefendant\n```\n");
}

#[test]
fn without_a_verdict_a_cite_resolves_through_the_manifest() {
    let root = support::worked("hover-no-verdict", &[]);
    let mut host = support::host(&root);
    let a = host.analyze();
    let pkg = a.hir("frcp-rule12-example").unwrap();
    let corpus = litgraph::cite::SourceCorpus::embedded_ref();
    let located = litigate_lsp::cite::locate(pkg, None, "FRCP 12(b)(6)", 1, corpus);
    let summary: Vec<(String, Option<String>, Vec<String>)> = located
        .into_iter()
        .map(|l| (l.heading, l.source, l.pinpoint))
        .collect();
    assert_eq!(
        summary,
        [(
            "12".to_owned(),
            Some("frcp".to_owned()),
            vec!["b".to_owned(), "6".to_owned()]
        )]
    );
}

#[test]
fn a_cursor_just_past_a_name_still_hovers_it() {
    let text = support::read("lit/examples/rule12.lit");
    let root = support::worked("hover-left", &[]);
    let mut host = support::host(&root);
    let a = host.analyze();
    // After `respond` and before `:`, the right token is punctuation.
    let after = value(a, MODULE, &text, "decision respond:", 16).unwrap();
    assert!(
        after.contains("decision `frcp-rule12-example::respond`"),
        "{after}"
    );
    let (_, range) = hover::at(a, MODULE, &text, offset(&text, "decision respond:", 16)).unwrap();
    assert_eq!(&text[std::ops::Range::<usize>::from(range)], "respond");
}
