//! `[family.*]`: the forum-family class matrix (docs/litigate/CHANGE.md
//! §3), data in `lit/core`'s manifest rather than compiler code (design
//! review 2026-10-23 §2.5 item 4). Each family lists its forums, whose
//! precedent binds there, and one cell per class that exists there; a cell
//! may only narrow its class's row.
use super::{is_kebab, range, Checker, Ordered};
use crate::hir::{Cell, Effect, Family, Kind, StateRule};
use serde::Deserialize;
use std::collections::BTreeMap;
use toml::Spanned;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawFamily {
    title: String,
    forums: Spanned<Vec<String>>,
    #[serde(default)]
    precedent: Vec<String>,
    #[serde(default)]
    circuits: Vec<String>,
    #[serde(default)]
    panels: Vec<String>,
    state: Option<Ordered<Spanned<RawState>>>,
    class: Ordered<Spanned<RawCell>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawState {
    courts: Vec<String>,
    #[serde(default)]
    federal: Vec<String>,
    cite: Spanned<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCell {
    cite: Spanned<String>,
    example: Option<Spanned<String>>,
    row: Option<Spanned<Vec<String>>>,
    window_days: Option<Spanned<u32>>,
}

/// The validated families of package `name`: only `core` declares them.
pub(super) fn check(c: &mut Checker, name: &str, raw: &Ordered<Spanned<RawFamily>>) -> Vec<Family> {
    if let Some((_, first)) = raw.0.first() {
        c.check(name == "core", range(first), || {
            "only `core` declares the forum-family matrix (`[family.*]`)".to_owned()
        });
    }
    let mut forums: BTreeMap<&str, &str> = BTreeMap::new();
    raw.0
        .iter()
        .map(|(id, f)| {
            let at = range(f);
            let f = f.get_ref();
            c.check(is_kebab(id), at, || {
                format!("family id `{id}` must be kebab-case")
            });
            c.check(!f.forums.get_ref().is_empty(), range(&f.forums), || {
                format!("family `{id}` names at least one forum")
            });
            for forum in f.forums.get_ref() {
                let other = forums.insert(forum, id);
                c.check(other.is_none(), range(&f.forums), || {
                    format!(
                        "forum `{forum}` is in families `{}` and `{id}`",
                        other.unwrap_or_default()
                    )
                });
            }
            let states = f.state.as_ref().map_or(&[][..], |s| s.0.as_slice());
            for court in &f.precedent {
                let ok = match court.as_str() {
                    "@circuit" => !f.circuits.is_empty(),
                    "@state" => !states.is_empty(),
                    _ => is_kebab(court),
                };
                c.check(ok, at, || {
                    format!(
                        "family `{id}`: precedent court `{court}` must be a kebab-case id, \
                         `@circuit` with `circuits` listed, or `@state` with `[family.{id}.state.*]`"
                    )
                });
            }
            for circuit in f.circuits.iter().chain(&f.panels) {
                c.check(is_kebab(circuit), at, || {
                    format!("family `{id}`: court `{circuit}` must be a kebab-case id")
                });
            }
            let states = self::states(c, id, states);
            c.check(f.class.0.iter().any(|(k, _)| k == "base"), at, || {
                format!("family `{id}` has a `base` class: every forum has primary law")
            });
            Family {
                id: id.clone(),
                title: f.title.clone(),
                forums: f.forums.get_ref().clone(),
                precedent: f.precedent.clone(),
                circuits: f.circuits.clone(),
                panels: f.panels.clone(),
                states,
                classes: f
                    .class
                    .0
                    .iter()
                    .filter_map(|(class, cell)| self::cell(c, id, class, cell))
                    .collect(),
            }
        })
        .collect()
}

/// A family's states: each names its binding courts by kebab-case id and
/// cites the decisions that settle them.
fn states(c: &mut Checker, id: &str, raw: &[(String, Spanned<RawState>)]) -> Vec<StateRule> {
    raw.iter()
        .map(|(state, raw)| {
            let r = raw.get_ref();
            c.check(is_kebab(state), range(raw), || {
                format!("family `{id}`: state id `{state}` must be kebab-case")
            });
            c.check(
                !r.courts.is_empty() && r.courts.iter().chain(&r.federal).all(|x| is_kebab(x)),
                range(raw),
                || {
                    format!(
                        "family `{id}`: state `{state}` names its binding `courts` (and any \
                         `federal` ones) by kebab-case id"
                    )
                },
            );
            c.check(!r.cite.get_ref().trim().is_empty(), range(&r.cite), || {
                format!(
                    "family `{id}`: state `{state}` cites the decisions settling whose \
                         precedent binds there"
                )
            });
            StateRule {
                id: state.clone(),
                courts: r.courts.clone(),
                federal: r.federal.clone(),
                cite: r.cite.get_ref().clone(),
            }
        })
        .collect()
}

/// One cell: a known class, a cite, an `https` example, a row inside the
/// class's own, and a window only on an emergency declaration.
fn cell(c: &mut Checker, family: &str, class: &str, raw: &Spanned<RawCell>) -> Option<Cell> {
    let at = range(raw);
    let Some(kind) = Kind::from_name(class) else {
        c.check(false, at, || {
            format!(
                "family `{family}`: `{class}` is not a class ({})",
                Kind::NAMES.map(|n| format!("`{n}`")).join(", ")
            )
        });
        return None;
    };
    let r = raw.get_ref();
    c.check(!r.cite.get_ref().trim().is_empty(), range(&r.cite), || {
        format!("family `{family}`: the `{class}` cell cites the instrument that makes it exist")
    });
    if let Some(e) = &r.example {
        c.check(e.get_ref().starts_with("https://"), range(e), || {
            format!("family `{family}`: the `{class}` example is an `https://` URL")
        });
    }
    let ceiling = kind.row();
    let row = r.row.as_ref().map(|names| {
        names
            .get_ref()
            .iter()
            .filter_map(|n| {
                let effect = Effect::from_name(n);
                c.check(
                    effect.is_some_and(|e| ceiling.has(e)),
                    range(names),
                    || match effect {
                        Some(_) => format!(
                            "family `{family}`: `{n}` is outside the `{class}` row: a cell \
                             only narrows its class"
                        ),
                        None => format!("family `{family}`: `{n}` is not an effect"),
                    },
                );
                effect
            })
            .collect()
    });
    if let Some(w) = &r.window_days {
        c.check(
            matches!(kind, Kind::EmergencyDeclaration) && *w.get_ref() > 0,
            range(w),
            || {
                format!(
                    "family `{family}`: `window_days` bounds an `emergency-declaration`'s \
                     grants, and is positive"
                )
            },
        );
    }
    Some(Cell {
        class: class.to_owned(),
        cite: r.cite.get_ref().clone(),
        example: r.example.as_ref().map(|e| e.get_ref().clone()),
        row,
        window_days: r.window_days.as_ref().map(|w| *w.get_ref()),
    })
}
