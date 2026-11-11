use super::parse;
use super::tree::{Atom, ExprKind, Item, ItemKind, NumericBasis, TestStmt};
use litigate_base::DiagnosticCode as C;
use proptest::prelude::*;

fn read(relative: &str) -> String {
    let path = format!("{}/../../{relative}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap_or_default()
}

fn find(items: &[Item], pred: impl Fn(&ItemKind) -> bool) -> Option<&ItemKind> {
    items.iter().map(|i| &i.kind).find(|k| pred(k))
}

#[test]
fn worked_example_lowers_cleanly() {
    let parsed = parse(&read("lit/examples/rule12.lit"));
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
            ItemKind::Node(n) => Some((n.name.text.as_str(), n.edges.len(), n.fields.len())),
            _ => None,
        })
        .collect();
    assert_eq!(
        nodes,
        [
            ("time-barred", 2, 1),
            ("respond", 3, 1),
            ("mtd-ruling", 2, 2),
            ("answered", 1, 1),
            ("default-risk", 1, 1),
            ("judgment-entered", 0, 2),
            ("dismissed", 0, 2)
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
    let (kind, name, clauses, inner) = tree
        .items
        .iter()
        .find_map(|i| match &i.kind {
            ItemKind::Rule(kind, name, clauses, inner) => Some((kind, name, clauses, inner)),
            _ => None,
        })
        .expect("rule missing");
    assert_eq!(
        (kind.text.as_str(), name.text.as_str()),
        ("duty", "answer-duty")
    );
    assert!(
        matches!(&clauses[0].values[0].0, Atom::Expr(e) if e.kind == ExprKind::Cite("FRCP 12(a)(1)(A)(i)".into(), false))
    );
    assert!(
        matches!(&inner[0].kind, ItemKind::Deadline { direction, .. } if direction.text == "from")
    );
}

#[test]
fn item_tree_keeps_numeric_provenance_as_a_typed_atom() {
    let parsed = parse(concat!(
        "module m\n",
        "terminal t: \"T\" { payoff 1.usd basis placeholder vintage 2026-01-01 }\n",
    ));
    assert!(parsed.reports.is_empty(), "{:?}", parsed.reports);
    let ItemKind::Node(node) = &parsed.tree.items[0].kind else {
        panic!("node")
    };
    assert!(node.fields.iter().any(|field| field.values.iter().any(|(atom, _)| {
        matches!(atom, Atom::NumericProvenance(p) if p.vintage.as_ref().is_some_and(|d| d.text == "2026-01-01") && p.basis == Some(NumericBasis::Placeholder))
    })));
}

#[test]
fn estimate_and_placeholder_without_vintage_warn_in_the_item_tree() {
    for basis in ["estimate", "placeholder"] {
        let parsed = parse(&format!(
            "module m\nterminal t: \"T\" {{ payoff 1.usd basis {basis} }}\n"
        ));
        assert!(
            parsed
                .reports
                .iter()
                .any(|r| r.code == C::NumericProvenanceIncomplete),
            "{basis}: {:?}",
            parsed.reports
        );
    }
    let duplicate = parse(concat!(
        "module m\n",
        "terminal t: \"T\" { payoff 1.usd basis estimate basis placeholder vintage 2026-01-01 }\n",
    ));
    assert!(duplicate
        .reports
        .iter()
        .any(|r| r.code == C::NumericProvenanceIncomplete));
    let duplicate_vintage = parse(concat!(
        "module m\n",
        "terminal t: \"T\" { payoff 1.usd basis estimate vintage 2026-01-01 vintage 2026-02-01 }\n",
    ));
    assert!(duplicate_vintage
        .reports
        .iter()
        .any(|r| r.code == C::NumericProvenanceIncomplete));
}

#[test]
fn every_construct_lowers_without_reports() {
    for file in [
        "crates/litigate-syntax/tests/fixtures/constructs.lit",
        "crates/litigate-syntax/tests/fixtures/links.lit",
    ] {
        let parsed = parse(&read(file));
        assert_eq!(parsed.reports, vec![], "{file}");
        assert!(!parsed.tree.items.is_empty(), "{file}");
    }
    let tree = parse(&read(
        "crates/litigate-syntax/tests/fixtures/constructs.lit",
    ))
    .tree;
    assert_eq!(tree.imports.len(), 2);
    assert_eq!(
        tree.imports[1].names.as_ref().map(Vec::len),
        Some(2),
        "import list"
    );
    let exception = find(
        &tree.items,
        |k| matches!(k, ItemKind::Exception { name, .. } if name.text == "local-rule"),
    );
    assert!(matches!(
        exception,
        Some(ItemKind::Exception { then: Some(_), priority: Some(_), clauses, .. }) if clauses.len() == 1
    ));
    let undercut = find(
        &tree.items,
        |k| matches!(k, ItemKind::Exception { name, .. } if name.text == "moot"),
    );
    assert!(matches!(
        undercut,
        Some(ItemKind::Exception {
            then: None,
            priority: None,
            ..
        })
    ));
    let (amends, arms) = tree
        .items
        .iter()
        .find_map(|i| match &i.kind {
            ItemKind::Transition(_, amends, _, arms) => Some((amends, arms)),
            _ => None,
        })
        .expect("transition missing");
    assert_eq!(
        amends.version.as_ref().map(|d| d.text.as_str()),
        Some("2009-12-01")
    );
    assert!(arms[0].unless.is_some());
    assert!(arms[1].unless.is_none());
    let links = parse(&read("crates/litigate-syntax/tests/fixtures/links.lit")).tree;
    let (base, origin, fields) = links
        .items
        .iter()
        .find_map(|i| match &i.kind {
            ItemKind::Instance(base, origin, fields) => Some((base, origin, fields)),
            _ => None,
        })
        .expect("instance missing");
    assert_eq!((base.text.as_str(), origin.text.as_str()), ("cafc", "cofc"));
    let patch = fields.iter().find(|f| f.key.text == "patch");
    assert!(matches!(
        patch.map(|f| &f.values[1].0),
        Some(Atom::Fields(fields)) if fields.len() == 4 && fields[3].key.text == "deadline"
    ));
}

fn codes(src: &str) -> Vec<C> {
    parse(src).reports.into_iter().map(|r| r.code).collect()
}

#[test]
fn syntax_errors_are_reported_and_later_items_survive() {
    for src in [
        "module m\nstate a: \"A\" { bogus 1 }\nstate b: \"B\"",
        "module m\nstate a \"A\"\nstate b: \"B\"",
        "module m\ndefault d when a < b < c => 1\nstate b: \"B\"",
    ] {
        let parsed = parse(src);
        assert!(
            parsed.reports.iter().any(|r| r.code == C::UnexpectedToken),
            "{src}: {:?}",
            parsed.reports
        );
        assert!(
            parsed
                .tree
                .items
                .iter()
                .any(|i| matches!(&i.kind, ItemKind::Node(n) if n.name.text == "b")),
            "{src}"
        );
    }
    assert_eq!(codes("module m\n/// dangling\n"), [C::DanglingDocComment]);
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

proptest! {
    /// Lowering is total: any input yields a tree, and every report lies
    /// inside the source.
    #[test]
    fn lowering_is_total(src in "(module m\n)?[a-z_ :\"{}()\\[\\],.@?>=0-9\n/-]{0,120}") {
        let parsed = parse(&src);
        let len = u32::try_from(src.len()).unwrap_or(u32::MAX);
        prop_assert!(parsed.reports.iter().all(|r| r.range.start <= r.range.end));
        prop_assert!(parsed.reports.iter().all(|r| r.range.end <= len));
    }
}
