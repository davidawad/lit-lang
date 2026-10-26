//! Choice points of a world (docs/litigate/CHANGE.md §5.1; design review
//! 2026-10-23 §3.4, §6 items 23 and 24): `[choice.ID]` tables of a
//! `.world` file, each a set of named options, an option a set of layers.
//! The world spans one alternate world per combination, one option taken
//! at every choice point: its own layers plus the options'. A choice
//! between leaving an instrument out and adding it is a change toggled;
//! a choice among readings of unsettled law (a circuit split) is a
//! contested question, one reading per world.
//!
//! The enumeration is exhaustive and finite: the product of the option
//! counts, at most [`MAX_WORLDS`]. Each world is the stratified model the
//! checker already computes (SPEC §0), so no solver is needed to list the
//! worlds; an answer-set solver would recompute the same product.
use super::world::{check_parsed, range, World, WorldCheck};
use crate::db::inputs::Workspace;
use crate::db::span::{Report, TextRange};
use litigate_base::{DiagnosticCode as C, Profile};
use serde::Serialize;
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
                ..world.clone()
            };
            Alternate {
                id: format!("w{}", i + 1),
                check: check_parsed(db, ws, path, text, &one, at, profile),
                combination,
            }
        })
        .collect()
}
