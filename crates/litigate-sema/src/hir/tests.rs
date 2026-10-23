//! Contract tests: the HIR JSON shape is frozen by snapshot.
use super::*;
use std::collections::BTreeMap;

fn meta(id: &str) -> Metadata {
    Metadata {
        id: id.into(),
        label: "Motion to dismiss ruling".into(),
        cite: Some("FRCP 12(b)(6)".into()),
        effective: Interval {
            from: "2009-12-01".into(),
            to: None,
            known_at: "2025-01-01".into(),
        },
        authority: "FederalRule".into(),
        relations: vec![Relation::SubjectTo("frcp.rule6::computing-time".into())],
        span: None,
        historical: false,
        imported: false,
    }
}

fn edge(id: Option<&str>, to: &str, p: Authored<Rational>) -> Edge {
    Edge {
        id: id.map(Into::into),
        to: to.into(),
        label: "Granted".into(),
        actor: "office".into(),
        authority: None,
        probability: Some(p),
        hours: Some(Rational::integer(8)),
        cost: None,
        event: None,
        deadline: Some(EdgeDeadline {
            window: DeadlineWindow::Inline {
                length: 21,
                unit: DayUnit::Calendar,
            },
            extendable: Some(true),
            extension_authority: None,
            note: None,
        }),
        duration: None,
        valence: Some(Valence::Bad),
        court_listener_url: None,
        action_id: None,
        note: None,
        tags: vec!["dispositive".into()],
        attrs: BTreeMap::new(),
        sets: vec![],
        clears: vec![],
        requires: vec![],
        forbids: vec!["defaulted".into()],
        guard: None,
        span: None,
        holes: BTreeMap::new(),
    }
}

fn chance_node() -> Node {
    Node {
        meta: meta("mtd-ruling"),
        kind: NodeKind::Chance { discretion: true },
        group: Some("motions".into()),
        note: None,
        valence: None,
        court_listener_url: None,
        tags: vec!["fact".into()],
        attrs: BTreeMap::new(),
        actor: Some("office".into()),
        info: None,
        edges: vec![
            edge(
                Some("granted"),
                "dismissed",
                Authored::Known(Rational {
                    numerator: 1,
                    denominator: 3,
                }),
            ),
            edge(
                None,
                "answered",
                Authored::Hole(Some("no calibration yet".into())),
            ),
        ],
        holes: BTreeMap::new(),
    }
}

#[test]
fn node_json_shape_is_frozen() {
    insta::assert_json_snapshot!(Declaration::Node(chance_node()));
}

#[test]
fn hir_round_trips_through_json() -> Result<(), serde_json::Error> {
    let decl = Declaration::Node(chance_node());
    let back: Declaration = serde_json::from_str(&serde_json::to_string(&decl)?)?;
    assert_eq!(back, decl);
    let stake = Stake::Value(Money {
        minor_units: -200_000_000,
        currency: "usd".into(),
    });
    let back: Stake = serde_json::from_value(serde_json::to_value(&stake)?)?;
    assert_eq!(back, stake);
    Ok(())
}

#[test]
fn actor_role_uses_litgraph_spelling() -> Result<(), serde_json::Error> {
    assert_eq!(serde_json::to_string(&ActorRole::Protagonist)?, "\"self\"");
    assert_eq!(
        serde_json::to_string(&RollOn::LegalHoliday)?,
        "\"legalHoliday\""
    );
    let date: HolidayDate = serde_json::from_str(r#"{"rule":"fixed","month":6,"day":19}"#)?;
    assert_eq!(date, HolidayDate::Fixed { month: 6, day: 19 });
    Ok(())
}

/// Cite syntax is empty only with neither rules nor shorthands.
#[test]
fn cite_syntax_is_empty_without_rules_or_shorthands() {
    let rules = CiteSyntax {
        rules: vec![CiteRule {
            prefixes: vec!["FRCP".into()],
            heading: "Rule {}".into(),
            source: None,
        }],
        shorthands: Vec::new(),
    };
    let shorthands = CiteSyntax {
        rules: Vec::new(),
        shorthands: vec![CiteShorthand {
            prefix: "cfr".into(),
            code: "cfr".into(),
            title: 37,
            section_parts: 2,
            source: None,
        }],
    };
    assert!(CiteSyntax::default().is_empty());
    assert!(!rules.is_empty());
    assert!(!shorthands.is_empty());
}

#[test]
fn a_bound_past_the_calendar_has_no_date() {
    let far = Bound::Offset {
        from: DateRef::GrantOrderEntered,
        days: u64::MAX,
    };
    let near = Bound::Offset {
        from: DateRef::PrescribedDue,
        days: 1,
    };
    assert_eq!(
        Bound::Min(vec![near.clone()]).date("2025-01-01", "2025-02-01"),
        Some("2025-01-02".to_owned())
    );
    assert_eq!(
        Bound::Max(vec![near, far]).date("2025-01-01", "2025-02-01"),
        None
    );
}
