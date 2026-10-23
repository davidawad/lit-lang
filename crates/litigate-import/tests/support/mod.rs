//! Shared test support: the repository corpus, the read-back oracle, and
//! SPEC §8.6 semantic normalization over litgraph's own serde types.
#![allow(dead_code)]
pub mod reader;

use litgraph::model::{local_edge_ids, LinkFile, NodeKind, Pack};
use litgraph::scenario::NamedScenario;
use litigate_import::{import, ImportedFile, Input};
use litigate_syntax::ast::{self, AstNode};
use litigate_syntax::{FileKind, SyntaxKind};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// The repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every legacy JSON input in the repository: packs, `links.json`, and the
/// named scenarios.
pub fn corpus() -> Vec<Input> {
    let mut paths = Vec::new();
    for dir in ["packs", "scenarios"] {
        let entries = std::fs::read_dir(repo_root().join(dir))
            .map(|r| r.filter_map(Result::ok).collect::<Vec<_>>());
        for entry in entries.unwrap_or_default() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_ext(&name, "json") {
                paths.push(format!("{dir}/{name}"));
            }
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(repo_root().join(&path)).unwrap_or_default();
            Input::new(path, text)
        })
        .collect()
}

/// Imports `inputs`, failing the test with every diagnostic on error.
pub fn import_ok(inputs: &[Input]) -> Vec<ImportedFile> {
    match import(inputs) {
        Ok(imported) => imported.files,
        Err(diagnostics) => panic!(
            "{}",
            serde_json::to_string_pretty(&diagnostics).unwrap_or_default()
        ),
    }
}

/// The files of the package directory `dir` (`lit/frcp`).
pub fn package<'a>(files: &'a [ImportedFile], dir: &str) -> Vec<&'a ImportedFile> {
    files
        .iter()
        .filter(|f| f.path.starts_with(&format!("{dir}/")))
        .collect()
}

/// What the `.lit` modules of a package contribute to its `Pack`.
struct Modules {
    groups: Vec<Value>,
    nodes: Vec<Value>,
    edges: Vec<Value>,
    start: Option<String>,
}

/// Reads the package's modules in path order (SPEC §4.1 module order).
fn read_modules(files: &[&ImportedFile]) -> Modules {
    let mut groups = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut start = None;
    let mut modules: Vec<&&ImportedFile> =
        files.iter().filter(|f| is_ext(&f.path, "lit")).collect();
    modules.sort_by(|a, b| a.path.cmp(&b.path));
    for file in modules {
        let parse = reader::parsed(&file.text, FileKind::Lit, &file.path);
        for child in parse.syntax().children() {
            match child.kind() {
                SyntaxKind::GroupDecl => {
                    let decl = ast::GroupDecl::cast(child.clone());
                    let id = decl
                        .as_ref()
                        .and_then(ast::GroupDecl::name)
                        .map(|t| t.text().to_owned());
                    let label = decl
                        .and_then(|d| d.label())
                        .map(|t| ast::unescape(t.text()));
                    groups.push(json!({ "id": id, "label": label }));
                }
                SyntaxKind::StartDecl => {
                    start = ast::StartDecl::cast(child.clone())
                        .and_then(|s| s.target())
                        .map(|r| r.text());
                }
                _ => {
                    if let Some(decl) = ast::NodeDecl::cast(child.clone()) {
                        let (node, out) = reader::node(&decl);
                        nodes.push(node);
                        edges.extend(out);
                    }
                }
            }
        }
    }
    Modules {
        groups,
        nodes,
        edges,
        start,
    }
}

/// Whether `path` has extension `ext`.
pub fn is_ext(path: &str, ext: &str) -> bool {
    Path::new(path).extension().is_some_and(|e| e == ext)
}

/// Reads a pack package back into a litgraph `Pack` (SPEC §8.1).
pub fn read_pack(files: &[&ImportedFile]) -> Pack {
    let manifest = files
        .iter()
        .find(|f| f.path.ends_with("/Lit.toml"))
        .map(|f| f.text.as_str());
    let manifest: toml::Table = manifest.unwrap_or_default().parse().unwrap_or_default();
    let package = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let get = |key: &str| {
        package
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let Modules {
        groups,
        nodes,
        edges,
        start,
    } = read_modules(files);
    let counting = get("counting");
    let mut pack = Map::new();
    pack.insert(
        "schemaVersion".into(),
        json!(if counting.is_some() { 3 } else { 2 }),
    );
    for (key, json_key) in [
        ("pack_id", "id"),
        ("title", "title"),
        ("description", "description"),
        ("jurisdiction", "jurisdiction"),
        ("forum", "forum"),
    ] {
        if let Some(v) = get(key) {
            pack.insert(json_key.into(), json!(v));
        }
    }
    if let Some(counting) = counting {
        pack.insert("counting".into(), json!(counting));
    }
    pack.insert("startNodeId".into(), json!(start));
    pack.insert("groups".into(), Value::Array(groups));
    if let Some(roles) = package.get("roles") {
        pack.insert("roles".into(), toml_json(Some(roles)));
    }
    sources_and_cite_syntax(&manifest, &mut pack);
    pack.insert("nodes".into(), Value::Array(nodes));
    pack.insert("edges".into(), Value::Array(edges));
    let text = Value::Object(pack).to_string();
    Pack::from_json(&text).unwrap_or_else(|e| panic!("read-back pack is invalid: {e}"))
}

/// `[sources.*]` and `[[cite_syntax.*]]` as `Pack.sources`/`citeSyntax`.
fn sources_and_cite_syntax(manifest: &toml::Table, pack: &mut Map<String, Value>) {
    let sources = manifest
        .get("sources")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let sources: Vec<Value> = sources
        .iter()
        .map(|(id, fields)| {
            let mut source = toml_json(Some(fields));
            if let Some(map) = source.as_object_mut() {
                map.insert("id".into(), json!(id));
                if let Some(as_of) = map.remove("as_of") {
                    map.insert("asOf".into(), as_of);
                }
            }
            source
        })
        .collect();
    pack.insert("sources".into(), Value::Array(sources));
    let mut cite_syntax = toml_json(manifest.get("cite_syntax"));
    if cite_syntax.is_null() {
        cite_syntax = json!({});
    }
    for shorthand in cite_syntax
        .get_mut("shorthands")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        if let Some(map) = shorthand.as_object_mut() {
            if let Some(parts) = map.remove("section_parts") {
                map.insert("sectionParts".into(), parts);
            }
        }
    }
    pack.insert("citeSyntax".into(), cite_syntax);
}

/// TOML to JSON (integers stay integers), `Null` when absent.
fn toml_json(value: Option<&toml::Value>) -> Value {
    value
        .and_then(|v| serde_json::to_value(v).ok())
        .unwrap_or(Value::Null)
}

/// Reads a links package back into a `LinkFile` (SPEC §8.4).
pub fn read_links(files: &[&ImportedFile]) -> LinkFile {
    let module = files
        .iter()
        .find(|f| is_ext(&f.path, "lit"))
        .map(|f| f.text.as_str());
    let value = reader::links(module.unwrap_or_default());
    serde_json::from_value(value).unwrap_or_else(|e| panic!("read-back links are invalid: {e}"))
}

/// Reads a `.case` back into a `NamedScenario` (SPEC §8.3).
pub fn read_case(file: &ImportedFile) -> NamedScenario {
    let parse = reader::parsed(&file.text, FileKind::Case, &file.path);
    let case = parse
        .case_file()
        .unwrap_or_else(|| panic!("{}: not a case", file.path));
    let mut named = Map::new();
    named.insert("id".into(), json!(case.name().map(|t| t.text().to_owned())));
    let token_string = |t: Option<litigate_syntax::SyntaxToken>| t.map(|t| ast::unescape(t.text()));
    named.insert("summary".into(), json!(token_string(case.summary())));
    named.insert(
        "notes".into(),
        json!(token_string(case.notes()).unwrap_or_default()),
    );
    header(&case, &mut named);
    let mut scenario = Map::new();
    if let Some(Value::Object(analysis)) = case
        .block(SyntaxKind::AnalysisBlock)
        .and_then(|b| b.record())
        .map(|r| reader::value(r.syntax()))
    {
        scenario.extend(analysis);
    }
    if let Some(start) = case.start().and_then(|s| s.target()) {
        scenario.insert("start".into(), json!(start.text()));
    }
    if let Some(roles) = case.perspective().and_then(|p| p.roles()) {
        scenario.insert("perspective".into(), reader::value(roles.syntax()));
    }
    let entries = |kind: SyntaxKind| -> Map<String, Value> {
        case.block(kind)
            .map(|b| b.entries())
            .unwrap_or_default()
            .iter()
            .map(|e| {
                (
                    e.key_text().unwrap_or_default(),
                    e.value().map_or(Value::Null, |n| reader::value(&n)),
                )
            })
            .collect()
    };
    for (kind, key) in [
        (SyntaxKind::FactsBlock, "facts"),
        (SyntaxKind::StakesBlock, "payoffs"),
    ] {
        let map = entries(kind);
        if !map.is_empty() {
            scenario.insert(key.into(), Value::Object(map));
        }
    }
    let observe: Map<String, Value> = case
        .block(SyntaxKind::BeliefsBlock)
        .map(|b| b.observations())
        .unwrap_or_default()
        .iter()
        .map(|o| {
            let node = o.node().map(|r| r.text()).unwrap_or_default();
            (
                node,
                o.counts()
                    .map_or(Value::Null, |r| reader::value(r.syntax())),
            )
        })
        .collect();
    if !observe.is_empty() {
        scenario.insert("observe".into(), Value::Object(observe));
    }
    named.insert("scenario".into(), Value::Object(scenario));
    let text = Value::Object(named).to_string();
    NamedScenario::from_json(&text)
        .unwrap_or_else(|e| panic!("{}: read-back scenario is invalid: {e}", file.path))
}

/// SPEC §8.6: absent kind is `state`, absent deadline unit is `calendar`,
/// absent edge id is the derived `from->to#n`, empty equals absent (serde
/// defaults already), and edges are ordered by their source node's position
/// (order within one source node is significant). The format version and
/// the `lowered` header are provenance, not meaning.
pub fn normalize_pack(mut pack: Pack) -> Value {
    pack.schema_version = 0;
    pack.lowered = None;
    let ids = local_edge_ids(&pack.edges);
    for (edge, id) in pack.edges.iter_mut().zip(ids) {
        edge.id = Some(id);
        if let Some(deadline) = &mut edge.deadline {
            deadline.unit.get_or_insert_with(|| "calendar".into());
        }
    }
    for node in &mut pack.nodes {
        node.kind.get_or_insert(NodeKind::State);
    }
    let position = |id: &str| {
        pack.nodes
            .iter()
            .position(|n| n.id == id)
            .unwrap_or(usize::MAX)
    };
    let mut keyed: Vec<(usize, litgraph::model::RawEdge)> = pack
        .edges
        .drain(..)
        .map(|e| (position(&e.from), e))
        .collect();
    keyed.sort_by_key(|(at, _)| *at);
    pack.edges = keyed.into_iter().map(|(_, e)| e).collect();
    numbers_as_f64(serde_json::to_value(&pack).unwrap_or(Value::Null))
}

/// SPEC §8.6 for a links file: link ids derived when absent; no format
/// version or `lowered` header.
pub fn normalize_links(mut file: LinkFile) -> Value {
    file.schema_version = None;
    file.lowered = None;
    let ids = local_edge_ids(&file.links);
    for (edge, id) in file.links.iter_mut().zip(ids) {
        edge.id = Some(id);
        if let Some(deadline) = &mut edge.deadline {
            deadline.unit.get_or_insert_with(|| "calendar".into());
        }
    }
    numbers_as_f64(serde_json::to_value(&file).unwrap_or(Value::Null))
}

/// SPEC §8.6 compares numbers by binary64 value: `45` equals `45.0`.
pub fn numbers_as_f64(value: Value) -> Value {
    match value {
        Value::Number(n) => json!(n.as_f64().unwrap_or(f64::NAN)),
        Value::Array(items) => Value::Array(items.into_iter().map(numbers_as_f64).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, numbers_as_f64(v)))
                .collect(),
        ),
        other => other,
    }
}

/// The first JSON pointer where `a` and `b` differ, for readable failures.
pub fn first_difference(a: &Value, b: &Value, at: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => x.keys().chain(y.keys()).find_map(|k| {
            first_difference(
                x.get(k).unwrap_or(&Value::Null),
                y.get(k).unwrap_or(&Value::Null),
                &format!("{at}/{k}"),
            )
        }),
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => x
            .iter()
            .zip(y)
            .enumerate()
            .find_map(|(i, (p, q))| first_difference(p, q, &format!("{at}/{i}"))),
        _ if a == b => None,
        _ => Some(format!("{at}: {a} != {b}")),
    }
}

/// `uses` and the `source`s of a case.
fn header(case: &ast::CaseFile, named: &mut Map<String, Value>) {
    let token_string = |t: Option<litigate_syntax::SyntaxToken>| t.map(|t| ast::unescape(t.text()));
    let packs: Vec<String> = case
        .uses()
        .map(|u| {
            u.items()
                .iter()
                .filter_map(ast::UseItem::package)
                .map(|t| t.text().to_owned())
                .collect()
        })
        .unwrap_or_default();
    named.insert("packs".into(), json!(packs));
    let sources: Vec<Value> = case
        .sources()
        .iter()
        .map(|source| {
            let mut out = reader::fields(source.fields().into_iter())
                .as_object()
                .cloned()
                .unwrap_or_default();
            out.insert("title".into(), json!(token_string(source.title())));
            Value::Object(out)
        })
        .collect();
    named.insert("sources".into(), json!(sources));
}
