//! Matter dates must remain validated when the case is supplied as HIR.
mod case_support;

use case_support::{acme, check_example, examples, examples_clock, lock};
use litigate_base::Profile;
use litigate_sema::case::{check, resolve, CaseSpans, Env};
use litigate_sema::hir::Package;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_invalid_as_of_dates_are_rejected(day in 29_u32..=31) {
        let packages = examples();
        let refs: Vec<&Package> = packages.iter().collect();
        let lock = lock(packages);
        let resolved = resolve("cases/examples/acme.case", &acme(), &refs, Some(&lock));
        let mut matter = resolved.case.unwrap_or_else(|| panic!("{:#?}", resolved.diagnostics));
        matter.as_of = format!("2025-02-{day:02}");
        let checked = check(
            matter,
            &CaseSpans::default(),
            Env { packages: &refs, lock: Some(&lock), clock: examples_clock(), profile: Profile::Dev },
        );
        prop_assert!(checked.diagnostics.iter().any(|diagnostic| diagnostic.code.as_str() == "L0004"));
    }

    #[test]
    fn prop_same_day_events_keep_source_order(day in 1_u32..=28) {
        let date = format!("2025-03-{day:02}");
        let source = acme()
            .replace("2025-03-05 Served", &format!("{date} Served"))
            .replace("2025-03-20 MovedToDismiss", &format!("{date} MovedToDismiss"));
        let checked = check_example(&source, Profile::Dev);
        prop_assert!(!checked.diagnostics.iter().any(|diagnostic| diagnostic.code.as_str() == "L4001"));
        prop_assert!(checked.trace.len() >= 3, "{checked:#?}");
        prop_assert!(checked.trace[1].event.starts_with("Served"), "{checked:#?}");
        prop_assert!(checked.trace[2].event.starts_with("MovedToDismiss"), "{checked:#?}");
    }
}
