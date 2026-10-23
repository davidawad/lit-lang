//! Accessors of rule-bearing declarations.
use super::{decl, lit, text, CONSTRUCTS, LINKS};
use crate::ast::{
    AstNode, CalendarDecl, CountingDecl, DeadlineDecl, DefaultDecl, ExceptionDecl, InstanceDecl,
    RuleDecl, TestDecl, TransitionDecl,
};
use crate::syntax_kind::SyntaxKind;

#[test]
fn rule_kinds_and_bodies() {
    let file = lit(
        "module m\nrule duty a { }\nrule power b { }\nrule liability c { state s: \"S\"\ndefault d when true => 1 }\n",
    );
    for (name, kind) in [
        ("a", SyntaxKind::KwDuty),
        ("b", SyntaxKind::KwPower),
        ("c", SyntaxKind::KwLiability),
    ] {
        let rule: RuleDecl = decl(&file, name);
        assert_eq!(rule.rule_kind().map(|t| t.kind()), Some(kind), "{name}");
    }
    let empty: RuleDecl = decl(&file, "a");
    assert!(empty.items().is_empty());
    let body: RuleDecl = decl(&file, "c");
    let kinds: Vec<SyntaxKind> = body
        .items()
        .iter()
        .map(crate::ast::items::Item::kind)
        .collect();
    assert_eq!(kinds, [SyntaxKind::StateDecl, SyntaxKind::DefaultDecl]);
}

#[test]
fn deadlines_expose_period_direction_trigger_and_counting() {
    let file = lit(CONSTRUCTS);
    let answer: DeadlineDecl = decl(&file, "answer-period");
    assert_eq!(text(answer.period()).as_deref(), Some("21.days"));
    assert_eq!(
        answer.direction().map(|t| t.kind()),
        Some(SyntaxKind::KwFrom)
    );
    let trigger = answer.trigger().expect("trigger");
    assert_eq!(trigger.kind(), SyntaxKind::CallExpr);
    assert_eq!(trigger.syntax().to_string(), "Served(complaint)");
    assert_eq!(
        answer.counted_by().map(|r| r.text()).as_deref(),
        Some("frcp6")
    );
    assert!(answer.options().is_empty());
    let old: DeadlineDecl = decl(&file, "old-answer-period");
    assert_eq!(old.direction().map(|t| t.kind()), Some(SyntaxKind::KwAfter));

    let before = lit("module m\ndeadline b: 3.days before Hearing(h) counted_by c\n");
    let b: DeadlineDecl = decl(&before, "b");
    assert_eq!(b.direction().map(|t| t.kind()), Some(SyntaxKind::KwBefore));
}

#[test]
fn counting_items_and_calendar_holidays() {
    let file = lit(CONSTRUCTS);
    let counting: CountingDecl = decl(&file, "itc-210-6");
    let kinds: Vec<_> = counting
        .items()
        .iter()
        .map(crate::ast::Field::keyword_kind)
        .collect();
    assert_eq!(
        kinds,
        [
            Some(SyntaxKind::KwAnchor),
            Some(SyntaxKind::KwShortPeriod),
            Some(SyntaxKind::KwServiceExtension)
        ]
    );

    let calendar: CalendarDecl = decl(&file, "rcfc-holidays");
    let holidays = calendar.holidays();
    let names: Vec<_> = holidays.iter().filter_map(|h| text(h.name())).collect();
    assert_eq!(
        names,
        [
            "inauguration-day",
            "thanksgiving",
            "memorial-day",
            "closure"
        ]
    );
    let inauguration = &holidays[0];
    let rule = inauguration.rule().expect("rule");
    assert_eq!(rule.callee().map(|r| r.text()).as_deref(), Some("every"));
    assert_eq!(rule.args().len(), 4);
    let effective = inauguration.effective().expect("effective");
    assert_eq!(
        effective.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwEffective)
    );
    assert!(effective.date().is_none());
    assert_eq!(
        inauguration.observed().map(|r| r.text()).as_deref(),
        Some("next_day")
    );
    assert!(holidays[1].observed().is_none());
    let closure = holidays[3]
        .effective()
        .and_then(|c| c.interval())
        .expect("interval");
    assert_eq!(text(closure.from()).as_deref(), Some("2018-12-05"));
    assert_eq!(text(closure.to()).as_deref(), Some("2018-12-06"));
}

#[test]
fn transitions_select_versions_by_arm() {
    let file = lit(CONSTRUCTS);
    let transition: TransitionDecl = decl(&file, "rule86");
    assert_eq!(
        transition.amends().map(|r| r.text()).as_deref(),
        Some("answer-period@2009-12-01")
    );
    let arms = transition.arms();
    assert_eq!(arms.len(), 2);
    assert!(!arms[0].is_otherwise());
    assert_eq!(
        arms[0]
            .condition()
            .map(|e| e.syntax().to_string())
            .as_deref(),
        Some("has_event(Answered(complaint))")
    );
    assert_eq!(
        arms[0].target().map(|r| r.text()).as_deref(),
        Some("answer-period@1993-12-01")
    );
    assert!(arms[1].is_otherwise());
    assert!(arms[1].condition().is_none());
    assert_eq!(
        arms[1].target().map(|r| r.text()).as_deref(),
        Some("answer-period@2009-12-01")
    );
}

#[test]
fn defaults_and_exceptions() {
    let file = lit(CONSTRUCTS);
    let default: DefaultDecl = decl(&file, "standing-order");
    assert_eq!(
        default.condition().map(|e| e.kind()),
        Some(SyntaxKind::CallExpr)
    );
    assert_eq!(
        default.value().map(|e| e.syntax().to_string()).as_deref(),
        Some("14.days")
    );

    let rebut: ExceptionDecl = decl(&file, "local-rule");
    assert_eq!(
        rebut.exception_kind().map(|t| t.kind()),
        Some(SyntaxKind::KwRebuts)
    );
    assert_eq!(
        rebut.target().map(|r| r.text()).as_deref(),
        Some("standing-order")
    );
    assert_eq!(
        rebut.condition().map(|e| e.kind()),
        Some(SyntaxKind::UnaryExpr)
    );
    assert_eq!(
        rebut.value().map(|e| e.syntax().to_string()).as_deref(),
        Some("7.days")
    );
    assert_eq!(text(rebut.priority()).as_deref(), Some("2"));

    let undercut: ExceptionDecl = decl(&file, "moot");
    assert_eq!(
        undercut.exception_kind().map(|t| t.kind()),
        Some(SyntaxKind::KwUndercuts)
    );
    assert_eq!(
        undercut.condition().map(|e| e.kind()),
        Some(SyntaxKind::BinaryExpr)
    );
    assert!(undercut.value().is_none());
    assert!(undercut.priority().is_none());
}

#[test]
fn tests_expose_givens_and_expects() {
    let file = lit(
        "module m\ntest \"served\" {\n  given Served(c) on 2025-01-02\n  expect due(d) == 2025-01-23\n}\n",
    );
    let test = TestDecl::cast(file.items()[0].syntax().clone()).expect("test");
    assert_eq!(text(test.title()).as_deref(), Some("\"served\""));
    let givens = test.givens();
    assert_eq!(givens.len(), 1);
    assert_eq!(
        givens[0].event().map(|e| e.syntax().to_string()).as_deref(),
        Some("Served(c)")
    );
    assert_eq!(text(givens[0].date()).as_deref(), Some("2025-01-02"));
    let expects = test.expects();
    assert_eq!(expects.len(), 1);
    assert_eq!(
        expects[0].expr().map(|e| e.kind()),
        Some(SyntaxKind::BinaryExpr)
    );
}

#[test]
fn instances_expose_origin_fields_and_patches() {
    let file = lit(LINKS);
    let instance: InstanceDecl = decl(&file, "cafc");
    assert_eq!(text(instance.base()).as_deref(), Some("cafc"));
    assert_eq!(text(instance.origin()).as_deref(), Some("cofc"));
    let kinds: Vec<_> = instance
        .fields()
        .iter()
        .map(crate::ast::Field::keyword_kind)
        .collect();
    assert_eq!(
        kinds,
        [
            Some(SyntaxKind::KwPack),
            Some(SyntaxKind::KwNote),
            Some(SyntaxKind::KwRemoveEdges),
            Some(SyntaxKind::KwProbabilities),
            Some(SyntaxKind::KwRoles),
            Some(SyntaxKind::KwPayoffTransform)
        ]
    );
    let patches = instance.patches();
    assert_eq!(patches.len(), 1);
    assert_eq!(text(patches[0].edge()).as_deref(), Some("affirm"));
    assert_eq!(patches[0].record().map(|r| r.fields().len()), Some(4));

    let empty: InstanceDecl = decl(&lit("module m\ninstance a@b { }\n"), "a");
    assert!(empty.fields().is_empty());
    assert!(empty.patches().is_empty());
}

#[test]
fn cites_collapse_whitespace_or_hold_a_hole() {
    let file = lit("module m\ndefault d cite FRCP   12(b)\t (6)\n  when true => 1\ndefault h cite ?(\"find it\") when true => 1\n");
    let spaced: DefaultDecl = decl(&file, "d");
    let cite = &spaced.cites()[0];
    assert_eq!(cite.text().as_deref(), Some("FRCP 12(b) (6)"));
    assert!(cite.hole().is_none());
    let holed: DefaultDecl = decl(&file, "h");
    let cite = &holed.cites()[0];
    assert!(cite.text().is_none());
    assert_eq!(
        cite.hole().and_then(|h| h.reason()).as_deref(),
        Some("find it")
    );
}
