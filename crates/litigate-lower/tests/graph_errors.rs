//! Nodes, edges, links and instances the engine cannot represent (SPEC
//! §8.1, §8.4): each lowering error as a diagnostic snapshot, and the
//! edge values that do lower. Each case mutates the checked HIR of
//! `lit/examples`.
mod support;

use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{
    self, CiteShorthand, DayUnit, DeadlineWindow, Declaration, EdgeDeadline, Money, NodeKind,
    Rational, TriangularDuration,
};
use std::collections::BTreeMap;
use support::{
    decls_mut, example, lowered, lowering_errors, node_mut, pack_of, render_diagnostics,
};

/// `1/0`: a rational with no binary64 value.
const UNDEFINED: Rational = Rational {
    numerator: 1,
    denominator: 0,
};

fn inline(length: u64, unit: DayUnit) -> EdgeDeadline {
    EdgeDeadline {
        window: DeadlineWindow::Inline { length, unit },
        extendable: None,
        extension_authority: None,
        note: None,
        counting: None,
    }
}

fn within(deadline: &str) -> EdgeDeadline {
    EdgeDeadline {
        window: DeadlineWindow::Declared {
            deadline: deadline.into(),
        },
        extendable: None,
        extension_authority: None,
        note: None,
        counting: None,
    }
}

#[test]
fn an_attribute_must_have_a_binary64_value() {
    let mut p = example();
    node_mut(&mut p, "respond")
        .attrs
        .insert("weight".into(), UNDEFINED);
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn a_duration_mode_must_have_a_binary64_value() {
    let mut p = example();
    node_mut(&mut p, "respond").edges[0].duration = Some(TriangularDuration {
        min: None,
        mode: UNDEFINED,
        max: None,
    });
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn a_window_must_have_an_exact_binary64_length() {
    let mut p = example();
    node_mut(&mut p, "respond").edges[2].deadline = Some(inline(1 << 53, DayUnit::Calendar));
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn within_must_name_a_deadline() {
    let mut p = example();
    node_mut(&mut p, "respond").edges[0].deadline = Some(within("respond"));
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn within_must_resolve() {
    let mut p = example();
    node_mut(&mut p, "respond").edges[0].deadline = Some(within("nowhere"));
    insta::assert_snapshot!(lowering_errors(&p));
}

/// Court-day windows lower to `unit: "court"`; a span-less `within` edge
/// still resolves (located at the manifest if it did not).
#[test]
fn edge_windows_that_lower() {
    let mut p = example();
    let respond = node_mut(&mut p, "respond");
    respond.edges[0].span = None;
    respond.edges[2].deadline = Some(inline(14, DayUnit::Court));
    let lowered = lowered(&p);
    let pack = pack_of(&lowered);
    let unit = |i: usize| {
        pack.edges
            .iter()
            .filter(|e| e.from == "respond")
            .nth(i)
            .and_then(|e| e.deadline.as_ref())
            .map(|d| (d.length, d.unit.clone()))
    };
    assert_eq!(unit(0), Some((21.0, None)));
    assert_eq!(unit(2), Some((14.0, Some("court".to_owned()))));
}

/// Week, month and year windows keep their unit (`Deadline.unit`).
#[test]
fn calendar_period_windows_keep_their_unit() {
    let units = [
        (DayUnit::Week, "week"),
        (DayUnit::Month, "month"),
        (DayUnit::Year, "year"),
    ];
    for (unit, name) in units {
        let mut p = example();
        node_mut(&mut p, "respond").edges[2].deadline = Some(inline(2, unit));
        let lowered = lowered(&p);
        let window = pack_of(&lowered)
            .edges
            .iter()
            .filter(|e| e.from == "respond")
            .nth(2)
            .and_then(|e| e.deadline.as_ref())
            .map(|d| (d.length, d.unit.clone()));
        assert_eq!(window, Some((2.0, Some(name.to_owned()))));
    }
}

#[test]
fn a_flag_payoff_needs_a_currency_with_a_lowering() {
    let mut p = example();
    let NodeKind::Terminal { payoff_by_flag, .. } = &mut node_mut(&mut p, "dismissed").kind else {
        panic!("a terminal")
    };
    payoff_by_flag.insert(
        "defaulted".into(),
        Money {
            minor_units: 100,
            currency: "gbp".into(),
        },
    );
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn the_start_node_must_be_in_force() {
    let mut p = example();
    p.start = "nowhere".into();
    insta::assert_snapshot!(lowering_errors(&p));
}

/// A qualified target belongs to another pack and is never dangling.
#[test]
fn a_qualified_target_is_not_dangling() {
    let mut p = example();
    node_mut(&mut p, "respond").edges[2].to = "frcp::default-entered".into();
    let lowered = lowered(&p);
    let to: Vec<&str> = pack_of(&lowered)
        .edges
        .iter()
        .map(|e| e.to.as_str())
        .collect();
    assert!(to.contains(&"frcp::default-entered"), "{to:?}");
}

/// A package without a spanned declaration reports at its bare
/// `Lit.toml`.
#[test]
fn span_less_findings_fall_back_to_the_manifest() {
    let mut p = example();
    p.modules.clear();
    insta::assert_snapshot!(lowering_errors(&p));
}

/// Modules with no spanned declaration sort by name; they add nothing.
#[test]
fn modules_without_spans_sort_by_name() {
    let mut p = example();
    let empty = |name: &str| hir::Module {
        name: name.into(),
        package: p.manifest.name.clone(),
        imports: Vec::new(),
        declarations: Vec::new(),
    };
    let (b, a) = (empty("frcp.b"), empty("frcp.a"));
    let before = lowered(&p);
    p.modules.extend([b, a]);
    assert_eq!(pack_of(&lowered(&p)), pack_of(&before));
}

#[test]
fn cite_syntax_shorthands_must_lower() {
    let mut p = example();
    let shorthand = |prefix: &str, code: &str, title: u32, section_parts: u32| CiteShorthand {
        prefix: prefix.into(),
        code: code.into(),
        title,
        section_parts,
        source: None,
    };
    p.manifest.cite_syntax.shorthands = vec![
        shorthand("UCC", "ucc", 1, 1),
        shorthand("big-title", "usc", 70_000, 1),
        shorthand("many-parts", "cfr", 37, 300),
        shorthand("§", "cfr", 37, 1),
    ];
    insta::assert_snapshot!(lowering_errors(&p));
}

fn instance(patch_edges: BTreeMap<String, hir::EdgePatch>) -> hir::Instance {
    hir::Instance {
        name: "copy".into(),
        pack: "frcp-rule12-example".into(),
        note: None,
        remove_edges: Vec::new(),
        probabilities: BTreeMap::from([("answer".into(), UNDEFINED)]),
        roles: BTreeMap::new(),
        payoff_transform: None,
        stake: None,
        patch_edges,
    }
}

#[test]
fn a_rule_package_holds_no_instances() {
    let mut p = example();
    p.modules[0]
        .declarations
        .push(Declaration::Instance(instance(BTreeMap::new())));
    insta::assert_snapshot!(lowering_errors(&p));
}

#[test]
fn a_links_package_declares_no_nodes() {
    let mut p = example();
    p.manifest.links = true;
    // Other declarations (here a counting rule) are not part of a link file.
    for m in &mut p.modules {
        m.declarations.retain(|d| match d {
            Declaration::Node(n) => n.meta.id == "answered",
            other => matches!(other, Declaration::Counting(_)),
        });
    }
    insta::assert_snapshot!(lowering_errors(&p));
}

/// A patch writing only `note`.
fn note(text: &str) -> hir::EdgePatch {
    hir::EdgePatch {
        note: Some(text.into()),
        ..hir::EdgePatch::default()
    }
}

/// An instance's patches are checked against its base pack, found among
/// the loaded rule packages (never a links package); a patch must name a
/// base edge and an instance probability must have a binary64 value.
#[test]
fn instance_patches_name_base_edges() {
    let base = example();
    let mut links = example();
    links.manifest.links = true;
    links.manifest.pack_id = "links".into();
    links.modules.truncate(1);
    links.modules[0].declarations = vec![Declaration::Instance(instance(BTreeMap::from([
        ("answer".into(), note("Answer it")),
        ("nope".into(), note("Nothing")),
    ])))];
    let found = Lowerer::new(&[&links, &base])
        .package(&links, &Options::default())
        .err()
        .unwrap_or_default();
    insta::assert_snapshot!(render_diagnostics(&found));

    links.modules[0].declarations = vec![Declaration::Instance(hir::Instance {
        probabilities: BTreeMap::new(),
        ..instance(BTreeMap::from([("answer".into(), note("Answer it"))]))
    })];
    let lowered = Lowerer::new(&[&links, &base])
        .package(&links, &Options::default())
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    let Document::Links(file) = lowered.document else {
        panic!("a link file")
    };
    assert_eq!(file.instances.len(), 1);
}

/// A patch replaces each field it writes whole: the members of a base
/// object field it lacks are null, a written empty list or record is
/// empty, and a `cost` hole is null.
#[test]
fn a_patch_replaces_whole_fields() {
    let mut base = example();
    for e in &mut node_mut(&mut base, "respond").edges {
        if let Some(d) = e
            .deadline
            .as_mut()
            .filter(|_| e.id.as_deref() == Some("answer"))
        {
            d.extendable = Some(true);
            d.note = Some("base".into());
        }
    }
    let mut links = example();
    links.manifest.links = true;
    links.manifest.pack_id = "links".into();
    links.modules.truncate(1);
    let patch = hir::EdgePatch {
        hours: Some(hir::Rational::integer(2)),
        cost: Some(hir::Authored::Hole(None)),
        deadline: Some(hir::EdgeDeadline {
            window: hir::DeadlineWindow::Inline {
                length: 30,
                unit: hir::DayUnit::Calendar,
            },
            extendable: None,
            extension_authority: None,
            note: None,
        }),
        tags: Some(Vec::new()),
        attrs: Some(BTreeMap::new()),
        holes: BTreeMap::from([("cost".into(), Some("the fee".into()))]),
        ..note("Answer it")
    };
    links.modules[0].declarations = vec![Declaration::Instance(hir::Instance {
        probabilities: BTreeMap::new(),
        ..instance(BTreeMap::from([("answer".into(), patch)]))
    })];
    let lowered = Lowerer::new(&[&links, &base])
        .package(&links, &Options::default())
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)));
    let Document::Links(file) = lowered.document else {
        panic!("a link file")
    };
    let patched = file
        .instances
        .values()
        .flat_map(|i| i.patch_edges.values())
        .next()
        .cloned();
    insta::assert_json_snapshot!(patched);
    // The cost hole stays visible in the lowered header.
    let holes: Vec<(String, Option<String>)> = file
        .lowered
        .iter()
        .flat_map(|l| l.holes.iter())
        .map(|h| (h.pointer.clone(), h.reason.clone()))
        .collect();
    assert_eq!(
        holes,
        [(
            "/instances/copy/patch_edges/answer/cost".to_owned(),
            Some("the fee".to_owned())
        )]
    );
}

/// Overlapping span-less versions are ambiguous at the declaration's
/// name.
#[test]
fn ambiguous_span_less_versions() {
    let mut p = example();
    node_mut(&mut p, "answered").meta.span = None;
    let twin = node_mut(&mut p, "answered").clone();
    p.modules[0].declarations.push(Declaration::Node(twin));
    for d in decls_mut(&mut p) {
        if let Declaration::Node(n) = d {
            n.edges.retain(|e| e.to != "answered");
        }
    }
    insta::assert_snapshot!(lowering_errors(&p));
}

/// An information set has no litgraph form: a solve could choose
/// differently at decisions the actor cannot tell apart (L6001).
#[test]
fn an_information_set_has_no_lowering() {
    let mut p = example();
    node_mut(&mut p, "respond").info = Some("filing".into());
    insta::assert_snapshot!(lowering_errors(&p));
}
