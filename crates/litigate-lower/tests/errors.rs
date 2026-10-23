//! Strictness: every construct the engine cannot represent is a precise
//! lowering error (SPEC §8: L6001/L6003/L6004, L3007), never a silent
//! approximation. Each case mutates the checked HIR of `lit/examples`.
mod support;

use litigate_base::DiagnosticCode as C;
use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{
    self, Anchor, Authored, DayUnit, DeadlineWindow, Declaration, Direction, Interval, Money,
    Observance, RollOn,
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
        .insert("Fax".into(), (1, DayUnit::Calendar));
    only(&p, C::LoweringUnsupported, "`Fax`");

    let mut p = example();
    counting(&mut p)
        .service_extensions
        .insert("Mail".into(), (3, DayUnit::Court));
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
        (4, Some("frcp6"))
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
    });
    only(&p, C::LoweringUnsupported, "business-day");

    let mut p = example();
    node(&mut p, "respond").edges[2].cost = Some(Authored::Known(Money {
        minor_units: 100,
        currency: "eur".into(),
    }));
    only(&p, C::LoweringUnsupported, "`eur`");

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

#[test]
fn versions_are_selected_at_the_build_dates() {
    let mut p = example();
    let mut old = node(&mut p, "answered").clone();
    node(&mut p, "answered").meta.effective.from = "2015-01-01".into();
    old.meta.effective = Interval {
        from: "2009-12-01".into(),
        to: Some("2015-01-01".into()),
        known_at: "2025-01-01".into(),
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
            i.payoff_transform = Some("payoff *".into());
            if let Some(patch) = i.patch_edges.values_mut().next() {
                *patch = serde_json::json!({"deadline": {"length": "45"}});
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
    transform: &str,
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
            i.payoff_transform = Some(transform.into());
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

#[test]
fn an_instance_stake_lowers_to_the_value_its_transform_binds() {
    let Ok(lowered) = lower_instances("stake - payoff", Some(&usd(2_000_000))) else {
        panic!("a known stake lowers");
    };
    let Document::Links(links) = lowered.document else {
        panic!("a links package lowers to a link file");
    };
    assert!(links.instances.values().all(|i| i.stake == Some(2e6)));
    let Ok(lowered) = lower_instances("stake - payoff", Some(&Authored::Hole(None))) else {
        panic!("a stake hole lowers");
    };
    let Document::Links(links) = lowered.document else {
        panic!("a links package lowers to a link file");
    };
    assert!(links.instances.values().all(|i| i.stake.is_none()));
}

#[test]
fn an_instance_transform_binds_only_payoff_and_a_declared_stake() {
    let messages = |transform: &str, stake: Option<&Authored<Money>>| -> Vec<(C, String)> {
        lower_instances(transform, stake)
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|d| (d.code, d.message))
            .collect()
    };
    let undeclared = messages("stake - payoff", None);
    assert!(
        undeclared
            .iter()
            .any(|(c, m)| *c == C::InvalidEngineExpression && m.contains("uses `stake`")),
        "{undeclared:?}"
    );
    let unknown = messages("rate * payoff", Some(&usd(1)));
    assert!(
        unknown
            .iter()
            .any(|(c, m)| *c == C::InvalidEngineExpression && m.contains("uses `rate`")),
        "{unknown:?}"
    );
    let unused = messages("payoff", Some(&usd(1)));
    assert!(
        unused
            .iter()
            .any(|(c, m)| *c == C::InvalidField && m.contains("does not use")),
        "{unused:?}"
    );
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
    assert_eq!((pack.schema_version, pack.counting.as_deref()), (4, None));
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
