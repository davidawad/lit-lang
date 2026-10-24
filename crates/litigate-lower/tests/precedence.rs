//! Lowering emits what precedence selects (SPEC §5.5, §8.1; design review
//! 2026-10-23 §5.1 P1): `within d` takes the claimant that displaces `d`,
//! a displaced node is absent, unordered claimants are L3003, and a
//! package's pack never changes because a downstream package claims its
//! rules. Each case mutates the checked HIR of `lit/examples`.
mod support;

use litigate_base::DiagnosticCode as C;
use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{self, Declaration, Relation};
use support::load_hir;

fn example() -> hir::Package {
    load_hir(&["lit/examples"]).remove(0)
}

fn decls(pkg: &mut hir::Package) -> impl Iterator<Item = &mut Declaration> {
    pkg.modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
}

fn answer_period(pkg: &mut hir::Package) -> &mut hir::Deadline {
    decls(pkg)
        .find_map(|d| match d {
            Declaration::Deadline(x) if x.meta.id == "answer-period" => Some(x),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no answer-period"))
}

/// A copy of `answer-period` named `id`, `days` long, with `relations`.
fn claimant(pkg: &mut hir::Package, id: &str, days: u64, relations: Vec<Relation>) -> Declaration {
    let mut d = answer_period(pkg).clone();
    d.meta.id = id.into();
    d.amount = days;
    d.meta.relations = relations;
    Declaration::Deadline(d)
}

/// The `within` lengths of the lowered pack's edges.
fn windows(packages: &[&hir::Package], pkg: &hir::Package) -> Vec<f64> {
    let lowered = Lowerer::new(packages)
        .package(pkg, &Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Pack(pack) = lowered.document else {
        panic!("expected a pack")
    };
    pack.edges
        .iter()
        .filter_map(|e| e.deadline.as_ref().and_then(|d| d.length))
        .collect()
}

fn errors(pkg: &hir::Package) -> Vec<(C, String)> {
    Lowerer::new(&[pkg])
        .package(pkg, &Options::default())
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|d| (d.code, d.message))
        .collect()
}

#[test]
fn within_takes_the_claimant_that_prevails() {
    let mut p = example();
    assert_eq!(windows(&[&p], &p), [21.0, 21.0]);
    let quick = claimant(
        &mut p,
        "quick",
        14,
        vec![Relation::Notwithstanding("answer-period".into())],
    );
    p.modules[0].declarations.push(quick);
    assert_eq!(windows(&[&p], &p), [14.0, 14.0]);
}

#[test]
fn unordered_claimants_are_an_error_not_a_choice() {
    let mut p = example();
    for id in ["quick", "quicker"] {
        let d = claimant(
            &mut p,
            id,
            14,
            vec![Relation::Notwithstanding("answer-period".into())],
        );
        p.modules[0].declarations.push(d);
    }
    let found = errors(&p);
    assert!(
        found.iter().any(|(c, m)| *c == C::UnresolvedPrecedence
            && m == "no single declaration governs `answer-period` on 2025-01-01"),
        "{found:#?}"
    );
}

#[test]
fn a_displaced_node_is_absent() {
    let mut p = example();
    let mut twin = decls(&mut p)
        .find_map(|d| match d {
            Declaration::Node(n) if n.meta.id == "answered" => Some(n.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no answered"));
    twin.meta.id = "answered-locally".into();
    twin.meta.relations = vec![Relation::Notwithstanding("answered".into())];
    p.modules[0].declarations.push(Declaration::Node(twin));
    let found = errors(&p);
    // `answer` and `denied` both lead to `answered`, which is displaced.
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found
            .iter()
            .all(|(c, m)| *c == C::NoVersionInForce && m.contains("targets `answered`")),
        "{found:#?}"
    );
}

#[test]
fn a_downstream_claim_never_changes_an_upstream_pack() {
    let mut base = example();
    answer_period(&mut base)
        .meta
        .delegations
        .push(hir::Delegation {
            to: "LocalRule".into(),
            ops: vec![hir::DelegatedOp::Set],
            max: None,
            cite: None,
        });
    let mut local = base.clone();
    local.manifest.name = "local-rules".into();
    local.manifest.pack_id = "local-rules".into();
    local.manifest.authority = "LocalRule".into();
    local
        .manifest
        .dependencies
        .insert(base.manifest.name.clone(), "^0".into());
    let quick = claimant(
        &mut local,
        "quick",
        14,
        vec![Relation::Override(format!(
            "{}::answer-period",
            base.manifest.pack_id
        ))],
    );
    for m in &mut local.modules {
        m.declarations.retain(|d| matches!(d, Declaration::Node(_)));
    }
    local.modules[0].declarations.push(quick);
    for d in decls(&mut local) {
        if let Declaration::Node(n) = d {
            for e in &mut n.edges {
                if let Some(hir::EdgeDeadline {
                    window: hir::DeadlineWindow::Declared { deadline },
                    ..
                }) = &mut e.deadline
                {
                    *deadline = format!("{}::answer-period", base.manifest.pack_id);
                }
            }
        }
    }
    // `local-rules` depends on the example, so its own edges see the
    // override; the example's pack does not.
    assert_eq!(windows(&[&base, &local], &base), [21.0, 21.0]);
    assert_eq!(windows(&[&base, &local], &local), [14.0, 14.0]);
}
