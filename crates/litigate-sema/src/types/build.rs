//! HIR construction: name resolution and type checking of every item of a
//! module, then package-level assembly (SPEC §4-§6, §11).
mod basis;
mod calendar;
mod clock;
mod cx;
pub(crate) use cx::refs;
mod delegation;
mod edge;
mod imports;
mod instance;
mod kind;
mod logic;
mod node;
mod obligation;
mod package;

pub use package::{check_package, expected_lock, PackageCheck};

use crate::db::ids::DeclId;
use crate::db::inputs::SourceFile;
use crate::db::parse::tree::{dotted, Import, Item, ItemKind, ItemTree, Name, TypeRef};
use crate::db::queries::{
    check_decl, decl_item, item_tree, manifest, module_context_of, signature, span_map,
};
use crate::db::span::{sort_diagnostics, Encoded, Positions, Report, TextRange};
use crate::hir::spans::Spans;
use crate::hir::{self, Declaration, Rule, RuleKind};
use crate::resolve::scope::Scope;
use crate::resolve::sig::{clause, clauses_of, flatten, value_of, ValueKind};
use crate::types::expr::Checker;
use cx::Cx;
use imports::{import_signatures, imports, pack_signatures};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, FileId, Span};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

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
        | ItemKind::Event(n, _, _)
        | ItemKind::Rule(_, n, _, _)
        | ItemKind::Counting(n, _, _)
        | ItemKind::Calendar(n, _, _)
        | ItemKind::Transition(n, _, _, _)
        | ItemKind::Grant(n, _, _)
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
        if let ItemKind::Test(name, ..) = &item.kind {
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
        let Some(name) = item_name(item) else {
            continue;
        };
        // Every other named item is a value (SPEC §4.3 namespaces).
        let (ns, kind) = match &item.kind {
            ItemKind::TypeAlias(..) | ItemKind::Enum(..) | ItemKind::Event(..) => (1, None),
            ItemKind::Group(..) => (2, None),
            _ => (0, value_of(item).map(|(_, k)| std::mem::discriminant(&k))),
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

/// Checks one module and builds its HIR: the module-level checks
/// (imports, duplicates, the package header, information sets) here, and
/// each top-level declaration through its own [`crate::db::queries::check_decl`],
/// whose position-free result is located with the file's [`SpanMap`].
pub fn check_module(db: &dyn salsa::Database, file: SourceFile) -> ModuleCheck {
    let parsed = item_tree(db, file);
    let tree = &parsed.tree;
    let file_id = FileId(file.path(db));
    let map = span_map(db, file);
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
    let span = |r: TextRange| map.span(&file_id, r);
    let mut out = Items::default();
    let mut infos: Vec<InfoMember> = Vec::new();
    let mut hole_ranges: Vec<(TextRange, Option<String>)> = Vec::new();
    for index in 0..tree.items.len() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let decl = check_decl(db, DeclId::new(db, file, index));
        reports.extend(decl.reports.iter().cloned());
        hole_ranges.extend(decl.holes.iter().cloned());
        infos.extend(decl.infos.iter().cloned());
        out.groups.extend(decl.groups.iter().cloned());
        out.starts
            .extend(decl.starts.iter().map(|(n, r)| (n.clone(), span(*r))));
        if out.first_node.is_none() {
            out.first_node = decl.first_node.map(span);
        }
        if out.first_link.is_none() {
            out.first_link = decl.first_link.map(span);
        }
        out.declarations
            .extend(decl.declarations.iter().cloned().map(|mut d| {
                d.spans_mut(&mut |s| map.decode(s));
                d
            }));
    }
    info_sets(&infos, &mut reports);
    let holes = hole_ranges
        .iter()
        .map(|(r, reason)| Hole {
            span: span(*r),
            reason: reason.clone(),
        })
        .collect();
    let has_error = reports.iter().any(|r| !r.code.is_lint());
    let mut diagnostics: Vec<Diagnostic> = reports
        .iter()
        .map(|r| r.to_diagnostic(&file_id, &map))
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

/// What one top-level declaration's module needs to know: the module's
/// imports, type aliases and counting rules, and whether it is a links
/// module. Ranges are anchored, so it is backdated unless one of those
/// changes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleContext {
    /// `import` declarations.
    pub imports: Vec<Import>,
    /// Type aliases by name (the last of a name wins).
    pub aliases: BTreeMap<String, TypeRef>,
    /// Counting rules by name (the first of a name) with their item keys.
    pub countings: BTreeMap<String, BTreeSet<String>>,
    /// The module declares a `link` or an `instance` (`::` qualification).
    pub links: bool,
}

/// The [`ModuleContext`] of `tree`.
#[must_use]
pub fn module_context(tree: &ItemTree) -> ModuleContext {
    let mut cx = ModuleContext {
        imports: tree.imports.clone(),
        ..ModuleContext::default()
    };
    for (item, _) in flatten(tree) {
        match &item.kind {
            ItemKind::TypeAlias(n, t) => {
                cx.aliases.insert(n.text.clone(), t.clone());
            }
            ItemKind::Counting(n, _, fields) => {
                cx.countings
                    .entry(n.text.clone())
                    .or_insert_with(|| fields.iter().map(|f| f.key.text.clone()).collect());
            }
            ItemKind::Link(_) | ItemKind::Instance(..) => cx.links = true,
            _ => {}
        }
    }
    cx
}

/// One top-level declaration checked: its HIR, whose spans are
/// [`crate::db::span::Encoded`], and everything else with anchored ranges,
/// so it is the same wherever the declaration moves in its file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeclCheck {
    /// HIR declarations (a `rule` and the declarations nested in it).
    pub declarations: Vec<Declaration>,
    /// `group` declarations.
    pub groups: Vec<hir::Group>,
    /// `start` declarations: resolved node and location.
    pub starts: Vec<(Option<String>, TextRange)>,
    /// The declaration, if it is a node.
    pub first_node: Option<TextRange>,
    /// The declaration, if it is a link or instance.
    pub first_link: Option<TextRange>,
    /// Reports.
    pub reports: Vec<Report>,
    /// Holes and their reasons.
    pub holes: Vec<(TextRange, Option<String>)>,
    /// Decisions in information sets, checked across the module.
    infos: Vec<InfoMember>,
}

/// Name resolution and type checking of top-level declaration `decl`, its
/// HIR spans position-free.
pub fn check_decl_in<'db>(db: &'db dyn salsa::Database, decl: DeclId<'db>) -> DeclCheck {
    check_decl_at(db, decl, &Encoded)
}

/// [`check_decl_in`] with HIR spans placed by `at` (the file's
/// [`crate::db::span::SpanMap`] places them where they are now: the
/// reference a located [`crate::db::queries::check_decl`] must equal).
pub fn check_decl_at<'db>(
    db: &'db dyn salsa::Database,
    decl: DeclId<'db>,
    at: &dyn Positions,
) -> DeclCheck {
    let file = decl.file(db);
    let Some(item) = decl_item(db, decl) else {
        return DeclCheck::default();
    };
    let file_id = FileId(file.path(db));
    let context = module_context_of(db, file);
    let m = manifest(db, file.package(db))
        .as_ref()
        .ok()
        .map(|p| &p.manifest);
    let own = signature(db, file);
    let (import_sigs, transitive) = import_signatures(db, file);
    let scope = Scope::new(
        own,
        &context.imports,
        &import_sigs,
        if context.links {
            pack_signatures(db, file)
        } else {
            Vec::new()
        },
    )
    .with_transitive(transitive);
    let mut ck = Checker::new(&scope);
    ck.actors = m
        .map(|m| m.roles.keys().cloned().collect())
        .unwrap_or_default();
    ck.file = Some((&file_id, at));
    let mut cx = Cx {
        ck,
        manifest: m,
        links: context.links,
        link_from: None,
    };
    let mut out = items(&mut cx, item, context);
    out.reports = cx.ck.reports;
    out.holes = cx.ck.holes;
    out
}

#[allow(clippy::too_many_lines)] // one arm per declaration kind
fn items(cx: &mut Cx<'_, '_>, top: &Item, context: &ModuleContext) -> DeclCheck {
    let mut out = DeclCheck::default();
    let aliases: BTreeMap<String, &crate::db::parse::tree::TypeRef> = context
        .aliases
        .iter()
        .map(|(n, t)| (n.clone(), t))
        .collect();
    let local_counting =
        |name: &str, key: &str| context.countings.get(name).map(|keys| keys.contains(key));
    let nested: &[Item] = match &top.kind {
        ItemKind::Rule(_, _, _, items) => items,
        _ => &[],
    };
    let flat = std::iter::once((top, None)).chain(nested.iter().map(|i| (i, Some(top))));
    for (item, rule) in flat {
        let d = match &item.kind {
            ItemKind::Start(r) => {
                let node = cx.value_ref(r, "node", |k| matches!(k, ValueKind::Node { .. }));
                out.starts.push((node, item.range));
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
            ItemKind::Event(n, p, effect) => cx.event(n, p, effect).map(Declaration::Event),
            ItemKind::Node(n) => {
                out.first_node.get_or_insert(item.range);
                let node = cx.node(n, item, rule);
                if let Some(set) = &node.info {
                    out.infos.push(InfoMember::of(n.name.range, set, &node));
                }
                Some(Declaration::Node(node))
            }
            ItemKind::Link(e) => {
                out.first_link.get_or_insert(item.range);
                cx.link(e).map(|l| Declaration::Link(Box::new(l)))
            }
            ItemKind::Instance(b, o, f) => {
                out.first_link.get_or_insert(item.range);
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
                    rule,
                    name,
                    amount,
                    direction,
                    trigger,
                    counted_by,
                    opts,
                    local_counting,
                )
                .map(Declaration::Deadline),
            ItemKind::Counting(n, _, fields) => cx
                .counting(item, rule, n, fields)
                .map(Declaration::Counting),
            ItemKind::Calendar(n, _, h) => cx.calendar(item, rule, n, h).map(Declaration::Calendar),
            ItemKind::Transition(n, amends, _, arms) => cx
                .transition(item, rule, n, amends, arms)
                .map(Declaration::Transition),
            ItemKind::Authority(chain) => {
                for a in Cx::authorities(chain) {
                    let a = Declaration::Authority(a);
                    kind::check(cx, item, &a);
                    out.declarations.push(a);
                }
                None
            }
            ItemKind::Default(n, _, when, then) => Some(Declaration::Default(
                cx.default_rule(item, rule, n, when, then),
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
                    rule,
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
            ItemKind::Grant(name, clauses, fields) => cx
                .grant(item, rule, name, clauses, fields)
                .map(Declaration::Grant),
        };
        if let Some(d) = &d {
            kind::check(cx, item, d);
        }
        out.declarations.extend(d);
    }
    out
}

/// A decision in an information set: where, which node and set, and the
/// choices it offers its actor, as sorted `(label, actor)` pairs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct InfoMember {
    at: TextRange,
    node: String,
    set: String,
    choices: Vec<(String, String)>,
}

impl InfoMember {
    fn of(at: TextRange, set: &str, node: &hir::Node) -> Self {
        let mut choices: Vec<(String, String)> = node
            .edges
            .iter()
            .map(|e| (e.label.clone(), e.actor.clone()))
            .collect();
        choices.sort();
        Self {
            at,
            node: node.meta.id.clone(),
            set: set.to_owned(),
            choices,
        }
    }
}

/// The decisions of one information set (SPEC §5.1) are indistinguishable
/// to their actor, so each offers the same choices to the same actor as
/// the first member declared (L2001).
fn info_sets(members: &[InfoMember], reports: &mut Vec<Report>) {
    for (j, m) in members.iter().enumerate() {
        let Some(first) = members[..j].iter().find(|f| f.set == m.set) else {
            continue;
        };
        if first.choices != m.choices {
            reports.push(
                Report::new(
                    C::TypeMismatch,
                    m.at,
                    format!(
                        "decision `{}` is in information set `{}` with `{}` but offers different choices",
                        m.node, m.set, first.node
                    ),
                )
                .note_at(first.at, "the set's first member")
                .note("an actor who cannot tell the decisions apart chooses among the same labelled edges, as the same actor"),
            );
        }
    }
}
