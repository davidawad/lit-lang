//! Declared `subject_to` relations keep every effective claimant in play.
mod legal;

use legal::{pkg, render, with_world, workspace};
use litigate_sema::precedence::{
    beats,
    conflict::{active_claimants, claimants},
    Lattice, Reason,
};
use litigate_sema::temporal::{Key, World};
use proptest::prelude::*;

const PACKAGE: &str = "module demo.rules
start s
enum Document { complaint }
event Served(document: Document)
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
deadline target: 21.days from Served(complaint) counted_by c6 effective [2009-12-01, ..)
deadline a: 14.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) subject_to target
deadline b: 7.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) subject_to target
";

const VERSIONED_PACKAGE: &str = "module demo.rules
start s
enum Document { complaint }
event Served(document: Document)
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
deadline target: 21.days from Served(complaint) counted_by c6 effective [2009-12-01, 2015-01-01)
deadline target: 22.days from Served(complaint) counted_by c6 effective [2015-01-01, ..)
deadline a: 14.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) subject_to target@2015-01-01
deadline superseder: 10.days from Served(complaint) counted_by c6 effective [2015-01-01, ..) supersedes target@2015-01-01
";

const UNRESOLVED_PACKAGE: &str = "module demo.rules
start s
enum Document { complaint }
event Served(document: Document)
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
deadline target: 21.days from Served(complaint) counted_by c6 effective [2009-12-01, 2015-01-01)
deadline a: 14.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) subject_to target
deadline b: 7.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) subject_to target
";

fn pair_result(a_relation: &str, b_relation: &str) -> Option<(bool, Reason)> {
    let source = format!(
        "module demo.rules\nstart s\nenum Document {{ complaint }}\nevent Served(document: Document)\n\
         counting c6 {{ holidays cal }}\ncalendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}\n\
         state s: \"S\" {{ _ \"go\" -> t }}\nterminal t: \"T\"\n\
         deadline a: 14.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) {a_relation}\n\
         deadline b: 7.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) {b_relation}\n"
    );
    let package = pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("rules.lit", &source)],
    );
    let (db, ws) = workspace(&[package]);
    with_world(&db, ws, |world| {
        let index = |name: &str| {
            world
                .decls
                .iter()
                .position(|declaration| declaration.meta.id == name)
                .unwrap_or_else(|| panic!("missing declaration {name}"))
        };
        beats(world, &Lattice::new(&world.ranks), index("a"), index("b"))
    })
}

proptest! {
    #[test]
    fn prop_subject_to_claimants_remain_active(year in 2010_i32..=2030) {
        let source = pkg("demo", "FederalRule", "2009-12-01", &[], &[("rules.lit", PACKAGE)]);
        let (db, ws) = workspace(&[source]);
        let target = Key { pack: "demo".into(), name: "target".into() };
        let date = format!("{year}-06-01");
        let count = with_world(&db, ws, |world| active_claimants(world, &target, &date).len());
        prop_assert_eq!(count, 3);
    }

    #[test]
    fn prop_versioned_claims_apply_only_to_the_named_target_version(after_amendment in any::<bool>()) {
        let package = pkg("demo", "FederalRule", "2009-12-01", &[], &[("rules.lit", VERSIONED_PACKAGE)]);
        let (db, ws) = workspace(&[package]);
        let target = Key { pack: "demo".into(), name: "target".into() };
        let date = if after_amendment { "2016-06-01" } else { "2014-06-01" };
        let count = with_world(&db, ws, |world| active_claimants(world, &target, date).len());
        prop_assert_eq!(count, if after_amendment { 2 } else { 1 });
    }

    #[test]
    fn prop_non_claim_relations_do_not_add_claimants(_seed in 0_u8..1) {
        let package = pkg("demo", "FederalRule", "2009-12-01", &[], &[("rules.lit", VERSIONED_PACKAGE)]);
        let (db, ws) = workspace(&[package]);
        let checks = litigate_sema::check_workspace(&db, ws);
        let packages: Vec<_> = checks.iter().filter_map(|check| check.package.as_ref()).collect();
        let world = World::from_packages(&packages);
        let target = Key { pack: "demo".into(), name: "target".into() };
        prop_assert_eq!(claimants(&world, &target).len(), 3);
    }

    #[test]
    fn prop_unresolved_precedence_points_to_the_relation_clause(_seed in 0_u8..1) {
        let package = pkg("demo", "FederalRule", "2009-12-01", &[], &[("rules.lit", UNRESOLVED_PACKAGE)]);
        let (db, ws) = workspace(&[package]);
        let diagnostics = render(&db, ws);
        let relation_span = diagnostics.iter().any(|diagnostic| {
            let mut pieces = diagnostic.split(':');
            let file = pieces.next().unwrap_or_default();
            let line = pieces.next().unwrap_or_default();
            let column = pieces
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .parse::<usize>()
                .unwrap_or_default();
            file.ends_with("rules.lit") && line == "11" && column == 84 && diagnostic.contains("L3003")
        });
        prop_assert!(relation_span, "{diagnostics:#?}");
    }

    #[test]
    fn prop_declared_relation_pairs_resolve_without_cycles(case in 0_u8..3) {
        let (a, b, expected) = match case {
            0 => ("subject_to b", "", Some((false, Reason::Declared("subject_to")))),
            1 => ("notwithstanding b", "", Some((true, Reason::Declared("notwithstanding")))),
            _ => ("", "", None),
        };
        prop_assert_eq!(pair_result(a, b), expected);
    }
}
