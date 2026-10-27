//! MCP resources over the library's compiled packs and matters: the
//! engine manual, `links.json`, every pack, and the named scenario of
//! every matter, under `litigate://` URIs.

use litigate_engine::api::{self, Catalog};
use serde_json::{json, Value};

/// The engine manual (the `describe` op over the library).
const DESCRIBE_URI: &str = "litigate://describe";
/// The links composing the library's packs.
const LINKS_URI: &str = "litigate://links";
/// One pack per resource; `<id>` is the pack's own `id`.
const PACK_PREFIX: &str = "litigate://packs/";
/// Every named scenario (id, summary, packs).
const SCENARIOS_INDEX_URI: &str = "litigate://scenarios";
/// One named scenario per resource; `<id>` is its own `id`.
const SCENARIO_PREFIX: &str = "litigate://scenarios/";

fn resource(uri: &str, name: &str, description: &str) -> Value {
    json!({
        "uri": uri,
        "name": name,
        "description": description,
        "mimeType": "application/json",
    })
}

/// Every resource `catalog` gives.
#[must_use]
pub fn list(catalog: &Catalog) -> Vec<Value> {
    let mut out = vec![
        resource(
            DESCRIBE_URI,
            "describe",
            "The engine's machine-readable manual over the library: ops, scenario fields, metrics, variables, params, and the named scenarios.",
        ),
        resource(LINKS_URI, "links", "The links composing the library's packs: where forums meet."),
        resource(
            SCENARIOS_INDEX_URI,
            "scenarios",
            "Every matter's named scenario: id, summary, the packs it needs. Read litigate://scenarios/<id> for one.",
        ),
    ];
    out.extend(
        catalog.packs.iter().map(|(_, pack, _)| {
            resource(&format!("{PACK_PREFIX}{}", pack.id), &pack.id, &pack.title)
        }),
    );
    out.extend(
        catalog
            .scenarios()
            .map(|s| resource(&format!("{SCENARIO_PREFIX}{}", s.id), &s.id, &s.summary)),
    );
    out
}

/// The document at `uri`, or `None` when `catalog` has none there.
fn document(catalog: &Catalog, uri: &str) -> Option<Value> {
    if uri == DESCRIBE_URI {
        return Some(api::describe(catalog));
    }
    if uri == LINKS_URI {
        return serde_json::to_value(&catalog.links).ok();
    }
    if uri == SCENARIOS_INDEX_URI {
        return Some(json!({
            "origin": catalog.scenarios_origin,
            "scenarios": catalog.scenarios().map(|s| json!({
                "id": s.id,
                "summary": s.summary,
                "packs": s.packs,
            })).collect::<Vec<_>>(),
        }));
    }
    if let Some(id) = uri.strip_prefix(PACK_PREFIX) {
        let (_, pack, _) = catalog.packs.iter().find(|(_, p, _)| p.id == id)?;
        return serde_json::to_value(pack).ok();
    }
    let id = uri.strip_prefix(SCENARIO_PREFIX)?;
    serde_json::to_value(catalog.scenario(id).ok()?).ok()
}

/// The `resources/read` contents of `uri`, or `None` when `catalog` has
/// no resource there.
#[must_use]
pub fn read(catalog: &Catalog, uri: &str) -> Option<Value> {
    // A `serde_json::Value` always prints.
    let text = serde_json::to_string_pretty(&document(catalog, uri)?).unwrap_or_default();
    Some(json!([{"uri": uri, "mimeType": "application/json", "text": text}]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        let lib = vec!["tests/fixtures/cov-cmds/lit".to_owned()];
        let cases = vec!["tests/fixtures/cov-cmds/cases".to_owned()];
        crate::library::library_catalog(&lib, &cases).unwrap()
    }

    fn text(contents: Option<Value>) -> Value {
        let text = contents.unwrap_or_default()[0]["text"].clone();
        serde_json::from_str(text.as_str().unwrap_or_default()).unwrap_or_default()
    }

    #[test]
    fn every_pack_and_scenario_is_a_resource() {
        let c = catalog();
        let uris: Vec<Value> = list(&c).into_iter().map(|r| r["uri"].clone()).collect();
        assert_eq!(
            uris,
            [
                "litigate://describe",
                "litigate://links",
                "litigate://scenarios",
                "litigate://packs/cov-pack",
                "litigate://scenarios/cov-matter"
            ]
        );
    }

    #[test]
    fn each_resource_reads_as_its_document() {
        let c = catalog();
        assert_eq!(text(read(&c, DESCRIBE_URI)), api::describe(&c));
        assert!(text(read(&c, LINKS_URI)).is_object());
        let index = text(read(&c, SCENARIOS_INDEX_URI));
        assert_eq!(index["scenarios"][0]["id"], json!("cov-matter"));
        assert_eq!(index["scenarios"][0]["packs"], json!(["cov-pack"]));
        assert_eq!(
            text(read(&c, "litigate://packs/cov-pack"))["id"],
            json!("cov-pack")
        );
        let matter = text(read(&c, "litigate://scenarios/cov-matter"));
        assert_eq!(matter["id"], json!("cov-matter"));
        assert!(matter["scenario"].is_object());
    }

    #[test]
    fn an_unknown_uri_has_no_resource() {
        let c = catalog();
        for uri in [
            "litigate://packs/nope",
            "litigate://scenarios/nope",
            "litigate://nope",
        ] {
            assert!(read(&c, uri).is_none(), "{uri}");
        }
    }
}
