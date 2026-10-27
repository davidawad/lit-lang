//! Strictness: every construct the engine cannot represent is a precise
//! lowering error (SPEC §8: L6001/L6003/L6004, L3007), never a silent
//! approximation. Each case mutates the checked HIR of `lit/examples`.
mod support;

use litigate_base::DiagnosticCode as C;
use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{
    self, Anchor, Authored, DayUnit, DeadlineWindow, Declaration, Direction, Interval, Money,
    Observance, RollOn, ServiceTime,
};
use support::load_hir;

fn example() -> hir::Package {
    load_hir(&["lit/examples"]).remove(0)
}

fn decls(pkg: &mut hir::Package) -> impl Iterator<Item = &mut Declaration> {
    pkg.modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
}

fn counting(pkg: &mut hir::Package) -> &mut hir::Counting {
    decls(pkg)
        .find_map(|d| match d {
            Declaration::Counting(c) => Some(c),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no counting"))
}

fn node<'p>(pkg: &'p mut hir::Package, id: &str) -> &'p mut hir::Node {
    decls(pkg)
        .find_map(|d| match d {
            Declaration::Node(n) if n.meta.id == id => Some(n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no node {id}"))
}

fn lower(
    pkg: &hir::Package,
    options: &Options,
) -> Result<litigate_lower::LoweredPackage, Vec<litigate_base::Diagnostic>> {
    Lowerer::new(&[pkg]).package(pkg, options)
}

fn codes(pkg: &hir::Package) -> Vec<(C, String)> {
    lower(pkg, &Options::default())
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|d| (d.code, d.message))
        .collect()
}

fn only(pkg: &hir::Package, code: C, needle: &str) {
    let found = codes(pkg);
    assert!(
        found.len() == 1 && found[0].0 == code && found[0].1.contains(needle),
        "expected one {code:?} containing {needle:?}, got {found:#?}"
    );
}

#[test]
fn clock_shapes_the_engine_cannot_run() {
    let mut p = example();
    counting(&mut p).roll_on = vec![RollOn::Weekend];
    only(&p, C::LoweringUnsupported, "`weekend` and `legal_holiday`");

    let mut p = example();
    counting(&mut p).exclude_trigger_day = false;
    only(&p, C::LoweringUnsupported, "exclude_trigger_day");

    let mut p = example();
    counting(&mut p).anchor = Anchor::NextBusinessDay;
    only(&p, C::LoweringUnsupported, "next_business_day");

    let mut p = example();
    counting(&mut p).include_last_day = false;
    only(&p, C::LoweringUnsupported, "include_last_day");

    let mut p = example();
    counting(&mut p)
        .service_extensions
        .insert("Fax".into(), ServiceTime::Adds(1, DayUnit::Calendar));
    only(&p, C::LoweringUnsupported, "`Fax`");

    let mut p = example();
    counting(&mut p)
        .service_extensions
        .insert("Mail".into(), ServiceTime::Adds(3, DayUnit::Court));
    only(&p, C::LoweringUnsupported, "calendar days");

    let mut p = example();
    counting(&mut p).meta.cite = Some("Local Rule 6".into());
    only(&p, C::LoweringUnsupported, "sourceUrl");

    let mut p = example();
    for d in decls(&mut p) {
        if let Declaration::Calendar(c) = d {
            c.holidays[0].observed = Some(Observance::NextDay);
        }
    }
    only(&p, C::LoweringUnsupported, "next_day");

    let mut p = example();
    p.manifest.counting = Some("rcfc6".into());
    only(&p, C::LoweringUnsupported, "names counting rule `rcfc6`");
}

#[test]
fn a_shared_library_rule_needs_no_fragment() {
    let mut p = example();
    p.modules.iter_mut().for_each(|m| {
        m.declarations
            .retain(|d| !matches!(d, Declaration::Counting(_) | Declaration::Calendar(_)));
    });
    p.manifest.counting = Some("frcp6".into());
    for d in decls(&mut p) {
        if let Declaration::Deadline(x) = d {
            x.counting = "frcp6".into();
        }
    }
    let lowered = lower(&p, &Options::default()).unwrap_or_else(|d| panic!("{d:#?}"));
    assert!(lowered.clock.is_none());
    let Document::Pack(pack) = lowered.document else {
        panic!("expected a pack")
    };
    assert_eq!(
        (pack.schema_version, pack.counting.as_deref()),
        (litigate_engine::model::PACK_SCHEMA_VERSION, Some("frcp6"))
    );
}

#[test]
fn edge_values_without_a_lowering() {
    let mut p = example();
    node(&mut p, "respond").edges[2].deadline = Some(hir::EdgeDeadline {
        window: DeadlineWindow::Inline {
            length: 3,
            unit: DayUnit::Business,
        },
        extendable: None,
        extension_authority: None,
        note: None,
        counting: None,
    });
    only(&p, C::LoweringUnsupported, "business-day");

    let mut p = example();
    node(&mut p, "respond").edges[2].cost = Some(Authored::Known(Money {
        minor_units: 100,
        currency: "eur".into(),
    }));
    only(&p, C::LoweringUnsupported, "`eur`");

    // Concurrent tracks lower: the engine expands their markings.
    let mut p = example();
    node(&mut p, "respond").edges[2].spawn = vec!["answered".into()];
    assert_eq!(codes(&p), []);
    let mut p = example();
    node(&mut p, "respond").edges[2].joins = vec!["answered".into()];
    assert_eq!(codes(&p), []);

    let mut p = example();
    for d in decls(&mut p) {
        if let Declaration::Deadline(x) = d {
            x.direction = Direction::Backward;
        }
    }
    let found = codes(&p);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found
        .iter()
        .all(|(c, m)| *c == C::LoweringUnsupported && m.contains("backward")));
}

/// `p` lowered to its pack.
fn pack_of(p: &hir::Package) -> Box<litigate_engine::model::Pack> {
    let lowered = lower(p, &Options::default()).unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Pack(pack) = lowered.document else {
        panic!("expected a pack")
    };
    pack
}

/// Whether each edge of `p`'s pack has a `deadline` window.
fn windows(p: &hir::Package) -> Vec<bool> {
    pack_of(p)
        .edges
        .iter()
        .map(|e| e.deadline.is_some())
        .collect()
}

/// The `(pointer, reason)` of each `lowered.holes` entry of `p`'s pack
/// under an edge's `deadline`.
fn deadline_holes(p: &hir::Package) -> Vec<(String, String)> {
    pack_of(p)
        .lowered
        .unwrap_or_else(|| panic!("a pack has a lowered header"))
        .holes
        .into_iter()
        .filter(|h| h.pointer.contains("/deadline/"))
        .map(|h| (h.pointer, h.reason.unwrap_or_default()))
        .collect()
}

/// Every deadline of `p` designated by its trigger's first argument.
fn designate(p: &mut hir::Package) {
    for d in decls(p) {
        if let Declaration::Deadline(x) = d {
            x.designated = Some(hir::Designated {
                event: "Noticed".into(),
                param: 0,
                name: "due".into(),
                interpretation: None,
            });
        }
    }
}

/// The `deadline` of each edge of `p`'s pack that has one.
fn deadlines(p: &hir::Package) -> Vec<litigate_engine::model::Deadline> {
    pack_of(p)
        .edges
        .into_iter()
        .filter_map(|e| e.deadline)
        .collect()
}

#[test]
fn a_designated_deadline_lowers_to_a_designated_window() {
    let mut p = example();
    let counted = windows(&p);
    assert!(counted.contains(&true));
    designate(&mut p);
    // The notice sets the date in each matter: its least time is never a
    // `length`, and every window the source has is kept (lg-flu.33).
    assert_eq!(windows(&p), counted);
    let found = deadlines(&p);
    assert!(
        found.iter().all(|d| d.length.is_none()
            && d.designated
                == Some(litigate_engine::model::Designated {
                    min_period: 21.0,
                    source_event: "Noticed".into(),
                    source_event_arg: "due".into(),
                    interpretation: None,
                })),
        "{found:#?}"
    );
    // Its extension lowers as a counted window's does: what the source
    // states, edge options winning; nothing stated is undeclared (absent),
    // never not-extendable, and no hole stands in for it.
    assert_eq!(deadline_holes(&p), Vec::<(String, String)>::new());
    assert!(found.iter().all(|d| d.extendable.is_none()));
    for d in decls(&mut p) {
        match d {
            Declaration::Deadline(x) => {
                x.extendable = Some(true);
                x.extension_authority = Some("Rule X".into());
                if let Some(designated) = x.designated.as_mut() {
                    designated.interpretation = Some("no start named".into());
                }
            }
            Declaration::Node(n) => {
                for e in &mut n.edges {
                    if let Some(w) = e.deadline.as_mut() {
                        w.extendable = Some(false);
                    }
                }
            }
            _ => {}
        }
    }
    let found = deadlines(&p);
    assert!(
        found.iter().all(|d| d.extendable == Some(false)
            && d.extension_authority.as_deref() == Some("Rule X")
            && d.designated
                .as_ref()
                .and_then(|x| x.interpretation.as_deref())
                == Some("no start named")),
        "{found:#?}"
    );
    assert_eq!(deadline_holes(&p), Vec::<(String, String)>::new());
}

/// The motion event pattern `Moved(answer)`.
fn moved() -> hir::Expr {
    let expr = |ty: &str, kind| hir::Expr {
        ty: hir::Type::Named(ty.into()),
        kind,
        span: None,
    };
    expr(
        "Moved",
        hir::ExprKind::Call {
            callee: "Moved".into(),
            args: vec![expr(
                "Doc",
                hir::ExprKind::Literal {
                    value: hir::Value::Variant("Doc::answer".into()),
                },
            )],
        },
    )
}

/// One `extend` grant of each shape, every bound and standard among them.
fn grants() -> Vec<hir::Extension> {
    use hir::{Bound, CauseStandard as S, DateRef as R, Extension, ExtensionWindow as W};
    let due = |days| Bound::Offset {
        from: R::PrescribedDue,
        days,
    };
    let order = |days| Bound::Offset {
        from: R::GrantOrderEntered,
        days,
    };
    vec![
        Extension::Bounded {
            by: "Court".into(),
            until: Bound::Max(vec![due(30), Bound::Min(vec![due(0), order(14)])]),
            after_expiry: true,
            standard: Some(S::GoodCauseOrExcusableNeglect),
            motion: Some(hir::Motion {
                event: moved(),
                within: Some(due(30)),
                unless: Some(moved()),
            }),
            interpretation: Some("read so".into()),
        },
        Extension::Bounded {
            by: "Court".into(),
            until: Bound::Before {
                from: R::CurrentDue,
                days: 10,
            },
            after_expiry: false,
            standard: Some(S::ExtraordinaryCircumstances),
            motion: None,
            interpretation: None,
        },
        Extension::Discretionary {
            by: "Court".into(),
            standard: S::GoodCause,
            window: W::BeforeExpiry,
            motion: None,
            interpretation: Some("read so".into()),
        },
        Extension::Discretionary {
            by: "Court".into(),
            standard: S::ExcusableNeglect,
            window: W::AfterExpiryOnMotion,
            motion: Some(hir::Motion {
                event: moved(),
                within: None,
                unless: None,
            }),
            interpretation: None,
        },
        Extension::NonExtendable {
            authority: "FRCP 6(b)(2)".into(),
            against: Vec::new(),
        },
    ]
}

/// [`grants`] as pack extension bounds.
fn bounds() -> Vec<litigate_engine::model::ExtensionBound> {
    use litigate_engine::model::{
        CauseStandard, DateBound, DateRef, ExtensionBound, ExtensionWindow, Motion,
    };
    let due = |days| DateBound::Offset {
        from: DateRef::PrescribedDue,
        days,
    };
    vec![
        ExtensionBound::Bounded {
            by: "Court".into(),
            until: DateBound::Max(vec![
                due(30),
                DateBound::Min(vec![
                    due(0),
                    DateBound::Offset {
                        from: DateRef::GrantOrderEntered,
                        days: 14,
                    },
                ]),
            ]),
            after_expiry: true,
            standard: Some(CauseStandard::GoodCauseOrExcusableNeglect),
            motion: Some(Motion {
                event: "Moved(answer)".into(),
                within: Some(due(30)),
                unless: Some("Moved(answer)".into()),
            }),
            interpretation: Some("read so".into()),
        },
        ExtensionBound::Bounded {
            by: "Court".into(),
            until: DateBound::Before {
                from: DateRef::CurrentDue,
                days: 10,
            },
            after_expiry: false,
            standard: Some(CauseStandard::ExtraordinaryCircumstances),
            motion: None,
            interpretation: None,
        },
        ExtensionBound::Discretionary {
            by: "Court".into(),
            standard: CauseStandard::GoodCause,
            window: ExtensionWindow::BeforeExpiry,
            motion: None,
            interpretation: Some("read so".into()),
        },
        ExtensionBound::Discretionary {
            by: "Court".into(),
            standard: CauseStandard::ExcusableNeglect,
            window: ExtensionWindow::AfterExpiryOnMotion,
            motion: Some(Motion {
                event: "Moved(answer)".into(),
                within: None,
                unless: None,
            }),
            interpretation: None,
        },
        ExtensionBound::NonExtendable {
            authority: "FRCP 6(b)(2)".into(),
        },
    ]
}

#[test]
fn extension_grants_lower_to_extension_bounds() {
    let mut p = example();
    assert!(deadlines(&p).iter().all(|d| d.extensions.is_empty()));
    let grants = grants();
    for d in decls(&mut p) {
        if let Declaration::Deadline(x) = d {
            x.extend.clone_from(&grants);
        }
    }
    let want = bounds();
    let found = deadlines(&p);
    assert!(!found.is_empty());
    assert!(found.iter().all(|d| d.extensions == want), "{found:#?}");
    // The pack reads back with them (schema v5).
    let pack = pack_of(&p);
    let text = serde_json::to_string(&pack).unwrap_or_default();
    let back = litigate_engine::model::Pack::from_json(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(back.edges, pack.edges);
}

#[test]
fn versions_are_selected_at_the_build_dates() {
    let mut p = example();
    let mut old = node(&mut p, "answered").clone();
    node(&mut p, "answered").meta.effective.from = "2015-01-01".into();
    old.meta.effective = Interval {
        from: "2009-12-01".into(),
        to: Some("2015-01-01".into()),
        known_at: "2025-01-01".into(),
        retracted: None,
        unknown_before: None,
        commenced: None,
    };
    old.meta.label = "Answer filed (pre-2015)".into();
    p.modules[0].declarations.push(Declaration::Node(old));
    let label = |options: &Options| -> Vec<String> {
        let lowered = lower(&p, options).unwrap_or_else(|d| panic!("{d:#?}"));
        let Document::Pack(pack) = lowered.document else {
            panic!("expected a pack")
        };
        pack.nodes
            .iter()
            .filter(|n| n.id == "answered")
            .map(|n| n.label.clone())
            .collect()
    };
    assert_eq!(label(&Options::default()), ["Answer filed"]);
    let early = Options {
        as_of: Some("2012-06-01".into()),
        known_at: None,
    };
    assert_eq!(label(&early), ["Answer filed (pre-2015)"]);
}

#[test]
fn a_target_out_of_force_is_not_left_dangling() {
    let mut p = example();
    node(&mut p, "answered").meta.effective.from = "2015-01-01".into();
    let early = Options {
        as_of: Some("2012-06-01".into()),
        known_at: None,
    };
    let found: Vec<(C, String)> = lower(&p, &early)
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|d| (d.code, d.message))
        .collect();
    // `answer` and `denied` both lead to `answered`.
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found
            .iter()
            .all(|(c, m)| *c == C::NoVersionInForce && m.contains("targets `answered`")),
        "{found:#?}"
    );
}

#[test]
fn overlapping_versions_are_ambiguous() {
    let mut p = example();
    let twin = node(&mut p, "answered").clone();
    p.modules[0].declarations.push(Declaration::Node(twin));
    let found = codes(&p);
    assert!(
        found.iter().any(|(c, _)| *c == C::NoVersionInForce),
        "{found:#?}"
    );
}

#[test]
fn instances_are_checked_against_their_base_pack() {
    let inputs = support::corpus();
    let packages = support::imported_hir(&inputs);
    let mut all: Vec<hir::Package> = packages.into_iter().map(|(_, p)| p).collect();
    let links = all
        .iter()
        .position(|p| p.manifest.links)
        .unwrap_or_else(|| panic!("no links"));
    let mut bad = all.remove(links);
    for d in decls(&mut bad) {
        if let Declaration::Instance(i) = d {
            i.payoff_transform = Some(literal(hir::Value::Boolean(true)));
            if let Some(patch) = i.patch_edges.values_mut().next() {
                // A patch lowers as an edge does: business days have no
                // litgraph unit.
                patch.deadline = Some(hir::EdgeDeadline {
                    window: hir::DeadlineWindow::Inline {
                        length: 45,
                        unit: hir::DayUnit::Business,
                    },
                    extendable: None,
                    extension_authority: None,
                    note: None,
                    counting: None,
                });
            }
        }
    }
    let refs: Vec<&hir::Package> = all.iter().collect();
    let found: Vec<C> = Lowerer::new(&refs)
        .package(&bad, &Options::default())
        .err()
        .unwrap_or_default()
        .iter()
        .map(|d| d.code)
        .collect();
    assert!(found.contains(&C::InvalidEngineExpression), "{found:?}");
    assert!(found.contains(&C::LoweringUnsupported), "{found:?}");
    // Without its base packs loaded, a patching instance cannot be checked.
    let alone = Lowerer::new(&[]).package(&bad, &Options::default());
    assert!(alone.is_err());
}

/// The links package with every instance given `transform` and `stake`,
/// lowered against its base packs.
fn lower_instances(
    transform: &hir::Expr,
    stake: Option<&Authored<Money>>,
) -> Result<litigate_lower::LoweredPackage, Vec<litigate_base::Diagnostic>> {
    let inputs = support::corpus();
    let packages = support::imported_hir(&inputs);
    let mut all: Vec<hir::Package> = packages.into_iter().map(|(_, p)| p).collect();
    let links = all
        .iter()
        .position(|p| p.manifest.links)
        .unwrap_or_else(|| panic!("no links"));
    let mut pkg = all.remove(links);
    for d in decls(&mut pkg) {
        if let Declaration::Instance(i) = d {
            i.payoff_transform = Some(transform.clone());
            i.stake = stake.cloned();
        }
    }
    let refs: Vec<&hir::Package> = all.iter().collect();
    Lowerer::new(&refs).package(&pkg, &Options::default())
}

fn usd(dollars: i64) -> Authored<Money> {
    Authored::Known(Money {
        minor_units: dollars * 100,
        currency: "usd".into(),
    })
}

fn expr(kind: hir::ExprKind) -> hir::Expr {
    hir::Expr {
        ty: hir::Type::Unit,
        kind,
        span: None,
    }
}

fn literal(value: hir::Value) -> hir::Expr {
    expr(hir::ExprKind::Literal { value })
}

fn local(name: &str) -> hir::Expr {
    expr(hir::ExprKind::Ref {
        target: name.into(),
    })
}

fn binary(op: hir::BinaryOp, lhs: hir::Expr, rhs: hir::Expr) -> hir::Expr {
    expr(hir::ExprKind::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn negated(operand: hir::Expr) -> hir::Expr {
    expr(hir::ExprKind::Unary {
        op: hir::UnaryOp::Neg,
        operand: Box::new(operand),
    })
}

fn dollars(amount: i64, currency: &str) -> hir::Expr {
    literal(hir::Value::Money(Money {
        minor_units: amount,
        currency: currency.into(),
    }))
}

fn stake_minus_payoff() -> hir::Expr {
    binary(hir::BinaryOp::Sub, local("stake"), local("payoff"))
}

#[test]
fn an_instance_stake_lowers_to_the_value_its_transform_binds() {
    let Ok(lowered) = lower_instances(&stake_minus_payoff(), Some(&usd(2_000_000))) else {
        panic!("a known stake lowers");
    };
    let Document::Links(links) = lowered.document else {
        panic!("a links package lowers to a link file");
    };
    assert!(links.instances.values().all(|i| i.stake == Some(2e6)));
    assert!(links
        .instances
        .values()
        .all(|i| i.payoff_transform.as_deref() == Some("stake - payoff")));
    let Ok(lowered) = lower_instances(&stake_minus_payoff(), Some(&Authored::Hole(None))) else {
        panic!("a stake hole lowers");
    };
    let Document::Links(links) = lowered.document else {
        panic!("a links package lowers to a link file");
    };
    assert!(links.instances.values().all(|i| i.stake.is_none()));
}

/// A typed transform lowers to litgraph syntax, parenthesized by
/// precedence; what has no engine spelling is reported.
#[test]
fn an_instance_transform_lowers_to_engine_syntax() {
    use hir::BinaryOp::{Add, Div, Lt, Mul, Sub};
    let lowered = |e: &hir::Expr| match lower_instances(e, None) {
        Ok(p) => match p.document {
            Document::Links(l) => l
                .instances
                .values()
                .next()
                .and_then(|i| i.payoff_transform.clone())
                .inspect(|t| assert!(litigate_engine::expr::parse(t).is_ok(), "{t}"))
                .unwrap_or_default(),
            other @ Document::Pack(_) => panic!("{other:?}"),
        },
        // Every instance reports alike; the first says it.
        Err(d) => d
            .first()
            .map(|d| format!("{} {}", d.code.as_str(), d.message))
            .unwrap_or_default(),
    };
    let half = literal(hir::Value::Number(hir::Rational {
        numerator: 1,
        denominator: 2,
    }));
    let rows = [
        (
            "sum",
            binary(Add, dollars(100_000_000, "usd"), local("payoff")),
        ),
        (
            "right-nested difference",
            binary(
                Sub,
                local("payoff"),
                binary(Sub, local("stake"), local("payoff")),
            ),
        ),
        (
            "product of a sum",
            binary(
                Mul,
                binary(Add, local("payoff"), local("stake")),
                half.clone(),
            ),
        ),
        (
            "quotient of a product",
            binary(Div, binary(Mul, local("payoff"), half.clone()), half),
        ),
        ("negated sum", negated(stake_minus_payoff())),
        (
            "negative literal",
            binary(Sub, local("payoff"), dollars(-150, "usd")),
        ),
        ("negated negative", negated(dollars(-150, "usd"))),
        ("comparison", binary(Lt, local("payoff"), local("stake"))),
        (
            "negated comparison",
            negated(binary(Lt, local("payoff"), local("stake"))),
        ),
        (
            "a sum of a non-value",
            binary(Add, literal(hir::Value::Boolean(true)), local("payoff")),
        ),
        ("not a value", literal(hir::Value::Boolean(true))),
        (
            "other currency",
            binary(Add, local("payoff"), dollars(1, "eur")),
        ),
        (
            "no binary64",
            binary(
                Mul,
                local("payoff"),
                literal(hir::Value::Number(hir::Rational {
                    numerator: 1,
                    denominator: 0,
                })),
            ),
        ),
    ];
    let table: Vec<(&str, String)> = rows.iter().map(|(n, e)| (*n, lowered(e))).collect();
    insta::assert_debug_snapshot!(table);
}

/// A package that declares several counting rules and names none in
/// `Lit.toml` is a counting-rule library (`lit/core`): its pack names no
/// rule, and its one clock fragment holds every rule and, once, each
/// calendar they close on. Naming a rule it does not declare stays L6001.
#[test]
fn a_counting_rule_library_lowers_every_rule() {
    let mut p = example();
    let mut copy = counting(&mut p).clone();
    copy.meta.id = "frcp6-copy".into();
    p.modules[0].declarations.push(Declaration::Counting(copy));
    let lowered = lower(&p, &Options::default()).unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Pack(pack) = &lowered.document else {
        panic!("expected a pack")
    };
    assert_eq!(
        (pack.schema_version, pack.counting.as_deref()),
        (litigate_engine::model::PACK_SCHEMA_VERSION, None)
    );
    let (clock, map) = lowered.clock.unwrap_or_else(|| panic!("no fragment"));
    let rules: Vec<&str> = clock.counting_rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(rules, ["frcp6", "frcp6-copy"]);
    assert_eq!(clock.calendars.len(), 1);
    let pointers: Vec<&str> = map.entries.iter().map(|e| e.pointer.as_str()).collect();
    assert_eq!(
        pointers,
        ["/countingRules/0", "/calendars/0", "/countingRules/1"]
    );

    p.manifest.counting = Some("frcp6-copy".into());
    let lowered = lower(&p, &Options::default()).unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Pack(pack) = &lowered.document else {
        panic!("expected a pack")
    };
    assert_eq!(pack.counting.as_deref(), Some("frcp6-copy"));
    assert_eq!(lowered.clock.map(|(c, _)| c.counting_rules.len()), Some(2));

    p.manifest.counting = Some("rcfc6".into());
    only(&p, C::LoweringUnsupported, "names counting rule `rcfc6`");
}

#[test]
fn a_layer_with_no_node_and_no_start_lowers_to_an_empty_pack() {
    // Bowles v. Russell (docs/litigate/examples/change/overruling) declares
    // an invalidation and no node, so no `start`: nothing dangles.
    let all = load_hir(&["docs/litigate/examples/change/overruling"]);
    let refs: Vec<&hir::Package> = all.iter().collect();
    let bowles = all
        .iter()
        .find(|p| p.manifest.name == "change-bowles-v-russell")
        .unwrap_or_else(|| panic!("no Bowles"));
    assert!(bowles.start.is_empty());
    let lowered = Lowerer::new(&refs)
        .package(bowles, &Options::default())
        .unwrap_or_else(|e| panic!("{e:?}"));
    let Document::Pack(pack) = lowered.document else {
        panic!("not a pack")
    };
    assert!(pack.nodes.is_empty() && pack.start_node_id.is_empty());
}
