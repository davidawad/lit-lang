//! Fee shifting (lg-vwwr): a terminal's `fee_shift` lowers to the pack's
//! `feeShift`, and a stake's to `scenario.fee_shifts`.
mod support;

use litigate_lower::Lowerer;
use litigate_sema::hir::{self, Rational};
use serde_json::json;
use support::{acme, example, lowered, node_mut, pack_of};

fn half() -> Rational {
    Rational {
        numerator: 1,
        denominator: 2,
    }
}

#[test]
fn a_terminals_fee_shift_lowers_to_fee_shift_on_the_pack() {
    let mut pkg = example();
    if let hir::NodeKind::Terminal { fee_shift, .. } = &mut node_mut(&mut pkg, "dismissed").kind {
        *fee_shift = Some(half());
    }
    let low = lowered(&pkg);
    let pack = pack_of(&low);
    let shift = |id: &str| {
        pack.nodes
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.fee_shift)
    };
    assert_eq!(shift("dismissed"), Some(0.5));
    assert_eq!(shift("judgment-entered"), None);
}

#[test]
fn a_stakes_fee_shift_lowers_to_the_scenarios_fee_shifts() {
    let pkg = example();
    let lower = |case: &hir::Case| {
        let named = Lowerer::new(&[&pkg])
            .case(case, "cases/examples/acme.case")
            .unwrap_or_else(|d| panic!("{d:#?}"));
        serde_json::to_value(&named.scenario).expect("serializes")
    };
    assert_eq!(lower(&acme())["scenario"].get("fee_shifts"), None);
    let mut case = acme();
    case.fee_shifts
        .insert("frcp-rule12-example::dismissed".into(), half());
    assert_eq!(
        lower(&case)["scenario"]["fee_shifts"],
        json!({ "frcp-rule12-example::dismissed": 0.5 })
    );
}
