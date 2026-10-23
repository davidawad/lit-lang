//! `links.json` to a links package (SPEC §8.4 in reverse): `instance` and
//! `link` declarations in one module.
use crate::issues::Issues;
use crate::json;
use crate::node::Spelling;
use crate::text;
use crate::Emitted;
use litgraph::cite::normalize::CiteGrammar;
use litgraph::model::{CiteSyntax, Instance, LinkFile, Role};
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
        module.push(instance_text(name, instance, issues));
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

fn instance_text(name: &str, instance: &Instance, issues: &mut Issues) -> String {
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
    if let Some(transform) = &instance.payoff_transform {
        fields.push(format!("payoff_transform {}", text::string(transform)));
    }
    let uses_stake = instance
        .payoff_transform
        .as_deref()
        .and_then(|t| litgraph::expr::parse(t).ok())
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
        let edge = issues.ident(edge, &at, false);
        match patch {
            serde_json::Value::Object(map) => {
                fields.push(format!("patch {edge} {}", json::record(map, &at, issues)));
            }
            _ => issues.report(&at, "an edge patch must be a JSON object (SPEC §8.4)"),
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
