//! A small independent oracle for layered overrides (SPEC §5.5).
//!
//! Generated packages stack `LocalRule` and `StandingOrder` overrides of
//! one delegated `FederalRule` deadline. The oracle reads plain year
//! windows: a higher authority prevails, and within one layer the later
//! `from` does. It never sees HIR, the lattice or the conflict code.
mod legal;

use legal::{pkg, render, with_world, workspace, CORE};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::precedence::{conflict::winner, Lattice};
use litigate_sema::temporal::Key;
use proptest::prelude::*;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// One override: in force from January 1 of `start` until January 1 of
/// `end`, or open-ended.
#[derive(Clone, Debug)]
struct Override {
    start: u16,
    end: Option<u16>,
}

/// The two delegated layers, highest authority first.
const LAYERS: [(&str, &str); 2] = [("local", "LocalRule"), ("standing", "StandingOrder")];

/// Overrides of one layer: distinct starts, so the layer never ties
/// (equal starts are an L3003 the checker reports, not a selection).
fn layer() -> impl Strategy<Value = Vec<Override>> {
    prop::collection::btree_set(2010_u16..2030, 0..4).prop_flat_map(|starts| {
        let starts: Vec<u16> = starts.into_iter().collect();
        let ends = prop::collection::vec(prop::option::of(1_u16..8), starts.len());
        (Just(starts), ends).prop_map(|(starts, ends)| {
            starts
                .into_iter()
                .zip(ends)
                .map(|(start, span)| Override {
                    start,
                    end: span.map(|span| start + span),
                })
                .collect()
        })
    })
}

/// Reference semantics: who prevails over `frcp::answer-period` on June 1
/// of `year`.
fn reference(layers: &[Vec<Override>; 2], year: u16) -> String {
    // The base rule is in force from 2009-12-01.
    if year < 2010 {
        return "none".to_owned();
    }
    for ((pack, _), overrides) in LAYERS.iter().zip(layers) {
        let latest = overrides
            .iter()
            .enumerate()
            .filter(|(_, o)| o.start <= year && o.end.is_none_or(|end| year < end))
            .max_by_key(|(_, o)| o.start);
        if let Some((index, _)) = latest {
            return format!("{pack}::o{index}");
        }
    }
    "frcp::answer-period".to_owned()
}

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

fn core() -> LoadedPackage {
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", CORE), ("start.lit", START)],
    )
}

fn frcp() -> LoadedPackage {
    let text = "module frcp.rules\nimport core.authority.{Statute, LocalRule, StandingOrder, CaseOrder}\nstart s\nenum Document { complaint }\nevent Served(document: Document)\ncounting c6 { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\nstate s: \"S\"\ndeadline answer-period: 21.days from Served(complaint) counted_by c6 effective [2009-12-01, ..) delegable { to [LocalRule, StandingOrder], op set }\n";
    pkg(
        "frcp",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", text)],
    )
}

/// The overrides of one layer as declarations, in `order`.
fn declarations(overrides: &[Override], order: &[usize]) -> String {
    let mut body = String::new();
    for &index in order {
        let o = &overrides[index];
        let end = o
            .end
            .map_or("..".to_owned(), |end| format!("{end:04}-01-01"));
        let _ = writeln!(
            body,
            "deadline o{index}: 14.days from Filed(complaint) counted_by c6 effective [{:04}-01-01, {end}) override answer-period",
            o.start
        );
    }
    body
}

/// A layer package `name` of `authority` holding `body`.
fn layer_package(name: &str, authority: &str, body: &str) -> LoadedPackage {
    let text = format!(
        "module {name}.rules\nimport frcp.rules.{{answer-period, c6}}\nstart l\nenum Paper {{ complaint }}\nevent Filed(paper: Paper)\nstate l: \"L\"\n{body}"
    );
    pkg(
        name,
        authority,
        "2009-12-01",
        &["core", "frcp"],
        &[("rules.lit", &text)],
    )
}

/// The whole workspace: base packages, then each layer, declarations in
/// source order.
fn packages(layers: &[Vec<Override>; 2]) -> Vec<LoadedPackage> {
    let mut all = vec![core(), frcp()];
    for ((name, authority), overrides) in LAYERS.iter().zip(layers) {
        let order: Vec<usize> = (0..overrides.len()).collect();
        all.push(layer_package(
            name,
            authority,
            &declarations(overrides, &order),
        ));
    }
    all
}

/// The engine's winner over `frcp::answer-period` on June 1 of each year
/// 2009-2039.
fn timeline(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    with_world(&db, ws, |world| {
        let lattice = Lattice::new(&world.ranks);
        let target = Key {
            pack: "frcp".into(),
            name: "answer-period".into(),
        };
        (2009..2040)
            .map(
                |year| match winner(world, &lattice, &target, &format!("{year:04}-06-01")) {
                    Ok(w) => format!("{}::{}", world.pack_of(w), world.decls[w].meta.id),
                    Err(pairs) if pairs.is_empty() => "none".to_owned(),
                    Err(pairs) => format!("unresolved {pairs:?}"),
                },
            )
            .collect()
    })
}

fn expected(layers: &[Vec<Override>; 2]) -> Vec<String> {
    (2009..2040).map(|year| reference(layers, year)).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn generated_layered_overrides_match_the_reference_interpreter(
        local in layer(), standing in layer()
    ) {
        let layers = [local, standing];
        prop_assert_eq!(timeline(&packages(&layers)), expected(&layers));
    }

    /// Permuting packages and declarations, renaming a layer's module and
    /// adding a layer that overrides nothing leave the timeline unchanged.
    #[test]
    fn layer_order_names_and_irrelevant_layers_preserve_the_timeline(
        local in layer(), standing in layer(), seed in any::<u64>()
    ) {
        let layers = [local, standing];
        let want = expected(&layers);
        let mut permuted = vec![core(), frcp()];
        for ((name, authority), overrides) in LAYERS.iter().zip(&layers).rev() {
            let mut order: Vec<usize> = (0..overrides.len()).collect();
            let shift = usize::try_from(seed).unwrap_or_default() % order.len().max(1);
            order.rotate_left(shift);
            order.reverse();
            let body = declarations(overrides, &order);
            permuted.push(layer_package(name, authority, &body));
        }
        permuted.reverse();
        prop_assert_eq!(timeline(&permuted), want.clone());

        let mut renamed = packages(&layers);
        for file in &mut renamed[2].files {
            file.text = file.text.replace("module local.rules", "module local.renamed");
            file.relative = "renamed.lit".to_owned();
        }
        prop_assert_eq!(timeline(&renamed), want.clone());

        let mut widened = packages(&layers);
        widened.push(layer_package(
            "chambers",
            "CaseOrder",
            "deadline unrelated: 3.days from Filed(complaint) counted_by c6 effective [2012-01-01, ..)\n",
        ));
        prop_assert_eq!(timeline(&widened), want);
    }
}

/// Hand-picked differential cases: an empty stack, a lower layer alone, a
/// higher layer shadowing it, an expiry handing back to the lower layer
/// and then to the base rule, and a later start within one layer.
#[test]
fn differential_cases_match_the_reference_interpreter() {
    let o = |start, end| Override { start, end };
    let cases: [[Vec<Override>; 2]; 5] = [
        [vec![], vec![]],
        [vec![], vec![o(2012, None)]],
        [vec![o(2014, None)], vec![o(2012, None)]],
        [vec![o(2014, Some(2016))], vec![o(2012, Some(2020))]],
        [vec![o(2011, None), o(2015, Some(2018))], vec![]],
    ];
    let mut seen = BTreeSet::new();
    for layers in &cases {
        let got = timeline(&packages(layers));
        assert_eq!(got, expected(layers), "{layers:?}");
        seen.extend(got);
    }
    // The cases reach every outcome the oracle distinguishes.
    for outcome in [
        "none",
        "frcp::answer-period",
        "local::o0",
        "local::o1",
        "standing::o0",
    ] {
        assert!(seen.contains(outcome), "{outcome} never selected");
    }
}
