const PACK: &str = "frcp-rule12-example";

#[test]
fn layer_classes_follow_the_package_authority() {
    assert_eq!(layer_class("LocalRule"), LayerClass::LocalRule);
    assert_eq!(layer_class("StandingOrder"), LayerClass::StandingOrder);
    assert_eq!(layer_class("CaseOrder"), LayerClass::CaseOrder);
    assert_eq!(layer_class("Regulation"), LayerClass::BaseRule);
}

#[test]
fn every_obligation_end_reads_as_a_status() {
    let closed = Some(("Answered(complaint)", "2025-03-20"));
    let read = |s, closed, due| {
        let t = obligation_status(s, closed, due, "2025-06-01", "Served");
        (t.state, t.reason)
    };
    let cases = [
            (Status::Open, RowState::Open, None),
            (
                Status::Discharged,
                RowState::Met,
                Some("discharged by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Late,
                RowState::Late,
                Some("acted by Answered(complaint) on 2025-03-20, after the due date"),
            ),
            (
                Status::Unjudged,
                RowState::Unknown,
                Some("timeliness unknown: Served due date not recorded; acted by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Unknown,
                RowState::Unknown,
                Some("Served due date not recorded"),
            ),
            (
                Status::Breached,
                RowState::Breached,
                Some("past due at 2025-06-01 with no act by its bearer"),
            ),
            (
                Status::Lapsed,
                RowState::Lapsed,
                Some("past due at 2025-06-01 with no act"),
            ),
            (
                Status::Waived,
                RowState::Superseded,
                Some("waived by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Superseded,
                RowState::Superseded,
                Some("restarted by Answered(complaint) on 2025-03-20 (a later trigger)"),
            ),
            (
                Status::Merged,
                RowState::Superseded,
                Some("absorbed by a `later_of` instance"),
            ),
        ];
    for (status, state, reason) in cases {
        assert_eq!(
            read(status, closed, Some("2025-03-26")),
            (state, reason.map(str::to_owned)),
            "{status:?}"
        );
    }
    assert_eq!(
        read(Status::Discharged, None, None),
        (RowState::Met, Some("discharged".to_owned()))
    );
    assert_eq!(
        read(Status::Open, None, None),
        (
            RowState::Unknown,
            Some("open at 2025-06-01 with no due date".to_owned())
        )
    );
}

#[test]
fn layers_are_enabled_named_or_disabled() {
    let ids = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    let spec = |items: &[&str]| {
        disabled_layers(
            &ids,
            &items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
        )
    };
    assert_eq!(spec(&[]), Ok(vec![]));
    assert_eq!(spec(&["-b"]), Ok(vec!["b".to_owned()]));
    assert_eq!(spec(&["a", "+c"]), Ok(vec!["b".to_owned()]));
    assert_eq!(
        spec(&["a", "-a"]),
        Ok(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()])
    );
    assert_eq!(
        spec(&["-d"]),
        Err("no layer `d`; layers: a, b, c".to_owned())
    );
}

/// The frozen example document.
fn example() -> Timeline {
    serde_json::from_str(include_str!(
        "../../../../docs/litigate/examples/timeline-ttab-opposition.json"
    ))
    .unwrap()
}

fn row_ids(t: &Timeline) -> Vec<&str> {
    t.rows.iter().map(|r| r.id.as_str()).collect()
}

fn world_ids(t: &Timeline) -> Vec<&str> {
    t.worlds.iter().map(|w| w.id.as_str()).collect()
}

/// The example with a world `w3` nested under `w1` that names no fork
/// row, and a docket row after the fork.
fn nested() -> Timeline {
    let mut t = example();
    let mut w3 = t.worlds[1].clone();
    w3.id = "w3".to_owned();
    w3.parent = Some("w1".to_owned());
    w3.fork_row = None;
    t.worlds.push(w3);
    let mut late = t.rows[0].clone();
    late.id = "r10".to_owned();
    t.rows.push(late);
    t
}

#[test]
fn a_disabled_layer_drops_its_rows_and_the_worlds_forking_from_them() {
    let mut t = nested();
    // r7 triggered by the reset (r6) loses its trigger with it.
    t.rows[6].trigger = Some("r6".to_owned());
    apply_layers(&mut t, &["order-reset-1".to_owned()]);
    assert_eq!(
        row_ids(&t),
        ["r1", "r2", "r3", "r4", "r5", "r7", "r9", "r10"]
    );
    assert_eq!(t.rows[5].trigger, None);
    // r7's replacement (r8) went with its layer, so r7 names none.
    assert_eq!(t.rows[5].replaced_by, None);
    assert!(!t.layers[1].enabled);
    assert!(t.layers[0].enabled);
    let ends: Vec<&str> = t
        .edges
        .iter()
        .flat_map(|e| [e.from.as_str(), e.to.as_str()])
        .collect();
    assert!(!ends.contains(&"r6"));
    let mut t = nested();
    apply_layers(&mut t, &["ttab".to_owned()]);
    assert_eq!(world_ids(&t), ["w0"]);
    assert!(t.rows.is_empty());
    assert!(t.edges.is_empty());
}

#[test]
fn a_world_keeps_its_ancestors_up_to_its_fork() {
    let mut t = nested();
    assert_eq!(apply_world(&mut t, "w1"), Ok(()));
    assert_eq!(world_ids(&t), ["w0", "w1"]);
    assert_eq!(
        row_ids(&t),
        ["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8"]
    );
    assert_eq!(t.rows[6].replaced_by.as_ref(), Some(&t.rows[7].key));
    assert!(t.edges.iter().all(|e| e.world != "w2"));
    // A world naming no fork row cuts nothing from its parent.
    let mut t = nested();
    assert_eq!(apply_world(&mut t, "w3"), Ok(()));
    assert_eq!(world_ids(&t), ["w0", "w1", "w3"]);
    assert_eq!(t.rows.len(), 8);
    let mut t = nested();
    assert_eq!(
        apply_world(&mut t, "w7"),
        Err("no world `w7`; worlds: w0, w1, w2, w3".to_owned())
    );
}

/// The worked example (plus `extra`) with matter `text`, checked: its
/// session, which `f` reads with the matter.
fn with_matter(extra: &str, text: &str, f: impl FnOnce(&Session, matter::CaseCheck)) {
    let session = testkit::example_locked(extra, text, None);
    let all = session.hir();
    let c = matter::check(
        &session,
        &session.cases[0],
        &crate::clock::Clocks::new(&all),
        litigate_base::Profile::Dev,
    );
    f(&session, c);
}

/// The timeline of `m` in `session`, as JSON.
fn build(session: &Session, m: &matter::CaseCheck, case: &hir::Case) -> serde_json::Value {
    let all = session.hir();
    let texts = session.sources();
    serde_json::to_value(of_case(m, case, &all, &texts, &m.diagnostics)).unwrap()
}

fn rows(t: &serde_json::Value) -> Vec<(String, String, String)> {
    t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["world"].as_str().unwrap().to_owned(),
                r["kind"].as_str().unwrap().to_owned(),
                r["status"]["state"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn the_lattice_is_the_package_s_its_dependencies_and_core() {
    let session = testkit::example_with("");
    let all = session.hir();
    let mut pkg = all[0].clone();
    assert!(lattice(&all, &pkg).contains("FederalRule"));
    // A dependency that is not loaded adds nothing; one met twice is
    // read once.
    pkg.manifest
        .dependencies
        .insert("nowhere".to_owned(), "^1".to_owned());
    pkg.manifest
        .dependencies
        .insert("core".to_owned(), "^1".to_owned());
    assert_eq!(lattice(&all, &pkg), lattice(&all, all[0]));
    with_matter("", ACME, |session, m| {
        let all = session.hir();
        let texts = session.sources();
        let mut t = of_case(&m, m.case.as_ref().unwrap(), &all, &texts, &[]);
        assert_eq!(authority_errors(&t, &all), Vec::<Diagnostic>::new());
        let mut bad = t.layers[0].clone();
        bad.authority = "Constitution".to_owned();
        let mut ghost = bad.clone();
        ghost.package = "ghost".to_owned();
        t.layers.extend([bad, ghost]);
        let errors = authority_errors(&t, &all);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, DiagnosticCode::InvalidManifest);
        assert_eq!(errors[0].span.file.0, "lit/examples/Lit.toml");
        assert!(errors[0]
                .message
                .starts_with("layer `frcp-rule12-example`: authority `Constitution` is not in the authority lattice package `frcp-rule12-example` declares (known: "));
    });
}

#[test]
fn rows_sharing_a_key_in_one_world_are_an_error_naming_both() {
    with_matter("", ACME, |session, m| {
        let all = session.hir();
        let texts = session.sources();
        let mut t = of_case(&m, m.case.as_ref().unwrap(), &all, &texts, &[]);
        assert_eq!(check_keys(&mut t), Vec::<Diagnostic>::new());
        let before = t.diagnostics.len();
        // The same key in another world is no collision.
        let mut elsewhere = t.rows[0].clone();
        elsewhere.id = "x1".to_owned();
        elsewhere.world = "w1".to_owned();
        let mut twin = t.rows[0].clone();
        twin.id = "x2".to_owned();
        let mut first = t.rows[1].clone();
        first.source = None;
        first.id = "x3".to_owned();
        first.key = "k".to_owned();
        let mut second = first.clone();
        second.id = "x4".to_owned();
        t.rows.extend([elsewhere, twin, first, second]);
        let errors = check_keys(&mut t);
        let messages: Vec<&str> = errors.iter().map(|d| d.message.as_str()).collect();
        assert_eq!(
                messages,
                [
                    "rows `r1` (Served(complaint, Personal) on widget (line 20)) and `x2` (Served(complaint, Personal) on widget (line 20)) of world `w0` share the key `event:Served@served-1`: nothing in their cause tells them apart; give the events docket numbers",
                    "rows `x3` (MovedToDismiss(complaint) by widget) and `x4` (MovedToDismiss(complaint) by widget) of world `w0` share the key `k`: nothing in their cause tells them apart; give the events docket numbers",
                ]
            );
        assert!(errors
            .iter()
            .all(|d| d.code == DiagnosticCode::DuplicateName));
        assert_eq!(errors[0].span.line, 20);
        assert_eq!(errors[1].span.file.0, "cases/examples/acme.case");
        assert_eq!(errors[1].span.line, 1);
        assert_eq!(t.diagnostics.len(), before + 2);
        assert_eq!(t.diagnostics[before].code, "L1001");
    });
}

#[test]
fn only_an_election_or_removal_names_an_option_edge() {
    with_matter("", ACME, |session, m| {
        let case = m.case.as_ref().unwrap();
        let all = session.hir();
        let served = &case.events[0].action;
        let moved = &case.events[1].action;
        assert_eq!(option_edge(&all, served), None);
        let mut pkg = all[0].clone();
        let declare = |pkg: &mut hir::Package, effect| {
            for module in &mut pkg.modules {
                for d in &mut module.declarations {
                    if let Declaration::Event(t) = d {
                        t.effect.clone_from(&effect);
                    }
                }
            }
        };
        declare(&mut pkg, Some(hir::EventEffect::Elects { param: 0 }));
        let elects = [&pkg];
        assert_eq!(option_edge(&elects, served).as_deref(), Some("complaint"));
        assert_eq!(option_edge(&elects, moved).as_deref(), Some("complaint"));
        declare(&mut pkg, Some(hir::EventEffect::Removes { param: 9 }));
        assert_eq!(option_edge(&[&pkg], served), None);
        declare(
            &mut pkg,
            Some(hir::EventEffect::Waives {
                deadline: "answer-period".to_owned(),
            }),
        );
        assert_eq!(option_edge(&[&pkg], served), None);
        let mut literal = served.clone();
        literal.kind = hir::ExprKind::Literal {
            value: hir::Value::Edge("e".to_owned()),
        };
        assert_eq!(option_edge(&[&pkg], &literal), None);
        assert_eq!(hirx::event_text(&literal), "");
    });
}
