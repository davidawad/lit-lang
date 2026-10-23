//! Computed compatibility classes (SPEC §4.4). A release's class against
//! an earlier release of the same package is computed from the two
//! packages' HIR, never declared, and decides the version bump the release
//! needs, as Elm and cargo-semver-checks enforce semver for code:
//!
//! - `interface`: an id or type a matter, a links package, or a dependent
//!   names is removed, renamed, or retyped;
//! - `value`: every such id and type remains, but what the package computes
//!   may differ (a period, a counting rule, an edge, a probability, a
//!   declaration added);
//! - `none`: nothing a consumer computes differs (formatting, comments,
//!   labels, notes, tests).
//!
//! `value` over-approximates "a computed due date or probability differs":
//! any change to the declarations' meaning counts, whether or not some
//! matter observes it, so the class is never too low.
use super::lock::canonical;
use super::semver::Version;
use crate::hir;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// What a change can break, least to most.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// Nothing a consumer computes differs.
    None,
    /// Computed values may differ; every named id and type remains.
    Value,
    /// A named id or type is removed, renamed, or retyped.
    Interface,
}

/// The version component a release bumps, least to most significant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Bump {
    /// Only the pre-release identifiers.
    Pre,
    /// `PATCH`.
    Patch,
    /// `MINOR`.
    Minor,
    /// `MAJOR`.
    Major,
}

/// One classified difference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    /// Its class.
    pub class: Class,
    /// `removed`, `renamed`, `retyped`, `added`, or `changed`.
    pub kind: &'static str,
    /// What changed: `<declaration kind> <id>`, `edge <node>.<edge>`,
    /// `actor <name>`, `manifest <key>`, or `package`.
    pub what: String,
    /// The new id of a renamed declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now: Option<String>,
}

/// A release classified against an earlier one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Compat {
    /// The most breaking class among the changes (`none` without any).
    pub class: Class,
    /// The least bump the class needs.
    pub required: Bump,
    /// The bump the two versions declare (`None` when the new version is
    /// not later).
    pub declared: Option<Bump>,
    /// Every difference, in declaration-key order.
    pub changes: Vec<Change>,
}

impl Compat {
    /// Whether the declared bump is enough: at least the required one, or
    /// an unchanged package at the same version.
    #[must_use]
    pub fn ok(&self, same_version: bool) -> bool {
        self.declared.is_some_and(|d| d >= self.required)
            || (same_version && self.changes.is_empty())
    }
}

/// Keys dropped before comparing meaning: source positions and display
/// text.
const PRESENTATION: [&str; 7] = [
    "span",
    "label",
    "note",
    "title",
    "description",
    "court_listener_url",
    "valence",
];

/// Declaration kinds that are types: any change to one is a retyping.
const TYPES: [&str; 3] = ["enum", "event", "type_alias"];

fn canon(value: &Value, skip: &[&str]) -> String {
    let mut out = String::new();
    canonical(value, skip, &mut out);
    out
}

type Key = (String, String);

/// `(kind, id)` of a serialized declaration: its `meta.id` or `name`, or a
/// link's `from#edge-id`.
fn key(v: &Value) -> Key {
    let id = [&v["meta"]["id"], &v["name"]]
        .into_iter()
        .find_map(Value::as_str)
        .map_or_else(
            || {
                format!(
                    "{}#{}",
                    canon(&v["from"], &[]),
                    canon(&v["edge"]["id"], &[])
                )
            },
            str::to_owned,
        );
    (v["decl"].as_str().unwrap_or_default().to_owned(), id)
}

/// Every declaration but tests, serialized, grouped by key (versions of
/// one declaration share a key).
fn declarations(p: &hir::Package) -> BTreeMap<Key, Vec<Value>> {
    let mut out: BTreeMap<Key, Vec<Value>> = BTreeMap::new();
    for d in p.modules.iter().flat_map(|m| &m.declarations) {
        let v = serde_json::to_value(d).unwrap_or(Value::Null);
        if v["decl"] != "test" {
            out.entry(key(&v)).or_default().push(v);
        }
    }
    out
}

/// The versions' canonical texts without `skip`, order-free.
fn texts(versions: &[Value], skip: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = versions.iter().map(|v| canon(v, skip)).collect();
    out.sort();
    out
}

/// Each node's former ids, to its current id.
fn former_nodes(decls: &BTreeMap<Key, Vec<Value>>) -> BTreeMap<String, String> {
    decls
        .iter()
        .filter(|((kind, _), _)| kind == "node")
        .flat_map(|((_, id), versions)| {
            versions
                .iter()
                .flat_map(|v| v["formerly"].as_array().into_iter().flatten())
                .filter_map(Value::as_str)
                .map(move |old| (old.to_owned(), id.clone()))
        })
        .collect()
}

/// Edges of `old` (one node's versions) missing from `new`: renamed when
/// a new edge lists the id in `formerly`, else removed.
fn edge_changes(node: &str, old: &[Value], new: &[Value]) -> Vec<Change> {
    let edges = |versions: &[Value]| -> Vec<Value> {
        versions
            .iter()
            .flat_map(|v| v["edges"].as_array().into_iter().flatten().cloned())
            .collect()
    };
    let (old, new) = (edges(old), edges(new));
    let live: BTreeSet<&str> = new.iter().filter_map(|e| e["id"].as_str()).collect();
    let formerly: BTreeMap<&str, &str> = new
        .iter()
        .flat_map(|e| {
            let now = e["id"].as_str().unwrap_or_default();
            e["formerly"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(move |f| (f, now))
        })
        .collect();
    let gone: BTreeSet<&str> = old
        .iter()
        .filter_map(|e| e["id"].as_str())
        .filter(|id| !live.contains(id))
        .collect();
    gone.into_iter()
        .map(|id| {
            let now = formerly.get(id).map(|n| (*n).to_owned());
            Change {
                class: Class::Interface,
                kind: if now.is_some() { "renamed" } else { "removed" },
                what: format!("edge {node}.{id}"),
                now,
            }
        })
        .collect()
}

/// Identity and actors (interface), then any other difference in the
/// manifest, `start`, groups, or resolved dependency versions (value).
fn package_changes(old: &hir::Package, new: &hir::Package) -> Vec<Change> {
    let changed = |class, what: String| Change {
        class,
        kind: "changed",
        what,
        now: None,
    };
    let (a, b) = (&old.manifest, &new.manifest);
    let mut out: Vec<Change> = [
        ("pack_id", &a.pack_id, &b.pack_id),
        ("forum", &a.forum, &b.forum),
    ]
    .into_iter()
    .filter(|(_, x, y)| x != y)
    .map(|(k, _, _)| changed(Class::Interface, format!("manifest {k}")))
    .collect();
    out.extend(
        a.roles
            .iter()
            .filter(|(actor, role)| b.roles.get(*actor) != Some(role))
            .map(|(actor, _)| changed(Class::Interface, format!("actor {actor}"))),
    );
    let rest = |p: &hir::Package| {
        let mut m = serde_json::to_value(&p.manifest).unwrap_or(Value::Null);
        m["version"] = Value::Null;
        let v = serde_json::json!([m, p.start, p.groups, p.resolved]);
        canon(&v, &PRESENTATION)
    };
    if rest(old) != rest(new) {
        out.push(changed(Class::Value, "package".to_owned()));
    }
    out
}

/// Classifies `new` against `old`, an earlier release of the same package.
#[must_use]
pub fn classify(old: &hir::Package, new: &hir::Package) -> Compat {
    let (a, b) = (declarations(old), declarations(new));
    let renamed = former_nodes(&b);
    let mut changes = package_changes(old, new);
    let mut targets = BTreeSet::new();
    for (k @ (kind, id), versions) in &a {
        let to = (kind == "node")
            .then(|| renamed.get(id))
            .flatten()
            .filter(|_| !b.contains_key(k));
        let target = to.map(|to| (kind.clone(), to.clone()));
        let Some(now) = b.get(k).or_else(|| target.as_ref().and_then(|t| b.get(t))) else {
            changes.push(Change {
                class: Class::Interface,
                kind: "removed",
                what: format!("{kind} {id}"),
                now: None,
            });
            continue;
        };
        if kind == "node" {
            changes.extend(edge_changes(id, versions, now));
        }
        if let Some(to) = to {
            targets.insert((kind.clone(), to.clone()));
            changes.push(Change {
                class: Class::Interface,
                kind: "renamed",
                what: format!("{kind} {id}"),
                now: Some(to.clone()),
            });
        } else if TYPES.contains(&kind.as_str())
            && texts(versions, &["span"]) != texts(now, &["span"])
        {
            changes.push(Change {
                class: Class::Interface,
                kind: "retyped",
                what: format!("{kind} {id}"),
                now: None,
            });
        } else if texts(versions, &PRESENTATION) != texts(now, &PRESENTATION) {
            changes.push(Change {
                class: Class::Value,
                kind: "changed",
                what: format!("{kind} {id}"),
                now: None,
            });
        }
    }
    changes.extend(
        b.keys()
            .filter(|k| !a.contains_key(*k) && !targets.contains(*k))
            .map(|(kind, id)| Change {
                class: Class::Value,
                kind: "added",
                what: format!("{kind} {id}"),
                now: None,
            }),
    );
    let class = changes.iter().map(|c| c.class).max().unwrap_or(Class::None);
    // Both manifests are valid (L1004), so both versions parse.
    let (required, declared) = Version::parse(&old.manifest.version)
        .zip(Version::parse(&new.manifest.version))
        .map_or((Bump::Major, None), |(o, n)| {
            (required(class, &o), declared(&o, &n))
        });
    Compat {
        class,
        required,
        declared,
        changes,
    }
}

/// The most significant component `new` raises over `old`; `None` unless
/// `new` is later.
#[must_use]
pub fn declared(old: &Version, new: &Version) -> Option<Bump> {
    let bump = if new.major != old.major {
        Bump::Major
    } else if new.minor != old.minor {
        Bump::Minor
    } else if new.patch != old.patch {
        Bump::Patch
    } else {
        Bump::Pre
    };
    (new > old).then_some(bump)
}

/// The least bump `class` needs from `old`: an interface change bumps the
/// leftmost non-zero component (what `^` holds fixed), a value change the
/// next one (what `~` holds fixed), and `none` any later version.
#[must_use]
pub fn required(class: Class, old: &Version) -> Bump {
    let breaking = if old.major > 0 {
        Bump::Major
    } else if old.minor > 0 {
        Bump::Minor
    } else {
        Bump::Patch
    };
    match class {
        Class::Interface => breaking,
        Class::Value if breaking == Bump::Major => Bump::Minor,
        Class::Value => Bump::Patch,
        Class::None => Bump::Pre,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    #[test]
    fn declared_bumps_name_the_most_significant_component() {
        for (old, new, bump) in [
            ("1.2.3", "2.0.0", Some(Bump::Major)),
            ("1.2.3", "1.3.0", Some(Bump::Minor)),
            ("1.2.3", "1.2.4", Some(Bump::Patch)),
            ("1.2.3-rc.1", "1.2.3", Some(Bump::Pre)),
            ("1.2.3", "1.2.3", None),
            ("1.2.3", "1.2.2", None),
        ] {
            assert_eq!(declared(&v(old), &v(new)), bump, "{old} -> {new}");
        }
    }

    #[test]
    fn required_bumps_follow_caret_and_tilde() {
        for (class, old, bump) in [
            (Class::Interface, "2024.12.0", Bump::Major),
            (Class::Value, "2024.12.0", Bump::Minor),
            (Class::None, "2024.12.0", Bump::Pre),
            (Class::Interface, "0.1.0", Bump::Minor),
            (Class::Value, "0.1.0", Bump::Patch),
            (Class::Interface, "0.0.3", Bump::Patch),
            (Class::Value, "0.0.3", Bump::Patch),
        ] {
            assert_eq!(required(class, &v(old)), bump, "{class:?} {old}");
        }
    }

    #[test]
    fn an_unchanged_package_at_the_same_version_is_compatible() {
        let compat = Compat {
            class: Class::None,
            required: Bump::Pre,
            declared: None,
            changes: Vec::new(),
        };
        assert!(compat.ok(true));
        assert!(!compat.ok(false));
        let changed = Compat {
            changes: vec![Change {
                class: Class::Value,
                kind: "changed",
                what: "deadline d".into(),
                now: None,
            }],
            ..compat.clone()
        };
        assert!(!changed.ok(true));
        let bumped = Compat {
            declared: Some(Bump::Patch),
            ..changed
        };
        assert!(bumped.ok(false));
    }
}
