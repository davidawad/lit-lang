//! HIR construction: name resolution and type checking of every item of a
//! module, then package-level assembly (SPEC §4-§6, §11).
mod calendar;
mod clock;
mod cx;
mod edge;
mod logic;
mod node;
mod package;

pub use package::{check_package, expected_lock, PackageCheck};

use crate::db::inputs::SourceFile;
use crate::db::parse::tree::{dotted, Item, ItemKind, ItemTree, Name};
use crate::db::queries::{item_tree, manifest, module_graph, module_header, signature};
use crate::db::span::{sort_diagnostics, Report, TextRange};
use crate::hir::{self, Declaration, Rule, RuleKind};
use crate::resolve::graph::ImportTarget;
use crate::resolve::scope::Scope;
use crate::resolve::sig::{clause, clauses_of, flatten, value_of, Signature, ValueKind};
use crate::types::expr::Checker;
use cx::Cx;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, FileId, LineIndex, Span};
use serde::Serialize;
use std::collections::BTreeMap;

/// An explicit hole and its reason (SPEC §7 `holes`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hole {
    /// Location.
    pub span: Span,
    /// `?("reason")` text.
    pub reason: Option<String>,
}

/// Result of checking one module.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleCheck {
    /// HIR, present only when the module has no errors.
    pub module: Option<hir::Module>,
    /// `group` declarations in source order.
    pub groups: Vec<hir::Group>,
    /// `start` declarations: resolved node and location.
    pub starts: Vec<(Option<String>, Span)>,
    /// First node declaration, if any (links packages may have none).
    pub first_node: Option<Span>,
    /// First link or instance, if any.
    pub first_link: Option<Span>,
    /// Diagnostics, sorted.
    pub diagnostics: Vec<Diagnostic>,
    /// Holes.
    pub holes: Vec<Hole>,
}

/// The declared name of an item, if it has one.
#[must_use]
pub fn item_name(item: &Item) -> Option<&Name> {
    match &item.kind {
        ItemKind::Group(n, _)
        | ItemKind::TypeAlias(n, _)
        | ItemKind::Enum(n, _)
        | ItemKind::Event(n, _)
        | ItemKind::Rule(_, n, _, _)
        | ItemKind::Counting(n, _, _)
        | ItemKind::Calendar(n, _, _)
        | ItemKind::Transition(n, _, _, _)
        | ItemKind::Default(n, _, _, _) => Some(n),
        ItemKind::Node(n) => Some(&n.name),
        ItemKind::Deadline { name, .. } | ItemKind::Exception { name, .. } => Some(name),
        _ => None,
    }
}

fn has_effective(item: &Item, rule: Option<&Item>) -> bool {
    clause(clauses_of(item), "effective").is_some()
        || rule.is_some_and(|r| clause(clauses_of(r), "effective").is_some())
}

/// First sighting of a name: value kind, versioned, and where.
type Seen = (Option<std::mem::Discriminant<ValueKind>>, bool, TextRange);

/// Duplicate declarations within one module (L1001); same-kind values that
/// all carry `effective` clauses are versions, not duplicates.
fn duplicates(tree: &ItemTree, reports: &mut Vec<Report>) {
    let mut seen: BTreeMap<(u8, String), Seen> = BTreeMap::new();
    for (item, rule) in flatten(tree) {
        let (ns, kind) = match &item.kind {
            ItemKind::TypeAlias(..) | ItemKind::Enum(..) | ItemKind::Event(..) => (1, None),
            ItemKind::Group(..) => (2, None),
            ItemKind::Test(name, ..) => {
                let key = (3, name.text.clone());
                if let Some((_, _, first)) = seen.get(&key) {
                    reports.push(
                        Report::new(
                            C::DuplicateName,
                            name.range,
                            format!("test \"{}\" is declared twice", name.text),
                        )
                        .note_at(*first, "first declared here"),
                    );
                } else {
                    seen.insert(key, (None, false, name.range));
                }
                continue;
            }
            _ => match value_of(item) {
                Some((_, kind)) => (0, Some(std::mem::discriminant(&kind))),
                None => continue,
            },
        };
        let Some(name) = item_name(item) else {
            continue;
        };
        let versioned = has_effective(item, rule);
        let key = (ns, name.text.clone());
        match seen.get(&key) {
            Some((k, v, _)) if ns == 0 && *k == kind && *v && versioned => {}
            Some((_, _, first)) => reports.push(
                Report::new(
                    C::DuplicateName,
                    name.range,
                    format!("`{}` is already declared in this module", name.text),
                )
                .note_at(*first, "first declared here")
                .note("two versions of one declaration each need an `effective` clause"),
            ),
            None => {
                seen.insert(key, (kind, versioned, name.range));
            }
        }
    }
}

/// Import-level diagnostics for `file`.
fn imports(
    db: &dyn salsa::Database,
    file: SourceFile,
    tree: &ItemTree,
    own: &Signature,
    reports: &mut Vec<Report>,
) -> Vec<String> {
    let graph = module_graph(db, file.workspace(db));
    let Some(node) = graph.node(file) else {
        return Vec::new();
    };
    if node.duplicate {
        if let (Some(first), Some(last)) = (tree.module.first(), tree.module.last()) {
            reports.push(Report::new(
                C::DuplicateName,
                first.range.cover(last.range),
                format!(
                    "module `{}` is declared by another file of this package",
                    node.module
                ),
            ));
        }
    }
    let mut resolved = Vec::new();
    let mut named: BTreeMap<String, String> = BTreeMap::new();
    for (import, target) in tree.imports.iter().zip(&node.imports) {
        let path = dotted(&import.path);
        let range = import.path.first().map_or(import.range, |f| {
            f.range
                .cover(import.path.last().map_or(f.range, |l| l.range))
        });
        match target {
            ImportTarget::Module(f) => {
                resolved.push(module_header(db, *f).module.clone());
                let sig = signature(db, *f);
                for n in import.names.iter().flatten() {
                    let exists = sig.values.contains_key(&n.text) || sig.types.contains_key(&n.text) || sig.groups.contains(&n.text) || sig.authorities.contains(&n.text);
                    if !exists {
                        let keys = sig.values.keys().chain(sig.types.keys()).chain(sig.groups.iter()).map(String::as_str);
                        let mut report = Report::new(C::UnresolvedName, n.range, format!("module `{path}` declares no `{}`", n.text));
                        if let Some(best) = crate::resolve::suggest::nearest(&n.text, keys) {
                            report = report.fix(n.range, best, Applicability::MaybeIncorrect);
                        }
                        reports.push(report);
                    } else if own.values.contains_key(&n.text) || own.types.contains_key(&n.text) || own.groups.contains(&n.text) {
                        reports.push(Report::new(C::DuplicateName, n.range, format!("`{}` is imported from `{path}` but also declared in this module", n.text)).note("shadowing is not allowed"));
                    } else if let Some(other) = named.insert(n.text.clone(), path.clone()).filter(|o| *o != path) {
                        reports.push(Report::new(C::DuplicateName, n.range, format!("`{}` is already imported from `{other}`", n.text)));
                    }
                }
            }
            ImportTarget::Unknown(best) => {
                let mut report = Report::new(C::UnresolvedName, range, format!("cannot find module `{path}`"));
                if let Some(best) = best {
                    report = report.fix(range, best.clone(), Applicability::MaybeIncorrect);
                }
                reports.push(report);
            }
            ImportTarget::Undeclared(pkg) => reports.push(
                Report::new(C::UndeclaredDependency, range, format!("module `{path}` belongs to package `{pkg}`, which is not a dependency"))
                    .note(format!("add `{pkg} = {{ version = \"^x.y.z\", path = \"...\" }}` to `[dependencies]` in Lit.toml")),
            ),
            ImportTarget::Cycle(cycle) => reports.push(Report::new(C::ImportCycle, range, format!("importing `{path}` creates a cycle")).note(format!("cycle: {}", cycle.join(" -> ")))),
        }
    }
    resolved
}

/// A `.lit` `package` header must agree with `Lit.toml` (L1005).
fn package_header(
    header: &crate::db::parse::tree::PackageHeader,
    m: &hir::Manifest,
    reports: &mut Vec<Report>,
) {
    let mut expect = vec![("name", &header.name, m.name.as_str())];
    for f in &header.fields {
        let want = if f.key.text == "forum" {
            m.forum.as_str()
        } else {
            m.authority.as_str()
        };
        if let Some(crate::db::parse::tree::ExprKind::Ref(r)) = cx::first_expr(f).map(|e| &e.kind) {
            expect.push((
                if f.key.text == "forum" {
                    "forum"
                } else {
                    "authority"
                },
                &r.head,
                want,
            ));
        }
    }
    for (what, got, want) in expect {
        if got.text != want {
            reports.push(
                Report::new(
                    C::PackageMismatch,
                    got.range,
                    format!(
                        "package {what} `{}` disagrees with Lit.toml (`{want}`)",
                        got.text
                    ),
                )
                .fix(got.range, want, Applicability::MachineApplicable),
            );
        }
    }
}

/// Signatures reachable through `::`: every module of this package and of
/// its declared dependencies.
fn pack_signatures(db: &dyn salsa::Database, file: SourceFile) -> Vec<&Signature> {
    let pkg = file.package(db);
    let deps: Vec<String> = manifest(db, pkg)
        .as_ref()
        .map(|m| m.manifest.dependencies.keys().cloned().collect())
        .unwrap_or_default();
    file.workspace(db)
        .packages(db)
        .iter()
        .filter(|p| {
            **p == pkg
                || manifest(db, **p)
                    .as_ref()
                    .is_ok_and(|m| deps.contains(&m.manifest.name))
        })
        .flat_map(|p| p.files(db).iter().map(|f| signature(db, *f)))
        .collect()
}

/// Checks one module and builds its HIR.
pub fn check_module(db: &dyn salsa::Database, file: SourceFile) -> ModuleCheck {
    let parsed = item_tree(db, file);
    let tree = &parsed.tree;
    let file_id = FileId(file.path(db));
    let index = LineIndex::new(file.text(db));
    let mut reports = parsed.reports.clone();
    let m = manifest(db, file.package(db))
        .as_ref()
        .ok()
        .map(|p| &p.manifest);
    let own = signature(db, file);
    let resolved_imports = imports(db, file, tree, own, &mut reports);
    duplicates(tree, &mut reports);
    if let (Some(header), Some(m)) = (&tree.package, m) {
        package_header(header, m, &mut reports);
    }
    let import_sigs: Vec<&Signature> = module_graph(db, file.workspace(db))
        .imports(file)
        .into_iter()
        .map(|f| signature(db, f))
        .collect();
    let links = tree
        .items
        .iter()
        .any(|i| matches!(i.kind, ItemKind::Link(_) | ItemKind::Instance(..)));
    let scope = Scope::new(
        own,
        tree,
        &import_sigs,
        if links {
            pack_signatures(db, file)
        } else {
            Vec::new()
        },
    );
    let mut ck = Checker::new(&scope);
    ck.actors = m
        .map(|m| m.roles.keys().cloned().collect())
        .unwrap_or_default();
    ck.file = Some((&file_id, &index));
    let mut cx = Cx {
        ck,
        manifest: m,
        links,
        link_from: None,
    };
    let out = items(&mut cx, tree, &index, &file_id);
    reports.append(&mut cx.ck.reports);
    let holes = cx
        .ck
        .holes
        .iter()
        .map(|(r, reason)| Hole {
            span: crate::db::span::to_span(&file_id, &index, *r),
            reason: reason.clone(),
        })
        .collect();
    let has_error = reports.iter().any(|r| !r.code.is_lint());
    let mut diagnostics: Vec<Diagnostic> = reports
        .iter()
        .map(|r| r.to_diagnostic(&file_id, &index))
        .collect();
    sort_diagnostics(&mut diagnostics);
    let module = (!has_error).then(|| hir::Module {
        name: dotted(&tree.module),
        package: m.map(|m| m.name.clone()).unwrap_or_default(),
        imports: resolved_imports,
        declarations: out.declarations,
    });
    ModuleCheck {
        module,
        groups: out.groups,
        starts: out.starts,
        first_node: out.first_node,
        first_link: out.first_link,
        diagnostics,
        holes,
    }
}

#[derive(Default)]
struct Items {
    declarations: Vec<Declaration>,
    groups: Vec<hir::Group>,
    starts: Vec<(Option<String>, Span)>,
    first_node: Option<Span>,
    first_link: Option<Span>,
}

#[allow(clippy::too_many_lines)] // one arm per declaration kind
fn items(cx: &mut Cx<'_, '_>, tree: &ItemTree, index: &LineIndex, file: &FileId) -> Items {
    let span = |r: TextRange| crate::db::span::to_span(file, index, r);
    let mut out = Items::default();
    let flat = flatten(tree);
    let aliases: BTreeMap<String, &crate::db::parse::tree::TypeRef> = flat
        .iter()
        .filter_map(|(i, _)| match &i.kind {
            ItemKind::TypeAlias(n, t) => Some((n.text.clone(), t)),
            _ => None,
        })
        .collect();
    let allow_hours = |name: &str| cx_allow_hours(tree, name);
    for (item, rule) in &flat {
        let d = match &item.kind {
            ItemKind::Start(r) => {
                let node = cx.value_ref(r, "node", |k| matches!(k, ValueKind::Node { .. }));
                out.starts.push((node, span(item.range)));
                None
            }
            ItemKind::Group(n, label) => {
                out.groups.push(hir::Group {
                    id: n.text.clone(),
                    label: label.text.clone(),
                });
                None
            }
            ItemKind::TypeAlias(n, t) => cx.alias(n, t, &aliases).map(Declaration::TypeAlias),
            ItemKind::Enum(n, v) => Some(Declaration::Enum(cx.enumeration(n, v))),
            ItemKind::Event(n, p) => cx.event(n, p).map(Declaration::Event),
            ItemKind::Node(n) => {
                out.first_node.get_or_insert_with(|| span(item.range));
                Some(Declaration::Node(cx.node(n, item, *rule)))
            }
            ItemKind::Link(e) => {
                out.first_link.get_or_insert_with(|| span(item.range));
                cx.link(e).map(|l| Declaration::Link(Box::new(l)))
            }
            ItemKind::Instance(b, o, f) => {
                out.first_link.get_or_insert_with(|| span(item.range));
                cx.instance(b, o, f).map(Declaration::Instance)
            }
            ItemKind::Rule(kind, name, _, nested) => {
                let rule_kind = match kind.text.as_str() {
                    "power" => RuleKind::Power,
                    "liability" => RuleKind::Liability,
                    _ => RuleKind::Duty,
                };
                let meta = cx.meta(name, None, item, None);
                let declarations = nested
                    .iter()
                    .filter_map(|i| item_name(i).map(|n| n.text.clone()))
                    .collect();
                Some(Declaration::Rule(Rule {
                    meta,
                    rule_kind,
                    declarations,
                }))
            }
            ItemKind::Deadline {
                name,
                amount,
                direction,
                trigger,
                counted_by,
                opts,
                ..
            } => cx
                .deadline(
                    item,
                    *rule,
                    name,
                    amount,
                    direction,
                    trigger,
                    counted_by,
                    opts,
                    allow_hours,
                )
                .map(Declaration::Deadline),
            ItemKind::Counting(n, _, fields) => cx
                .counting(item, *rule, n, fields)
                .map(Declaration::Counting),
            ItemKind::Calendar(n, _, h) => {
                cx.calendar(item, *rule, n, h).map(Declaration::Calendar)
            }
            ItemKind::Transition(n, amends, _, arms) => cx
                .transition(item, *rule, n, amends, arms)
                .map(Declaration::Transition),
            ItemKind::Authority(chain) => {
                out.declarations.extend(
                    Cx::authorities(chain)
                        .into_iter()
                        .map(Declaration::Authority),
                );
                None
            }
            ItemKind::Default(n, _, when, then) => Some(Declaration::Default(
                cx.default_rule(item, *rule, n, when, then),
            )),
            ItemKind::Exception {
                name,
                kind,
                target,
                when,
                then,
                priority,
                ..
            } => cx
                .exception(
                    item,
                    *rule,
                    name,
                    kind,
                    target,
                    when,
                    then.as_ref(),
                    priority.as_ref(),
                )
                .map(Declaration::Exception),
            ItemKind::Test(name, clauses, stmts) => {
                Some(Declaration::Test(cx.test(item, name, clauses, stmts)))
            }
        };
        out.declarations.extend(d);
    }
    out
}

/// Whether counting rule `name` (declared in this module) has `allow_hours`.
fn cx_allow_hours(tree: &ItemTree, name: &str) -> bool {
    flatten(tree).iter().any(|(i, _)| matches!(&i.kind, ItemKind::Counting(n, _, f) if n.text == name && f.iter().any(|x| x.key.text == "allow_hours")))
}
