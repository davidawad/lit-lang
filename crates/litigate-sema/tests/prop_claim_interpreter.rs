//! A small independent oracle for effective-time claim selection.
//!
//! Generated source consists only of well-typed deadlines and relations.
//! The oracle operates on plain date tuples, not HIR or temporal code.
mod legal;

use legal::{pkg, workspace};
use litigate_sema::precedence::conflict::active_claimants;
use litigate_sema::temporal::{Key, World};
use proptest::prelude::*;
use std::fmt::Write as _;

#[derive(Clone, Debug)]
struct Claim {
    start: u16,
    end: u16,
}

fn claims() -> impl Strategy<Value = Vec<Claim>> {
    prop::collection::vec((2009_u16..2027, 2010_u16..2029), 1..8)
        .prop_filter("claim windows are non-empty", |windows| {
            windows.iter().all(|(start, end)| start < end)
        })
        .prop_map(|windows| {
            windows
                .into_iter()
                .map(|(start, end)| Claim { start, end })
                .collect()
        })
}

/// Reference semantics for a finite set of half-open ISO date intervals.
fn reference(claims: &[Claim], year: u16) -> Vec<String> {
    let date = format!("{year:04}-06-01");
    let mut active = vec!["target".to_owned()];
    active.extend(
        claims
            .iter()
            .enumerate()
            .filter(|(_, claim)| {
                let from = format!("{:04}-01-01", claim.start);
                let until = format!("{:04}-01-01", claim.end);
                from.as_str() <= date.as_str() && date.as_str() < until.as_str()
            })
            .map(|(index, _)| format!("c{index}")),
    );
    active.sort();
    active
}

fn program(claims: &[Claim]) -> String {
    let mut source = String::from(
        "module demo.rules\nstart s\nenum Document { complaint }\nevent Served(document: Document)\n\
         counting clock { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n\
         state s: \"S\" { _ \"go\" -> t }\nterminal t: \"T\"\n\
         deadline target: 21.days from Served(complaint) counted_by clock effective [2009-01-01, ..)\n",
    );
    for (index, claim) in claims.iter().enumerate() {
        let _ = writeln!(
            source,
            "deadline c{index}: 7.days from Served(complaint) counted_by clock effective [{:04}-01-01, {:04}-01-01) subject_to target",
            claim.start, claim.end
        );
    }
    source
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn generated_well_typed_claims_match_the_reference_interpreter(
        claims in claims(), year in 2009_u16..2029
    ) {
        let source = program(&claims);
        let package = pkg("demo", "FederalRule", "2009-01-01", &[], &[("rules.lit", &source)]);
        let (db, ws) = workspace(&[package]);
        let checked = litigate_sema::check_workspace(&db, ws);
        let packages: Vec<_> = checked.iter().filter_map(|item| item.package.as_ref()).collect();
        prop_assert_eq!(packages.len(), 1, "generated input must typecheck: {:?}", checked.iter().flat_map(|item| &item.diagnostics).collect::<Vec<_>>());
        let world = World::from_packages(&packages);
        let key = Key { pack: "demo".into(), name: "target".into() };
        let mut actual: Vec<_> = active_claimants(&world, &key, &format!("{year:04}-06-01"))
            .into_iter().map(|index| world.decls[index].meta.id.clone()).collect();
        actual.sort();
        prop_assert_eq!(actual, reference(&claims, year));
    }
}

/// The selection timeline must not depend on source identity or unrelated items.
#[test]
fn renaming_reordering_and_irrelevant_items_preserve_selection_timeline() {
    let claims = vec![
        Claim {
            start: 2010,
            end: 2020,
        },
        Claim {
            start: 2016,
            end: 2025,
        },
    ];
    let original = program(&claims);
    let renamed = original.replace("module demo.rules", "module renamed.package");
    let with_irrelevant = format!("{original}\nenum Unused {{ filing, hearing }}\n");
    let reversed = {
        let mut lines: Vec<_> = original.lines().map(str::to_owned).collect();
        let first = lines
            .iter()
            .position(|line| line.starts_with("deadline target:"))
            .unwrap();
        let rest = lines.split_off(first);
        let (header, deadlines) = rest.split_at(1);
        lines.extend(header.iter().cloned());
        lines.extend(deadlines.iter().rev().cloned());
        lines.join("\n")
    };
    let expected = (2009..2029)
        .map(|year| reference(&claims, year))
        .collect::<Vec<_>>();
    for source in [original, renamed, with_irrelevant, reversed] {
        let package = pkg(
            "demo",
            "FederalRule",
            "2009-01-01",
            &[],
            &[("rules.lit", &source)],
        );
        let (db, ws) = workspace(&[package]);
        let checked = litigate_sema::check_workspace(&db, ws);
        let packages: Vec<_> = checked
            .iter()
            .filter_map(|item| item.package.as_ref())
            .collect();
        assert_eq!(
            packages.len(),
            1,
            "metamorphic input must typecheck: {:?}",
            checked
                .iter()
                .flat_map(|item| &item.diagnostics)
                .collect::<Vec<_>>()
        );
        let world = World::from_packages(&packages);
        let key = Key {
            pack: "demo".into(),
            name: "target".into(),
        };
        let timeline = (2009..2029)
            .map(|year| {
                let mut active = active_claimants(&world, &key, &format!("{year:04}-06-01"))
                    .into_iter()
                    .map(|index| world.decls[index].meta.id.clone())
                    .collect::<Vec<_>>();
                active.sort();
                active
            })
            .collect::<Vec<_>>();
        assert_eq!(timeline, expected);
    }
}
