//! `Lit.toml` for an imported package (SPEC §4.1), written by hand so the
//! comments that tell the porter what is a placeholder survive.
use crate::links::role_name as role;
use litgraph::model::{CiteCode, Pack};
use std::fmt::Write as _;

/// The version every imported package starts at.
pub(crate) const VERSION: &str = "0.1.0";

/// Default `in_force` of an imported package. Legacy JSON records no
/// effective date, so this is an explicit placeholder: every `as_of` falls
/// inside it, and the porter replaces it with the real date.
pub(crate) const IN_FORCE_PLACEHOLDER: &str = "0001-01-01";

/// The placeholder authority level of an imported package (legacy JSON
/// records none); the porter sets the real one.
pub(crate) const AUTHORITY_PLACEHOLDER: &str = "Unclassified";

/// A TOML basic string.
fn toml_string(value: &str) -> String {
    let mut out = String::from('"');
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A TOML key: bare when it can be, else quoted.
fn toml_key(key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if bare {
        key.to_owned()
    } else {
        toml_string(key)
    }
}

fn toml_list(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|s| toml_string(s)).collect();
    format!("[{}]", items.join(", "))
}

/// The newest `asOf` among the pack's sources: the corpus date the pack's
/// text reflects (`known_at`).
pub(crate) fn known_at(pack: &Pack) -> String {
    pack.sources
        .iter()
        .filter_map(|source| source.as_of.clone())
        .max()
        .unwrap_or_else(|| IN_FORCE_PLACEHOLDER.to_owned())
}

/// `Lit.toml` for `pack`, imported from `from`.
pub(crate) fn pack_manifest(pack: &Pack, from: &str, forum: &str) -> String {
    let mut out = vec![
        format!("# Imported by `litigate import` from {from}."),
        format!("# in_force and authority are placeholders: legacy JSON records neither."),
        "[package]".to_owned(),
        format!("name = {}", toml_string(&pack.id)),
        format!("version = {}", toml_string(VERSION)),
        "language = 1".to_owned(),
        format!("pack_id = {}", toml_string(&pack.id)),
        format!("forum = {}", toml_string(forum)),
        format!("authority = {}", toml_string(AUTHORITY_PLACEHOLDER)),
        format!("title = {}", toml_string(&pack.title)),
    ];
    let optional = [
        ("description", &pack.description),
        ("jurisdiction", &pack.jurisdiction),
    ];
    out.extend(optional.iter().filter_map(|(key, value)| {
        value
            .as_ref()
            .map(|v| format!("{key} = {}", toml_string(v)))
    }));
    out.push(format!("in_force = {}", toml_string(IN_FORCE_PLACEHOLDER)));
    out.push(format!("known_at = {}", toml_string(&known_at(pack))));
    if let Some(counting) = &pack.counting {
        out.push(format!("counting = {}", toml_string(counting)));
    }
    out.push(String::new());
    out.push("[package.roles]".to_owned());
    out.extend(
        pack.roles
            .iter()
            .map(|(actor, r)| format!("{} = {}", toml_key(actor), toml_string(role(*r)))),
    );
    for source in &pack.sources {
        out.push(String::new());
        out.push(format!("[sources.{}]", toml_key(&source.id)));
        let fields = [
            ("title", &source.title),
            ("url", &source.url),
            ("path", &source.path),
            ("sha256", &source.sha256),
            ("as_of", &source.as_of),
        ];
        out.extend(fields.iter().filter_map(|(key, value)| {
            value
                .as_ref()
                .map(|v| format!("{key} = {}", toml_string(v)))
        }));
    }
    for rule in &pack.cite_syntax.rules {
        out.push(String::new());
        out.push("[[cite_syntax.rules]]".to_owned());
        out.push(format!("prefixes = {}", toml_list(&rule.prefixes)));
        out.push(format!("heading = {}", toml_string(&rule.heading)));
        if let Some(source) = &rule.source {
            out.push(format!("source = {}", toml_string(source)));
        }
    }
    for shorthand in &pack.cite_syntax.shorthands {
        out.push(String::new());
        out.push("[[cite_syntax.shorthands]]".to_owned());
        out.push(format!("prefix = {}", toml_string(&shorthand.prefix)));
        let code = match shorthand.code {
            CiteCode::Usc => "usc",
            CiteCode::Cfr => "cfr",
        };
        out.push(format!("code = {}", toml_string(code)));
        out.push(format!("title = {}", shorthand.title));
        out.push(format!("section_parts = {}", shorthand.section_parts));
        if let Some(source) = &shorthand.source {
            out.push(format!("source = {}", toml_string(source)));
        }
    }
    out.push(String::new());
    out.join("\n")
}

/// `Lit.toml` for the links package: it has no pack identity of its own
/// (SPEC §8.4), only the packages its links and instances reference.
pub(crate) fn links_manifest(
    from: &str,
    known_at: &str,
    dependencies: &[(String, String)],
) -> String {
    let mut out = vec![
        format!("# Imported by `litigate import` from {from}."),
        "# in_force is a placeholder: legacy JSON records no effective date.".to_owned(),
        "[package]".to_owned(),
        "name = \"links\"".to_owned(),
        format!("version = {}", toml_string(VERSION)),
        "language = 1".to_owned(),
        format!("in_force = {}", toml_string(IN_FORCE_PLACEHOLDER)),
        format!("known_at = {}", toml_string(known_at)),
        String::new(),
        "[dependencies]".to_owned(),
    ];
    out.extend(dependencies.iter().map(|(name, dir)| {
        format!(
            "{} = {{ version = {}, path = {} }}",
            toml_key(name),
            toml_string(&format!("^{VERSION}")),
            toml_string(&format!("../{dir}"))
        )
    }));
    out.push(String::new());
    out.join("\n")
}

#[cfg(test)]
mod tests;
