//! Accessors of `.case` matters.
use super::{case, text, MATTER};
use crate::ast::{AstNode, CaseBlock, RecordField, VersionReq, VersionText};
use crate::parse::parse_case;
use crate::syntax_kind::SyntaxKind;
use rowan::{GreenNodeBuilder, SyntaxNode};

/// The required `.case` header items.
const HEAD: &str = "case c\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"S\"\n";

/// Parses the required header followed by `rest`.
fn matter(rest: &str) -> crate::ast::CaseFile {
    case(&format!("{HEAD}{rest}"))
}

fn version(op: &str, version: &str) -> VersionText {
    VersionText {
        op: op.to_owned(),
        version: version.to_owned(),
    }
}

#[test]
fn header_items_of_a_full_matter() {
    let file = case(MATTER);
    assert_eq!(text(file.name()).as_deref(), Some("acme-appeal"));
    let pins = file.uses().expect("uses").items();
    let packages: Vec<_> = pins.iter().filter_map(|pin| text(pin.package())).collect();
    assert_eq!(packages, ["frcp-rule12-example", "cafc"]);
    let reqs: Vec<_> = pins
        .iter()
        .filter_map(|pin| pin.requirement().and_then(|r| r.text()))
        .collect();
    assert_eq!(
        reqs,
        [version("^", "0.1.0"), version("~", "2024.12.0-rc.1")]
    );
    assert_eq!(text(file.as_of()).as_deref(), Some("2025-06-01"));
    assert_eq!(text(file.known_at()).as_deref(), Some("2025-06-15"));
    assert_eq!(
        text(file.summary()).as_deref(),
        Some("\"Appeal posture with every optional case item.\"")
    );
    assert_eq!(
        text(file.notes()).as_deref(),
        Some("\"Fixture for SPEC §3.2.\"")
    );
    let sources = file.sources();
    assert_eq!(sources.len(), 2);
    assert_eq!(text(sources[0].title()).as_deref(), Some("\"Docket\""));
    assert!(sources[0].fields().is_empty());
    let url = &sources[1].fields()[0];
    assert_eq!(url.keyword_kind(), Some(SyntaxKind::KwUrl));
    let start = file.start().expect("start");
    assert_eq!(start.keyword().map(|t| t.kind()), Some(SyntaxKind::KwStart));
    assert_eq!(
        start.target().map(|r| r.text()).as_deref(),
        Some("frcp-rule12-example::mtd-ruling")
    );
}

#[test]
fn every_version_operator_is_recognized() {
    let file = case("case c\nuses a >=1.0.0, b =2.3.4\nas_of 2025-01-01\nsummary \"S\"\n");
    let reqs: Vec<_> = file
        .uses()
        .expect("uses")
        .items()
        .iter()
        .filter_map(|pin| pin.requirement().and_then(|r| r.text()))
        .collect();
    assert_eq!(reqs, [version(">=", "1.0.0"), version("=", "2.3.4")]);
}

/// The requirement of the single pin of a recovered `uses p <req>` header.
fn recovered_requirement(req: &str) -> Option<VersionReq> {
    let parsed = parse_case(&format!(
        "case c\nuses p {req}\nas_of 2025-01-01\nsummary \"S\"\n"
    ));
    assert!(!parsed.is_ok());
    let file = parsed.case_file().expect("case root");
    file.uses().expect("uses").items()[0].requirement()
}

#[test]
fn incomplete_requirements_have_no_version_text() {
    let missing_op = recovered_requirement("1.0.0").expect("requirement node");
    assert!(missing_op.text().is_none());
    let missing_version = recovered_requirement("^").expect("requirement node");
    assert_eq!(missing_version.text(), Some(version("^", "")));

    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind::VersionReq.into());
    builder.token(SyntaxKind::Caret.into(), "^");
    builder.finish_node();
    let root = SyntaxNode::new_root(builder.finish());
    let without_version = VersionReq::cast(root).expect("version req");
    assert!(without_version.text().is_none());
}

#[test]
fn minimal_matter_has_no_optional_items() {
    let recovered = parse_case("case c\n");
    assert!(!recovered.is_ok());
    let bare = recovered.case_file().expect("case root");
    assert!(bare.uses().is_none());
    assert!(bare.as_of().is_none());
    assert!(bare.summary().is_none());
    let file = matter("");
    assert!(file.known_at().is_none());
    assert!(file.notes().is_none());
    assert!(file.sources().is_empty());
    assert!(file.start().is_none());
    assert!(file.perspective().is_none());
    assert!(file.parties().is_empty());
    assert!(file.events().is_empty());
    assert!(file.blocks().is_empty());
    assert!(file.block(SyntaxKind::FactsBlock).is_none());
}

#[test]
fn perspective_by_roles_or_by_party() {
    let file = case(MATTER);
    let perspective = file.perspective().expect("perspective");
    assert!(perspective.party().is_none());
    let roles: Vec<_> = perspective
        .roles()
        .expect("roles")
        .fields()
        .iter()
        .filter_map(RecordField::key_text)
        .collect();
    assert_eq!(roles, ["Plaintiff", "Defendant"]);

    let file = matter("perspective acme\n");
    let perspective = file.perspective().expect("perspective");
    assert_eq!(
        perspective.party().map(|r| r.text()).as_deref(),
        Some("acme")
    );
    assert!(perspective.roles().is_none());
}

#[test]
fn parties_and_events() {
    let file = case(MATTER);
    let parties = file.parties();
    assert_eq!(parties.len(), 1);
    assert_eq!(text(parties[0].name()).as_deref(), Some("acme"));
    assert_eq!(
        parties[0].actor().map(|r| r.text()).as_deref(),
        Some("Plaintiff")
    );
    assert_eq!(
        parties[0].entity().map(|r| r.text()).as_deref(),
        Some("Corporation")
    );

    let event = &file.events()[0];
    assert_eq!(text(event.date()).as_deref(), Some("2025-03-05"));
    let call = event.call().expect("call");
    assert_eq!(call.callee().map(|r| r.text()).as_deref(), Some("Served"));
    assert_eq!(event.mode().map(|t| t.kind()), Some(SyntaxKind::KwOn));
    assert_eq!(event.party().map(|r| r.text()).as_deref(), Some("acme"));
    assert_eq!(
        event.cite().and_then(|c| c.text()).as_deref(),
        Some("FRCP 4(h)")
    );

    let file = matter("events { 2025-01-01 Answered(x) by acme }\n");
    let event = &file.events()[0];
    assert_eq!(event.mode().map(|t| t.kind()), Some(SyntaxKind::KwBy));
    assert!(event.cite().is_none());
    assert!(event.docket().is_none());

    let file = matter("events { 2025-01-01 docket 12 Answered(x) by acme }\n");
    let event = &file.events()[0];
    assert_eq!(text(event.docket()).as_deref(), Some("12"));
    assert_eq!(text(event.date()).as_deref(), Some("2025-01-01"));
    assert_eq!(
        event
            .call()
            .and_then(|c| c.callee())
            .map(|r| r.text())
            .as_deref(),
        Some("Answered")
    );
}

#[test]
fn blocks_expose_keywords_records_and_observations() {
    let file = case(MATTER);
    let keywords: Vec<_> = file
        .blocks()
        .iter()
        .filter_map(|block| block.keyword().map(|t| t.kind()))
        .collect();
    assert_eq!(
        keywords,
        [
            SyntaxKind::KwFacts,
            SyntaxKind::KwStakes,
            SyntaxKind::KwAnalysis
        ]
    );
    let analysis = file.block(SyntaxKind::AnalysisBlock).expect("analysis");
    let keys: Vec<_> = analysis
        .entries()
        .iter()
        .filter_map(RecordField::key_text)
        .collect();
    assert_eq!(keys, ["objective", "horizon", "mixed", "seed", "x"]);
    assert!(analysis.observations().is_empty());

    let file = matter("beliefs { prior n: { e: 6, f: 4 }\n observe n: { e: 3, f: 1 } }\n");
    let beliefs: CaseBlock = file.block(SyntaxKind::BeliefsBlock).expect("beliefs");
    assert_eq!(
        beliefs.keyword().map(|t| t.kind()),
        Some(SyntaxKind::KwBeliefs)
    );
    assert!(beliefs.record().is_none());
    assert!(beliefs.entries().is_empty());
    let observation = &beliefs.observations()[0];
    assert_eq!(observation.node().map(|r| r.text()).as_deref(), Some("n"));
    assert_eq!(observation.counts().map(|r| r.fields().len()), Some(2));
    let prior = &beliefs.priors()[0];
    assert_eq!(prior.node().map(|r| r.text()).as_deref(), Some("n"));
    assert_eq!(prior.counts().map(|r| r.fields().len()), Some(2));
    assert_eq!(beliefs.observations().len(), 1);
}

#[test]
fn docket_is_a_keyword_only_before_a_number() {
    // `docket` not followed by a number is the event call's callee.
    let file = matter("events { 2025-01-01 docket(x) by acme }\n");
    let event = &file.events()[0];
    assert!(event.docket().is_none());
    let callee = event.call().and_then(|c| c.callee()).map(|r| r.text());
    assert_eq!(callee.as_deref(), Some("docket"));
    // A number missing after `docket` leaves no docket and no call.
    let parsed = parse_case(&format!(
        "{HEAD}events {{ 2025-01-01 docket Answered(x) by acme }}\n"
    ));
    assert!(!parsed.is_ok());
    let file = parsed.case_file().expect("case root");
    assert!(file.events()[0].docket().is_none());
}

#[test]
fn an_event_may_declare_an_id_before_its_date() {
    let file = matter("events {\n  answer: 2025-01-01 docket 3 Answered(x) by acme\n  2025-01-02 Moved() by acme\n}\n");
    let events = file.events();
    assert_eq!(text(events[0].id()).as_deref(), Some("answer"));
    assert_eq!(text(events[0].date()).as_deref(), Some("2025-01-01"));
    assert_eq!(text(events[0].docket()).as_deref(), Some("3"));
    assert_eq!(events[0].party().map(|r| r.text()).as_deref(), Some("acme"));
    assert!(events[1].id().is_none());
    // An id must be followed by the event's date.
    let parsed = parse_case(&format!("{HEAD}events {{ answer: Answered(x) by acme }}\n"));
    assert!(!parsed.is_ok());
    let file = parsed.case_file().expect("case root");
    assert_eq!(text(file.events()[0].id()).as_deref(), Some("answer"));
    assert!(file.events()[0].date().is_none());
}
