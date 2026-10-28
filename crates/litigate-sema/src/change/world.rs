//! Worlds (docs/litigate/CHANGE.md §5; design review 2026-10-23 §3.2,
//! §3.4): `W = (Λ, ι, μ)`, a finite set of layers, a location index and a
//! modality, written as a `.world` file. The layers merge as a set (order
//! and repetition in the file mean nothing); precedence comes from the
//! instruments, never from the file. `as_of` and `known_at` are query
//! coordinates, not world fields. A world may also declare choice points
//! (`[choice.ID]`, [`super::choice`]): the alternate worlds it spans.
use super::choice::{self, Choices, Contests};
use super::{families, family_of, scope};
use crate::db::ids::DeclId;
use crate::db::inputs::{PackageInput, Workspace};
use crate::db::queries::{check_decl, check_package, item_tree, manifest};
use crate::db::span::{sort_diagnostics, Report, TextRange};
use crate::hir::{Family, Kind, Manifest, Place, Row};
use crate::pkg::semver::Version;
use litigate_base::{Diagnostic, DiagnosticCode as C, FileId, LineIndex, Profile};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use toml::Spanned;

/// A world's modality (design review §2.5 item 2): `hypothetical` is not
/// a class of file but a world that is not the actual one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// The law as it is: every layer authoritative.
    #[default]
    Actual,
    /// A what-if: any layer, synthetic ones included; never strict.
    Counterfactual,
}

/// The location index ι: where the world's law is applied, so precedent
/// binds per circuit and per state (design review §3.2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// The forum family (a `[family.*]` id of `core`).
    pub family: String,
    /// The court.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court: Option<String>,
    /// The regional circuit (a court id the family lists in `circuits`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit: Option<String>,
    /// The state (an id the family lists in `[family.*.state.*]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The judge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<String>,
}

impl Location {
    /// Where precedent binds: its circuit and state.
    #[must_use]
    pub fn place(&self) -> Place<'_> {
        Place {
            circuit: self.circuit.as_deref(),
            state: self.state.as_deref(),
        }
    }
}

/// A parsed world.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    /// Its modality.
    pub modality: Modality,
    /// Its layers, by package name: a set.
    pub layers: BTreeSet<String>,
    /// Its location.
    pub location: Location,
    /// Its choice points, by id: each a set of named options, an option a
    /// set of layers ([`super::choice`]). Empty: the world is one world.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub choices: Choices,
    /// The choice points that are contested questions, by id, with their
    /// priors ([`super::choice::chances`]).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contested: Contests,
}

/// One layer as the world has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Layer {
    /// Its package name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// Its class (a precedent's with its scope at the world's location).
    pub class: String,
    /// Whether its claims are in force in the world: false for a precedent
    /// that does not bind at the location yet has binding effects, and for
    /// a precedent an overruling or abrogation of the world withdraws.
    pub in_force: bool,
    /// A precedent's date, from which it applies: its designation's (the
    /// decision's where the published opinion is the designation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// The overruling or abrogation withdrawing it, and from when.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overruled_by: Option<Overruled>,
    /// Whether its claims apply at the location, overruled or not: false
    /// only for a precedent that does not bind there yet has binding
    /// effects.
    #[serde(skip)]
    pub applies: bool,
}

impl Layer {
    /// Whether its claims are in force in a matter judged at `as_of` with
    /// the corpus known at `known_at` (`None`: the latest): a precedent
    /// applies from its date, and an overruling or abrogation withdraws it
    /// from its own, retroactively for every case still open (Harper v.
    /// Virginia Dep't of Taxation, 509 U.S. 86 (1993)), so a matter judged
    /// before that date, or at a `known_at` before it, still has it.
    #[must_use]
    pub fn in_force_at(&self, as_of: &str, known_at: Option<&str>) -> bool {
        self.applies
            && self
                .from
                .as_deref()
                .is_none_or(|f| f <= as_of && known_at.is_none_or(|k| f <= k))
            && self.overruled_by.as_ref().is_none_or(|o| {
                o.from.as_str() > as_of || known_at.is_some_and(|k| o.from.as_str() > k)
            })
    }
}

/// How a precedent is withdrawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Withdrawal {
    /// Overruled: by its own court (en banc where it sits in panels) or by
    /// one whose precedent binds it (`overrules`).
    Overruled,
    /// Abrogated: a decision of a court whose precedent binds its court is
    /// clearly irreconcilable with it (`abrogates`; Miller v. Gammie, 335
    /// F.3d 889, 900 (9th Cir. 2003) (en banc)).
    Abrogated,
}

/// An overruling or abrogation of a world's precedent layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Overruled {
    /// The overruling or abrogating package.
    pub by: String,
    /// Whether it overrules or abrogates.
    pub how: Withdrawal,
    /// The overruling's date, from which the precedent's claims are
    /// withdrawn (to every case still open, Harper v. Virginia Dep't of
    /// Taxation, 509 U.S. 86 (1993)).
    pub from: String,
}

/// A checked world file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldCheck {
    /// Its path.
    pub path: String,
    /// The world, when the file parsed.
    pub world: Option<World>,
    /// Its layers, by name.
    pub layers: Vec<Layer>,
    /// Every diagnostic.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorld {
    modality: Option<Spanned<String>>,
    layers: Spanned<Vec<Spanned<String>>>,
    location: Spanned<RawLocation>,
    choice: Option<choice::Raw>,
    contested: Option<choice::RawContests>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLocation {
    family: Spanned<String>,
    court: Option<String>,
    circuit: Option<Spanned<String>>,
    state: Option<Spanned<String>>,
    judge: Option<String>,
}

pub(super) fn range<T>(s: &Spanned<T>) -> TextRange {
    let span = s.span();
    let f = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    TextRange::new(f(span.start), f(span.end))
}

/// Parses a `.world` text: its world and where each layer (a choice
/// point's too) is named.
///
/// # Errors
/// The L1013 reports of a malformed file.
pub fn parse_world(text: &str) -> Result<(World, BTreeMap<String, TextRange>), Vec<Report>> {
    let raw: RawWorld = toml::from_str(text).map_err(|e| {
        let f = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        let at = e.span().map_or(TextRange::default(), |s| {
            TextRange::new(f(s.start), f(s.end))
        });
        vec![Report::new(
            C::InvalidWorld,
            at,
            format!("invalid world file: {}", e.message().trim()),
        )]
    })?;
    let modality = match raw.modality.as_ref().map(|m| m.get_ref().as_str()) {
        None | Some("actual") => Modality::Actual,
        Some("counterfactual") => Modality::Counterfactual,
        Some(other) => {
            let at = raw.modality.as_ref().map(range).unwrap_or_default();
            return Err(vec![Report::new(
                C::InvalidWorld,
                at,
                format!("`modality` is `actual` or `counterfactual`, found `{other}`"),
            )]);
        }
    };
    let mut at: BTreeMap<String, TextRange> = BTreeMap::new();
    for layer in raw.layers.get_ref() {
        // A repeated layer is the same member of the set: the first names it.
        at.entry(layer.get_ref().clone())
            .or_insert_with(|| range(layer));
    }
    if at.is_empty() {
        return Err(vec![Report::new(
            C::InvalidWorld,
            range(&raw.layers),
            "a world has at least one layer",
        )]);
    }
    let l = raw.location.get_ref();
    at.insert("@family".to_owned(), range(&l.family));
    if let Some(c) = &l.circuit {
        at.insert("@circuit".to_owned(), range(c));
    }
    if let Some(s) = &l.state {
        at.insert("@state".to_owned(), range(s));
    }
    if let Some(m) = &raw.modality {
        at.insert("@modality".to_owned(), range(m));
    }
    let layers: BTreeSet<String> = at.keys().filter(|k| !k.starts_with('@')).cloned().collect();
    let choices = match &raw.choice {
        Some(c) => choice::parse(c, &layers, &mut at)?,
        None => Choices::new(),
    };
    let contested = match &raw.contested {
        Some(c) => choice::parse_contested(c, &choices, &mut at)?,
        None => Contests::new(),
    };
    let world = World {
        modality,
        layers,
        choices,
        contested,
        location: Location {
            family: l.family.get_ref().clone(),
            court: l.court.clone(),
            circuit: l.circuit.as_ref().map(|c| c.get_ref().clone()),
            state: l.state.as_ref().map(|s| s.get_ref().clone()),
            judge: l.judge.clone(),
        },
    };
    Ok((world, at))
}

/// The highest loaded version of each package with a valid manifest, by
/// name.
fn loaded(db: &dyn salsa::Database, ws: Workspace) -> BTreeMap<String, (PackageInput, &Manifest)> {
    let mut out: BTreeMap<String, (PackageInput, &Manifest)> = BTreeMap::new();
    for p in ws.packages(db) {
        let Ok(m) = manifest(db, *p) else { continue };
        let m = &m.manifest;
        let newer = out
            .get(&m.name)
            .is_none_or(|(_, old)| Version::parse(&m.version) > Version::parse(&old.version));
        if newer {
            out.insert(m.name.clone(), (*p, m));
        }
    }
    out
}

/// Every effect `pkg`'s declarations have.
fn effects_of(db: &dyn salsa::Database, pkg: PackageInput) -> Row {
    pkg.files(db)
        .iter()
        .flat_map(|f| {
            let n = u32::try_from(item_tree(db, *f).tree.items.len()).unwrap_or(u32::MAX);
            (0..n).map(move |i| check_decl(db, DeclId::new(db, *f, i)).effects)
        })
        .fold(Row::default(), Row::union)
}

/// Checks the world file `text` at `path` against the packages of `ws`
/// under `profile` (docs/litigate/CHANGE.md §5): the world its `layers`
/// make, and every layer its choice points name is a loaded package. The
/// worlds the choices span are checked one by one where they are run
/// ([`super::choice::enumerate`]).
#[must_use]
pub fn check_world(
    db: &dyn salsa::Database,
    ws: Workspace,
    path: &str,
    text: &str,
    profile: Profile,
) -> WorldCheck {
    let file = FileId(path.to_owned());
    let index = LineIndex::new(text);
    let (world, at) = match parse_world(text) {
        Ok(parsed) => parsed,
        Err(reports) => {
            return WorldCheck {
                path: path.to_owned(),
                world: None,
                layers: Vec::new(),
                diagnostics: reports
                    .iter()
                    .map(|r| r.to_diagnostic(&file, &index))
                    .collect(),
            };
        }
    };
    let fixed = World {
        choices: Choices::new(),
        contested: Contests::new(),
        ..world.clone()
    };
    let mut out = check_parsed(db, ws, path, text, &fixed, &at, profile);
    let all = loaded(db, ws);
    let unknown = choice::layers(&world.choices).filter(|name| !all.contains_key(*name));
    out.diagnostics.extend(unknown.map(|name| {
        Report::new(
            C::InvalidWorld,
            at[name],
            format!("layer `{name}` is not a loaded package"),
        )
        .to_diagnostic(&file, &index)
    }));
    out.diagnostics
        .extend(settled(db, &world, &at, &all).map(|r| r.to_diagnostic(&file, &index)));
    sort_diagnostics(&mut out.diagnostics);
    out.world = Some(world);
    out
}

/// L1013 for each contested question of `world` one of whose readings is
/// a precedent binding at its location: a controlling precedent settles
/// the question there, so it is no chance (design review §3.4).
fn settled<'a>(
    db: &'a dyn salsa::Database,
    world: &'a World,
    at: &'a BTreeMap<String, TextRange>,
    all: &'a BTreeMap<String, (PackageInput, &Manifest)>,
) -> impl Iterator<Item = Report> + 'a {
    world.contested.keys().flat_map(move |id| {
        let options = world.choices.get(id).into_iter().flat_map(|o| o.values());
        options.flatten().filter_map(move |name| {
            let (pkg, m) = all.get(name)?;
            let Kind::Precedent { .. } = m.kind else {
                return None;
            };
            let family = families(db, *pkg)
                .iter()
                .find(|f| f.id == world.location.family);
            (scope(m, family, world.location.place()) == crate::hir::Scope::Binding).then(|| {
                Report::new(
                    C::InvalidWorld,
                    at.get(&format!("@contested.{id}"))
                        .copied()
                        .unwrap_or_default(),
                    format!(
                        "precedent `{name}` binds at the world's location: it settles \
                         contested question `{id}` there"
                    ),
                )
            })
        })
    })
}

/// Checks `world`, parsed from `text` at `path` (`at`: where each of its
/// layers and location keys is named), against the packages of `ws` under
/// `profile`.
#[must_use]
pub fn check_parsed(
    db: &dyn salsa::Database,
    ws: Workspace,
    path: &str,
    text: &str,
    world: &World,
    at: &BTreeMap<String, TextRange>,
    profile: Profile,
) -> WorldCheck {
    let file = FileId(path.to_owned());
    let index = LineIndex::new(text);
    let mut out = WorldCheck {
        path: path.to_owned(),
        world: None,
        layers: Vec::new(),
        diagnostics: Vec::new(),
    };
    let mut reports: Vec<Report> = Vec::new();
    let all = loaded(db, ws);
    let layers: Vec<(PackageInput, &Manifest)> = world
        .layers
        .iter()
        .filter_map(|name| {
            let found = all.get(name).copied();
            if found.is_none() {
                reports.push(Report::new(
                    C::InvalidWorld,
                    at[name],
                    format!("layer `{name}` is not a loaded package"),
                ));
            }
            found
        })
        .collect();
    let families: &[Family] = layers.first().map_or(&[], |(p, _)| families(db, *p).as_slice());
    let family = location(world, at, families, &mut reports);
    if world.modality == Modality::Counterfactual && profile == Profile::Strict {
        reports.push(Report::new(
            C::NotActualLaw,
            at.get("@modality").copied().unwrap_or_default(),
            "a counterfactual world is never checked under the strict profile: its law is not \
             the law in force",
        ));
    }
    let mut located: Vec<Diagnostic> = Vec::new();
    let overrulings = overrulings(db, &layers, families, &mut located);
    let here = Here { world, at, family };
    let (checked, fits): (Vec<Layer>, Vec<bool>) = layers
        .iter()
        .map(|&(pkg, m)| here.layer(db, pkg, m, overrulings.get(&m.name), &mut reports))
        .unzip();
    out.layers = checked;
    out.diagnostics = reports
        .iter()
        .map(|r| r.to_diagnostic(&file, &index))
        .collect();
    out.diagnostics.extend(located);
    // A precedent that does not bind here has no claim in force here, so
    // it conflicts with nothing (design review §3.3, S1).
    let visible: Vec<(PackageInput, &Manifest)> = layers
        .iter()
        .zip(fits)
        .filter_map(|(l, fits)| fits.then_some(*l))
        .collect();
    out.diagnostics.extend(merged(db, &visible));
    sort_diagnostics(&mut out.diagnostics);
    out.world = Some(world.clone());
    out
}

/// What a world's layers are checked against: the world, where its names
/// are written, and its location's family.
struct Here<'w> {
    world: &'w World,
    at: &'w BTreeMap<String, TextRange>,
    family: Option<&'w Family>,
}

impl Here<'_> {
    /// Layer `m` (package `pkg`) as the world has it, and whether its
    /// effects fit its class here: L1014 for a synthetic layer in an
    /// actual world, L9009 for a precedent that does not bind here yet has
    /// binding effects.
    fn layer(
        &self,
        db: &dyn salsa::Database,
        pkg: PackageInput,
        m: &Manifest,
        overruled_by: Option<&Overruled>,
        reports: &mut Vec<Report>,
    ) -> (Layer, bool) {
        let name = &m.name;
        let authoritative = m.instrument.as_ref().is_none_or(|i| i.authoritative);
        if !authoritative && self.world.modality == Modality::Actual {
            reports.push(Report::new(
                C::NotActualLaw,
                self.at[name],
                format!(
                    "layer `{name}` is not authoritative (a synthetic what-if): it may appear \
                     only in a counterfactual world"
                ),
            ));
        }
        let (kind, fits) = match &m.kind {
            Kind::Precedent { .. } => {
                let s = scope(m, self.family, self.world.location.place());
                let kind = Kind::Precedent { scope: s };
                let fits = effects_of(db, pkg).within(kind.row());
                if !fits {
                    reports.push(not_binding(self.world, m, self.at[name], self.family));
                }
                (kind, fits)
            }
            other => (other.clone(), true),
        };
        let from = matches!(m.kind, Kind::Precedent { .. })
            .then(|| m.instrument.as_ref())
            .flatten()
            .map(|i| i.designated.clone().unwrap_or_else(|| i.date.clone()));
        let layer = Layer {
            name: name.clone(),
            version: m.version.clone(),
            class: kind.label().to_owned(),
            in_force: fits && overruled_by.is_none(),
            from,
            overruled_by: overruled_by.cloned(),
            applies: fits,
        };
        (layer, fits)
    }
}

/// The location's family, checking its circuit (L1013).
fn location<'f>(
    world: &World,
    at: &BTreeMap<String, TextRange>,
    families: &'f [Family],
    reports: &mut Vec<Report>,
) -> Option<&'f Family> {
    let l = &world.location;
    let Some(family) = families.iter().find(|f| f.id == l.family) else {
        reports.push(
            Report::new(
                C::InvalidWorld,
                at["@family"],
                format!("`{}` is not a forum family of `core`'s matrix", l.family),
            )
            .note(format!(
                "the families: {}",
                families
                    .iter()
                    .map(|f| format!("`{}`", f.id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
        return None;
    };
    let needs = family.precedent.iter().any(|p| p == "@circuit");
    match &l.circuit {
        None if needs => reports.push(Report::new(
            C::InvalidWorld,
            at["@family"],
            format!(
                "a world in the `{}` family names its `circuit`: circuit precedent binds per \
                 circuit there",
                family.id
            ),
        )),
        Some(c) if !family.circuits.contains(c) => reports.push(Report::new(
            C::InvalidWorld,
            at["@circuit"],
            if family.circuits.is_empty() {
                format!("the `{}` family has no regional circuits", family.id)
            } else {
                format!(
                    "`{c}` is not a circuit of the `{}` family ({})",
                    family.id,
                    family.circuits.join(", ")
                )
            },
        )),
        _ => {}
    }
    let needs = family.precedent.iter().any(|p| p == "@state");
    match &l.state {
        None if needs => reports.push(Report::new(
            C::InvalidWorld,
            at["@family"],
            format!(
                "a world in the `{}` family names its `state`: a state's courts bind only in it",
                family.id
            ),
        )),
        Some(s) if !family.states.iter().any(|x| &x.id == s) => reports.push(Report::new(
            C::InvalidWorld,
            at["@state"],
            if family.states.is_empty() {
                format!("the `{}` family has no states", family.id)
            } else {
                format!(
                    "`{s}` is not a state of the `{}` family ({})",
                    family.id,
                    family
                        .states
                        .iter()
                        .map(|x| x.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        )),
        _ => {}
    }
    Some(family)
}

/// L9009: a precedent that does not bind at the location, with effects
/// only binding precedent has.
fn not_binding(world: &World, m: &Manifest, at: TextRange, family: Option<&Family>) -> Report {
    let i = m.instrument.as_ref();
    let court = i.and_then(|i| i.court.as_deref()).unwrap_or_default();
    let why = if i.is_some_and(|i| i.designated.is_none()) {
        "it was never designated precedential".to_owned()
    } else {
        let l = &world.location;
        format!(
            "`{court}` precedent does not bind in the `{}` family{}",
            l.family,
            l.circuit
                .iter()
                .chain(&l.state)
                .fold(String::new(), |at, c| at + " at `" + c + "`")
        )
    };
    let mut r = Report::new(
        C::PrecedentNotBinding,
        at,
        format!(
            "precedent `{}` is persuasive in this world ({why}): its binding effects are not in \
             force here",
            m.name
        ),
    );
    if let Some(f) = family {
        r = r.note(format!(
            "precedent binding in `{}`: {}",
            f.id,
            f.precedent.join(", ")
        ));
    }
    r
}

/// The precedents the overrulings and abrogations of `layers` withdraw,
/// by name, with the earliest valid one and its date. L2010, in the
/// withdrawing package's `Lit.toml`, for a target that is not a
/// precedent; an overruling by a court that is neither the precedent's
/// own (en banc, where it sits in panels: Miller v. Gammie, 335 F.3d 889,
/// 899-900 (9th Cir. 2003) (en banc)) nor one whose precedent binds it;
/// or an abrogation by a court whose precedent does not bind it.
fn overrulings(
    db: &dyn salsa::Database,
    layers: &[(PackageInput, &Manifest)],
    families: &[Family],
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, Overruled> {
    let by_name: BTreeMap<&str, &Manifest> =
        layers.iter().map(|(_, m)| (m.name.as_str(), *m)).collect();
    let mut out: BTreeMap<String, Overruled> = BTreeMap::new();
    for (pkg, o) in layers {
        let decision = matches!(o.kind, Kind::Overruling | Kind::Precedent { .. });
        let Some(i) = o.instrument.as_ref().filter(|_| decision) else {
            continue;
        };
        let court = i.court.as_deref().unwrap_or_default();
        let text = pkg.manifest_text(db);
        let targets = i
            .overrules
            .iter()
            .filter(|_| o.kind == Kind::Overruling)
            .map(|t| (t, Withdrawal::Overruled))
            .chain(i.abrogates.iter().map(|t| (t, Withdrawal::Abrogated)));
        for (target, how) in targets {
            let key = match how {
                Withdrawal::Overruled => "overrules",
                Withdrawal::Abrogated => "abrogates",
            };
            let at = super::key_range(text, key);
            let mut error = |message: String| {
                diagnostics.push(
                    Report::new(C::ForbiddenEffect, at, message)
                        .to_diagnostic(&FileId(pkg.manifest_path(db)), &LineIndex::new(text)),
                );
            };
            let Some(p) = by_name.get(target.as_str()) else {
                continue;
            };
            let Some(pi) = p
                .instrument
                .as_ref()
                .filter(|_| matches!(p.kind, Kind::Precedent { .. }))
            else {
                error(format!(
                    "`{}` {key} `{target}`, which is not a precedent",
                    o.name
                ));
                continue;
            };
            let theirs = pi.court.as_deref().unwrap_or_default();
            let family = family_of(families, &p.forum);
            if let Some(why) = unwithdrawable(family, how, (court, i.en_banc), theirs, target) {
                error(why);
                continue;
            }
            // Withdrawn from the earliest: a later one finds it already
            // withdrawn (ties go to the first name).
            let earlier = |old: &Overruled| (&i.date, &o.name) < (&old.from, &old.by);
            if out.get(target).is_none_or(earlier) {
                out.insert(
                    target.clone(),
                    Overruled {
                        by: o.name.clone(),
                        how,
                        from: i.date.clone(),
                    },
                );
            }
        }
    }
    out
}

/// Why a decision of `court` (`en_banc`: sitting en banc) cannot withdraw
/// `target`, a precedent of `theirs` in `family`, the way `how` says:
/// `None` when it can.
fn unwithdrawable(
    family: Option<&Family>,
    how: Withdrawal,
    (court, en_banc): (&str, bool),
    theirs: &str,
    target: &str,
) -> Option<String> {
    let higher = family.is_some_and(|f| court != theirs && f.binds(court, f.seat(theirs)));
    let own = court == theirs && (en_banc || !family.is_some_and(|f| f.sits_in_panels(theirs)));
    match how {
        Withdrawal::Overruled if !(own || higher) => Some(if court == theirs {
            format!(
                "a `{court}` panel cannot overrule `{target}`: a panel is bound by its court's \
                 prior panels until the court en banc (`en_banc = true`) or a court whose \
                 precedent binds it overrules them"
            )
        } else {
            format!(
                "`{court}` cannot overrule `{theirs}` precedent `{target}`: an overruling is by \
                 the same court (en banc) or one whose precedent binds it"
            )
        }),
        Withdrawal::Abrogated if !higher => Some(format!(
            "`{court}` cannot abrogate `{theirs}` precedent `{target}`: only a decision of a \
             court whose precedent binds `{theirs}` abrogates it (its own court overrules)"
        )),
        _ => None,
    }
}

/// The temporal and precedence diagnostics the layers have together but
/// none has alone: conflicts the set merge makes (design review §3.4).
fn merged(db: &dyn salsa::Database, layers: &[(PackageInput, &Manifest)]) -> Vec<Diagnostic> {
    let mut packages: Vec<(PackageInput, &Manifest)> = Vec::new();
    let mut own: Vec<usize> = Vec::new();
    for &(pkg, m) in layers {
        for (p, pm) in crate::precedence::query::closure(db, pkg, m) {
            let i = packages
                .iter()
                .position(|(q, _)| *q == p)
                .unwrap_or_else(|| {
                    packages.push((p, pm));
                    packages.len() - 1
                });
            if p == pkg {
                own.push(i);
            }
        }
    }
    let alone: Vec<&Diagnostic> = layers
        .iter()
        .flat_map(|(p, _)| check_package(db, *p).diagnostics.iter())
        .collect();
    crate::precedence::query::checks_over(db, &packages, &own)
        .into_iter()
        .filter(|d| !alone.contains(&d))
        .collect()
}
