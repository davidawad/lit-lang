//! JSON values as `JsonRecord`/`JsonValue` source (SPEC §3.1): instance
//! `patch` records and the `.case` `analysis` block.
use crate::issues::Issues;
use crate::text;
use serde_json::Value;

/// A JSON object as a `JsonRecord`. Keys that are identifiers stay bare;
/// any other key is a string literal.
pub(crate) fn record(
    map: &serde_json::Map<String, Value>,
    pointer: &str,
    issues: &mut Issues,
) -> String {
    let entries: Vec<String> = map
        .iter()
        .map(|(key, value)| {
            let spelled_key = if text::is_ident(key) {
                key.clone()
            } else {
                text::string(key)
            };
            format!(
                "{spelled_key}: {}",
                value_text(value, &format!("{pointer}/{key}"), issues)
            )
        })
        .collect();
    if entries.is_empty() {
        "{}".to_owned()
    } else {
        format!("{{ {} }}", entries.join(", "))
    }
}

/// One JSON value: numbers exact (integers verbatim, floats as their
/// shortest round-tripping decimal), `null` as `unset`.
pub(crate) fn value_text(value: &Value, pointer: &str, issues: &mut Issues) -> String {
    match value {
        Value::Null => "unset".to_owned(),
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
        Value::Object(map) => record(map, pointer, issues),
    }
}
