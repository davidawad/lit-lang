//! Strict `Lit.toml` parsing and validation (SPEC §4.1). Unknown keys,
//! missing required keys, and ill-typed or invalid values are all L1004.
use super::semver::{Requirement, Version};
use crate::db::span::{Report, TextRange};
use crate::hir::{
    ActorRole, CiteSyntax, Holding, Instrument, Interval, Kind, Law, Manifest, Scope, SourceBinding,
};
use crate::types::date;
use litigate_base::DiagnosticCode as C;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use toml::Spanned;

/// A TOML table deserialized in document order.
#[derive(Debug)]
struct Ordered<T>(Vec<(String, T)>);

impl<T> Default for Ordered<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ordered<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = Ordered<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a table")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    out.push(entry);
                }
                Ok(Ordered(out))
            }
        }
        deserializer.deserialize_map(V(PhantomData))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    package: RawPackage,
    #[serde(default)]
    dependencies: Ordered<Spanned<RawDependency>>,
    #[serde(default)]
    sources: Ordered<Spanned<RawSource>>,
    cite_syntax: Option<Spanned<cite_syntax::RawCiteSyntax>>,
    #[serde(default)]
    family: Ordered<Spanned<family::RawFamily>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackage {
    name: Spanned<String>,
    version: Spanned<String>,
    language: Spanned<i64>,
    // Pack identity: required in a pack, absent in a links package.
    pack_id: Option<Spanned<String>>,
    forum: Option<Spanned<String>>,
    // The forum family of `core`'s matrix the package's forum is in.
    family: Option<Spanned<String>>,
    authority: Option<Spanned<String>>,
    title: Option<Spanned<String>>,
    roles: Option<Spanned<Ordered<Spanned<String>>>>,
    description: Option<Spanned<String>>,
    jurisdiction: Option<Spanned<String>>,
    // The US state, DC or territory of a state-court pack (`litigate coverage`).
    jurisdiction_id: Option<Spanned<String>>,
    counting: Option<Spanned<String>>,
    // Dependencies whose periods `counting` counts in a matter using this
    // package (a court's own counting rule over a national rule set's).
    recounts: Option<Spanned<Vec<String>>>,
    // Procedure (default) or substantive law (SPEC §4.1, §5.6).
    law: Option<Spanned<String>>,
    in_force: Spanned<String>,
    in_force_until: Option<Spanned<String>>,
    known_at: Spanned<String>,
    // The date before which no declaration's law is encoded (SPEC §4.1).
    unknown_before: Option<Spanned<String>>,
    // The instrument class and the instrument justifying it (SPEC §4.1).
    kind: Option<Spanned<String>>,
    instrument: Option<Spanned<RawInstrument>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInstrument {
    source: Spanned<String>,
    issuer: Spanned<String>,
    date: Spanned<String>,
    designation: Option<String>,
    binds: Option<Spanned<Vec<String>>>,
    court: Option<Spanned<String>>,
    designated: Option<Spanned<String>>,
    overrules: Option<Spanned<Vec<String>>>,
    reach: Option<Spanned<Vec<String>>>,
    holdings: Option<Spanned<Vec<Spanned<RawHolding>>>>,
    en_banc: Option<Spanned<bool>>,
    abrogates: Option<Spanned<Vec<String>>>,
    authoritative: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHolding {
    holding: Spanned<String>,
    pin: Option<Spanned<String>>,
    edits: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDependency {
    version: Spanned<String>,
    path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSource {
    title: Option<String>,
    url: Option<String>,
    path: Option<Spanned<String>>,
    sha256: Option<Spanned<String>>,
    path_sha256: Option<Spanned<String>>,
    as_of: Option<Spanned<String>>,
    #[serde(default)]
    cite_prefixes: Vec<String>,
    section_heading: Option<Spanned<String>>,
}

/// A validated manifest plus the dependency paths the loader follows.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedManifest {
    /// HIR manifest.
    pub manifest: Manifest,
    /// Dependency name to its `path`, when given.
    pub dependency_paths: BTreeMap<String, String>,
}

/// True for an identifier per SPEC §2 (letters, digits, `_`, inner `-`).
#[must_use]
pub fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let start_ok = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_');
    start_ok
        && !text.ends_with('-')
        && !text.contains("--")
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// True for a kebab-case package name or pack id.
#[must_use]
pub fn is_kebab(text: &str) -> bool {
    !text.is_empty()
        && text.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
        && text.starts_with(|c: char| c.is_ascii_lowercase())
}

fn range<T>(s: &Spanned<T>) -> TextRange {
    let span = s.span();
    let at = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    TextRange::new(at(span.start), at(span.end))
}

struct Checker {
    reports: Vec<Report>,
}

impl Checker {
    fn check(&mut self, ok: bool, at: TextRange, message: impl FnOnce() -> String) {
        if !ok {
            self.reports
                .push(Report::new(C::InvalidManifest, at, message()));
        }
    }

    fn date(&mut self, key: &str, value: &Spanned<String>) {
        self.check(date::is_valid(value.get_ref()), range(value), || {
            format!(
                "`{key}` must be a YYYY-MM-DD date, found `{}`",
                value.get_ref()
            )
        });
    }
}

/// Parses and validates `Lit.toml` text.
///
/// # Errors
/// Every problem as an L1004 report located in the manifest text.
pub fn parse_manifest(text: &str) -> Result<ParsedManifest, Vec<Report>> {
    let raw: RawManifest = toml::from_str(text).map_err(|e| {
        let at = e.span().map_or(TextRange::default(), |s| {
            let f = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
            TextRange::new(f(s.start), f(s.end))
        });
        vec![Report::new(
            C::InvalidManifest,
            at,
            format!("invalid Lit.toml: {}", e.message().trim()),
        )]
    })?;
    let mut c = Checker {
        reports: Vec::new(),
    };
    let p = &raw.package;
    let links = identity(&mut c, text, &raw);
    package_table(&mut c, p);
    let empty = Ordered::default();
    let roles = roles(&mut c, p.roles.as_ref().map_or(&empty, |r| r.get_ref()));
    let (dependencies, dependency_paths) = dependencies(&mut c, &raw.dependencies);
    let sources: Vec<SourceBinding> = raw
        .sources
        .0
        .iter()
        .map(|(id, s)| source(&mut c, id, s))
        .collect();
    let cite_syntax = raw
        .cite_syntax
        .as_ref()
        .map_or_else(CiteSyntax::default, |cs| {
            cite_syntax::check(&mut c, cs.get_ref(), &raw.sources)
        });
    counting(&mut c, p, &dependencies);
    let law = law(&mut c, p);
    let (kind, instrument) = kind(&mut c, p, &roles, &sources);
    let families = family::check(&mut c, p.name.get_ref(), &raw.family);
    if !c.reports.is_empty() {
        return Err(c.reports);
    }
    let text_of = |s: &Option<Spanned<String>>| s.as_ref().map(|s| s.get_ref().clone());
    let manifest = Manifest {
        name: p.name.get_ref().clone(),
        version: p.version.get_ref().clone(),
        language: 1,
        pack_id: text_of(&p.pack_id).unwrap_or_default(),
        forum: text_of(&p.forum).unwrap_or_default(),
        family: text_of(&p.family),
        authority: text_of(&p.authority).unwrap_or_default(),
        title: text_of(&p.title).unwrap_or_default(),
        description: text_of(&p.description),
        jurisdiction: text_of(&p.jurisdiction),
        jurisdiction_id: text_of(&p.jurisdiction_id),
        roles,
        in_force: in_force(p),
        sources,
        dependencies,
        counting: text_of(&p.counting),
        recounts: p
            .recounts
            .as_ref()
            .map(|r| r.get_ref().clone())
            .unwrap_or_default(),
        cite_syntax,
        links,
        kind,
        instrument,
        families,
        law,
    };
    Ok(ParsedManifest {
        manifest,
        dependency_paths,
    })
}

/// The package's `in_force` interval from its date fields.
fn in_force(p: &RawPackage) -> Interval {
    Interval {
        from: p.in_force.get_ref().clone(),
        to: p.in_force_until.as_ref().map(|s| s.get_ref().clone()),
        known_at: p.known_at.get_ref().clone(),
        retracted: None,
        unknown_before: p.unknown_before.as_ref().map(|s| s.get_ref().clone()),
        commenced: None,
        repealed: false,
    }
}

/// `package.counting` must be a rule id; `package.recounts` needs that
/// rule and may name only dependencies.
fn counting(c: &mut Checker, p: &RawPackage, dependencies: &BTreeMap<String, String>) {
    if let Some(counting) = &p.counting {
        c.check(is_ident(counting.get_ref()), range(counting), || {
            format!(
                "`counting` must be a counting-rule id, found `{}`",
                counting.get_ref()
            )
        });
    }
    if let Some(recounts) = &p.recounts {
        c.check(p.counting.is_some(), range(recounts), || {
            "`recounts` needs the package's own `counting` rule".to_owned()
        });
        for name in recounts.get_ref() {
            c.check(dependencies.contains_key(name), range(recounts), || {
                format!("`recounts` names `{name}`, which is not a dependency")
            });
        }
    }
}

/// `package.law`: `procedure` (the default) or `substantive`.
fn law(c: &mut Checker, p: &RawPackage) -> Law {
    let Some(law) = &p.law else {
        return Law::Procedure;
    };
    match law.get_ref().as_str() {
        "procedure" => Law::Procedure,
        "substantive" => Law::Substantive,
        other => {
            c.check(false, range(law), || {
                format!("`law` must be `procedure` or `substantive`, found `{other}`")
            });
            Law::Procedure
        }
    }
}

fn package_table(c: &mut Checker, p: &RawPackage) {
    c.check(is_kebab(p.name.get_ref()), range(&p.name), || {
        format!("package name `{}` must be kebab-case", p.name.get_ref())
    });
    c.check(
        Version::parse(p.version.get_ref()).is_some(),
        range(&p.version),
        || {
            format!(
                "`{}` is not a SemVer version (MAJOR.MINOR.PATCH[-pre])",
                p.version.get_ref()
            )
        },
    );
    c.check(*p.language.get_ref() == 1, range(&p.language), || {
        "`language` must be 1".to_owned()
    });
    if let Some(id) = &p.pack_id {
        c.check(is_kebab(id.get_ref()), range(id), || {
            format!("pack id `{}` must be kebab-case", id.get_ref())
        });
    }
    for (key, value) in [
        ("forum", &p.forum),
        ("authority", &p.authority),
        ("jurisdiction_id", &p.jurisdiction_id),
    ] {
        if let Some(value) = value {
            c.check(is_ident(value.get_ref()), range(value), || {
                format!("`{key}` must be an identifier, found `{}`", value.get_ref())
            });
        }
    }
    if let Some(family) = &p.family {
        c.check(is_kebab(family.get_ref()), range(family), || {
            format!(
                "`family` must be a forum-family id such as `district-civil`, found `{}`",
                family.get_ref()
            )
        });
    }
    c.date("in_force", &p.in_force);
    c.date("known_at", &p.known_at);
    if let Some(before) = &p.unknown_before {
        c.date("unknown_before", before);
    }
    if let Some(until) = &p.in_force_until {
        c.date("in_force_until", until);
        c.check(until.get_ref() > p.in_force.get_ref(), range(until), || {
            "`in_force_until` must be after `in_force`".to_owned()
        });
    }
}

/// Pack identity (SPEC §4.1): a pack's manifest has every identity key; a
/// links package's has none of them, nor any other pack-only key. Returns
/// whether the manifest is a links manifest.
fn identity(c: &mut Checker, text: &str, raw: &RawManifest) -> bool {
    let p = &raw.package;
    let at = text.find("[package]").map_or(TextRange::default(), |s| {
        let f = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        TextRange::new(f(s), f(s + "[package]".len()))
    });
    let required = [
        ("pack_id", p.pack_id.is_some()),
        ("forum", p.forum.is_some()),
        ("authority", p.authority.is_some()),
        ("title", p.title.is_some()),
        ("package.roles", p.roles.is_some()),
    ];
    if required.iter().any(|(_, present)| *present) {
        for (key, present) in required {
            c.check(present, at, || {
                format!("invalid Lit.toml: missing field `{key}`")
            });
        }
        return false;
    }
    let pack_only = [
        ("description", p.description.as_ref().map(range)),
        ("jurisdiction", p.jurisdiction.as_ref().map(range)),
        ("jurisdiction_id", p.jurisdiction_id.as_ref().map(range)),
        ("counting", p.counting.as_ref().map(range)),
        ("recounts", p.recounts.as_ref().map(range)),
        ("law", p.law.as_ref().map(range)),
        ("cite_syntax", raw.cite_syntax.as_ref().map(range)),
        ("sources", raw.sources.0.first().map(|(_, s)| range(s))),
        ("kind", p.kind.as_ref().map(range)),
        ("package.instrument", p.instrument.as_ref().map(range)),
    ];
    for (key, found) in pack_only {
        if let Some(found) = found {
            c.check(false, found, || {
                format!(
                    "`{key}` needs a pack identity: a links package's manifest has \
                     only `name`, `version`, `language`, `in_force`, `known_at`, and \
                     `[dependencies]` (add `pack_id` if this is a pack)"
                )
            });
        }
    }
    true
}

fn roles(c: &mut Checker, table: &Ordered<Spanned<String>>) -> BTreeMap<String, ActorRole> {
    let mut roles = BTreeMap::new();
    for (actor, role) in &table.0 {
        let parsed = match role.get_ref().as_str() {
            "self" => Some(ActorRole::Protagonist),
            "opponent" => Some(ActorRole::Opponent),
            "nature" => Some(ActorRole::Nature),
            _ => None,
        };
        c.check(parsed.is_some(), range(role), || {
            format!("role of `{actor}` must be \"self\", \"opponent\", or \"nature\"")
        });
        // `either` is a valid key: the implicit actor's role (SPEC §4.3).
        c.check(is_ident(actor), range(role), || {
            format!("`{actor}` is not a valid actor name")
        });
        roles.insert(actor.clone(), parsed.unwrap_or(ActorRole::Nature));
    }
    roles
}

type Dependencies = (BTreeMap<String, String>, BTreeMap<String, String>);

fn dependencies(c: &mut Checker, table: &Ordered<Spanned<RawDependency>>) -> Dependencies {
    let mut dependencies = BTreeMap::new();
    let mut dependency_paths = BTreeMap::new();
    for (name, dep) in &table.0 {
        let d = dep.get_ref();
        c.check(is_kebab(name), range(dep), || {
            format!("dependency name `{name}` must be kebab-case")
        });
        c.check(
            Requirement::parse(d.version.get_ref()).is_some(),
            range(&d.version),
            || {
                format!(
                    "`{}` is not a requirement (`^`, `~`, `>=`, or `=` plus a version)",
                    d.version.get_ref()
                )
            },
        );
        dependencies.insert(name.clone(), d.version.get_ref().clone());
        if let Some(path) = &d.path {
            dependency_paths.insert(name.clone(), path.clone());
        }
    }
    (dependencies, dependency_paths)
}

fn source(c: &mut Checker, id: &str, s: &Spanned<RawSource>) -> SourceBinding {
    let raw = s.get_ref();
    c.check(is_ident(id), range(s), || {
        format!("source id `{id}` must be an identifier")
    });
    c.check(raw.url.is_some() || raw.path.is_some(), range(s), || {
        format!("source `{id}` needs `url`, `path`, or both")
    });
    if let Some(path) = &raw.path {
        let p = path.get_ref();
        let relative = !p.starts_with('/')
            && !p.contains('\\')
            && !p.split('/').any(|seg| seg == ".." || seg.is_empty());
        c.check(relative, range(path), || {
            format!("source path `{p}` must be repository-relative")
        });
    }
    for (key, sha) in [("sha256", &raw.sha256), ("path_sha256", &raw.path_sha256)] {
        if let Some(sha) = sha {
            let ok = sha.get_ref().len() == 64
                && sha
                    .get_ref()
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
            c.check(ok, range(sha), || {
                format!("`{key}` must be 64 lowercase hex digits")
            });
        }
    }
    if let Some(sha) = &raw.path_sha256 {
        c.check(raw.path.is_some(), range(sha), || {
            format!("source `{id}` has `path_sha256` but no `path` to pin")
        });
    }
    if let Some(as_of) = &raw.as_of {
        c.date("as_of", as_of);
    }
    c.check(
        raw.cite_prefixes.iter().all(|p| !p.trim().is_empty()),
        range(s),
        || format!("source `{id}` has an empty cite prefix"),
    );
    if let Some(h) = &raw.section_heading {
        c.check(h.get_ref().contains("{n}"), range(h), || {
            "`section_heading` must contain `{n}`".to_owned()
        });
    }
    SourceBinding {
        id: id.to_owned(),
        title: raw.title.clone(),
        url: raw.url.clone(),
        path: raw.path.as_ref().map(|p| p.get_ref().clone()),
        sha256: raw.sha256.as_ref().map(|p| p.get_ref().clone()),
        path_sha256: raw.path_sha256.as_ref().map(|p| p.get_ref().clone()),
        as_of: raw.as_of.as_ref().map(|p| p.get_ref().clone()),
        cite_prefixes: raw.cite_prefixes.clone(),
        section_heading: raw.section_heading.as_ref().map(|p| p.get_ref().clone()),
    }
}

/// The instrument class and its justification (SPEC §4.1, design review
/// 2026-10-23 §2.5): a class other than `base` needs the instrument that
/// makes it one ([`instrument`]); a modification layer's class names its
/// package's authority; guidance names the actors it binds;
/// `hypothetical` is a world modality.
fn kind(
    c: &mut Checker,
    p: &RawPackage,
    roles: &BTreeMap<String, ActorRole>,
    sources: &[SourceBinding],
) -> (Kind, Option<Instrument>) {
    let binds = p
        .instrument
        .as_ref()
        .and_then(|i| i.get_ref().binds.as_ref());
    let name = p.kind.as_ref().map_or("base", |n| n.get_ref().as_str());
    let kind = match Kind::from_name(name) {
        Some(Kind::Guidance { .. }) => Kind::Guidance {
            binds: binds.map(|b| b.get_ref().clone()).unwrap_or_default(),
        },
        // Binding scope is computed from the court hierarchy (the forum
        // families of `core`, a world's location), never declared: the
        // manifest alone makes a precedent persuasive, failing closed.
        Some(Kind::Precedent { .. }) => Kind::Precedent {
            scope: Scope::Persuasive,
        },
        Some(kind) => kind,
        None => {
            let why = if name == "hypothetical" {
                "`hypothetical` is the modality of a world, not a file kind: a counterfactual world may hold a layer of any kind".to_owned()
            } else {
                format!(
                    "`kind` must be one of {}, found `{name}`",
                    Kind::NAMES.map(|n| format!("`{n}`")).join(", ")
                )
            };
            c.check(
                false,
                p.kind.as_ref().map(range).unwrap_or_default(),
                || why,
            );
            return (Kind::Base, None);
        }
    };
    let at = p.kind.as_ref().map(range).unwrap_or_default();
    if let Some(level) = kind.level() {
        let authority = p.authority.as_ref().map(|a| a.get_ref().as_str());
        c.check(authority == Some(level), at, || {
            format!(
                "a `{}` package has authority `{level}`, found `{}`",
                kind.name(),
                authority.unwrap_or_default()
            )
        });
    }
    let guidance = matches!(kind, Kind::Guidance { .. });
    match binds {
        Some(b) if !guidance => c.check(false, range(b), || {
            "`binds` names the addressees of guidance only".to_owned()
        }),
        Some(b) => {
            c.check(!b.get_ref().is_empty(), range(b), || {
                "guidance binds at least one actor".to_owned()
            });
            for actor in b.get_ref() {
                c.check(roles.contains_key(actor), range(b), || {
                    format!("`{actor}` is not a `[package.roles]` actor")
                });
            }
        }
        None => c.check(!guidance, at, || {
            "guidance names the actors it binds: add `binds = [...]` to `[package.instrument]`"
                .to_owned()
        }),
    }
    let instrument = instrument(c, &kind, at, p.instrument.as_ref(), sources);
    (kind, instrument)
}

/// The instrument `[package.instrument]` names: present exactly when the
/// class is not `base`, its text vendored under a `[sources.*]` key.
fn instrument(
    c: &mut Checker,
    kind: &Kind,
    at: TextRange,
    raw: Option<&Spanned<RawInstrument>>,
    sources: &[SourceBinding],
) -> Option<Instrument> {
    let Some(i) = raw else {
        c.check(kind.is_base(), at, || {
            format!(
                "kind `{}` must be justified by its instrument: add `[package.instrument]` with \
                 `source`, `issuer` and `date`",
                kind.name()
            )
        });
        return None;
    };
    c.check(!kind.is_base(), range(i), || {
        "`[package.instrument]` justifies a `kind` other than `base`".to_owned()
    });
    let raw = i.get_ref();
    let source = raw.source.get_ref();
    c.check(
        sources.iter().any(|s| &s.id == source && s.path.is_some()),
        range(&raw.source),
        || {
            format!(
                "instrument source `{source}` must be a `[sources.*]` key with a vendored `path`"
            )
        },
    );
    c.check(
        !raw.issuer.get_ref().trim().is_empty(),
        range(&raw.issuer),
        || "`issuer` must name who issued the instrument".to_owned(),
    );
    c.date("date", &raw.date);
    decision(c, kind, raw);
    Some(Instrument {
        source: source.clone(),
        issuer: raw.issuer.get_ref().clone(),
        date: raw.date.get_ref().clone(),
        designation: raw.designation.clone(),
        court: raw.court.as_ref().map(|c| c.get_ref().clone()),
        designated: raw.designated.as_ref().map(|d| d.get_ref().clone()),
        overrules: raw
            .overrules
            .as_ref()
            .map(|o| o.get_ref().clone())
            .unwrap_or_default(),
        reach: raw.reach.as_ref().map(|r| r.get_ref().clone()),
        holdings: holdings(c, kind, raw),
        en_banc: raw.en_banc.as_ref().is_some_and(|b| *b.get_ref()),
        abrogates: raw
            .abrogates
            .as_ref()
            .map(|a| a.get_ref().clone())
            .unwrap_or_default(),
        authoritative: raw.authoritative.unwrap_or(true),
    })
}

/// A precedent's holdings and binding reach (docs/litigate/CHANGE.md
/// §2.8): it names its `reach` and at least one holding, each with the
/// pin cite to the opinion page it is held at (`79 F.3d at 1215`) and the
/// one rule element (`pack::id`) it edits. Only a precedent has either.
fn holdings(c: &mut Checker, kind: &Kind, raw: &RawInstrument) -> Vec<Holding> {
    let precedent = matches!(kind, Kind::Precedent { .. });
    let at = range(&raw.source);
    match &raw.reach {
        Some(r) => {
            c.check(precedent, range(r), || {
                "`reach` names the binding reach of a `precedent` only".to_owned()
            });
            for family in r.get_ref() {
                c.check(is_kebab(family), range(r), || {
                    format!(
                        "reach `{family}` must be a forum-family id such as `circuit-appellate`"
                    )
                });
            }
        }
        None => c.check(!precedent, at, || {
            "a precedent names its binding reach: add `reach = [...]` (the forum families where \
             its court's precedent binds; `[]` where it binds nowhere) to `[package.instrument]`"
                .to_owned()
        }),
    }
    let Some(list) = &raw.holdings else {
        c.check(!precedent, at, || {
            "a precedent names its holdings: add `[[package.instrument.holdings]]` with \
             `holding`, `pin` (the opinion page) and `edits` (the rule element)"
                .to_owned()
        });
        return Vec::new();
    };
    c.check(precedent, range(list), || {
        "`holdings` are a `precedent`'s only".to_owned()
    });
    c.check(!list.get_ref().is_empty(), range(list), || {
        "a precedent has at least one holding".to_owned()
    });
    let mut out = Vec::new();
    for h in list.get_ref() {
        let h_at = range(h);
        let h = h.get_ref();
        c.check(
            !h.holding.get_ref().trim().is_empty(),
            range(&h.holding),
            || "`holding` states what the court held".to_owned(),
        );
        match &h.pin {
            Some(pin) => c.check(is_pin(pin.get_ref()), range(pin), || {
                format!(
                    "pin `{}` must cite the opinion page the holding is at, as `<reporter> at \
                     <page>` (`79 F.3d at 1215`)",
                    pin.get_ref()
                )
            }),
            None => c.check(false, h_at, || {
                "a holding without a pin cite is not a holding: add `pin` (the opinion page, \
                 `79 F.3d at 1215`)"
                    .to_owned()
            }),
        }
        match &h.edits {
            Some(e) => c.check(is_element(e.get_ref()), range(e), || {
                format!(
                    "edits `{}` must name one rule element as `pack::id`",
                    e.get_ref()
                )
            }),
            None => c.check(false, h_at, || {
                "a holding names the rule element it edits: add `edits = \"pack::id\"`".to_owned()
            }),
        }
        if let (Some(pin), Some(edits)) = (&h.pin, &h.edits) {
            out.push(Holding {
                holding: h.holding.get_ref().clone(),
                pin: pin.get_ref().clone(),
                edits: edits.get_ref().clone(),
            });
        }
    }
    out
}

/// True for a pin cite to an opinion page: `<reporter cite> at <page>`,
/// the page starting with a digit (`375 U.S. at 387`, `Paper 30 at 3`).
fn is_pin(text: &str) -> bool {
    text.rsplit_once(" at ").is_some_and(|(cite, page)| {
        !cite.trim().is_empty() && page.trim_start().starts_with(|c: char| c.is_ascii_digit())
    })
}

/// True for a rule element named `pack::id`.
fn is_element(text: &str) -> bool {
    text.split_once("::")
        .is_some_and(|(pack, id)| is_kebab(pack) && is_ident(id))
}

/// A judicial decision's keys (docs/litigate/CHANGE.md §2.8-§2.9): a
/// precedent or overruling names its `court`, from which binding scope is
/// computed; only a precedent is `designated` precedential, on or after
/// its decision; only an overruling `overrules`, and names what; only a
/// decision sits `en_banc` or `abrogates`, and an abrogation names what.
fn decision(c: &mut Checker, kind: &Kind, raw: &RawInstrument) {
    let decision = matches!(kind, Kind::Precedent { .. } | Kind::Overruling);
    let at = range(&raw.source);
    match &raw.court {
        Some(court) => c.check(decision, range(court), || {
            "`court` names the court of a `precedent` or `overruling` only".to_owned()
        }),
        None => c.check(!decision, at, || {
            format!(
                "the `{}` instrument names its `court` (a forum-family court id such as \
                 `scotus` or `ca9`)",
                kind.name()
            )
        }),
    }
    if let Some(court) = &raw.court {
        c.check(is_kebab(court.get_ref()), range(court), || {
            format!("court `{}` must be a kebab-case id", court.get_ref())
        });
    }
    if let Some(d) = &raw.designated {
        c.check(matches!(kind, Kind::Precedent { .. }), range(d), || {
            "only a `precedent` is `designated` precedential".to_owned()
        });
        c.date("designated", d);
        c.check(d.get_ref() >= raw.date.get_ref(), range(d), || {
            "a decision is designated precedential on or after its `date`".to_owned()
        });
    }
    let overruling = matches!(kind, Kind::Overruling);
    match &raw.overrules {
        Some(o) => c.check(overruling && !o.get_ref().is_empty(), range(o), || {
            if overruling {
                "an overruling `overrules` at least one precedent package".to_owned()
            } else {
                "only an `overruling` `overrules`".to_owned()
            }
        }),
        None => c.check(!overruling, at, || {
            "an overruling names the precedent packages it `overrules`".to_owned()
        }),
    }
    if let Some(b) = &raw.en_banc {
        c.check(decision, range(b), || {
            "only a `precedent` or `overruling` sits `en_banc`".to_owned()
        });
    }
    if let Some(a) = &raw.abrogates {
        c.check(decision && !a.get_ref().is_empty(), range(a), || {
            if decision {
                "a decision that `abrogates` names at least one precedent package".to_owned()
            } else {
                "only a `precedent` or `overruling` `abrogates`".to_owned()
            }
        });
    }
}

mod cite_syntax;
mod family;

#[cfg(test)]
mod tests;
