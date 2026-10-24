//! Strict `Lit.toml` parsing and validation (SPEC §4.1). Unknown keys,
//! missing required keys, and ill-typed or invalid values are all L1004.
use super::semver::{Requirement, Version};
use crate::db::span::{Report, TextRange};
use crate::hir::{ActorRole, CiteSyntax, Interval, Manifest, SourceBinding};
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
    authority: Option<Spanned<String>>,
    title: Option<Spanned<String>>,
    roles: Option<Spanned<Ordered<Spanned<String>>>>,
    description: Option<Spanned<String>>,
    jurisdiction: Option<Spanned<String>>,
    counting: Option<Spanned<String>>,
    in_force: Spanned<String>,
    in_force_until: Option<Spanned<String>>,
    known_at: Spanned<String>,
    // The date before which no declaration's law is encoded (SPEC §4.1).
    unknown_before: Option<Spanned<String>>,
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
    let sources = raw
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
    if let Some(counting) = &p.counting {
        c.check(is_ident(counting.get_ref()), range(counting), || {
            format!(
                "`counting` must be a counting-rule id, found `{}`",
                counting.get_ref()
            )
        });
    }
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
        authority: text_of(&p.authority).unwrap_or_default(),
        title: text_of(&p.title).unwrap_or_default(),
        description: text_of(&p.description),
        jurisdiction: text_of(&p.jurisdiction),
        roles,
        in_force: Interval {
            from: p.in_force.get_ref().clone(),
            to: p.in_force_until.as_ref().map(|s| s.get_ref().clone()),
            known_at: p.known_at.get_ref().clone(),
            retracted: None,
            unknown_before: p.unknown_before.as_ref().map(|s| s.get_ref().clone()),
            commenced: None,
        },
        sources,
        dependencies,
        counting: text_of(&p.counting),
        cite_syntax,
        links,
    };
    Ok(ParsedManifest {
        manifest,
        dependency_paths,
    })
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
    for (key, value) in [("forum", &p.forum), ("authority", &p.authority)] {
        if let Some(value) = value {
            c.check(is_ident(value.get_ref()), range(value), || {
                format!("`{key}` must be an identifier, found `{}`", value.get_ref())
            });
        }
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
        ("counting", p.counting.as_ref().map(range)),
        ("cite_syntax", raw.cite_syntax.as_ref().map(range)),
        ("sources", raw.sources.0.first().map(|(_, s)| range(s))),
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

mod cite_syntax;

#[cfg(test)]
mod tests;
