//! A test oracle: reads imported `.lit`/`.case` source back into litgraph
//! JSON through the parser's CST, independently of the importer's writer.
//! It covers exactly the constructs `import` emits and follows SPEC §8 for
//! their lowering, so `read(import(json)) ≡ json` (§8.6) shows the import
//! is lossless before `litigate-lower` exists.
use litigate_syntax::ast::{self, AstNode};
use litigate_syntax::{parse, FileKind, SyntaxKind, SyntaxNode};
use serde_json::{json, Map, Value};

/// Significant text of a node (trivia dropped).
fn text_of(node: &SyntaxNode) -> String {
    node.descendants_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(|t| {
            !matches!(
                t.kind(),
                SyntaxKind::Whitespace | SyntaxKind::Comment | SyntaxKind::DocComment
            )
        })
        .map(|t| t.text().to_owned())
        .collect()
}

fn number(text: &str) -> f64 {
    text.replace('_', "").parse().unwrap_or(f64::NAN)
}

/// The amount of a `Money` or `Duration` token (`2_000.usd`, `4.5.days`).
fn amount(text: &str) -> f64 {
    number(text.rsplit_once('.').map_or(text, |(n, _)| n))
}

/// A value node as JSON: literals by type, refs and cites as strings, id
/// lists as arrays, records and field lists as objects, holes as their
/// reason (the lowering of a cite or `as_of` hole, SPEC §8.1, §8.3).
pub fn value(node: &SyntaxNode) -> Value {
    match node.kind() {
        SyntaxKind::Literal => {
            let Some(token) = node.first_token() else {
                return Value::Null;
            };
            match token.kind() {
                SyntaxKind::Number => json!(number(token.text())),
                SyntaxKind::Money | SyntaxKind::Duration => json!(amount(token.text())),
                SyntaxKind::String => json!(ast::unescape(token.text())),
                SyntaxKind::Date => json!(token.text()),
                SyntaxKind::KwTrue => json!(true),
                SyntaxKind::KwFalse => json!(false),
                _ => Value::Null,
            }
        }
        SyntaxKind::UnaryExpr => {
            let inner = node.children().next().map_or(Value::Null, |n| value(&n));
            json!(-inner.as_f64().unwrap_or(f64::NAN))
        }
        // A rate, `650.usd / 1.hours`: major units per unit of time.
        SyntaxKind::BinaryExpr => {
            let operands: Vec<f64> = node
                .children()
                .map(|n| value(&n).as_f64().unwrap_or(f64::NAN))
                .collect();
            json!(operands[0] / operands[1])
        }
        SyntaxKind::HoleExpr => ast::Hole::cast(node.clone())
            .and_then(|h| h.reason())
            .map_or(Value::Null, Value::String),
        SyntaxKind::CiteExpr => {
            let cite = ast::Cite::cast(node.clone());
            let text = cite.as_ref().and_then(ast::Cite::text);
            let hole = cite.and_then(|c| c.hole()).and_then(|h| h.reason());
            text.or(hole).map_or(Value::Null, Value::String)
        }
        SyntaxKind::IdList | SyntaxKind::ListExpr => {
            Value::Array(node.children().map(|n| value(&n)).collect())
        }
        SyntaxKind::RecordExpr => {
            let mut map = Map::new();
            for field in node.children().filter_map(ast::RecordField::cast) {
                let key = field.key_text().unwrap_or_default();
                let v = field.value().map_or(Value::Null, |n| value(&n));
                map.insert(key, v);
            }
            Value::Object(map)
        }
        SyntaxKind::FieldList => fields(node.children().filter_map(ast::Field::cast)),
        _ => json!(text_of(node)),
    }
}

/// A `payoff_transform` expression as the litgraph text SPEC §8.4 lowers
/// it to, with its precedence level: money in major units, operators
/// spaced, parentheses only where precedence needs them.
fn engine(node: &SyntaxNode) -> (String, u8) {
    let wrap = |(text, level): (String, u8), least: u8| {
        if level < least {
            format!("({text})")
        } else {
            text
        }
    };
    let operands: Vec<SyntaxNode> = node.children().collect();
    match node.kind() {
        SyntaxKind::ParenExpr => engine(&operands[0]),
        SyntaxKind::UnaryExpr => (format!("-{}", wrap(engine(&operands[0]), 3)), 3),
        SyntaxKind::BinaryExpr => {
            let op = node
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .find(|t| !matches!(t.kind(), SyntaxKind::Whitespace | SyntaxKind::Comment))
                .map(|t| t.text().to_owned())
                .unwrap_or_default();
            let level = if op == "+" || op == "-" { 1 } else { 2 };
            let (l, r) = (engine(&operands[0]), engine(&operands[1]));
            (
                format!("{} {op} {}", wrap(l, level), wrap(r, level + 1)),
                level,
            )
        }
        _ => match value(node) {
            Value::Number(n) => (n.as_f64().unwrap_or(f64::NAN).to_string(), 4),
            other => (other.as_str().unwrap_or_default().to_owned(), 4),
        },
    }
}

/// `keyword value` fields as an object keyed by keyword text. A field with
/// options (a deadline) keeps them under `"{keyword}:opts"`.
pub fn fields(list: impl Iterator<Item = ast::Field>) -> Value {
    let mut map = Map::new();
    for field in list {
        let key = field
            .keyword()
            .map(|k| k.text().to_owned())
            .unwrap_or_default();
        let nodes: Vec<SyntaxNode> = field.syntax().children().collect();
        if let Some(first) = nodes.first() {
            let v = if key == "payoff_transform" {
                json!(engine(first).0)
            } else {
                value(first)
            };
            map.insert(key.clone(), v);
            let unit = first
                .first_token()
                .filter(|t| key == "deadline" && t.kind() == SyntaxKind::Duration);
            if let Some((_, unit)) = unit.as_ref().and_then(|t| t.text().rsplit_once('.')) {
                map.insert(format!("{key}:unit"), json!(unit));
            }
        }
        if let Some(opts) = nodes.get(1) {
            map.insert(format!("{key}:opts"), value(opts));
        }
    }
    Value::Object(map)
}

fn take(map: &mut Map<String, Value>, from: &str, to: &str, out: &mut Map<String, Value>) {
    if let Some(v) = map.remove(from) {
        out.insert(to.to_owned(), v);
    }
}

/// Edge fields (`EdgeFields`) to `RawEdge` JSON keys.
fn edge_fields(mut f: Map<String, Value>, out: &mut Map<String, Value>) {
    take(&mut f, "authority", "authority", out);
    take(&mut f, "effort", "hours", out);
    take(&mut f, "cost", "cost", out);
    if let Some(length) = f.remove("deadline") {
        let mut deadline = Map::new();
        deadline.insert("length".into(), length);
        let unit = match f.remove("deadline:unit") {
            Some(Value::String(u)) if u == "court_days" => Some("court"),
            Some(Value::String(u)) if u == "weeks" => Some("week"),
            Some(Value::String(u)) if u == "months" => Some("month"),
            Some(Value::String(u)) if u == "years" => Some("year"),
            _ => None,
        };
        if let Some(unit) = unit {
            deadline.insert("unit".into(), json!(unit));
        }
        if let Some(Value::Object(mut opts)) = f.remove("deadline:opts") {
            take(&mut opts, "extendable", "extendable", &mut deadline);
            take(
                &mut opts,
                "extension_authority",
                "extensionAuthority",
                &mut deadline,
            );
            take(&mut opts, "note", "note", &mut deadline);
            take(&mut opts, "counting", "counting", &mut deadline);
        }
        out.insert("deadline".into(), Value::Object(deadline));
    }
    match f.remove("duration") {
        Some(Value::Object(range)) => {
            out.insert("duration".into(), Value::Object(range));
        }
        Some(mode) => {
            out.insert("duration".into(), json!({ "mode": mode }));
        }
        None => {}
    }
    take(&mut f, "court_listener_url", "courtListenerUrl", out);
    take(&mut f, "action_id", "actionId", out);
    for key in [
        "valence", "note", "tags", "attrs", "sets", "clears", "requires", "forbids", "replaces",
    ] {
        take(&mut f, key, key, out);
    }
}

/// The head of an edge/link (`by`, `p`) and its `EdgeFields` block, as a
/// `RawEdge` object (without `from`).
fn edge_object(node: &SyntaxNode, default_actor: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let id = node
        .children_with_tokens()
        .find_map(|e| e.into_token().filter(|t| t.kind() == SyntaxKind::Ident));
    if let Some(id) = id {
        out.insert("id".into(), json!(id.text()));
    }
    let label = node
        .children_with_tokens()
        .find_map(|e| e.into_token().filter(|t| t.kind() == SyntaxKind::String));
    out.insert(
        "label".into(),
        json!(label.map(|t| ast::unescape(t.text())).unwrap_or_default()),
    );
    let head = fields(node.children().filter_map(ast::Field::cast));
    let mut head = head.as_object().cloned().unwrap_or_default();
    out.insert(
        "actor".into(),
        head.remove("by").unwrap_or_else(|| json!(default_actor)),
    );
    take(&mut head, "p", "probability", &mut out);
    if let Some(list) = node.children().find(|n| n.kind() == SyntaxKind::FieldList) {
        let body = fields(list.children().filter_map(ast::Field::cast));
        edge_fields(body.as_object().cloned().unwrap_or_default(), &mut out);
    }
    out
}

/// One node declaration: the `RawNode` and its edges in source order.
pub fn node(decl: &ast::NodeDecl) -> (Value, Vec<Value>) {
    let syntax = decl.syntax();
    let id = decl.name().map(|t| t.text().to_owned()).unwrap_or_default();
    let kind = match decl.keyword().map(|k| k.kind()) {
        Some(SyntaxKind::KwDecision) => "decision",
        Some(SyntaxKind::KwTerminal) => "terminal",
        _ => "state",
    };
    let mut out = Map::new();
    out.insert("id".into(), json!(id));
    out.insert("kind".into(), json!(kind));
    out.insert(
        "label".into(),
        json!(decl
            .label()
            .map(|t| ast::unescape(t.text()))
            .unwrap_or_default()),
    );
    let mut head = fields(decl.clause_fields().into_iter())
        .as_object()
        .cloned()
        .unwrap_or_default();
    let actor = head
        .remove("by")
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "either".into());
    take(&mut head, "outcome", "outcome", &mut out);
    if let Some(cite) = syntax.children().find(|n| n.kind() == SyntaxKind::CiteExpr) {
        out.insert("cite".into(), value(&cite));
    }
    let mut edges = Vec::new();
    if let Some(body) = decl.body() {
        let mut f = fields(body.syntax().children().filter_map(ast::Field::cast))
            .as_object()
            .cloned()
            .unwrap_or_default();
        for key in ["group", "note", "valence", "tags", "attrs", "payoff"] {
            take(&mut f, key, key, &mut out);
        }
        take(&mut f, "court_listener_url", "courtListenerUrl", &mut out);
        take(&mut f, "payoff_by_flag", "payoffByFlag", &mut out);
        for edge in body
            .syntax()
            .children()
            .filter(|n| n.kind() == SyntaxKind::EdgeDecl)
        {
            let mut e = edge_object(&edge, &actor);
            e.insert("from".into(), json!(id));
            let to = edge
                .children()
                .find(|n| n.kind() == SyntaxKind::Ref)
                .map(|r| text_of(&r));
            e.insert("to".into(), json!(to.unwrap_or_default()));
            edges.push(Value::Object(e));
        }
    }
    (Value::Object(out), edges)
}

/// Parses `text`, asserting it is clean.
pub fn parsed(text: &str, kind: FileKind, path: &str) -> litigate_syntax::Parse {
    let parse = parse(text, kind);
    assert!(parse.is_ok(), "{path}: {:#?}", parse.errors());
    parse
}

/// A links module: the `LinkFile` JSON.
pub fn links(text: &str) -> Value {
    let parse = parsed(text, FileKind::Lit, "links");
    let mut instances = Map::new();
    let mut links = Vec::new();
    for child in parse.syntax().children() {
        match child.kind() {
            SyntaxKind::InstanceDecl => {
                let idents: Vec<String> = child
                    .children_with_tokens()
                    .filter_map(|e| e.into_token().filter(|t| t.kind() == SyntaxKind::Ident))
                    .map(|t| t.text().to_owned())
                    .collect();
                let name = idents.join("@");
                let list = child.children().find(|n| n.kind() == SyntaxKind::FieldList);
                let mut f = list
                    .map(|l| fields(l.children().filter_map(ast::Field::cast)))
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                let mut instance = Map::new();
                for key in [
                    "pack",
                    "note",
                    "remove_edges",
                    "probabilities",
                    "roles",
                    "payoff_transform",
                ] {
                    take(&mut f, key, key, &mut instance);
                }
                let mut patches = Map::new();
                for patch in child.descendants().filter_map(ast::PatchDecl::cast) {
                    let edge = patch
                        .edge()
                        .map(|t| t.text().to_owned())
                        .unwrap_or_default();
                    let written = fields(patch.fields().into_iter());
                    let mut record = Map::new();
                    edge_fields(
                        written.as_object().cloned().unwrap_or_default(),
                        &mut record,
                    );
                    patches.insert(edge, Value::Object(record));
                }
                if !patches.is_empty() {
                    instance.insert("patch_edges".into(), Value::Object(patches));
                }
                instances.insert(name, Value::Object(instance));
            }
            SyntaxKind::LinkDecl => {
                let mut e = edge_object(&child, "either");
                let refs: Vec<String> = child
                    .children()
                    .filter(|n| n.kind() == SyntaxKind::Ref)
                    .map(|r| text_of(&r))
                    .collect();
                e.insert(
                    "from".into(),
                    json!(refs.first().cloned().unwrap_or_default()),
                );
                e.insert("to".into(), json!(refs.get(1).cloned().unwrap_or_default()));
                links.push(Value::Object(e));
            }
            _ => {}
        }
    }
    json!({ "instances": instances, "links": links })
}
