//! Determinism properties for the compiler's serialized output.
mod support;

use litigate_lower::{render, Document, Lowerer, Options};
use proptest::prelude::*;
use support::load_hir;

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, .. ProptestConfig::default() })]

    #[test]
    fn prop_lowering_ignores_loaded_package_order(reverse in any::<bool>()) {
        let packages = load_hir(&["lit/examples"]);
        let refs: Vec<_> = packages.iter().collect();
        let mut permuted = refs.clone();
        if reverse { permuted.reverse(); }
        let lower = |inputs: &[&litigate_sema::hir::Package]| {
            let output = Lowerer::new(inputs).package(&packages[0], &Options::default());
            let lowered = output.unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
            let Document::Pack(pack) = lowered.document else { panic!("expected pack") };
            render(&pack).unwrap_or_else(|error| panic!("{error:?}"))
        };
        prop_assert_eq!(lower(&refs), lower(&permuted));
    }
}
