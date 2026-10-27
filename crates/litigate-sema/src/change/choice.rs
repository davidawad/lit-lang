//! Choice points of a world (docs/litigate/CHANGE.md §5.1; design review
//! 2026-10-23 §3.4, §6 items 23 and 24): `[choice.ID]` tables of a
//! `.world` file, each a set of named options, an option a set of layers.
//! The world spans one alternate world per combination, one option taken
//! at every choice point: its own layers plus the options'. A choice
//! between leaving an instrument out and adding it is a change toggled;
//! a choice among readings of unsettled law (a circuit split) is a
//! contested question, one reading per world.
//!
//! A choice point the file also declares `[contested.ID]` is a contested
//! question: its options are drawn, not chosen, so it lowers to a chance
//! node ([`chances`]) whose branches weigh its options by a
//! basis-labelled prior; the uncontested choice points stay alternate
//! worlds ([`timelines`]), each weighing the worlds its draws reach.
//!
//! The enumeration is exhaustive and finite: the product of the option
//! counts, at most [`MAX_WORLDS`]. Each world is the stratified model the
//! checker already computes (SPEC §0), so no solver is needed to list the
//! worlds; an answer-set solver would recompute the same product.
use super::world::{check_parsed, range, World, WorldCheck};
use crate::db::inputs::Workspace;
use crate::db::span::{Report, TextRange};
use litigate_base::{DiagnosticCode as C, Profile};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use toml::Spanned;

/// The most worlds one file's choice points may span.
pub const MAX_WORLDS: usize = 4096;

/// Choice points by id: each option's layers by option name.
pub type Choices = BTreeMap<String, BTreeMap<String, BTreeSet<String>>>;

/// `[choice.ID]` tables as written.
pub(super) type Raw = BTreeMap<String, Spanned<BTreeMap<String, Spanned<Vec<Spanned<String>>>>>>;

/// One alternate world: the option taken at each choice point and the
/// layers that makes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Combination {
    /// The option taken, by choice id.
    pub choices: BTreeMap<String, String>,
    /// The world's layers: its own and the options'.
    pub layers: BTreeSet<String>,
}

/// One alternate world, checked.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Alternate {
    /// Its id: `w1`, `w2`, ... in combination order.
    pub id: String,
    /// Its choices and layers.
    pub combination: Combination,
    /// The world with these layers, checked.
    pub check: WorldCheck,
}

/// Every layer some option of `choices` names, once each.
pub fn layers(choices: &Choices) -> impl Iterator<Item = &String> {
    choices
        .values()
        .flat_map(|options| options.values().flatten())
        .collect::<BTreeSet<_>>()
        .into_iter()
}

/// Parses the `[choice.ID]` tables `raw` of a world whose own layers are
/// `fixed`, recording where each option's layer is named in `at` (the
/// first naming), and each choice at `@choice.ID`.
///
/// # Errors
/// The L1013 reports of a malformed choice: fewer than two options, an
/// option naming one of the world's own layers or another choice's, two
/// options with the same layers, or more than [`MAX_WORLDS`] worlds.
pub(super) fn parse(
    raw: &Raw,
    fixed: &BTreeSet<String>,
    at: &mut BTreeMap<String, TextRange>,
) -> Result<Choices, Vec<Report>> {
    let mut out = Choices::new();
    let mut reports: Vec<Report> = Vec::new();
    // The choice each layer belongs to.
    let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
    let mut worlds: usize = 1;
    for (id, options) in raw {
        let here = range(options);
        let options = options.get_ref();
        if options.len() < 2 {
            reports.push(Report::new(
                C::InvalidWorld,
                here,
                format!(
                    "choice `{id}` has {} option(s): a choice point has at least two",
                    options.len()
                ),
            ));
        }
        let mut parsed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (label, names) in options {
            let mut set = BTreeSet::new();
            for name in names.get_ref() {
                let layer = name.get_ref();
                let other = owner.get(layer.as_str()).copied();
                if fixed.contains(layer) {
                    reports.push(Report::new(
                        C::InvalidWorld,
                        range(name),
                        format!(
                            "layer `{layer}` is in every world (`layers`): choice `{id}` cannot \
                             leave it out"
                        ),
                    ));
                } else if let Some(other) = other.filter(|o| o != id) {
                    reports.push(Report::new(
                        C::InvalidWorld,
                        range(name),
                        format!(
                            "layer `{layer}` belongs to choice `{other}`: a layer is in one choice \
                             point, so each world is one combination"
                        ),
                    ));
                } else {
                    owner.insert(layer, id);
                    at.entry(layer.clone()).or_insert_with(|| range(name));
                    set.insert(layer.clone());
                }
            }
            if let Some(same) = parsed.iter().find(|(_, s)| **s == set).map(|(l, _)| l) {
                reports.push(Report::new(
                    C::InvalidWorld,
                    range(names),
                    format!("options `{same}` and `{label}` of choice `{id}` hold the same layers"),
                ));
            }
            parsed.insert(label.clone(), set);
        }
        worlds = worlds.saturating_mul(options.len());
        at.insert(format!("@choice.{id}"), here);
        out.insert(id.clone(), parsed);
    }
    if worlds > MAX_WORLDS {
        let first = raw.values().next().map(range).unwrap_or_default();
        reports.push(Report::new(
            C::InvalidWorld,
            first,
            format!("the choice points span {worlds} worlds, more than {MAX_WORLDS}"),
        ));
    }
    if reports.is_empty() {
        Ok(out)
    } else {
        Err(reports)
    }
}

/// Every combination of `world`'s choice points, in order: by choice id,
/// then option name. A world without choices is its one combination.
#[must_use]
pub fn combinations(world: &World) -> Vec<Combination> {
    let mut out = vec![Combination {
        choices: BTreeMap::new(),
        layers: world.layers.clone(),
    }];
    for (id, options) in &world.choices {
        out = out
            .iter()
            .flat_map(|c| {
                options.iter().map(move |(label, layers)| {
                    let mut next = c.clone();
                    next.choices.insert(id.clone(), label.clone());
                    next.layers.extend(layers.iter().cloned());
                    next
                })
            })
            .collect();
    }
    out
}

/// Every world `world` (parsed from `text` at `path`, its names at `at`)
/// spans, each checked against the packages of `ws` under `profile`.
#[must_use]
pub fn enumerate(
    db: &dyn salsa::Database,
    ws: Workspace,
    path: &str,
    text: &str,
    world: &World,
    at: &BTreeMap<String, TextRange>,
    profile: Profile,
) -> Vec<Alternate> {
    combinations(world)
        .into_iter()
        .enumerate()
        .map(|(i, combination)| {
            let one = World {
                layers: combination.layers.clone(),
                choices: Choices::new(),
                contested: Contests::new(),
                ..world.clone()
            };
            Alternate {
                id: world_id(i),
                check: check_parsed(db, ws, path, text, &one, at, profile),
                combination,
            }
        })
        .collect()
}

/// The bases a contested question's prior may have (design review
/// §2.10): read from a source, measured on a dataset, an expert's, or an
/// estimate.
pub const BASES: [&str; 4] = ["sourced", "calibrated", "expert", "estimate"];

/// The prior of a contested question (design review §3.4, §6 item 23): a
/// choice point whose options are readings of unsettled law at the
/// world's location, drawn by the tribunal rather than chosen by anyone.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contest {
    /// Its basis, one of [`BASES`].
    pub basis: String,
    /// What the basis rests on (a cite, `dataset@date`, an expert); none
    /// for an estimate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Each option's Dirichlet pseudo-count, by option name: every option
    /// of the choice, positive; 1 each when the file gives none.
    pub prior: BTreeMap<String, u64>,
}

/// Contested questions by choice id.
pub type Contests = BTreeMap<String, Contest>;

/// A `[contested.ID]` table as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawContest {
    basis: Option<Spanned<String>>,
    source: Option<Spanned<String>>,
    prior: Option<Spanned<BTreeMap<String, Spanned<i64>>>>,
}

/// `[contested.ID]` tables as written.
pub(super) type RawContests = BTreeMap<String, Spanned<RawContest>>;

/// Parses the `[contested.ID]` tables `raw` of a world whose choice points
/// are `choices`, recording each at `@contested.ID` in `at`.
///
/// # Errors
/// The L1013 reports of a malformed one: not a choice point, an unknown
/// basis, a source missing (or given for an estimate), or a prior that is
/// not a positive pseudo-count for exactly each option.
pub(super) fn parse_contested(
    raw: &RawContests,
    choices: &Choices,
    at: &mut BTreeMap<String, TextRange>,
) -> Result<Contests, Vec<Report>> {
    let mut out = Contests::new();
    let mut reports: Vec<Report> = Vec::new();
    let mut bad = |at: TextRange, message: String| {
        reports.push(Report::new(C::InvalidWorld, at, message));
    };
    for (id, table) in raw {
        let here = range(table);
        at.insert(format!("@contested.{id}"), here);
        let Some(options) = choices.get(id) else {
            bad(
                here,
                format!("`contested.{id}` names no choice point: declare `[choice.{id}]`"),
            );
            continue;
        };
        let table = table.get_ref();
        let basis = table
            .basis
            .as_ref()
            .map_or("estimate", |b| b.get_ref().as_str());
        let basis_at = table.basis.as_ref().map_or(here, range);
        if !BASES.contains(&basis) {
            bad(
                basis_at,
                format!("`basis` is one of {}, found `{basis}`", BASES.join(", ")),
            );
        }
        let source = table.source.as_ref().map(|s| s.get_ref().clone());
        match (&source, basis) {
            (Some(_), "estimate") => bad(
                table.source.as_ref().map_or(here, range),
                format!("contested question `{id}` is an estimate: it has no `source`"),
            ),
            (None, b) if b != "estimate" && BASES.contains(&b) => bad(
                basis_at,
                format!("contested question `{id}` has basis `{b}`: name its `source`"),
            ),
            _ => {}
        }
        let prior = prior_of(id, options, table.prior.as_ref(), &mut bad);
        out.insert(
            id.clone(),
            Contest {
                basis: basis.to_owned(),
                source,
                prior,
            },
        );
    }
    if reports.is_empty() {
        Ok(out)
    } else {
        Err(reports)
    }
}

/// The prior of contested question `id` over `options`, `given` as
/// written (1 each when absent), each malformed pseudo-count reported to
/// `bad`.
fn prior_of(
    id: &str,
    options: &BTreeMap<String, BTreeSet<String>>,
    given: Option<&Spanned<BTreeMap<String, Spanned<i64>>>>,
    bad: &mut impl FnMut(TextRange, String),
) -> BTreeMap<String, u64> {
    let mut prior: BTreeMap<String, u64> = options.keys().map(|o| (o.clone(), 1)).collect();
    let Some(given) = given else {
        return prior;
    };
    for option in options.keys().filter(|o| !given.get_ref().contains_key(*o)) {
        bad(
            range(given),
            format!("the prior of `{id}` gives option `{option}` no pseudo-count"),
        );
    }
    for (option, alpha) in given.get_ref() {
        if !options.contains_key(option) {
            bad(
                range(alpha),
                format!("choice `{id}` has no option `{option}`"),
            );
        } else if let Ok(a @ 1..) = u64::try_from(*alpha.get_ref()) {
            prior.insert(option.clone(), a);
        } else {
            bad(
                range(alpha),
                format!(
                    "the pseudo-count of `{id}` option `{option}` is a positive integer, found {}",
                    alpha.get_ref()
                ),
            );
        }
    }
    prior
}

/// The id of the world at `index` of [`combinations`].
fn world_id(index: usize) -> String {
    format!("w{}", index + 1)
}

/// One branch of a chance node: an option of a contested question, its
/// probability, and the worlds that take it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Branch {
    /// The option.
    pub option: String,
    /// Its pseudo-count.
    pub alpha: u64,
    /// Its probability, exact: `alpha / Σ alpha`, reduced, as `n/d`.
    pub exact: String,
    /// Its probability.
    pub p: f64,
    /// The worlds taking it, by id.
    pub worlds: Vec<String>,
}

/// A contested question lowered to a chance node (a `chance discretion`
/// draw by the tribunal, design review §2.11, §3.4): its branches are its
/// options, each weighted by its prior's mean.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chance {
    /// Its node id: `choice.ID`.
    pub node: String,
    /// The choice point.
    pub choice: String,
    /// Its prior's basis.
    pub basis: String,
    /// What the basis rests on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Its branches, in option order.
    pub branches: Vec<Branch>,
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// `n / d` as a probability (exact text and nearest float).
#[allow(clippy::cast_precision_loss)]
fn ratio(n: u64, d: u64) -> (String, f64) {
    let g = gcd(n, d).max(1);
    (format!("{}/{}", n / g, d / g), n as f64 / d as f64)
}

/// The chance node of each contested question of `world`, its branches'
/// worlds those [`combinations`] (and [`enumerate`]) give the ids of.
/// An uncontested choice point lowers to no node: its options stay
/// alternate worlds ([`timelines`]).
#[must_use]
pub fn chances(world: &World) -> Vec<Chance> {
    let combos = combinations(world);
    world
        .contested
        .iter()
        .filter_map(|(id, contest)| {
            let options = world.choices.get(id)?;
            let total = contest
                .prior
                .values()
                .fold(0u64, |s, a| s.saturating_add(*a));
            let branches = options
                .keys()
                .map(|option| {
                    let alpha = contest.prior.get(option).copied().unwrap_or(1);
                    let (exact, p) = ratio(alpha, total);
                    let worlds = combos
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.choices.get(id) == Some(option))
                        .map(|(i, _)| world_id(i))
                        .collect();
                    Branch {
                        option: option.clone(),
                        alpha,
                        exact,
                        p,
                        worlds,
                    }
                })
                .collect();
            Some(Chance {
                node: format!("choice.{id}"),
                choice: id.clone(),
                basis: contest.basis.clone(),
                source: contest.source.clone(),
                branches,
            })
        })
        .collect()
}

/// One world of the uncontested choices: the option taken at each, and
/// the alternate worlds its contested questions' draws reach, each with
/// the probability of its draws.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Timeline {
    /// The option taken at each uncontested choice point.
    pub choices: BTreeMap<String, String>,
    /// Each world it holds, by id, with its probability in the timeline.
    pub worlds: Vec<(String, f64)>,
}

/// The worlds of `world`'s uncontested choice points, in combination
/// order; each holds every combination of its contested ones, weighted by
/// their [`chances`]. Without a contested question each is one world, of
/// probability 1.
#[must_use]
pub fn timelines(world: &World) -> Vec<Timeline> {
    let chances = chances(world);
    let p_of = |choice: &str, option: &str| {
        chances
            .iter()
            .find(|c| c.choice == choice)
            .and_then(|c| c.branches.iter().find(|b| b.option == option))
            .map_or(1.0, |b| b.p)
    };
    let mut out: Vec<Timeline> = Vec::new();
    for (i, combo) in combinations(world).iter().enumerate() {
        let (contested, fixed): (BTreeMap<_, _>, BTreeMap<_, _>) = combo
            .choices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .partition(|(k, _)| world.contested.contains_key(k));
        let p = contested.iter().map(|(k, v)| p_of(k, v)).product();
        match out.iter_mut().find(|t| t.choices == fixed) {
            Some(t) => t.worlds.push((world_id(i), p)),
            None => out.push(Timeline {
                choices: fixed,
                worlds: vec![(world_id(i), p)],
            }),
        }
    }
    out
}
