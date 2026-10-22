//! Module signatures: every name a module declares, with the kind and
//! resolved type information dependents need. Signatures carry no source
//! ranges, so an edit that keeps a module's interface intact is backdated
//! and its importers are not re-checked.
use crate::db::inputs::SourceFile;
use crate::db::parse::tree::{Atom, ExprKind, Field, Item, ItemKind, ItemTree};
use crate::db::queries::{manifest, module_graph, parse_file, signature};
use crate::hir::{Manifest, Type};
use crate::resolve::scope::Scope;
use crate::types::{expr::Checker, ty};
use std::collections::{BTreeMap, BTreeSet};

/// What a value-namespace name denotes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    /// Graph node.
    Node {
        /// `chance` node.
        chance: bool,
        /// `terminal` node.
        terminal: bool,
        /// Tagged `fact`.
        fact: bool,
        /// Named outgoing edges.
        edges: Vec<String>,
    },
    /// Deadline.
    Deadline,
    /// Counting rule.
    Counting,
    /// Holiday calendar.
    Calendar,
    /// Default with its conclusion type (unknown when ill-typed).
    Default(Option<Type>),
    /// Exception.
    Exception,
    /// Transition.
    Transition,
    /// Rule wrapper.
    Rule,
    /// Pack instance `base@origin` and its base pack id.
    Instance(Option<String>),
}

impl ValueKind {
    /// Noun for diagnostics.
    #[must_use]
    pub const fn noun(&self) -> &'static str {
        match self {
            Self::Node { .. } => "node",
            Self::Deadline => "deadline",
            Self::Counting => "counting rule",
            Self::Calendar => "calendar",
            Self::Default(_) => "default",
            Self::Exception => "exception",
            Self::Transition => "transition",
            Self::Rule => "rule",
            Self::Instance(_) => "instance",
        }
    }
}

/// A value name and the `from` dates of its versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueSig {
    /// Kind.
    pub kind: ValueKind,
    /// In-force start of each version, in source order.
    pub versions: Vec<String>,
}

/// A type-namespace name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSig {
    /// Closed enum and its variants.
    Enum(Vec<String>),
    /// Transparent alias.
    Alias(Option<Type>),
    /// Event constructor parameters.
    Event(Vec<(String, Option<Type>)>),
}

/// Everything a module exports.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
    /// Module name.
    pub module: String,
    /// Owning package name.
    pub package: String,
    /// Owning package's pack id.
    pub pack_id: String,
    /// Values.
    pub values: BTreeMap<String, ValueSig>,
    /// Types.
    pub types: BTreeMap<String, TypeSig>,
    /// Groups.
    pub groups: BTreeSet<String>,
    /// Authority levels.
    pub authorities: BTreeSet<String>,
}

/// Items with `rule` bodies hoisted, each paired with its enclosing rule.
#[must_use]
pub fn flatten(tree: &ItemTree) -> Vec<(&Item, Option<&Item>)> {
    tree.items
        .iter()
        .flat_map(|item| {
            let nested: Vec<(&Item, Option<&Item>)> = match &item.kind {
                ItemKind::Rule(_, _, _, items) => items.iter().map(|i| (i, Some(item))).collect(),
                _ => Vec::new(),
            };
            std::iter::once((item, None)).chain(nested)
        })
        .collect()
}

/// The clause `key` of `fields`, if present.
#[must_use]
pub fn clause<'a>(fields: &'a [Field], key: &str) -> Option<&'a Field> {
    fields.iter().find(|f| f.key.text == key)
}

/// Header clauses of an item (version/relation/cite clauses).
#[must_use]
pub fn clauses_of(item: &Item) -> &[Field] {
    match &item.kind {
        ItemKind::Node(n) => &n.clauses,
        ItemKind::Rule(_, _, c, _)
        | ItemKind::Counting(_, c, _)
        | ItemKind::Calendar(_, c, _)
        | ItemKind::Transition(_, _, c, _)
        | ItemKind::Default(_, c, _, _)
        | ItemKind::Test(_, c, _)
        | ItemKind::Deadline { clauses: c, .. }
        | ItemKind::Exception { clauses: c, .. } => c,
        _ => &[],
    }
}

/// In-force start: own `effective`, else the rule's, else the package's.
#[must_use]
pub fn effective_from(item: &Item, rule: Option<&Item>, manifest: Option<&Manifest>) -> String {
    let own = |i: &Item| match clause(clauses_of(i), "effective").and_then(|f| f.values.first()) {
        Some((Atom::Interval(from, _), _)) => Some(from.text.clone()),
        _ => None,
    };
    own(item)
        .or_else(|| rule.and_then(own))
        .or_else(|| manifest.map(|m| m.in_force.from.clone()))
        .unwrap_or_default()
}

/// The value declared by `item`, if any.
#[must_use]
pub fn value_of(item: &Item) -> Option<(&str, ValueKind)> {
    Some(match &item.kind {
        ItemKind::Node(n) => {
            let fact = n.fields.iter().any(|f| {
                f.key.text == "tags"
                    && f.values.iter().any(|(a, _)| matches!(a, Atom::Expr(e) if matches!(&e.kind, ExprKind::List(l) if l.iter().any(|x| matches!(&x.kind, ExprKind::Ref(r) if r.is_simple() && r.head.text == "fact")))))
            });
            let kind = ValueKind::Node {
                chance: n.keyword.text == "chance",
                terminal: n.keyword.text == "terminal",
                fact,
                edges: n
                    .edges
                    .iter()
                    .filter_map(|e| e.name.as_ref().map(|n| n.text.clone()))
                    .collect(),
            };
            (n.name.text.as_str(), kind)
        }
        ItemKind::Deadline { name, .. } => (name.text.as_str(), ValueKind::Deadline),
        ItemKind::Counting(name, ..) => (name.text.as_str(), ValueKind::Counting),
        ItemKind::Calendar(name, ..) => (name.text.as_str(), ValueKind::Calendar),
        ItemKind::Default(name, ..) => (name.text.as_str(), ValueKind::Default(None)),
        ItemKind::Exception { name, .. } => (name.text.as_str(), ValueKind::Exception),
        ItemKind::Transition(name, ..) => (name.text.as_str(), ValueKind::Transition),
        ItemKind::Rule(_, name, ..) => (name.text.as_str(), ValueKind::Rule),
        _ => return None,
    })
}

/// Builds the signature of `file`.
pub fn build(db: &dyn salsa::Database, file: SourceFile) -> Signature {
    let tree = &parse_file(db, file).tree;
    let parsed = manifest(db, file.package(db)).as_ref().ok();
    let m = parsed.map(|p| &p.manifest);
    let mut sig = Signature {
        module: crate::db::parse::tree::dotted(&tree.module),
        package: m.map(|m| m.name.clone()).unwrap_or_default(),
        pack_id: m.map(|m| m.pack_id.clone()).unwrap_or_default(),
        ..Signature::default()
    };
    let items = flatten(tree);
    for (item, rule) in &items {
        if let Some((name, kind)) = value_of(item) {
            let from = effective_from(item, *rule, m);
            match sig.values.get_mut(name) {
                Some(existing)
                    if std::mem::discriminant(&existing.kind) == std::mem::discriminant(&kind) =>
                {
                    existing.versions.push(from)
                }
                Some(_) => {}
                None => {
                    sig.values.insert(
                        name.to_owned(),
                        ValueSig {
                            kind,
                            versions: vec![from],
                        },
                    );
                }
            }
        }
        match &item.kind {
            ItemKind::Group(name, _) => {
                sig.groups.insert(name.text.clone());
            }
            ItemKind::Authority(chain) => {
                sig.authorities.extend(chain.iter().map(|n| n.text.clone()))
            }
            ItemKind::Enum(name, variants) => {
                sig.types.entry(name.text.clone()).or_insert_with(|| {
                    TypeSig::Enum(variants.iter().map(|v| v.text.clone()).collect())
                });
            }
            ItemKind::Instance(base, origin, fields) => {
                let pack = clause(fields, "pack").and_then(|f| match f.values.first() {
                    Some((Atom::Expr(e), _)) => match &e.kind {
                        ExprKind::Ref(r) => Some(r.display()),
                        _ => None,
                    },
                    _ => None,
                });
                sig.values
                    .entry(format!("{}@{}", base.text, origin.text))
                    .or_insert(ValueSig {
                        kind: ValueKind::Instance(pack),
                        versions: Vec::new(),
                    });
            }
            _ => {}
        }
    }
    let imports: Vec<&Signature> = module_graph(db, file.workspace(db))
        .imports(file)
        .into_iter()
        .map(|f| signature(db, f))
        .collect();
    resolve_types(&mut sig, tree, &imports);
    let default_types: Vec<(String, Option<Type>)> = {
        let scope = Scope::new(&sig, tree, &imports, Vec::new());
        let mut checker = Checker::new(&scope);
        items
            .iter()
            .filter_map(|(item, _)| match &item.kind {
                ItemKind::Default(name, _, _, then) => {
                    Some((name.text.clone(), checker.infer(then).ty))
                }
                _ => None,
            })
            .collect()
    };
    for (name, ty) in default_types {
        if let Some(v) = sig.values.get_mut(&name) {
            if matches!(v.kind, ValueKind::Default(None)) {
                v.kind = ValueKind::Default(ty);
            }
        }
    }
    sig
}

/// Resolves alias targets and event parameter types (aliases in
/// declaration order; an alias seeing a later or cyclic alias gets `None`).
fn resolve_types(sig: &mut Signature, tree: &ItemTree, imports: &[&Signature]) {
    for (item, _) in flatten(tree) {
        let resolved = match &item.kind {
            ItemKind::TypeAlias(name, target) => {
                let scope = Scope::new(sig, tree, imports, Vec::new());
                let ty = ty::resolve(&scope, target, &mut Vec::new());
                Some((name.text.clone(), TypeSig::Alias(ty)))
            }
            ItemKind::Event(name, params) => {
                let scope = Scope::new(sig, tree, imports, Vec::new());
                let params = params
                    .iter()
                    .map(|(p, t)| (p.text.clone(), ty::resolve(&scope, t, &mut Vec::new())))
                    .collect();
                Some((name.text.clone(), TypeSig::Event(params)))
            }
            _ => None,
        };
        if let Some((name, ts)) = resolved {
            sig.types.entry(name).or_insert(ts);
        }
    }
}
