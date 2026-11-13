#[test]
fn the_worked_example_forks_at_the_motion_ruling() {
    with_matter("", ACME, |session, m| {
        let t = build(session, &m, m.case.as_ref().unwrap());
        assert_eq!(t["worlds"][1]["choice"]["kind"], "chance");
        assert_eq!(t["worlds"][1]["choice"]["probability"], json!(0.5));
        assert_eq!(t["rows"][2]["status"]["state"], "met");
        assert_eq!(t["edges"][0]["kind"], "sequence");
    });
}

#[test]
fn instances_without_a_version_or_declaration_are_left_out() {
    with_matter("", ACME, |session, mut m| {
        let case = m.case.clone().unwrap();
        let mut superseded = m.counted[0].clone();
        superseded.status = Status::Superseded;
        superseded.closed_by = Some(1);
        let mut unclosed = superseded.clone();
        unclosed.closed_by = None;
        m.counted[0].version = "nope@2000-01-01".to_owned();
        m.counted.extend([superseded, unclosed]);
        let mut uncounted = m.counted[0].clone();
        uncounted.deadline = format!("{PACK}::answer-period");
        uncounted.due = None;
        uncounted.unknown = Some("no counting rule".to_owned());
        uncounted.status = Status::Discharged;
        uncounted.closed_by = Some(1);
        let mut undeclared = uncounted.clone();
        undeclared.deadline = format!("{PACK}::nope");
        undeclared.status = Status::Open;
        undeclared.closed_by = None;
        let mut unjudged = uncounted.clone();
        unjudged.status = Status::Unjudged;
        m.counted.extend([uncounted, undeclared, unjudged]);
        let t = build(session, &m, &case);
        let deadlines: Vec<&serde_json::Value> = t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "deadline")
            .collect();
        assert_eq!(deadlines.len(), 4, "{t:#}");
        assert_eq!(
            deadlines[2]["status"],
            json!({"state": "met", "reason": "discharged by MovedToDismiss(complaint) on 2025-03-20: no counting rule"})
        );
        // An act the engine cannot judge says why itself.
        assert_eq!(
            deadlines[3]["status"],
            json!({"state": "unknown", "reason": "timeliness unknown: Served due date not recorded; acted by MovedToDismiss(complaint) on 2025-03-20"})
        );
        let supersedes = t["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "supersedes")
            .count();
        assert_eq!(supersedes, 1);
    });
}

#[test]
fn events_fall_back_to_the_primary_layer() {
    with_matter("", ACME, |session, mut m| {
        let mut case = m.case.clone().unwrap();
        // No replay step for the first event, an unknown stage for the second.
        m.replay.remove(0);
        m.replay[0].from = format!("{PACK}::nope");
        m.counted.clear();
        let t = build(session, &m, &case);
        assert_eq!(t["rows"][0]["provenance"], json!([PACK]));
        // No used package: no layer, and no posture to fork at.
        m.packages.clear();
        case.start = None;
        let t = build(session, &m, &case);
        assert_eq!(t["rows"][0]["provenance"], json!([""]));
        assert_eq!(t["worlds"].as_array().unwrap().len(), 1);
        case.start = Some(format!("{PACK}::nope"));
        let t = build(session, &m, &case);
        assert_eq!(t["worlds"].as_array().unwrap().len(), 1);
    });
}

#[test]
fn a_matter_at_the_start_forks_with_probabilities() {
    let text = ACME
        .replace(
            "  served-1: 2025-03-05 Served(complaint, Personal) on widget\n",
            "",
        )
        .replace(
            "  moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n",
            "",
        )
        .replace("facts { time-barred: not-barred }\n", "");
    with_matter("", &text, |session, m| {
        let case = m.case.clone().unwrap();
        assert_eq!(case.start, None);
        let t = build(session, &m, &case);
        assert_eq!(t["worlds"][1]["choice"]["probability"], json!(0.1));
        assert_eq!(t["worlds"][2]["choice"]["probability"], json!(0.9));
    });
}

#[test]
fn a_decision_forks_into_elections_with_unknown_governing_deadlines() {
    let text = ACME
        .replace(
            "  served-1: 2025-03-05 Served(complaint, Personal) on widget\n",
            "",
        )
        .replace(
            "  moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n",
            "",
        )
        .replace(
            "perspective widget",
            "start frcp-rule12-example::respond\nperspective widget",
        );
    with_matter("", &text, |session, m| {
        let mut case = m.case.clone().unwrap();
        assert_eq!(case.start.as_deref(), Some("frcp-rule12-example::respond"));
        let t = build(session, &m, &case);
        let keys: Vec<&str> = t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            keys,
            [
                "frcp.rule12.answer-period@event:Served",
                "fork:frcp.rule12.respond@start",
                "event:Answered@fork:frcp.rule12.respond@start",
                "event:MovedToDismiss@fork:frcp.rule12.respond@start",
                "event:default@fork:frcp.rule12.respond@start",
            ]
        );
        assert_eq!(
            rows(&t),
            [
                ("w0", "deadline", "unknown"),
                ("w0", "fork", "unknown"),
                ("w1", "event", "none"),
                ("w2", "event", "none"),
                ("w3", "event", "none"),
            ]
            .map(|(a, b, c)| (a.to_owned(), b.to_owned(), c.to_owned()))
        );
        assert_eq!(t["worlds"][1]["choice"]["kind"], "election");
        assert_eq!(
            t["rows"][0]["status"]["reason"],
            "counted from Served(complaint), which no recorded event dates as of 2025-06-01"
        );
        // A removed option forks no world.
        case.removals.push(hir::Removal {
            edge: format!("{PACK}::default"),
            date: "2025-04-01".to_owned(),
            event: 0,
            authority: "FRCP 12".to_owned(),
        });
        let t = build(session, &m, &case);
        assert_eq!(t["worlds"].as_array().unwrap().len(), 3);
    });
}

#[test]
fn forks_read_the_actor_and_skip_what_does_not_branch() {
    with_matter("", ACME, |session, m| {
        let case = m.case.clone().unwrap();
        let all = session.hir();
        let texts = session.sources();
        let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
        assert!(b.node("nope").is_none());
        assert!(b.node("nope::x").is_none());
        assert!(b.node(&format!("{PACK}::nope")).is_none());
        assert!(b.deadline("nope").is_none());
        assert!(b.deadline("nope::x").is_none());
        assert!(b.deadline(&format!("{PACK}::nope")).is_none());
        let (pkg, terminal) = b.node(&format!("{PACK}::dismissed")).unwrap();
        fork(&mut b, &case, pkg, terminal, &[], None);
        assert!(b.rows.is_empty());
        let (_, respond) = b.node(&format!("{PACK}::respond")).unwrap();
        let mut state = respond.clone();
        state.kind = NodeKind::State;
        state.edges[0].actor = "court".to_owned();
        state.edges[1].to = "nope::x".to_owned();
        // An edge back to the node itself is not an option.
        state.edges[2].to = "respond".to_owned();
        fork(&mut b, &case, pkg, &state, &[], None);
        let kinds: Vec<ChoiceKind> = b.worlds[1..]
            .iter()
            .map(|w| w.choice.as_ref().unwrap().kind)
            .collect();
        assert_eq!(
            kinds,
            [ChoiceKind::HypotheticalDecision, ChoiceKind::Election]
        );
        // A deadline that does not resolve has no row.
        let mut odd = respond.clone();
        odd.edges[0].deadline = Some(hir::EdgeDeadline {
            window: DeadlineWindow::Declared {
                deadline: "nope".to_owned(),
            },
            ..respond.edges[0].deadline.clone().unwrap()
        });
        let before = b.rows.len();
        b.governing(
            0,
            pkg,
            &odd,
            &mut vec![format!("{PACK}::answer-period")],
            None,
        );
        assert_eq!(b.rows.len(), before);
    });
}

#[test]
fn keys_derive_from_causes() {
    with_matter("", ACME, |session, m| {
        let case = m.case.clone().unwrap();
        let t = build(session, &m, &case);
        let keys: Vec<&str> = t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            keys,
            [
                "event:Served@served-1",
                "event:MovedToDismiss@moved-to-dismiss-1",
                "frcp.rule12.answer-period@event:Served@served-1",
                "fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
                "event:granted@fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
                "event:denied@fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
            ]
        );
        // A named option without an event uses its stable edge ID.
        let all = session.hir();
        let texts = session.sources();
        let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
        let (pkg, answered) = b.node(&format!("{PACK}::answered")).unwrap();
        fork(&mut b, &case, pkg, answered, &[], None);
        assert_eq!(
            b.rows[1].key,
            "event:edge-answered-judgment-entered@fork:frcp.rule12.answered@start"
        );
        let mut unnamed = answered.clone();
        unnamed.edges[0].event = None;
        unnamed.edges[0].id = None;
        let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
        fork(&mut b, &case, pkg, &unnamed, &[], None);
        assert_eq!(
            b.rows[1].key,
            "event:judgment-entered@fork:frcp.rule12.answered@start"
        );
        let mut served = case.events[0].clone();
        served.id = None;
        served.docket = Some(2);
        assert_eq!(event_key(&all, &served), "event:Served@2025-03-05/2");
        served.docket = None;
        assert_eq!(event_key(&all, &served), "event:Served@2025-03-05");
        let mut electing = all[0].clone();
        for module in &mut electing.modules {
            for declaration in &mut module.declarations {
                if let Declaration::Event(event) = declaration {
                    if event.name == "Served" {
                        event.effect = Some(hir::EventEffect::Elects { param: 0 });
                    }
                }
            }
        }
        assert_eq!(
            event_key(&[&electing], &served),
            "event:Served@2025-03-05/complaint"
        );
        // An explicit ID overrides the case event's own key.
        served.id = Some("service".to_owned());
        assert_eq!(event_key(&all, &served), "event:Served@service");
        assert_eq!(decl_key(all[0], "nope"), "frcp-rule12-example.nope");
    });
}

#[test]
fn a_replacement_is_the_restart_or_the_absorbing_instance() {
    with_matter("", ACME, |_, m| {
        let at = |deadline: &str, trigger, status, closed_by, floor: Option<&str>| {
            let mut s = m.counted[0].clone();
            s.deadline = deadline.to_owned();
            s.trigger_event = trigger;
            s.status = status;
            s.closed_by = closed_by;
            s.floor = floor.map(str::to_owned);
            s
        };
        let counted = [
            at("p::a", 0, Status::Merged, Some(1), None),
            // Triggered by the absorbing event, but absorbs nothing:
            // the same deadline, then one with no floor.
            at("p::a", 1, Status::Open, None, None),
            at("p::c", 1, Status::Open, None, None),
            at("p::b", 1, Status::Open, None, Some("2025-07-02")),
            at("p::a", 2, Status::Superseded, Some(3), None),
            at("p::c", 3, Status::Open, None, None),
            at("p::a", 3, Status::Open, None, None),
            at("p::a", 4, Status::Discharged, Some(5), None),
        ];
        assert_eq!(replacement(&counted, 0), Some(3));
        assert_eq!(replacement(&counted, 1), None);
        assert_eq!(replacement(&counted, 4), Some(6));
        assert_eq!(replacement(&counted, 7), None);
    });
}

#[test]
fn deadline_labels_read_like_the_source() {
    let session = testkit::example_with("");
    let all = session.hir();
    let decl = hirx::deadlines(all[0], "answer-period").next().unwrap();
    assert_eq!(
        deadline_label(decl),
        "answer-period: 21 days after Served(complaint)"
    );
    let mut d = decl.clone();
    d.amount = 1;
    d.direction = hir::Direction::Backward;
    let units = [
        (hir::DayUnit::Calendar, "day"),
        (hir::DayUnit::Court, "court day"),
        (hir::DayUnit::Business, "business day"),
        (hir::DayUnit::Week, "week"),
        (hir::DayUnit::Month, "month"),
        (hir::DayUnit::Year, "year"),
    ];
    for (unit, name) in units {
        d.unit = unit;
        assert_eq!(
            deadline_label(&d),
            format!("answer-period: 1 {name} before Served(complaint)")
        );
    }
    d.designated = Some(hir::Designated {
        event: "Served".to_owned(),
        param: 0,
        name: "due".to_owned(),
        interpretation: None,
    });
    assert_eq!(
        deadline_label(&d),
        "answer-period: the date Served(complaint) designates, at least 1 year before it"
    );
}

#[test]
fn diagnostics_keep_errors_and_the_file_s_own_warnings() {
    let session = testkit::example_with("");
    let all = session.hir();
    let texts = session.sources();
    let b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
    let at = |file: &str, severity| {
        let mut d = Diagnostic::error(
            DiagnosticCode::InvalidStart,
            Span {
                file: file.into(),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 7,
            },
            format!("{severity:?}"),
        );
        d.severity = severity;
        d
    };
    let given = [
        at("other.lit", Severity::Error),
        at("other.lit", Severity::Warning),
        at("lit/examples/rule12.lit", Severity::Warning),
        at("lit/examples/rule12.lit", Severity::Info),
        at("lit/examples/rule12.lit", Severity::Hint),
    ];
    let kept: Vec<(TimelineSeverity, usize)> = diagnostics(&b, "lit/examples/rule12.lit", &given)
        .into_iter()
        .map(|d| (d.severity, d.source.unwrap().end))
        .collect();
    assert_eq!(
        kept,
        [
            (TimelineSeverity::Note, 6),
            (TimelineSeverity::Note, 6),
            (TimelineSeverity::Warning, 6),
            (TimelineSeverity::Error, 0),
        ]
    );
}
