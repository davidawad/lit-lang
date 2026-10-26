//! `lowered.basis` (design review 2026-10-23 §2.10): packs and link files
//! label each node or edge that declares a `basis`; a scenario carries the
//! join over every number of the packs it uses, with a matter prior's
//! probabilities its own estimate.
mod support;

use litigate_engine::model::{Basis, BasisLabel};
use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{self, Authored, Declaration, Prior, Rational};
use support::{acme, corpus, decls_mut, example, imported_hir, lowered, node_mut, pack_of, usd};

fn label(pointer: &str, basis: Basis, source: Option<&str>) -> BasisLabel {
    BasisLabel {
        pointer: pointer.into(),
        basis,
        source: source.map(str::to_owned),
    }
}

#[test]
fn a_pack_labels_what_declares_a_basis() {
    let mut pkg = example();
    node_mut(&mut pkg, "dismissed").basis = Some(hir::Basis::Sourced {
        cite: "FRCP 41(b)".into(),
    });
    node_mut(&mut pkg, "time-barred").basis = Some(hir::Basis::Estimate);
    let barred = node_mut(&mut pkg, "time-barred");
    barred.edges[0].basis = Some(hir::Basis::Calibrated {
        dataset: "fjc-idb".into(),
        as_of: "2024-09-30".into(),
    });
    node_mut(&mut pkg, "respond").edges[0].basis = Some(hir::Basis::Expert {
        name: "Jane Roe".into(),
    });
    let out = lowered(&pkg);
    let pack = pack_of(&out);
    let node = |id: &str| pack.nodes.iter().position(|n| n.id == id).unwrap_or(99);
    let edge = |id: &str| {
        pack.edges
            .iter()
            .position(|e| e.id.as_deref() == Some(id))
            .unwrap_or(99)
    };
    let mut want = vec![
        label(
            &format!("/nodes/{}", node("dismissed")),
            Basis::Sourced,
            Some("FRCP 41(b)"),
        ),
        label(
            &format!("/nodes/{}", node("time-barred")),
            Basis::Estimate,
            None,
        ),
        label(
            &format!("/edges/{}", edge("barred")),
            Basis::Calibrated,
            Some("fjc-idb@2024-09-30"),
        ),
        label(
            &format!("/edges/{}", edge("answer")),
            Basis::Expert,
            Some("Jane Roe"),
        ),
    ];
    want.sort();
    assert_eq!(pack.lowered.as_ref().map(|l| l.basis.clone()), Some(want));
}

#[test]
fn a_link_file_labels_its_links() {
    let mut packages = imported_hir(&corpus());
    let links = packages
        .iter_mut()
        .map(|(_, p)| p)
        .find(|p| p.manifest.links)
        .unwrap_or_else(|| panic!("no links package"));
    let mut count = 0;
    for d in decls_mut(links) {
        if let Declaration::Link(l) = d {
            l.edge.basis = Some(hir::Basis::Estimate);
            count += 1;
        }
    }
    let links = links.clone();
    let refs: Vec<&hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let out = Lowerer::new(&refs)
        .package(&links, &Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Links(file) = &out.document else {
        panic!("not a link file");
    };
    let labels = file
        .lowered
        .as_ref()
        .map(|l| l.basis.clone())
        .unwrap_or_default();
    assert_eq!(labels.len(), count);
    assert!(labels
        .iter()
        .all(|l| l.basis == Basis::Estimate && l.pointer.starts_with("/links/")));
}

/// The scenario's label for `pkg` and `case`.
fn scenario_basis(pkg: &hir::Package, case: &hir::Case) -> Vec<BasisLabel> {
    Lowerer::new(&[pkg])
        .case(case, "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"))
        .scenario
        .lowered
        .map(|l| l.basis)
        .unwrap_or_default()
}

#[test]
fn a_scenario_joins_every_number_it_rests_on() {
    let at = |basis| vec![label("/scenario", basis, None)];
    // The worked example leaves holes: the motion's odds and a payoff.
    let mut pkg = example();
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Hole));
    // Filled, every unlabelled number is an estimate.
    let half = Authored::Known(Rational {
        numerator: 1,
        denominator: 2,
    });
    for e in &mut node_mut(&mut pkg, "mtd-ruling").edges {
        e.probability = Some(half.clone());
    }
    if let hir::NodeKind::Terminal { payoff, .. } = &mut node_mut(&mut pkg, "judgment-entered").kind
    {
        *payoff = Some(Authored::Known(usd(-100)));
    }
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Estimate));
    // Labelled sourced everywhere, the join is sourced; one calibrated
    // edge makes it calibrated.
    let sourced = hir::Basis::Sourced {
        cite: "FRCP 12".into(),
    };
    for d in decls_mut(&mut pkg) {
        if let Declaration::Node(n) = d {
            n.basis = Some(sourced.clone());
            for e in &mut n.edges {
                e.basis = Some(sourced.clone());
            }
        }
    }
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Sourced));
    // A terminal valued only by flag has numbers too.
    if let hir::NodeKind::Terminal {
        payoff,
        payoff_by_flag,
        ..
    } = &mut node_mut(&mut pkg, "dismissed").kind
    {
        *payoff = None;
        payoff_by_flag.insert("settled".into(), usd(5));
    }
    node_mut(&mut pkg, "dismissed").basis = Some(hir::Basis::Expert {
        name: "Jane Roe".into(),
    });
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Expert));
    // So has one that only shifts fees.
    if let hir::NodeKind::Terminal {
        payoff_by_flag,
        fee_shift,
        ..
    } = &mut node_mut(&mut pkg, "dismissed").kind
    {
        payoff_by_flag.clear();
        *fee_shift = Some(Rational::integer(1));
    }
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Expert));
    node_mut(&mut pkg, "dismissed").basis = Some(sourced.clone());
    node_mut(&mut pkg, "mtd-ruling").edges[0].basis = Some(hir::Basis::Calibrated {
        dataset: "d".into(),
        as_of: "2024-01-01".into(),
    });
    assert_eq!(scenario_basis(&pkg, &acme()), at(Basis::Calibrated));
    // A matter prior over the motion makes its odds the matter's estimate;
    // one over a node with no odds adds nothing.
    let q = |id: &str| format!("frcp-rule12-example::{id}");
    let mut case = acme();
    case.priors = vec![
        Prior {
            node: q("respond"),
            pseudo_counts: Vec::new(),
        },
        Prior {
            node: q("mtd-ruling"),
            pseudo_counts: vec![
                (q("granted"), Rational::integer(1)),
                (q("denied"), Rational::integer(1)),
            ],
        },
    ];
    assert_eq!(scenario_basis(&pkg, &case), at(Basis::Estimate));
    case.priors.remove(1);
    assert_eq!(scenario_basis(&pkg, &case), at(Basis::Calibrated));
}
