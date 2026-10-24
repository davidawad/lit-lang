//! Module signatures: every name a module declares, with the kind and
//! resolved type information dependents need. Signatures carry no source
//! ranges, so an edit that keeps a module's interface intact is backdated
//! and its importers are not re-checked.
use crate::db::inputs::SourceFile;
use crate::db::parse::tree::{Atom, Expr, ExprKind, Field, Item, ItemKind, ItemTree, RefPath};
use crate::db::queries::{item_tree, manifest, module_graph, signature};
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
    /// Counting rule, with the forums whose periods its versions compute
    /// (`forums [...]`, else the declaring package's forum).
    Counting(std::collections::BTreeSet<String>),
    /// Holiday calendar; `true` when it lists only declared days: every
    /// holiday a `once(date)` with no observance.
    Calendar(bool),
    /// Default with its conclusion type (unknown when ill-typed).
    Default(Option<Type>),
    /// Exception, with the conclusion type of the default its chain of
    /// targets ends at (unknown when that default is ill-typed or the chain
    /// does not resolve).
    Exception(Option<Type>),
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
            Self::Counting(_) => "counting rule",
            Self::Calendar(_) => "calendar",
            Self::Default(_) => "default",
            Self::Exception(_) => "exception",
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
    /// Actors declared in the package's `[package.roles]`.
    pub actors: BTreeSet<String>,
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
                    && f.values.iter().any(|(a, _)| {
                        matches!(a, Atom::Expr(Expr { kind: ExprKind::List(l), .. })
                            if l.iter().any(|x| matches!(&x.kind, ExprKind::Ref(r) if r.is_simple() && r.head.text == "fact")))
                    })
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
        ItemKind::Counting(name, _, fields) => (
            name.text.as_str(),
            ValueKind::Counting(
                crate::types::build::refs(clause(fields, "forums").as_ref())
                    .into_iter()
                    .map(RefPath::display)
                    .collect(),
            ),
        ),
        ItemKind::Calendar(name, _, holidays) => (
            name.text.as_str(),
            ValueKind::Calendar(holidays.iter().all(|h| {
                h.observed.is_none()
                    && matches!(&h.rule.kind, ExprKind::Call(f, _) if f.display() == "once")
            })),
        ),
        ItemKind::Default(name, ..) => (name.text.as_str(), ValueKind::Default(None)),
        ItemKind::Exception { name, .. } => (name.text.as_str(), ValueKind::Exception(None)),
        ItemKind::Transition(name, ..) => (name.text.as_str(), ValueKind::Transition),
        ItemKind::Rule(_, name, ..) => (name.text.as_str(), ValueKind::Rule),
        _ => return None,
    })
}

/// Builds the signature of `file`.
pub fn build(db: &dyn salsa::Database, file: SourceFile) -> Signature {
    let tree = &item_tree(db, file).tree;
    let parsed = manifest(db, file.package(db)).as_ref().ok();
    let m = parsed.map(|p| &p.manifest);
    let mut sig = Signature {
        module: crate::db::parse::tree::dotted(&tree.module),
        package: m.map(|m| m.name.clone()).unwrap_or_default(),
        pack_id: m.map(|m| m.pack_id.clone()).unwrap_or_default(),
        actors: m
            .map(|m| m.roles.keys().cloned().collect())
            .unwrap_or_default(),
        ..Signature::default()
    };
    let items = flatten(tree);
    collect(&mut sig, &items, m);
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
        // The first default of a name types it; a same-named declaration of
        // another kind keeps its kind (L1001 reports the duplicate).
        if let Some(ValueSig {
            kind: kind @ ValueKind::Default(None),
            ..
        }) = sig.values.get_mut(&name)
        {
            *kind = ValueKind::Default(ty);
        }
    }
    exception_types(&mut sig, tree, &imports, &items);
    sig
}

/// Types every exception by the default its targets lead to, to a fixpoint
/// so an exception may target one declared after it (a cycle stays unknown;
/// L3006 reports it).
fn exception_types(
    sig: &mut Signature,
    tree: &ItemTree,
    imports: &[&Signature],
    items: &[(&Item, Option<&Item>)],
) {
    loop {
        let found: Vec<(String, Type)> = {
            let scope = Scope::new(sig, tree, imports, Vec::new());
            items
                .iter()
                .filter_map(|(item, _)| match &item.kind {
                    ItemKind::Exception { name, target, .. } => {
                        let (_, f) = scope.value(target).ok()?;
                        let ty = match &f.item.kind {
                            ValueKind::Default(t) | ValueKind::Exception(t) => t.as_ref()?,
                            _ => return None,
                        };
                        Some((name.text.clone(), scope.localize(f.sig, ty)))
                    }
                    _ => None,
                })
                .collect()
        };
        let mut changed = false;
        for (name, ty) in found {
            if let Some(ValueSig {
                kind: kind @ ValueKind::Exception(None),
                ..
            }) = sig.values.get_mut(&name)
            {
                *kind = ValueKind::Exception(Some(ty));
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
}

/// Records every declared name of `items` in `sig`.
fn collect(sig: &mut Signature, items: &[(&Item, Option<&Item>)], m: Option<&Manifest>) {
    for (item, rule) in items {
        if let Some((name, mut kind)) = value_of(item) {
            let from = effective_from(item, *rule, m);
            // A counting rule with no `forums` computes its own package's.
            if let ValueKind::Counting(forums) = &mut kind {
                if forums.is_empty() {
                    forums.insert(m.map(|m| m.forum.clone()).unwrap_or_default());
                }
            }
            match sig.values.get_mut(name) {
                Some(existing)
                    if std::mem::discriminant(&existing.kind) == std::mem::discriminant(&kind) =>
                {
                    existing.versions.push(from);
                    if let (ValueKind::Counting(all), ValueKind::Counting(more)) =
                        (&mut existing.kind, kind)
                    {
                        all.extend(more);
                    }
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
                sig.authorities.extend(chain.iter().map(|n| n.text.clone()));
            }
            ItemKind::Enum(name, variants) => {
                sig.types.entry(name.text.clone()).or_insert_with(|| {
                    TypeSig::Enum(variants.iter().map(|v| v.text.clone()).collect())
                });
            }
            ItemKind::Instance(base, origin, fields) => {
                let pack = clause(fields, "pack").and_then(|f| match f.values.first() {
                    Some((
                        Atom::Expr(Expr {
                            kind: ExprKind::Ref(r),
                            ..
                        }),
                        _,
                    )) => Some(r.display()),
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
}

/// Resolves alias targets and event parameter types. Aliases resolve to a
/// fixpoint, so one may name an alias declared after it; a cyclic alias
/// stays `None` (L3006 reports it). Event parameters resolve once every
/// alias has.
fn resolve_types(sig: &mut Signature, tree: &ItemTree, imports: &[&Signature]) {
    let mut aliases = Vec::new();
    let mut events = Vec::new();
    for (item, _) in flatten(tree) {
        let (name, placeholder) = match &item.kind {
            ItemKind::TypeAlias(name, target) => {
                aliases.push((name.text.clone(), target));
                (name, TypeSig::Alias(None))
            }
            ItemKind::Event(name, params, _) => {
                events.push((name.text.clone(), params));
                (name, TypeSig::Event(Vec::new()))
            }
            _ => continue,
        };
        sig.types.entry(name.text.clone()).or_insert(placeholder);
    }
    loop {
        let resolved: Vec<(String, Type)> = {
            let scope = Scope::new(sig, tree, imports, Vec::new());
            aliases
                .iter()
                .filter(|(name, _)| matches!(sig.types.get(name), Some(TypeSig::Alias(None))))
                .filter_map(|(name, target)| {
                    Some((name.clone(), ty::resolve(&scope, target, &mut Vec::new())?))
                })
                .collect()
        };
        if resolved.is_empty() {
            break;
        }
        for (name, ty) in resolved {
            sig.types.insert(name, TypeSig::Alias(Some(ty)));
        }
    }
    let typed: Vec<(String, TypeSig)> = {
        let scope = Scope::new(sig, tree, imports, Vec::new());
        events
            .iter()
            .filter(
                |(name, _)| matches!(sig.types.get(name), Some(TypeSig::Event(p)) if p.is_empty()),
            )
            .map(|(name, params)| {
                let params = params
                    .iter()
                    .map(|(p, t)| (p.text.clone(), ty::resolve(&scope, t, &mut Vec::new())))
                    .collect();
                (name.clone(), TypeSig::Event(params))
            })
            .collect()
    };
    for (name, ts) in typed {
        sig.types.insert(name, ts);
    }
}
