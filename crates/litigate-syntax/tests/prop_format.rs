//! Formatting remains stable across every corpus source kind.
mod support;

use litigate_syntax::format;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]

    #[test]
    fn prop_corpus_formatting_is_idempotent(index in any::<usize>()) {
        let corpus = support::corpus();
        let (path, kind) = &corpus[index % corpus.len()];
        let source = std::fs::read_to_string(path)
            .map_err(|error| TestCaseError::fail(format!("{}: {error}", path.display())))?;
        let once = format(&source, *kind)
            .map_err(|error| TestCaseError::fail(format!("{}: {error}", path.display())))?;
        let twice = format(&once, *kind)
            .map_err(|error| TestCaseError::fail(format!("{}: {error}", path.display())))?;
        prop_assert_eq!(once, twice);
    }
}
