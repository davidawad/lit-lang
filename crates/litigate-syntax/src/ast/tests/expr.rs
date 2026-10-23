//! Accessors of references, literals, and expressions.
use super::{case, decl, lit, text, CONSTRUCTS, LINKS, MATTER};
use crate::ast::{AstNode, CallExpr, DefaultDecl, Expr, Hole, InstanceDecl, Literal, Ratio, Ref};
use crate::syntax_kind::SyntaxKind;

/// The `when` condition of `default d when <expr> => 1`.
fn condition(expr: &str) -> Expr {
    let file = lit(&format!("module m\ndefault d when {expr} => 1\n"));
    let default: DefaultDecl = decl(&file, "d");
    default.condition().expect("condition")
}

#[test]
fn refs_report_text_idents_and_qualification() {
    let file = lit(LINKS);
    let link = crate::ast::LinkDecl::cast(file.items()[1].syntax().clone()).expect("link");
    let from: Ref = link.from().expect("from");
    assert_eq!(from.text(), "cafc@cofc::mandate");
    let idents: Vec<String> = from.idents().iter().map(|t| t.text().to_owned()).collect();
    assert_eq!(idents, ["cafc", "cofc", "mandate"]);
    assert!(from.is_qualified());
    let parts = from.parts();
    assert_eq!(parts.head, "cafc");
    assert_eq!(parts.instance.as_deref(), Some("cofc"));
    assert_eq!(parts.segments, [("::".to_owned(), "mandate".to_owned())]);
    assert!(parts.version.is_none());

    let module = lit(CONSTRUCTS)
        .module()
        .and_then(|m| m.path())
        .expect("path");
    assert!(!module.is_qualified());
    let parts = module.parts();
    assert_eq!(parts.head, "frcp");
    assert!(parts.instance.is_none());
    assert_eq!(parts.segments.len(), 2);
}

#[test]
fn literals_holes_and_ratios() {
    let file = lit(LINKS);
    let instance: InstanceDecl = decl(&file, "cafc");
    let fields = instance.fields();
    let note = fields[1].value_expr().expect("note");
    let note = Literal::cast(note.syntax().clone()).expect("literal");
    assert_eq!(
        note.string_value().as_deref(),
        Some("CAFC reviewing a COFC judgment")
    );
    let probabilities = fields[3].record().expect("record").fields();
    let ratio = probabilities[0]
        .value_expr()
        .and_then(|e| Ratio::cast(e.syntax().clone()))
        .expect("ratio");
    assert_eq!(text(ratio.numerator()).as_deref(), Some("7"));
    assert_eq!(text(ratio.denominator()).as_deref(), Some("10"));
    let hole = probabilities[1]
        .value_expr()
        .and_then(|e| Hole::cast(e.syntax().clone()))
        .expect("hole");
    assert_eq!(hole.reason().as_deref(), Some("no data"));

    let number = condition("3");
    let number = Literal::cast(number.syntax().clone()).expect("literal");
    assert_eq!(number.token().map(|t| t.kind()), Some(SyntaxKind::Number));
    assert!(number.string_value().is_none());
    let bare = Hole::cast(condition("?").syntax().clone()).expect("hole");
    assert!(bare.reason().is_none());
}

#[test]
fn calls_records_and_fields() {
    let call = condition("f(1, x)");
    let typed = CallExpr::cast(call.syntax().clone()).expect("call");
    assert_eq!(typed.callee().map(|r| r.text()).as_deref(), Some("f"));
    assert_eq!(typed.args().len(), 2);
    let operands: Vec<SyntaxKind> = call.operands().iter().map(Expr::kind).collect();
    assert_eq!(operands, [SyntaxKind::Literal, SyntaxKind::RefExpr]);
    assert!(call.operator().is_none());
    assert!(call.reference().is_none());
    assert!(call.field_name().is_none());
    assert!(call.arms().is_empty());
    let empty = CallExpr::cast(condition("g()").syntax().clone()).expect("call");
    assert!(empty.args().is_empty());

    let field = condition("f(x).due");
    assert_eq!(field.kind(), SyntaxKind::FieldExpr);
    assert_eq!(text(field.field_name()).as_deref(), Some("due"));
    assert_eq!(field.operands().len(), 1);

    let reference = condition("a.b");
    assert_eq!(reference.kind(), SyntaxKind::RefExpr);
    assert_eq!(
        reference.reference().map(|r| r.text()).as_deref(),
        Some("a.b")
    );
    assert!(reference.operands().is_empty());
}

#[test]
fn operators_of_unary_and_binary_expressions() {
    let binary = condition("1 + 2");
    assert_eq!(binary.operator().map(|t| t.kind()), Some(SyntaxKind::Plus));
    assert_eq!(binary.operands().len(), 2);
    let unary = condition("!x");
    assert_eq!(unary.operator().map(|t| t.kind()), Some(SyntaxKind::Bang));
    assert_eq!(unary.operands().len(), 1);
}

#[test]
fn match_arms_expose_patterns_and_values() {
    let expr = condition("match x { granted => 1, _ => 0 }");
    assert_eq!(expr.kind(), SyntaxKind::MatchExpr);
    let arms = expr.arms();
    assert_eq!(arms.len(), 2);
    assert_eq!(
        arms[0].pattern().map(|r| r.text()).as_deref(),
        Some("granted")
    );
    assert_eq!(
        arms[0].value().map(|e| e.syntax().to_string()).as_deref(),
        Some("1")
    );
    assert!(arms[1].pattern().is_none());
    assert_eq!(
        arms[1].value().map(|e| e.syntax().to_string()).as_deref(),
        Some("0")
    );
}

#[test]
fn record_fields_decode_keys_and_values() {
    let file = lit("module m\ninstance a@b { patch e { \"la\\tbel\": \"x\", tags: [\"t\"] } }\n");
    let instance: InstanceDecl = decl(&file, "a");
    let fields = instance.patches()[0].record().expect("record").fields();
    let quoted = &fields[0];
    assert_eq!(quoted.key().map(|t| t.kind()), Some(SyntaxKind::String));
    assert_eq!(quoted.key_text().as_deref(), Some("la\tbel"));
    assert!(quoted.key_ref().is_none());
    assert_eq!(quoted.value().map(|n| n.kind()), Some(SyntaxKind::Literal));
    assert!(quoted.value_ref().is_none());
    let plain = &fields[1];
    assert_eq!(plain.key_text().as_deref(), Some("tags"));
    assert_eq!(
        plain.value_expr().map(|e| e.kind()),
        Some(SyntaxKind::ListExpr)
    );

    let matter = case(MATTER);
    let facts = matter.block(SyntaxKind::FactsBlock).expect("facts");
    let fact = &facts.entries()[0];
    assert_eq!(
        fact.key_ref().map(|r| r.text()).as_deref(),
        Some("frcp-rule12-example::time-barred")
    );
    assert_eq!(
        fact.key_text().as_deref(),
        Some("frcp-rule12-example::time-barred")
    );
    assert!(fact.key().is_none());
    assert_eq!(
        fact.value_ref().map(|r| r.text()).as_deref(),
        Some("frcp-rule12-example::not-barred")
    );
    assert!(fact.value_expr().is_none());
}
