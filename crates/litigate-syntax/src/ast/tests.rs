use super::{AstNode, CaseFile, EdgeDecl, Expr, LitFile, NodeDecl, Token};
use crate::parse::{parse_case, parse_lit};
use crate::syntax_kind::SyntaxKind;
use rowan::{GreenNodeBuilder, SyntaxNode};

mod case;
mod decls;
mod expr;
mod items;

/// The `.lit` fixture exercising SPEC §3-§8 constructs.
const CONSTRUCTS: &str = include_str!("../../tests/fixtures/constructs.lit");
/// The `.lit` fixture with cross-pack links and an instance.
const LINKS: &str = include_str!("../../tests/fixtures/links.lit");
/// The `.case` fixture with every optional case item.
const MATTER: &str = include_str!("../../tests/fixtures/matter.case");

/// Parses error-free `.lit` text and returns its typed root.
fn lit(text: &str) -> LitFile {
    let parsed = parse_lit(text);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    parsed.lit_file().expect("lit root")
}

/// Parses error-free `.case` text and returns its typed root.
fn case(text: &str) -> CaseFile {
    let parsed = parse_case(text);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    parsed.case_file().expect("case root")
}

/// The item of `file` of wrapper type `N` declaring `name`.
fn decl<N: AstNode<Language = crate::LitLanguage>>(file: &LitFile, name: &str) -> N {
    file.items()
        .iter()
        .filter_map(|item| N::cast(item.syntax().clone()))
        .find(|node| text(super::token(node.syntax(), SyntaxKind::Ident)).as_deref() == Some(name))
        .expect("declaration of the expected kind and name")
}

/// Token text, owned.
fn text(token: Option<Token>) -> Option<String> {
    token.map(|token| token.text().to_owned())
}

#[test]
fn multi_kind_wrappers_cast_every_listed_kind() {
    assert!(NodeDecl::can_cast(SyntaxKind::ChanceDecl));
    assert!(NodeDecl::can_cast(SyntaxKind::TerminalDecl));
    assert!(!NodeDecl::can_cast(SyntaxKind::EdgeDecl));
    assert!(Expr::can_cast(SyntaxKind::MatchExpr) && !Expr::can_cast(SyntaxKind::Ident));
}

#[test]
fn name_is_first_identifier() {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind::StateDecl.into());
    builder.token(SyntaxKind::KwState.into(), "state");
    builder.token(SyntaxKind::Whitespace.into(), " ");
    builder.token(SyntaxKind::Ident.into(), "answer-due");
    builder.finish_node();
    let root = SyntaxNode::new_root(builder.finish());
    let decl = NodeDecl::cast(root).map(|d| d.name().map(|t| t.text().to_string()));
    assert_eq!(decl, Some(Some("answer-due".to_string())));
}

const NODE: &str = "module m\n/// Is it barred?\n/// Second line.\nchance c: \"C\" by Court discretion cite \"FRCP 8(c)(1)\"\n{\n  tags [fact]\n  barred \"Barred\" -> done p 1/10\n  _ \"Timely\" -> next p ?(\"unknown\") { within d { extendable true }, cost -5.usd }\n}\n";

#[test]
fn node_accessors_expose_every_part() {
    let parsed = parse_lit(NODE);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    let file = parsed.lit_file().expect("lit root");
    let items = file.items();
    let node = NodeDecl::cast(items[0].syntax().clone()).expect("node decl");
    assert_eq!(node.docs().as_deref(), Some("Is it barred?\nSecond line."));
    assert_eq!(node.keyword().map(|t| t.kind()), Some(SyntaxKind::KwChance));
    assert_eq!(
        node.label().map(|t| t.text().to_owned()).as_deref(),
        Some("\"C\"")
    );
    assert_eq!(node.by().map(|r| r.text()).as_deref(), Some("Court"));
    assert!(node.discretion());
    assert_eq!(node.cites()[0].text().as_deref(), Some("FRCP 8(c)(1)"));
    assert_eq!(node.fields().len(), 1);
    let edges: Vec<EdgeDecl> = node.edges();
    assert_eq!(edges.len(), 2);
    assert_eq!(
        edges[0].id().map(|t| t.text().to_owned()).as_deref(),
        Some("barred")
    );
    assert!(edges[1].id().is_none());
    let p = edges[0].probability().expect("p");
    assert_eq!(p.kind(), SyntaxKind::Ratio);
    let hole = edges[1].probability().expect("hole");
    let reason = super::Hole::cast(hole.syntax().clone()).and_then(|h| h.reason());
    assert_eq!(reason.as_deref(), Some("unknown"));
    let fields = edges[1].fields();
    assert_eq!(fields[0].keyword_kind(), Some(SyntaxKind::KwWithin));
    assert_eq!(
        fields[0].options()[0].keyword_kind(),
        Some(SyntaxKind::KwExtendable)
    );
    assert_eq!(edges[1].target().map(|r| r.text()).as_deref(), Some("next"));
}

#[test]
fn ref_parts_split_instances_paths_and_versions() {
    let parsed = parse_lit("module m\nstart cafc@cofc::entry.x@2009-12-01\n");
    let file = parsed.lit_file().expect("root");
    let start = super::StartDecl::cast(file.items()[0].syntax().clone()).expect("start");
    let parts = start.target().expect("ref").parts();
    assert_eq!(parts.head, "cafc");
    assert_eq!(parts.instance.as_deref(), Some("cofc"));
    assert_eq!(
        parts.segments,
        vec![("::".into(), "entry".into()), (".".into(), "x".into())]
    );
    assert_eq!(parts.version.as_deref(), Some("2009-12-01"));
}

#[test]
fn rule_bodies_and_deadlines_are_typed() {
    let text = "module m\nrule duty r effective [2020-01-01, ..) cite \"X 1\"\n{\n  deadline d: 21.days from Served(c) counted_by frcp6 { extendable false }\n}\n";
    let parsed = parse_lit(text);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    let file = parsed.lit_file().expect("root");
    let rule = super::RuleDecl::cast(file.items()[0].syntax().clone()).expect("rule");
    assert_eq!(rule.rule_kind().map(|t| t.kind()), Some(SyntaxKind::KwDuty));
    let interval = rule.version_clauses()[0].interval().expect("interval");
    assert_eq!(
        interval.from().map(|t| t.text().to_owned()).as_deref(),
        Some("2020-01-01")
    );
    assert!(interval.to().is_none());
    let deadline = super::DeadlineDecl::cast(rule.items()[0].syntax().clone()).expect("deadline");
    assert_eq!(
        deadline.period().map(|t| t.text().to_owned()).as_deref(),
        Some("21.days")
    );
    assert_eq!(
        deadline.direction().map(|t| t.kind()),
        Some(SyntaxKind::KwFrom)
    );
    assert_eq!(
        deadline.trigger().map(|e| e.kind()),
        Some(SyntaxKind::CallExpr)
    );
    assert_eq!(
        deadline.counted_by().map(|r| r.text()).as_deref(),
        Some("frcp6")
    );
    assert_eq!(deadline.options().len(), 1);
}

#[test]
fn case_accessors_expose_blocks() {
    let text = "case c\nuses p ^1.2.3-rc.1\nas_of 2025-01-01\nsummary \"S\"\n\nevents {\n  2025-01-02 Served(x) on b cite \"R 4\"\n}\n\nstakes { j: -1.usd, k: unvalued }\nbeliefs { observe n: { e: 3 } }\n";
    let parsed = parse_case(text);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    let file = parsed.case_file().expect("case root");
    let pin = &file.uses().expect("uses").items()[0];
    let req = pin.requirement().and_then(|r| r.text()).expect("req");
    assert_eq!((req.op.as_str(), req.version.as_str()), ("^", "1.2.3-rc.1"));
    assert_eq!(
        file.as_of().map(|t| t.text().to_owned()).as_deref(),
        Some("2025-01-01")
    );
    let event = &file.events()[0];
    assert_eq!(event.mode().map(|t| t.kind()), Some(SyntaxKind::KwOn));
    assert_eq!(event.cite().and_then(|c| c.text()).as_deref(), Some("R 4"));
    let stakes = file.block(SyntaxKind::StakesBlock).expect("stakes");
    let keys: Vec<String> = stakes
        .entries()
        .iter()
        .filter_map(super::RecordField::key_text)
        .collect();
    assert_eq!(keys, ["j", "k"]);
    let beliefs = file.block(SyntaxKind::BeliefsBlock).expect("beliefs");
    assert_eq!(
        beliefs.observations()[0]
            .node()
            .map(|r| r.text())
            .as_deref(),
        Some("n")
    );
}

#[test]
fn unescape_decodes_escapes() {
    assert_eq!(super::unescape(r#""a\"b\\c\nd""#), "a\"b\\c\nd");
}

#[test]
fn unescape_decodes_control_escapes_and_drops_a_trailing_backslash() {
    assert_eq!(super::unescape(r#""a\rb\tc""#), "a\rb\tc");
    assert_eq!(super::unescape("unquoted\\"), "unquoted");
}

#[test]
fn root_views_cast_only_their_own_file_kind() {
    let lit_root = parse_lit("module m\n").syntax();
    let case_root = parse_case("case c\n").syntax();
    assert!(super::lit_file(lit_root.clone()).is_some());
    assert!(super::lit_file(case_root.clone()).is_none());
    assert!(super::case_file(case_root).is_some());
    assert!(super::case_file(lit_root).is_none());
}

#[test]
fn trait_syntax_matches_the_inherent_view() {
    let file = lit("module m\n");
    let via_trait = AstNode::syntax(&file);
    assert_eq!(via_trait.kind(), SyntaxKind::SourceFile);
    assert_eq!(via_trait, file.syntax());
}
