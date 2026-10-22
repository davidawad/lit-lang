//! Name lookup for one module (SPEC §4.3): the module itself, then names
//! imported with `import m.{x}`, then `m.x` for imported modules, then
//! `pack::x` / `base@origin::x` where qualification is allowed. Failures
//! are L1002 reports with nearest-match suggestions.
use super::sig::{Signature, TypeSig, ValueKind, ValueSig};
use super::suggest;
use crate::db::parse::tree::{dotted, ItemTree, Name, PathSep, RefPath};
use crate::db::span::Report;
use litigate_base::{Applicability, DiagnosticCode as C};

/// One import as seen by lookup.
#[derive(Debug, Clone)]
pub struct Binding<'a> {
    /// Dotted module path.
    pub path: String,
    /// `import m.{x, y}` names; `None` for a whole-module import.
    pub names: Option<Vec<String>>,
    /// The imported module's signature.
    pub sig: &'a Signature,
}

/// A resolved name.
#[derive(Debug, Clone, Copy)]
pub struct Found<'a, T> {
    /// The declaring module's signature.
    pub sig: &'a Signature,
    /// Declared entity.
    pub item: &'a T,
}

/// Lookup environment of one module.
#[derive(Debug, Clone)]
pub struct Scope<'a> {
    /// The module's own signature.
    pub own: &'a Signature,
    /// Successful imports in source order.
    pub bindings: Vec<Binding<'a>>,
    /// Every module signature reachable through `::` qualification.
    pub packs: Vec<&'a Signature>,
    /// Names whose `import m.{x}` failed; lookups of them are suppressed.
    pub poisoned: Vec<String>,
}

impl<'a> Scope<'a> {
    /// Builds a scope; `imports` are the signatures the graph resolved for
    /// `tree.imports`, matched by module name.
    #[must_use]
    pub fn new(
        own: &'a Signature,
        tree: &ItemTree,
        imports: &[&'a Signature],
        packs: Vec<&'a Signature>,
    ) -> Self {
        let poisoned = tree
            .imports
            .iter()
            .flat_map(|i| {
                let sig = imports.iter().find(|s| s.module == dotted(&i.path));
                i.names
                    .iter()
                    .flatten()
                    .filter(move |n| {
                        sig.is_none_or(|s| {
                            !s.values.contains_key(&n.text)
                                && !s.types.contains_key(&n.text)
                                && !s.groups.contains(&n.text)
                                && !s.authorities.contains(&n.text)
                        })
                    })
                    .map(|n| n.text.clone())
            })
            .collect();
        let bindings = tree
            .imports
            .iter()
            .filter_map(|i| {
                let path = dotted(&i.path);
                let sig = imports.iter().find(|s| s.module == path)?;
                let names = i
                    .names
                    .as_ref()
                    .map(|n| n.iter().map(|n| n.text.clone()).collect());
                Some(Binding { path, names, sig })
            })
            .collect();
        Self {
            own,
            bindings,
            packs,
            poisoned,
        }
    }

    /// Canonical spelling of `name` declared in `sig` as seen from here.
    #[must_use]
    pub fn canonical(&self, sig: &Signature, name: &str) -> String {
        if sig.package == self.own.package {
            name.to_owned()
        } else {
            format!("{}::{name}", sig.pack_id)
        }
    }

    fn unqualified<T>(
        &self,
        name: &str,
        get: impl Fn(&'a Signature) -> Option<&'a T>,
    ) -> Option<Found<'a, T>> {
        get(self.own)
            .map(|item| Found {
                sig: self.own,
                item,
            })
            .or_else(|| {
                self.bindings
                    .iter()
                    .filter(|b| {
                        b.names
                            .as_ref()
                            .is_some_and(|n| n.iter().any(|x| x == name))
                    })
                    .find_map(|b| get(b.sig).map(|item| Found { sig: b.sig, item }))
            })
    }

    fn candidates<'s>(&'s self, keys: impl Fn(&'a Signature) -> Vec<&'a str>) -> Vec<&'s str> {
        let mut out = keys(self.own);
        for b in &self.bindings {
            match &b.names {
                Some(names) => out.extend(
                    names
                        .iter()
                        .map(String::as_str)
                        .filter(|n| keys(b.sig).contains(n)),
                ),
                None => out.extend(keys(b.sig)),
            }
        }
        out
    }

    fn unresolved(&self, name: &Name, what: &str, candidates: &[&str]) -> Report {
        if self.poisoned.contains(&name.text) {
            let mut report = Report::new(
                C::UnresolvedName,
                name.range,
                format!("`{}` was not imported", name.text),
            );
            report.suppressed = true;
            return report;
        }
        let mut report = Report::new(
            C::UnresolvedName,
            name.range,
            match what.split_once(" in ") {
                Some((noun, place)) => format!("cannot find {noun} `{}` in {place}", name.text),
                None => format!("cannot find {what} `{}`", name.text),
            },
        );
        if let Some(best) = suggest::nearest(&name.text, candidates.iter().copied()) {
            report = report
                .note(format!("a {what} with a similar name exists: `{best}`"))
                .fix(name.range, best, Applicability::MaybeIncorrect);
        }
        report
    }

    /// Resolves a value reference (node, deadline, counting rule, ...).
    ///
    /// # Errors
    /// L1002 with a suggestion when nothing matches.
    pub fn value(&self, r: &RefPath) -> Result<(String, Found<'a, ValueSig>), Report> {
        let value_keys = |s: &'a Signature| s.values.keys().map(String::as_str).collect::<Vec<_>>();
        let (canonical, found) = if r.rest.iter().any(|(sep, _)| *sep == PathSep::Colons) {
            self.qualified(r)?
        } else if r.rest.is_empty() && r.instance.is_none() {
            let found = self
                .unqualified(&r.head.text, |s| s.values.get(&r.head.text))
                .ok_or_else(|| self.unresolved(&r.head, "name", &self.candidates(value_keys)))?;
            (self.canonical(found.sig, &r.head.text), found)
        } else {
            let module: Vec<&str> = std::iter::once(r.head.text.as_str())
                .chain(
                    r.rest[..r.rest.len() - 1]
                        .iter()
                        .map(|(_, n)| n.text.as_str()),
                )
                .collect();
            let module = module.join(".");
            let last = r.last();
            let binding = self
                .bindings
                .iter()
                .find(|b| b.path == module)
                .ok_or_else(|| {
                    Report::new(
                        C::UnresolvedName,
                        r.range,
                        format!("`{module}` is not an imported module"),
                    )
                    .note(format!(
                        "add `import {module}` to refer to `{}`",
                        r.display()
                    ))
                })?;
            let item = binding.sig.values.get(&last.text).ok_or_else(|| {
                self.unresolved(
                    last,
                    &format!("name in `{module}`"),
                    &value_keys(binding.sig),
                )
            })?;
            (
                self.canonical(binding.sig, &last.text),
                Found {
                    sig: binding.sig,
                    item,
                },
            )
        };
        match &r.version {
            None => Ok((canonical, found)),
            Some(date) if found.item.versions.contains(&date.text) => {
                Ok((format!("{canonical}@{}", date.text), found))
            }
            Some(date) => Err(Report::new(
                C::UnresolvedName,
                date.range,
                format!(
                    "no version of `{}` takes effect on {}",
                    r.display(),
                    date.text
                ),
            )
            .note(format!(
                "versions take effect on: {}",
                found.item.versions.join(", ")
            ))),
        }
    }

    fn qualified(&self, r: &RefPath) -> Result<(String, Found<'a, ValueSig>), Report> {
        let last = r.last();
        if r.rest.len() != 1 {
            return Err(Report::new(
                C::UnresolvedName,
                r.range,
                format!(
                    "`{}` is not a valid qualified name (`pack::id`)",
                    r.display()
                ),
            ));
        }
        let (prefix, pack) = match &r.instance {
            Some(origin) => {
                let key = format!("{}@{}", r.head.text, origin.text);
                match self.own.values.get(&key).map(|v| &v.kind) {
                    Some(ValueKind::Instance(pack)) => (key, pack.clone().unwrap_or_default()),
                    _ => {
                        return Err(Report::new(
                            C::UnresolvedName,
                            r.head.range.cover(origin.range),
                            format!("no instance `{key}` is declared in this package"),
                        ))
                    }
                }
            }
            None => (r.head.text.clone(), r.head.text.clone()),
        };
        let sigs: Vec<&'a Signature> = self
            .packs
            .iter()
            .copied()
            .filter(|s| s.pack_id == pack)
            .collect();
        if sigs.is_empty() {
            let ids: Vec<&str> = self.packs.iter().map(|s| s.pack_id.as_str()).collect();
            let name = Name {
                text: pack,
                range: r.head.range,
            };
            return Err(self.unresolved(&name, "pack", &ids));
        }
        let found = sigs
            .iter()
            .find_map(|s| s.values.get(&last.text).map(|item| Found { sig: s, item }));
        let found = found.ok_or_else(|| {
            let keys: Vec<&str> = sigs
                .iter()
                .flat_map(|s| s.values.keys().map(String::as_str))
                .collect();
            self.unresolved(last, &format!("name in pack `{prefix}`"), &keys)
        })?;
        Ok((format!("{prefix}::{}", last.text), found))
    }

    /// Resolves a user type name (enum, alias, or event).
    ///
    /// # Errors
    /// L1002 with a suggestion.
    pub fn user_type(
        &self,
        name: &Name,
        builtins: &[&str],
    ) -> Result<(String, Found<'a, TypeSig>), Report> {
        self.unqualified(&name.text, |s| s.types.get(&name.text))
            .map(|f| (self.canonical(f.sig, &name.text), f))
            .ok_or_else(|| {
                let mut candidates =
                    self.candidates(|s| s.types.keys().map(String::as_str).collect());
                candidates.extend_from_slice(builtins);
                self.unresolved(name, "type", &candidates)
            })
    }

    /// Looks up an enum by canonical name among visible signatures.
    #[must_use]
    pub fn enum_variants(&self, canonical: &str) -> Option<&'a [String]> {
        let (pack, local) = canonical
            .split_once("::")
            .map_or((None, canonical), |(p, l)| (Some(p), l));
        std::iter::once(self.own)
            .chain(self.bindings.iter().map(|b| b.sig))
            .chain(self.packs.iter().copied())
            .filter(|s| {
                pack.map_or(s.package == self.own.package, |p| {
                    s.pack_id == p && s.package != self.own.package
                })
            })
            .find_map(|s| match s.types.get(local) {
                Some(TypeSig::Enum(v)) => Some(v.as_slice()),
                _ => None,
            })
    }

    /// Canonical names of visible enums whose variants include every key
    /// (the enum a `Dist` record literal denotes).
    #[must_use]
    pub fn enums_covering(&self, keys: &[&str]) -> Vec<String> {
        let mut out: Vec<String> = std::iter::once(self.own)
            .chain(self.bindings.iter().map(|b| b.sig))
            .flat_map(|s| s.types.iter().map(move |(n, t)| (s, n, t)))
            .filter_map(|(s, n, t)| match t {
                TypeSig::Enum(v)
                    if !keys.is_empty() && keys.iter().all(|k| v.iter().any(|x| x == k)) =>
                {
                    Some(self.canonical(s, n))
                }
                _ => None,
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Checks a group name.
    ///
    /// # Errors
    /// L1002 with a suggestion.
    pub fn group(&self, name: &Name) -> Result<(), Report> {
        self.unqualified(&name.text, |s| s.groups.get(&name.text))
            .map(|_| ())
            .ok_or_else(|| {
                self.unresolved(
                    name,
                    "group",
                    &self.candidates(|s| s.groups.iter().map(String::as_str).collect()),
                )
            })
    }

    /// Checks an authority level name.
    ///
    /// # Errors
    /// L1002 with a suggestion.
    pub fn authority(&self, name: &Name) -> Result<(), Report> {
        self.unqualified(&name.text, |s| s.authorities.get(&name.text))
            .map(|_| ())
            .ok_or_else(|| {
                self.unresolved(
                    name,
                    "authority",
                    &self.candidates(|s| s.authorities.iter().map(String::as_str).collect()),
                )
            })
    }

    /// Every value name visible unqualified (for suggestions).
    #[must_use]
    pub fn value_names(&self) -> Vec<&str> {
        self.candidates(|s| s.values.keys().map(String::as_str).collect())
    }
}
