//! Semantic equality and diff of litgraph documents (SPEC §8.6): the
//! migration oracle behind `litigate diff`.
//!
//! Both sides are read into litgraph's own serde types (which applies serde
//! defaults), normalized, and compared: objects by key, numbers by binary64
//! bits, arrays in order, except that nodes, edges, links, groups and
//! sources are matched by id so a change is reported against the id it
//! belongs to (`/nodes/answer-due/label`) and a pure reordering is one
//! `order` change (`/links/order` for links, whose whole order counts:
//! unlike pack edges they have no node list to regroup under). Source maps and the generated marker are sidecars and
//! never part of a document.
use litgraph::clock::ClockLibrary;
use litgraph::model::{local_edge_ids, LinkFile, NodeKind, Pack, RawEdge};
use litgraph::scenario::NamedScenario;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// What happened at a pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Present only in the second document.
    Added,
    /// Present only in the first document.
    Removed,
    /// Present in both with different values.
    Changed,
    /// The same ids in a different order (node and link order, or edge
    /// order within one source node, is significant).
    Order,
}

/// One difference. `pointer` addresses id-keyed arrays by id
/// (`/nodes/answer-due/label`, `/edges/respond->answered#0/cost`) and other
/// arrays by index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    /// Kind of difference.
    pub kind: ChangeKind,
    /// Where.
    pub pointer: String,
    /// The node, edge, link, group, source or instance id it belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The first document's value (absent when added).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    /// The second document's value (absent when removed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

/// The kind of a litgraph JSON document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    /// A pack.
    Pack,
    /// `links.json`.
    Links,
    /// A named scenario.
    Scenario,
    /// A clock-library fragment.
    Clock,
}

/// Differences between two packs, empty when they are semantically equal.
#[must_use]
pub fn semantic_diff(a: &Pack, b: &Pack) -> Vec<Change> {
    compare(&normalize_pack(a), &normalize_pack(b))
}

/// Differences between two link files.
#[must_use]
pub fn semantic_diff_links(a: &LinkFile, b: &LinkFile) -> Vec<Change> {
    compare(&normalize_links(a), &normalize_links(b))
}

/// Differences between two named scenarios.
#[must_use]
pub fn semantic_diff_scenarios(a: &NamedScenario, b: &NamedScenario) -> Vec<Change> {
    compare(&normalized(a), &normalized(b))
}

/// Differences between two clock-library fragments.
#[must_use]
pub fn semantic_diff_clock(a: &ClockLibrary, b: &ClockLibrary) -> Vec<Change> {
    compare(&normalized(a), &normalized(b))
}

/// Which kind of document `value` is, from its keys.
#[must_use]
pub fn detect(value: &Value) -> Option<DocumentKind> {
    let has = |k: &str| value.get(k).is_some();
    if has("nodes") || has("startNodeId") {
        Some(DocumentKind::Pack)
    } else if has("countingRules") || has("calendars") {
        Some(DocumentKind::Clock)
    } else if has("summary") || has("scenario") {
        Some(DocumentKind::Scenario)
    } else if has("links") || has("instances") {
        Some(DocumentKind::Links)
    } else {
        None
    }
}

/// Differences between two documents of the same kind given as JSON text
/// (the kind is detected from the first).
///
/// # Errors
/// A message when either text is not JSON, the kind is unknown or differs,
/// or a document does not deserialize as that kind.
pub fn semantic_diff_json(a: &str, b: &str) -> Result<(DocumentKind, Vec<Change>), String> {
    let parse = |label: &str, text: &str| -> Result<Value, String> {
        serde_json::from_str(text).map_err(|e| format!("{label}: not JSON: {e}"))
    };
    let (x, y) = (parse("first", a)?, parse("second", b)?);
    let kind = detect(&x).ok_or("first: not a litgraph pack, links, scenario or clock file")?;
    if detect(&y) != Some(kind) {
        return Err(format!("second: not a {kind:?} document").to_lowercase());
    }
    let changes = match kind {
        DocumentKind::Pack => semantic_diff(&read("first", x)?, &read("second", y)?),
        DocumentKind::Links => semantic_diff_links(&read("first", x)?, &read("second", y)?),
        DocumentKind::Scenario => semantic_diff_scenarios(&read("first", x)?, &read("second", y)?),
        DocumentKind::Clock => semantic_diff_clock(&read("first", x)?, &read("second", y)?),
    };
    Ok((kind, changes))
}

fn read<T: serde::de::DeserializeOwned>(label: &str, v: Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| format!("{label}: {e}"))
}

fn normalized<T: Serialize>(value: &T) -> Value {
    numbers_as_f64(serde_json::to_value(value).unwrap_or(Value::Null))
}

/// Derived ids and default units on edges.
fn normalize_edges(edges: &mut [RawEdge]) {
    let ids = local_edge_ids(edges);
    for (edge, id) in edges.iter_mut().zip(ids) {
        edge.id = Some(id);
    }
    for edge in edges {
        if let Some(deadline) = &mut edge.deadline {
            deadline.unit.get_or_insert_with(|| "calendar".into());
        }
    }
}

/// SPEC §8.6 for a pack: absent kind is `state`, absent deadline unit is
/// `calendar`, absent edge id is the derived `from->to#n`, and edges are
/// stably grouped by their source node's position.
#[must_use]
pub fn normalize_pack(pack: &Pack) -> Value {
    let mut pack = pack.clone();
    normalize_edges(&mut pack.edges);
    for node in &mut pack.nodes {
        node.kind.get_or_insert(NodeKind::State);
    }
    let position: BTreeMap<&str, usize> = pack
        .nodes
        .iter()
        .enumerate()
        .rev()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut keyed: Vec<(usize, RawEdge)> = pack
        .edges
        .iter()
        .map(|e| {
            (
                position.get(e.from.as_str()).copied().unwrap_or(usize::MAX),
                e.clone(),
            )
        })
        .collect();
    keyed.sort_by_key(|(at, _)| *at);
    pack.edges = keyed.into_iter().map(|(_, e)| e).collect();
    normalized(&pack)
}

/// SPEC §8.6 for a link file: derived link ids and default units.
#[must_use]
pub fn normalize_links(file: &LinkFile) -> Value {
    let mut file = file.clone();
    normalize_edges(&mut file.links);
    normalized(&file)
}

/// Every JSON number as binary64, so `45` equals `45.0`.
fn numbers_as_f64(value: Value) -> Value {
    match value {
        Value::Number(n) => n
            .as_f64()
            .and_then(serde_json::Number::from_f64)
            .map_or(Value::Null, Value::Number),
        Value::Array(items) => Value::Array(items.into_iter().map(numbers_as_f64).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, numbers_as_f64(v)))
                .collect(),
        ),
        other => other,
    }
}

/// Arrays matched by an id field, and whether their order is compared as
/// a whole or within groups. Pack edges are regrouped under their source
/// node, so only order within one node is significant; links have no node
/// list to group by, so their whole order is (SPEC §8.6).
fn keyed(pointer: &str) -> Option<(&'static str, Option<&'static str>)> {
    match pointer {
        "/nodes" | "/groups" | "/sources" | "/calendars" | "/countingRules" | "/links" => {
            Some(("id", None))
        }
        "/edges" => Some(("id", Some("from"))),
        _ => None,
    }
}

fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// Equality of two leaves (`walk` descends into arrays and objects itself):
/// numbers by binary64 bits.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            x.as_f64().map(f64::to_bits) == y.as_f64().map(f64::to_bits)
        }
        _ => a == b,
    }
}

fn compare(a: &Value, b: &Value) -> Vec<Change> {
    let mut out = Vec::new();
    walk(&mut out, "", None, a, b);
    out
}

fn change(
    out: &mut Vec<Change>,
    kind: ChangeKind,
    pointer: String,
    id: Option<&str>,
    before: Option<&Value>,
    after: Option<&Value>,
) {
    out.push(Change {
        kind,
        pointer,
        id: id.map(str::to_owned),
        before: before.cloned(),
        after: after.cloned(),
    });
}

fn walk(out: &mut Vec<Change>, pointer: &str, id: Option<&str>, a: &Value, b: &Value) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => objects(out, pointer, id, x, y),
        (Value::Array(x), Value::Array(y)) => match keyed(pointer) {
            Some((key, group)) if unique(x, key) && unique(y, key) => {
                by_id(out, pointer, key, group, x, y);
            }
            _ => arrays(out, pointer, id, x, y),
        },
        _ if same(a, b) => {}
        _ => change(
            out,
            ChangeKind::Changed,
            pointer.to_owned(),
            id,
            Some(a),
            Some(b),
        ),
    }
}

fn objects(
    out: &mut Vec<Change>,
    pointer: &str,
    id: Option<&str>,
    x: &Map<String, Value>,
    y: &Map<String, Value>,
) {
    let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
    for k in keys {
        let at = format!("{pointer}/{}", escape(k));
        let id = if pointer == "/instances" {
            Some(k.as_str())
        } else {
            id
        };
        match (x.get(k), y.get(k)) {
            (Some(p), Some(q)) => walk(out, &at, id, p, q),
            (p, q) => change(
                out,
                if p.is_some() {
                    ChangeKind::Removed
                } else {
                    ChangeKind::Added
                },
                at,
                id,
                p,
                q,
            ),
        }
    }
}

fn arrays(out: &mut Vec<Change>, pointer: &str, id: Option<&str>, x: &[Value], y: &[Value]) {
    for i in 0..x.len().max(y.len()) {
        let at = format!("{pointer}/{i}");
        match (x.get(i), y.get(i)) {
            (Some(p), Some(q)) => walk(out, &at, id, p, q),
            (p, q) => change(
                out,
                if p.is_some() {
                    ChangeKind::Removed
                } else {
                    ChangeKind::Added
                },
                at,
                id,
                p,
                q,
            ),
        }
    }
}

fn id_of<'v>(v: &'v Value, key: &str) -> Option<&'v str> {
    v.get(key).and_then(Value::as_str)
}

fn unique(items: &[Value], key: &str) -> bool {
    let ids: BTreeSet<Option<&str>> = items.iter().map(|v| id_of(v, key)).collect();
    ids.len() == items.len() && !ids.contains(&None)
}

/// Id-keyed arrays: added/removed/changed per id, then one `order` change
/// per group whose common ids are permuted.
fn by_id(
    out: &mut Vec<Change>,
    pointer: &str,
    key: &str,
    group: Option<&str>,
    x: &[Value],
    y: &[Value],
) {
    let index = |items: &[Value]| -> BTreeMap<String, Value> {
        items
            .iter()
            .map(|v| (id_of(v, key).unwrap_or_default().to_owned(), v.clone()))
            .collect()
    };
    let (xs, ys) = (index(x), index(y));
    for v in x {
        let id = id_of(v, key).unwrap_or_default();
        let at = format!("{pointer}/{}", escape(id));
        match ys.get(id) {
            Some(w) => walk(out, &at, Some(id), v, w),
            None => change(out, ChangeKind::Removed, at, Some(id), Some(v), None),
        }
    }
    for w in y {
        let id = id_of(w, key).unwrap_or_default();
        if !xs.contains_key(id) {
            change(
                out,
                ChangeKind::Added,
                format!("{pointer}/{}", escape(id)),
                Some(id),
                None,
                Some(w),
            );
        }
    }
    let order =
        |items: &[Value], other: &BTreeMap<String, Value>| -> BTreeMap<String, Vec<Value>> {
            let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
            for v in items {
                let id = id_of(v, key).unwrap_or_default();
                if other.contains_key(id) {
                    let g = group.and_then(|g| id_of(v, g)).unwrap_or_default();
                    groups
                        .entry(g.to_owned())
                        .or_default()
                        .push(Value::String(id.to_owned()));
                }
            }
            groups
        };
    let (before, after) = (order(x, &ys), order(y, &xs));
    for (g, ids) in &before {
        if after.get(g) != Some(ids) {
            let at = if group.is_some() {
                format!("{pointer}/order/{}", escape(g))
            } else {
                format!("{pointer}/order")
            };
            let id = group.map(|_| g.as_str());
            let (p, q) = (
                Value::Array(ids.clone()),
                Value::Array(after.get(g).cloned().unwrap_or_default()),
            );
            change(out, ChangeKind::Order, at, id, Some(&p), Some(&q));
        }
    }
}
