//! The procedure graph a matter replays over: the `uses` packages in order
//! (the first is primary), their versioned nodes and deadlines through the
//! temporal [`World`], link edges, and the event and enum vocabulary.
//!
//! Names follow sema's nominal convention: a name is bare inside the
//! package that declares it and `pack::Name` everywhere else. Case HIR is
//! written in the primary package's view; [`Graph::view`] respells an
//! expression into another package's view before it is compared with that
//! package's patterns.
use crate::hir::{Declaration, Edge, EventType, Expr, ExprKind, Node, Package, Type, Value};
use crate::temporal::{self, Key, MatterEvent, World};
use std::collections::{BTreeMap, BTreeSet};

/// Event type names that excuse a missed deadline when cited (SPEC §5.6).
pub(super) const EXCUSALS: [&str; 3] = ["waived", "extended", "excused"];

/// The used packages as one replayable graph.
pub(super) struct Graph<'a> {
    /// Packages in `uses` order.
    pub packages: Vec<&'a Package>,
    /// Versioned declarations of every used package.
    pub world: World<'a>,
    links: BTreeMap<String, Vec<(String, &'a Edge)>>,
}

impl<'a> Graph<'a> {
    /// The graph over `packages` (non-empty, `uses` order).
    pub fn new(packages: Vec<&'a Package>) -> Self {
        let world = World::from_packages(&packages);
        let mut links: BTreeMap<String, Vec<(String, &'a Edge)>> = BTreeMap::new();
        for p in &packages {
            for d in declarations(p) {
                if let Declaration::Link(l) = d {
                    let pack = l.from.split_once("::").map_or("", |(p, _)| p).to_owned();
                    links
                        .entry(l.from.clone())
                        .or_default()
                        .push((pack, &l.edge));
                }
            }
        }
        Self {
            packages,
            world,
            links,
        }
    }

    /// The primary package's pack id.
    pub fn primary(&self) -> &'a str {
        self.packages
            .first()
            .map_or("", |p| p.manifest.pack_id.as_str())
    }

    fn is_pack(&self, id: &str) -> bool {
        self.packages.iter().any(|p| p.manifest.pack_id == id)
    }

    /// `name` written in `pack`'s view, fully qualified (`pack::name`).
    pub fn qualify(&self, pack: &str, name: &str) -> String {
        match name.split_once("::") {
            Some((p, _)) if self.is_pack(p) => name.to_owned(),
            _ => format!("{pack}::{name}"),
        }
    }

    /// `name` (in the primary view) respelled for `pack`'s view.
    pub fn respell(&self, pack: &str, name: &str) -> String {
        let qualified = self.qualify(self.primary(), name);
        match qualified.split_once("::") {
            Some((p, local)) if p == pack => local.to_owned(),
            _ => qualified,
        }
    }

    /// `e` (in the primary view) respelled for `pack`'s view.
    pub fn view(&self, pack: &str, e: &Expr) -> Expr {
        let ty = self.view_type(pack, &e.ty);
        let kind = match &e.kind {
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: self.respell(pack, callee),
                args: args.iter().map(|a| self.view(pack, a)).collect(),
            },
            ExprKind::Literal {
                value: Value::Variant(v),
            } => ExprKind::Literal {
                value: Value::Variant(self.respell(pack, v)),
            },
            other => other.clone(),
        };
        Expr {
            ty,
            kind,
            span: e.span.clone(),
        }
    }

    fn view_type(&self, pack: &str, t: &Type) -> Type {
        match t {
            Type::Named(n) => Type::Named(self.respell(pack, n)),
            other => other.clone(),
        }
    }

    /// The temporal key of qualified `pack::id`.
    pub fn key(qualified: &str) -> Key {
        let (pack, name) = qualified.split_once("::").unwrap_or(("", qualified));
        Key {
            pack: pack.to_owned(),
            name: name.to_owned(),
        }
    }

    /// Every version of node `qualified`, in declaration order.
    pub fn node_versions(&self, qualified: &str) -> Vec<&'a Node> {
        self.world
            .versions(&Self::key(qualified))
            .versions
            .iter()
            .filter_map(|v| match self.world.decls[v.decl].decl {
                Declaration::Node(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Whether some version of node `qualified` exists.
    pub fn has_node(&self, qualified: &str) -> bool {
        !self.node_versions(qualified).is_empty()
    }

    /// The node version in force on `date` (SPEC §5.4, no transitions).
    pub fn node_at(&self, qualified: &str, date: &str, known_at: Option<&str>) -> Option<&'a Node> {
        let key = Self::key(qualified);
        let set = self.world.versions(&key);
        temporal::active(&self.world, &key, &set, date, known_at)
            .ok()
            .and_then(|v| match self.world.decls[v.decl].decl {
                Declaration::Node(n) => Some(n),
                _ => None,
            })
    }

    /// The version of declaration `qualified` that governs `events[at]`
    /// (transitions applied), as a declaration.
    pub fn governing(
        &self,
        qualified: &str,
        events: &[MatterEvent<'_>],
        at: usize,
        known_at: Option<&str>,
    ) -> Result<&'a Declaration, temporal::Ungoverned> {
        temporal::governing(&self.world, &Self::key(qualified), events, at, known_at)
            .map(|g| self.world.decls[g.version].decl)
    }

    /// The outgoing edges of node version `node` at qualified id `at`: its
    /// own edges (in its pack), then link edges, with each edge's pack.
    pub fn edges(&self, at: &str, node: &'a Node) -> Vec<(String, &'a Edge)> {
        let pack = at.split_once("::").map_or("", |(p, _)| p).to_owned();
        node.edges
            .iter()
            .map(|e| (pack.clone(), e))
            .chain(self.links.get(at).into_iter().flatten().cloned())
            .collect()
    }

    /// The qualified target of `edge` declared in `pack`.
    pub fn target(&self, pack: &str, edge: &Edge) -> String {
        self.qualify(pack, &edge.to)
    }

    /// Every event type of the used packages: (pack id, declaration).
    pub fn event_types(&self) -> impl Iterator<Item = (&'a str, &'a EventType)> + '_ {
        self.packages.iter().flat_map(|p| {
            declarations(p).filter_map(move |d| match d {
                Declaration::Event(e) => Some((p.manifest.pack_id.as_str(), e)),
                _ => None,
            })
        })
    }

    /// The event type `name` declared in `pack`.
    pub fn event_type(&self, pack: &str, name: &str) -> Option<&'a EventType> {
        self.event_types()
            .find(|(p, e)| *p == pack && e.name == name)
            .map(|(_, e)| e)
    }

    /// The variants of enum `name` declared in `pack`.
    pub fn variants(&self, pack: &str, name: &str) -> Option<&'a [String]> {
        let p = self.packages.iter().find(|p| p.manifest.pack_id == pack)?;
        declarations(p).find_map(|d| match d {
            Declaration::Enum(e) if e.name == name => Some(e.variants.as_slice()),
            _ => None,
        })
    }

    /// Qualified ids of every logical deadline of the used packages.
    pub fn deadlines(&self) -> BTreeSet<String> {
        self.world
            .decls
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.decl, Declaration::Deadline(_)))
            .map(|(i, _)| {
                let k = self.world.key(i);
                format!("{}::{}", k.pack, k.name)
            })
            .collect()
    }

    /// The `has_event` patterns of every transition arm, with the pack each
    /// is written in: matter history the transitions read (an order
    /// keeping the prior rules, FRCP 86(a)(2)(B)), so an event matching
    /// one is a legal timing-only event.
    pub fn transition_patterns(&self) -> Vec<(&'a str, &'a Expr)> {
        let mut out = Vec::new();
        for p in &self.packages {
            for d in declarations(p) {
                if let Declaration::Transition(t) = d {
                    for c in t.arms.iter().filter_map(|a| a.unless.as_ref()) {
                        atoms(c, &mut |e| out.push((p.manifest.pack_id.as_str(), e)));
                    }
                }
            }
        }
        out
    }

    /// Whether the used packages declare an excusing event type.
    pub fn declares_excusal(&self) -> bool {
        self.event_types().any(|(_, e)| is_excusal(&e.name))
    }
}

/// Whether event type `name` (any spelling) is `Waived`/`Extended`/`Excused`.
pub(super) fn is_excusal(name: &str) -> bool {
    let local = name.rsplit("::").next().unwrap_or(name);
    EXCUSALS.contains(&local.to_ascii_lowercase().as_str())
}

/// Calls `f` with the pattern of every `has_event(P)` inside `e`.
fn atoms<'e>(e: &'e Expr, f: &mut dyn FnMut(&'e Expr)) {
    if let Some(p) = temporal::condition::has_event(e) {
        f(p);
        return;
    }
    match &e.kind {
        ExprKind::Unary { operand, .. } => atoms(operand, f),
        ExprKind::Binary { lhs, rhs, .. } => {
            atoms(lhs, f);
            atoms(rhs, f);
        }
        _ => {}
    }
}

/// Every declaration of `p`, module by module.
pub(super) fn declarations(p: &Package) -> impl Iterator<Item = &Declaration> {
    p.modules.iter().flat_map(|m| m.declarations.iter())
}

/// Compact source-like spelling of an event call (`Served(complaint, Mail)`).
pub(super) fn display(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Call { callee, args } => format!(
            "{callee}({})",
            args.iter().map(display).collect::<Vec<_>>().join(", ")
        ),
        ExprKind::Literal { value } => match value {
            Value::Variant(v) => v.rsplit("::").next().unwrap_or(v).to_owned(),
            Value::Date(d) => d.clone(),
            Value::String(s) => format!("{s:?}"),
            Value::Boolean(b) => b.to_string(),
            Value::Number(r) => crate::types::rational::display(*r),
            Value::Money(m) => format!("{} {} minor units", m.minor_units, m.currency),
            _ => "_".to_owned(),
        },
        ExprKind::Ref { target } => target.clone(),
        _ => "_".to_owned(),
    }
}
