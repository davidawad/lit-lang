//! Accessors of `.lit` headers, items, and graph declarations.
use super::{decl, lit, text, CONSTRUCTS, LINKS};
use crate::ast::items::Item;
use crate::ast::{
    AstNode, AuthorityDecl, CalendarDecl, CountingDecl, DeadlineDecl, DefaultDecl, EnumDecl,
    EventDecl, ExceptionDecl, Field, GroupDecl, LinkDecl, NodeDecl, RuleDecl, TestDecl,
    TransitionDecl, TypeDecl, TypeExpr,
};
use crate::parse::parse_lit;
use crate::syntax_kind::SyntaxKind;

fn texts(tokens: Vec<super::Token>) -> Vec<String> {
    tokens.into_iter().map(|t| t.text().to_owned()).collect()
}

#[test]
fn file_header_exposes_module_imports_and_package() {
    let file = lit(CONSTRUCTS);
    let module = file.module().expect("module");
    assert_eq!(
        module.path().map(|r| r.text()).as_deref(),
        Some("frcp.rule6.computing-time")
    );
    let imports = file.imports();
    assert_eq!(imports.len(), 2);
    assert_eq!(
        imports[0].path().map(|r| r.text()).as_deref(),
        Some("core.calendar")
    );
    assert!(imports[0].names().is_empty());
    assert_eq!(
        imports[1].path().map(|r| r.text()).as_deref(),
        Some("frcp.rule12")
    );
    assert_eq!(texts(imports[1].names()), ["answer-period", "frcp6"]);
    let package = file.package().expect("package");
    assert_eq!(
        text(package.name()).as_deref(),
        Some("frcp-civil-procedure")
    );
    let fields = package.fields();
    let kinds: Vec<_> = fields.iter().map(Field::keyword_kind).collect();
    assert_eq!(
        kinds,
        [Some(SyntaxKind::KwForum), Some(SyntaxKind::KwAuthority)]
    );
    assert_eq!(
        fields[1].value_ref().map(|r| r.text()).as_deref(),
        Some("FederalRule")
    );
}

#[test]
fn file_without_header_has_no_module_or_package() {
    let file = lit("module m\nstart s\n");
    assert!(file.package().is_none());
    assert!(file.imports().is_empty());
    let recovered = parse_lit("start s\n");
    assert!(!recovered.is_ok());
    let file = recovered.lit_file().expect("lit root");
    assert!(file.module().is_none());
    let items = file.items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind(), SyntaxKind::StartDecl);
    assert_eq!(items[0].syntax().kind(), SyntaxKind::StartDecl);
}

#[test]
fn item_casts_only_declarations() {
    let file = lit(CONSTRUCTS);
    let kinds: Vec<SyntaxKind> = file.items().iter().map(Item::kind).collect();
    assert_eq!(kinds.first(), Some(&SyntaxKind::TypeDecl));
    assert_eq!(kinds.last(), Some(&SyntaxKind::TestDecl));
    assert_eq!(kinds.len(), 21);
    let module = file.module().expect("module");
    assert!(Item::cast(module.syntax().clone()).is_none());
}

#[test]
fn group_type_enum_event_and_authority_items() {
    let file = lit("module m\ngroup pleadings: \"Pleadings\"\n");
    let group: GroupDecl = decl(&file, "pleadings");
    assert_eq!(text(group.label()).as_deref(), Some("\"Pleadings\""));

    let file = lit(CONSTRUCTS);
    let dates: TypeDecl = decl(&file, "Dates");
    let list = dates.ty().expect("type");
    assert_eq!(text(list.name()).as_deref(), Some("List"));
    let args = list.args();
    assert_eq!(args.len(), 1);
    assert_eq!(text(args[0].name()).as_deref(), Some("Date"));
    assert!(args[0].args().is_empty());
    let weights: TypeDecl = decl(&file, "Weights");
    let map: Vec<TypeExpr> = weights.ty().expect("type").args();
    assert_eq!(map.len(), 2);
    assert_eq!(text(map[1].name()).as_deref(), Some("Dist"));
    assert_eq!(map[1].args().len(), 1);

    let outcome: EnumDecl = decl(&file, "Outcome");
    assert_eq!(texts(outcome.variants()), ["granted", "denied"]);

    let event: EventDecl = decl(&file, "Inaccessible");
    assert_eq!(
        event.docs().as_deref(),
        Some("A filing became due on an inaccessible day.")
    );
    let params = event.params();
    assert_eq!(params.len(), 1);
    assert_eq!(text(params[0].name()).as_deref(), Some("date"));
    assert_eq!(
        params[0].ty().and_then(|ty| text(ty.name())).as_deref(),
        Some("Date")
    );
    let bare: EventDecl = decl(&lit("module m\nevent Tick()\n"), "Tick");
    assert!(bare.params().is_empty());
    assert!(bare.docs().is_none());

    let authority: AuthorityDecl = file
        .items()
        .iter()
        .find_map(|item| AuthorityDecl::cast(item.syntax().clone()))
        .expect("authority");
    assert_eq!(
        texts(authority.levels()),
        ["Statute", "FederalRule", "LocalRule"]
    );
}

#[test]
fn clause_accessors_cover_every_clause_bearing_declaration() {
    let file = lit(CONSTRUCTS);
    let pending: NodeDecl = decl(&file, "pending");
    assert!(pending.cites().is_empty());
    assert!(pending.version_clauses().is_empty());
    let relations = pending.relations();
    assert_eq!(relations.len(), 2);

    let counting: CountingDecl = decl(&file, "itc-210-6");
    assert_eq!(counting.cites()[0].text().as_deref(), Some("19 CFR 210.6"));
    assert_eq!(counting.version_clauses().len(), 1);
    assert!(counting.relations().is_empty());

    let calendar: CalendarDecl = decl(&file, "rcfc-holidays");
    assert_eq!(calendar.cites()[0].text().as_deref(), Some("RCFC 6(a)(6)"));
    let known_at = &calendar.version_clauses()[0];
    assert_eq!(
        known_at.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwKnownAt)
    );
    assert_eq!(text(known_at.date()).as_deref(), Some("2025-01-01"));
    assert!(known_at.interval().is_none());
    assert!(calendar.relations().is_empty());

    let old: DeadlineDecl = decl(&file, "old-answer-period");
    assert!(old.cites().is_empty());
    assert!(old.version_clauses().is_empty());
    let supersedes = &old.relations()[0];
    assert_eq!(
        supersedes.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwSupersedes)
    );

    let transition: TransitionDecl = decl(&file, "rule86");
    assert_eq!(transition.cites()[0].text().as_deref(), Some("FRCP 86(a)"));
    assert!(transition.version_clauses().is_empty());
    assert!(transition.relations().is_empty());

    let default: DefaultDecl = decl(&file, "standing-order");
    assert!(default.cites().is_empty());
    assert!(default.version_clauses().is_empty());
    assert!(default.relations().is_empty());

    let exception: ExceptionDecl = decl(&file, "local-rule");
    assert_eq!(exception.cites()[0].text().as_deref(), Some("LR 7.1"));
    assert!(exception.version_clauses().is_empty());
    assert!(exception.relations().is_empty());

    let test = file
        .items()
        .iter()
        .find_map(|item| TestDecl::cast(item.syntax().clone()))
        .expect("test");
    assert!(test.cites().is_empty());
    assert!(test.version_clauses().is_empty());
    assert!(test.relations().is_empty());

    let rule_file =
        lit("module m\nrule power r known_at 2024-01-01 notwithstanding o cite X 2 { }\n");
    let rule: RuleDecl = decl(&rule_file, "r");
    assert_eq!(rule.cites()[0].text().as_deref(), Some("X 2"));
    assert_eq!(rule.version_clauses().len(), 1);
    assert_eq!(
        rule.relations()[0].target().map(|r| r.text()).as_deref(),
        Some("o")
    );
}

#[test]
fn relations_expose_targets_and_delegates() {
    let file = lit(CONSTRUCTS);
    let pending: NodeDecl = decl(&file, "pending");
    let relations = pending.relations();
    assert_eq!(
        relations[0].keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwSubjectTo)
    );
    assert_eq!(
        relations[0].target().map(|r| r.text()).as_deref(),
        Some("standing-order")
    );
    assert!(relations[0].delegates().is_none());
    let delegable = &relations[1];
    assert_eq!(
        delegable.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwDelegableTo)
    );
    assert!(delegable.target().is_none());
    let delegates = delegable.delegates().expect("delegates");
    let items: Vec<String> = delegates
        .items()
        .iter()
        .map(super::super::Ref::text)
        .collect();
    assert_eq!(items, ["LocalRule"]);
    assert!(delegates.strings().is_empty());
}

#[test]
fn node_headers_and_bodies() {
    let file = lit(CONSTRUCTS);
    let pending: NodeDecl = decl(&file, "pending");
    assert_eq!(
        pending.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwState)
    );
    assert_eq!(text(pending.label()).as_deref(), Some("\"Pending\""));
    assert!(pending.clause_fields().is_empty());
    assert!(pending.by().is_none());
    assert!(!pending.discretion());
    assert!(pending.outcome().is_none());
    assert!(pending.body().is_some());
    assert!(pending.fields().is_empty());
    assert_eq!(pending.edges().len(), 2);

    let done: NodeDecl = decl(&file, "done");
    assert_eq!(
        done.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwTerminal)
    );
    assert_eq!(done.clause_fields().len(), 1);
    let outcome = done.outcome().expect("outcome");
    assert_eq!(outcome.items()[0].text(), "judgment");
    let body: Vec<Option<SyntaxKind>> = done.fields().iter().map(Field::keyword_kind).collect();
    assert_eq!(
        body,
        [Some(SyntaxKind::KwPayoff), Some(SyntaxKind::KwPayoffByFlag)]
    );
    assert!(done.edges().is_empty());

    let bare: NodeDecl = decl(&lit("module m\ndecision d: \"D\" by Plaintiff\n"), "d");
    assert_eq!(
        bare.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwDecision)
    );
    assert_eq!(bare.by().map(|r| r.text()).as_deref(), Some("Plaintiff"));
    assert!(bare.body().is_none());
    assert!(bare.fields().is_empty());
    assert!(bare.edges().is_empty());
}

#[test]
fn edges_expose_clauses_and_fields() {
    let file = lit(CONSTRUCTS);
    let pending: NodeDecl = decl(&file, "pending");
    let edges = pending.edges();
    let ruling = &edges[0];
    assert_eq!(text(ruling.id()).as_deref(), Some("ruling"));
    assert_eq!(text(ruling.label()).as_deref(), Some("\"Ruling\""));
    assert_eq!(ruling.clause_fields().len(), 1);
    assert!(ruling.by().is_none());
    assert_eq!(
        ruling.probability().map(|e| e.kind()),
        Some(SyntaxKind::ParenExpr)
    );
    assert_eq!(ruling.target().map(|r| r.text()).as_deref(), Some("done"));
    let fields = ruling.fields();
    assert_eq!(
        fields[0].value_expr().map(|e| e.kind()),
        Some(SyntaxKind::HoleExpr)
    );
    let duration = &fields[1];
    assert_eq!(duration.keyword_kind(), Some(SyntaxKind::KwDuration));
    let bounds: Vec<Option<String>> = duration
        .options()
        .iter()
        .map(|f| {
            f.value_expr()
                .and_then(|e| text(super::super::Literal::cast(e.syntax().clone())?.token()))
        })
        .collect();
    assert_eq!(
        bounds,
        [
            Some("1.days".into()),
            Some("5.days".into()),
            Some("30.days".into())
        ]
    );

    let stay = &edges[1];
    assert_eq!(stay.by().map(|r| r.text()).as_deref(), Some("Court"));
    assert_eq!(
        stay.probability().map(|e| e.kind()),
        Some(SyntaxKind::Ratio)
    );
    let deadline = &stay.fields()[0];
    assert_eq!(deadline.keyword_kind(), Some(SyntaxKind::KwDeadline));
    assert_eq!(deadline.options().len(), 2);

    let bare = &lit("module m\nstate s: \"S\" { go \"Go\" -> t }\n").items()[0];
    let edge = &NodeDecl::cast(bare.syntax().clone()).expect("node").edges()[0];
    assert!(edge.probability().is_none());
    assert!(edge.fields().is_empty());
    assert!(edge.clause_fields().is_empty());
}

#[test]
fn links_expose_endpoints_and_clauses() {
    let file = lit(LINKS);
    let links: Vec<LinkDecl> = file
        .items()
        .iter()
        .filter_map(|item| LinkDecl::cast(item.syntax().clone()))
        .collect();
    assert_eq!(links.len(), 2);
    let appeal = &links[0];
    assert_eq!(text(appeal.id()).as_deref(), Some("appeal"));
    assert_eq!(
        text(appeal.label()).as_deref(),
        Some("\"Appeal to the Federal Circuit\"")
    );
    assert_eq!(
        appeal.from().map(|r| r.text()).as_deref(),
        Some("cofc::judgment")
    );
    assert_eq!(
        appeal.to().map(|r| r.text()).as_deref(),
        Some("cafc::entry")
    );
    assert_eq!(appeal.clause_fields().len(), 2);
    assert_eq!(appeal.by().map(|r| r.text()).as_deref(), Some("self"));
    assert_eq!(
        appeal.probability().map(|e| e.kind()),
        Some(SyntaxKind::Ratio)
    );
    let fields = appeal.fields();
    assert_eq!(fields[0].keyword_kind(), Some(SyntaxKind::KwReplaces));
    assert_eq!(fields[0].id_list().map(|l| l.items().len()), Some(1));
    assert_eq!(
        fields[1].value_ref().map(|r| r.text()).as_deref(),
        Some("cafc::notice-period")
    );

    let remand = &links[1];
    assert!(remand.id().is_none());
    assert_eq!(
        remand.from().map(|r| r.text()).as_deref(),
        Some("cafc@cofc::mandate")
    );
    assert_eq!(
        remand.to().map(|r| r.text()).as_deref(),
        Some("cofc::pending")
    );
    assert!(remand.by().is_none());
    assert!(remand.probability().is_none());
    assert!(remand.fields().is_empty());
}

#[test]
fn fields_view_values_by_shape() {
    let file = lit(CONSTRUCTS);
    let counting: CountingDecl = decl(&file, "itc-210-6");
    let items = counting.items();
    assert_eq!(items.len(), 3);
    let anchor = &items[0];
    assert_eq!(text(anchor.keyword()).as_deref(), Some("anchor"));
    assert_eq!(anchor.value().map(|n| n.kind()), Some(SyntaxKind::Ref));
    assert!(anchor.record().is_none());
    assert!(anchor.cite().is_none());
    assert!(anchor.id_list().is_none());
    assert!(anchor.value_expr().is_none());
    let extension = &items[2];
    assert_eq!(
        extension.keyword_kind(),
        Some(SyntaxKind::KwServiceExtension)
    );
    assert_eq!(extension.record().map(|r| r.fields().len()), Some(2));
    assert_eq!(
        extension.cite().and_then(|c| c.text()).as_deref(),
        Some("19 CFR 210.6(b)")
    );
    assert!(extension.value_ref().is_none());
    assert!(extension.options().is_empty());

    let file = lit("module m\nstate s: \"S\" { tags [] }\n");
    let node: NodeDecl = decl(&file, "s");
    let tags = &node.fields()[0];
    assert!(tags.id_list().is_some_and(|list| list.items().is_empty()));

    let flag = lit("module m\ncounting c { allow_hours }\n");
    let counting: CountingDecl = decl(&flag, "c");
    let allow = &counting.items()[0];
    assert_eq!(allow.keyword_kind(), Some(SyntaxKind::KwAllowHours));
    assert!(allow.value().is_none());
}

#[test]
fn id_lists_decode_string_entries() {
    let file = lit("module m\nstate s: \"S\" { e \"E\" -> t { replaces [old, \"anon\\n1\"] } }\n");
    let node: NodeDecl = decl(&file, "s");
    let replaces = node.edges()[0].fields()[0].id_list().expect("replaces");
    assert_eq!(replaces.items().len(), 1);
    assert_eq!(replaces.strings(), ["anon\n1"]);
}
