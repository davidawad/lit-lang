//! Determinism properties for the compiler's serialized output.
mod support;

use litigate_lower::{render, Document, Lowerer, Options};
use proptest::prelude::*;
use support::load_hir;

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, .. ProptestConfig::default() })]

    #[test]
    fn prop_lowering_is_deterministic(repeat in 0_u8..8) {
        let packages = load_hir(&["lit/examples"]);
        let refs: Vec<_> = packages.iter().collect();
        let mut rendered = Vec::new();
        for _ in 0..=repeat {
            let output = Lowerer::new(&refs).package(&packages[0], &Options::default());
            let lowered = output.unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
            let Document::Pack(pack) = lowered.document else { panic!("expected pack") };
            rendered.push(render(&pack).unwrap_or_else(|error| panic!("{error:?}")));
        }
        prop_assert!(rendered.windows(2).all(|pair| pair[0] == pair[1]));
    }
}
