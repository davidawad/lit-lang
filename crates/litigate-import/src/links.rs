//! `links.json` to a links package (SPEC §8.4 in reverse): `instance` and
//! `link` declarations in one module.
use crate::issues::Issues;
use crate::node::Spelling;
use crate::text;
use crate::Emitted;
use litigate_engine::cite::normalize::CiteGrammar;
use litigate_engine::model::{CiteSyntax, Instance, LinkFile, RawEdge, Role, PACK_SCHEMA_VERSION};
use std::collections::BTreeMap;

/// Packs known to this import: pack id to (package directory, citation
/// syntax, `known_at`).
pub(crate) type Known<'a> = BTreeMap<&'a str, (&'a str, &'a CiteSyntax, String)>;

/// Imports `file` into the `links` package.
pub(crate) fn import_links(
    file: &LinkFile,
    from: &str,
    known: &Known<'_>,
    issues: &mut Issues,
) -> Vec<Emitted> {
    let mut module = vec![
        format!("// Imported by `litigate import` from {from}."),
        "module links".to_owned(),
    ];
    let mut packs: Vec<&str> = Vec::new();
    for (name, instance) in &file.instances {
        packs.push(instance.pack.as_str());
        // A patch cites in the vocabulary of the pack it patches.
        let empty = CiteSyntax::default();
        let syntax = known
            .get(instance.pack.as_str())
            .map_or(&empty, |(_, syntax, _)| *syntax);
        let mut spelling = Spelling {
            grammar: CiteGrammar::new(syntax),
            issues,
        };
        module.push(instance_text(name, instance, &mut spelling));
    }
    for (index, link) in file.links.iter().enumerate() {
        packs.extend([pack_of(&link.from), pack_of(&link.to)]);
        packs.extend(link.replaces.iter().map(|r| pack_of(r)));
        // A link cites in the vocabulary of the pack it leaves.
        let empty = CiteSyntax::default();
        let syntax = known
            .get(pack_of(&link.from))
            .map_or(&empty, |(_, syntax, _)| *syntax);
        let mut spelling = Spelling {
            grammar: CiteGrammar::new(syntax),
            issues,
        };
        module.push(spelling.link(index, link));
    }
    packs.sort_unstable();
    packs.dedup();
    let dependencies: Vec<(String, String)> = packs
        .iter()
        .map(|pack| {
            let dir = known.get(pack).map_or(*pack, |(dir, _, _)| *dir);
            ((*pack).to_owned(), dir.to_owned())
        })
        .collect();
    let known_at = packs
        .iter()
        .filter_map(|pack| known.get(pack).map(|(_, _, at)| at.clone()))
        .max()
        .unwrap_or_else(|| crate::manifest::IN_FORCE_PLACEHOLDER.to_owned());
    vec![
        Emitted::toml(
            "links/Lit.toml".to_owned(),
            crate::manifest::links_manifest(from, &known_at, &dependencies),
        ),
        Emitted::lit("links/links.lit".to_owned(), module.join("\n\n")),
    ]
}

/// The pack id of a qualified reference (`base@origin::entry` gives `base`).
fn pack_of(reference: &str) -> &str {
    let head = reference.split("::").next().unwrap_or(reference);
    head.split('@').next().unwrap_or(head)
}

fn instance_text(name: &str, instance: &Instance, spelling: &mut Spelling<'_>) -> String {
    let issues = &mut *spelling.issues;
    let pointer = format!("/instances/{name}");
    let Some((base, origin)) = name.split_once('@') else {
        issues.report(&pointer, "an instance name is `base@origin` (SPEC §3.1)");
        return String::new();
    };
    let base = issues.ident(base, &pointer, false);
    let origin = issues.ident(origin, &pointer, false);
    let pack = issues.ident(&instance.pack, &format!("{pointer}/pack"), false);
    let mut fields = vec![format!("pack {pack}")];
    if let Some(note) = &instance.note {
        fields.push(format!("note {}", text::string(note)));
    }
    if !instance.remove_edges.is_empty() {
        let items: Vec<String> = instance
            .remove_edges
            .iter()
            .enumerate()
            .map(|(i, edge)| issues.ident(edge, &format!("{pointer}/remove_edges/{i}"), false))
            .collect();
        fields.push(format!("remove_edges [{}]", items.join(", ")));
    }
    if !instance.probabilities.is_empty() {
        let entries: Vec<String> = instance
            .probabilities
            .iter()
            .map(|(edge, p)| {
                let at = format!("{pointer}/probabilities/{edge}");
                let edge = issues.ident(edge, &at, false);
                format!(
                    "{edge}: {}",
                    issues.spell(text::number(*p), &at, "probability")
                )
            })
            .collect();
        fields.push(format!("probabilities {{ {} }}", entries.join(", ")));
    }
    if !instance.roles.is_empty() {
        let entries: Vec<String> = instance
            .roles
            .iter()
            .map(|(actor, role)| {
                let actor = issues.ident(actor, &format!("{pointer}/roles/{actor}"), false);
                format!("{actor}: {}", role_name(*role))
            })
            .collect();
        fields.push(format!("roles {{ {} }}", entries.join(", ")));
    }
    if let Some(source) = &instance.payoff_transform {
        let at = format!("{pointer}/payoff_transform");
        if let Some(spelled) = crate::transform::transform(source, &at, issues) {
            fields.push(format!("payoff_transform {spelled}"));
        }
    }
    let uses_stake = instance
        .payoff_transform
        .as_deref()
        .and_then(|t| litigate_engine::expr::parse(t).ok())
        .is_some_and(|e| e.vars().iter().any(|v| v == "stake"));
    if let Some(stake) = instance.stake {
        let at = format!("{pointer}/stake");
        fields.push(format!(
            "stake {}",
            issues.spell(text::money(stake), &at, "stake")
        ));
    } else if uses_stake {
        fields.push("stake ?".to_owned());
    }
    for (edge, patch) in &instance.patch_edges {
        let at = format!("{pointer}/patch_edges/{edge}");
        let edge = spelling.issues.ident(edge, &at, false);
        if let Some(body) = patch_fields(patch, &at, spelling) {
            fields.push(format!("patch {edge} {{\n{}\n}}", body.join("\n")));
        }
    }
    format!("instance {base}@{origin} {{\n{}\n}}", fields.join("\n"))
}

/// The `.lit` spelling of a role.
pub(crate) fn role_name(role: Role) -> &'static str {
    match role {
        Role::Me => "self",
        Role::Opponent => "opponent",
        Role::Nature => "nature",
    }
}

/// Keys of `RawEdge` JSON a `patch` writes (SPEC §8.4), with the empty
/// value each list or record field lowers to when written empty.
const PATCHABLE: [(&str, Option<&str>); 15] = [
    ("authority", None),
    ("hours", None),
    ("cost", None),
    ("deadline", None),
    ("duration", None),
    ("valence", None),
    ("courtListenerUrl", None),
    ("actionId", None),
    ("note", None),
    ("tags", Some("[]")),
    ("attrs", Some("{}")),
    ("sets", Some("[]")),
    ("clears", Some("[]")),
    ("requires", Some("[]")),
    ("forbids", Some("[]")),
];

/// A merge patch as the typed edge fields of a `patch`, each replacing
/// the base edge's whole field: a null member of an object field is left
/// out, a null `cost` is a hole, and a written empty list or record is
/// spelled empty. A key that is not an edge field a patch writes, another
/// null, or a value that is not a whole field has no `.lit` spelling.
fn patch_fields(
    patch: &serde_json::Value,
    pointer: &str,
    spelling: &mut Spelling<'_>,
) -> Option<Vec<String>> {
    let serde_json::Value::Object(map) = patch else {
        spelling
            .issues
            .report(pointer, "an edge patch must be a JSON object (SPEC §8.4)");
        return None;
    };
    let mut whole = serde_json::Map::new();
    let mut extra = Vec::new();
    for (key, value) in map {
        let at = format!("{pointer}/{key}");
        let Some((_, empty)) = PATCHABLE.iter().find(|(k, _)| k == key) else {
            spelling.issues.report(
                &at,
                format!("a `patch` writes edge fields only; `{key}` has no `.lit` spelling"),
            );
            continue;
        };
        match (value, empty) {
            (serde_json::Value::Null, _) if key == "cost" => extra.push("cost ?".to_owned()),
            (serde_json::Value::Null, _) => spelling.issues.report(
                &at,
                format!("a `patch` cannot unset `{key}`; it replaces a field with a value"),
            ),
            (serde_json::Value::Array(items), Some(empty)) if items.is_empty() => {
                extra.push(format!("{key} {empty}"));
            }
            (serde_json::Value::Object(members), Some(empty)) if members.is_empty() => {
                extra.push(format!("{key} {empty}"));
            }
            (serde_json::Value::Object(members), _) => {
                let kept = members.iter().filter(|(_, v)| !v.is_null());
                whole.insert(
                    key.clone(),
                    kept.map(|(k, v)| (k.clone(), v.clone())).collect(),
                );
            }
            _ => {
                whole.insert(key.clone(), value.clone());
            }
        }
    }
    whole.insert("from".into(), "".into());
    whole.insert("to".into(), "".into());
    whole.insert("label".into(), "".into());
    // A window must be whole too: a `length` or a designated date.
    let edge = serde_json::from_value::<RawEdge>(serde_json::Value::Object(whole))
        .map_err(|e| e.to_string())
        .and_then(|edge| {
            edge.deadline
                .iter()
                .try_for_each(|d| d.check("the patch", PACK_SCHEMA_VERSION, PACK_SCHEMA_VERSION))
                .map(|()| edge)
                .map_err(|e| e.to_string())
        });
    match edge {
        Ok(edge) => {
            let mut fields = spelling.edge_fields(&edge, pointer);
            fields.extend(extra);
            Some(fields)
        }
        Err(e) => {
            spelling.issues.report(
                pointer,
                format!("a `patch` replaces whole edge fields; this one is not: {e}"),
            );
            None
        }
    }
}
