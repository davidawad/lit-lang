use super::parse;
use super::tree::{ItemKind, TestStmt};
use litigate_base::DiagnosticCode as C;

fn fixture() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../lit/examples/rule12.lit");
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn worked_example_parses_cleanly() {
    let parsed = parse(&fixture());
    assert_eq!(parsed.reports, vec![]);
    let tree = parsed.tree;
    assert_eq!(super::tree::dotted(&tree.module), "frcp.rule12");
    assert_eq!(
        tree.package.as_ref().map(|p| p.name.text.as_str()),
        Some("frcp-rule12-example")
    );
    let nodes: Vec<_> = tree
        .items
        .iter()
        .filter_map(|i| match &i.kind {
            ItemKind::Node(n) => Some((n.name.text.as_str(), n.edges.len())),
            _ => None,
        })
        .collect();
    assert_eq!(
        nodes,
        [
            ("time-barred", 2),
            ("respond", 3),
            ("mtd-ruling", 2),
            ("answered", 1),
            ("default-risk", 1),
            ("judgment-entered", 0),
            ("dismissed", 0)
        ]
    );
    let docs: Vec<_> = tree.items.iter().filter_map(|i| i.doc.clone()).collect();
    assert_eq!(docs, ["Service of a pleading on a party."]);
    let tests = tree
        .items
        .iter()
        .filter(
            |i| matches!(&i.kind, ItemKind::Test(_, _, s) if matches!(s[0], TestStmt::Given(..))),
        )
        .count();
    assert_eq!(tests, 2);
}

fn codes(src: &str) -> Vec<C> {
    parse(src).reports.into_iter().map(|r| r.code).collect()
}

#[test]
fn reports_and_recovers() {
    assert_eq!(
        codes("module m\nstate a: \"A\" { bogus 1 }\nstate b: \"B\""),
        [C::InvalidField]
    );
    assert_eq!(
        codes("module m\nstate a \"A\"\nstate b: \"B\""),
        [C::UnexpectedToken]
    );
    assert_eq!(
        codes("module m\nenum E { a, b\nstate b: \"B\""),
        [C::UnterminatedLiteral]
    );
    assert_eq!(
        codes("module m\ndefault d when a < b < c => 1"),
        [C::UnexpectedToken]
    );
    assert_eq!(codes("module m\n/// dangling\n"), [C::DanglingDocComment]);
    assert_eq!(codes("state a: \"A\""), [C::UnexpectedToken]);
}

#[test]
fn unclosed_brace_offers_machine_fix() {
    let parsed = parse("module m\nstate a: \"A\" { note \"x\"");
    let report = parsed
        .reports
        .iter()
        .find(|r| r.code == C::UnterminatedLiteral);
    assert!(report.is_some_and(|r| r.fixes.len() == 1 && r.fixes[0].1 == "}"));
}

#[test]
fn parses_links_instances_transitions_and_exceptions() {
    let src = r#"module links
link appeal "Appeal" : cofc::judgment -> cafc@cofc::entry p 1/2 { replaces [cofc::old] }
instance cafc @ cofc { pack cafc, note "x", probabilities { a: 1/3 }, patch e { "hours": unset } }
transition t amends d@2009-12-01 cite FRCP 86(a) { unless has_event(X()) -> d@2007-12-01, otherwise -> d@2009-12-01 }
exception e undercuts d when !has_event(X()) priority 2
default d when true => match k { a => 1, _ => 2 }
"#;
    let parsed = parse(src);
    assert_eq!(parsed.reports, vec![]);
    assert_eq!(parsed.tree.items.len(), 5);
}
