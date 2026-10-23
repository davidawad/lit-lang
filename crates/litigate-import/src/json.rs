//! A scenario's remaining fields as the `.case` `analysis` block (SPEC
//! §3.2, §8.3 in reverse): a record of expressions, with each edge of
//! `remove_edges` an unquoted `EdgeRef`.
use crate::issues::Issues;
use crate::text;
use serde_json::{Map, Value};

/// Fields that name engine nodes or edges by string, which `analysis`
/// does not type.
const UNTYPED: [&str; 2] = ["probabilities", "policy"];

/// `rest` as an `analysis` record; `None` when nothing is left to write.
pub(crate) fn analysis(rest: &Map<String, Value>, issues: &mut Issues) -> Option<String> {
    let entries: Vec<String> = rest
        .iter()
        .filter_map(|(key, value)| {
            let at = format!("/scenario/{key}");
            if UNTYPED.contains(&key.as_str()) {
                issues.report(
                    &at,
                    format!(
                        "`{key}` names engine nodes or edges by string; `analysis` has no `{key}`"
                    ),
                );
                return None;
            }
            let spelled = match (key.as_str(), value) {
                ("remove_edges", Value::Array(items)) => edges(items, &at, issues),
                _ => value_text(value, &at, issues),
            };
            Some(format!("{key}: {spelled}"))
        })
        .collect();
    (!entries.is_empty()).then(|| format!("analysis {{ {} }}", entries.join(", ")))
}

/// Edge ids as a list of `EdgeRef`s.
fn edges(items: &[Value], pointer: &str, issues: &mut Issues) -> String {
    let items: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let at = format!("{pointer}/{i}");
            issues.ident(item.as_str().unwrap_or_default(), &at, true)
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// One value as an expression: numbers exact (integers verbatim, floats as
/// their shortest round-tripping decimal), records keyed by identifiers.
/// `null` has no spelling.
fn value_text(value: &Value, pointer: &str, issues: &mut Issues) -> String {
    match value {
        Value::Null => {
            issues.report(pointer, "`null` has no `.case` spelling");
            "?".to_owned()
        }
        Value::Bool(b) => b.to_string(),
        Value::String(s) => text::string(s),
        Value::Number(n) if n.is_i64() || n.is_u64() => n.to_string(),
        Value::Number(n) => issues.spell(n.as_f64().and_then(text::number), pointer, "number"),
        Value::Array(items) => {
            let items: Vec<String> = items
                .iter()
                .enumerate()
                .map(|(i, item)| value_text(item, &format!("{pointer}/{i}"), issues))
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(key, value)| {
                    let at = format!("{pointer}/{key}");
                    let key = issues.ident(key, &at, false);
                    format!("{key}: {}", value_text(value, &at, issues))
                })
                .collect();
            if entries.is_empty() {
                "{}".to_owned()
            } else {
                format!("{{ {} }}", entries.join(", "))
            }
        }
    }
}
