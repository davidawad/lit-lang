//! The procedure graph a matter replays over: the `uses` packages in order
//! (the first is primary), their versioned nodes and deadlines through the
//! temporal [`World`], link edges, and the event and enum vocabulary.
//!
//! Names follow sema's nominal convention: a name is bare inside the
//! package that declares it and `pack::Name` everywhere else. Case HIR is
//! written in the primary package's view; [`Graph::view`] respells an
//! expression into another package's view before it is compared with that
//! package's patterns.
use crate::hir::{
    DeadlineWindow, Declaration, Edge, EdgeDeadline, EventType, Expr, ExprKind, Extension, GuardOp,
    Node, Package, Relation, Type, Value,
};
use crate::precedence;
use crate::temporal::{self, Key, MatterEvent, World};
use std::collections::{BTreeMap, BTreeSet};

/// The used packages as one replayable graph.
pub(super) struct Graph<'a> {
    /// Packages in `uses` order.
    pub packages: Vec<&'a Package>,
    /// Versioned declarations of every used package.
    pub world: World<'a>,
    /// Link edges by qualified `from` node, each with the from node's pack
    /// and respelled into its view (see [`Graph::new`]).
    links: BTreeMap<String, Vec<(String, Edge)>>,
    /// The guard operations of the used links packages' patches (SPEC
    /// §3.6) by qualified instance edge `base@origin::edge`, respelled into
    /// the base pack's view.
    guard_ops: BTreeMap<String, Vec<GuardOp>>,
    /// Loaded transitive dependencies of the used packages that are not
    /// themselves used: vocabulary (their enums) and law a used package's
    /// declarations read (a chained period's base, a counting rule), but
    /// their own triggers are never replayed.
    deps: Vec<&'a Package>,
}

impl<'a> Graph<'a> {
    /// The graph over `packages` (non-empty, `uses` order), with their
    /// dependencies found by name among `loaded`.
    ///
    /// A link edge belongs to the pack of its `from` node (its actors are
    /// that pack's, and `pack::edge` names it for removal), so its `event`
    /// pattern, written in the links package, is respelled into the view
    /// of that pack (an instance `base@origin` reads in `base`'s view).
    pub fn new(packages: Vec<&'a Package>, loaded: &[&'a Package]) -> Self {
        let deps = dependencies(&packages, loaded);
        // The law the used packages see, as `check` and lowering select it
        // (`World::closure`): their dependencies' versioned declarations
        // (a chain's base deadline, a counting rule and its transitions)
        // and the authority lattice precedence ranks claimants by (SPEC
        // §5.5), which the dependencies (and `core`) declare.
        let world = World::from_packages(&[packages.as_slice(), deps.as_slice()].concat());
        let mut graph = Self {
            world,
            deps,
            packages,
            links: BTreeMap::new(),
            guard_ops: BTreeMap::new(),
        };
        let mut links: BTreeMap<String, Vec<(String, Edge)>> = BTreeMap::new();
        for p in &graph.packages {
            for d in declarations(p) {
                if let Declaration::Instance(i) = d {
                    let from = p.manifest.pack_id.as_str();
                    for (edge, patch) in &i.patch_edges {
                        let ops: Vec<GuardOp> = patch
                            .guard_ops
                            .iter()
                            .map(|op| GuardOp {
                                condition: crate::guard::respell(&op.condition, from, base(&i.pack)),
                                ..op.clone()
                            })
                            .collect();
                        graph
                            .guard_ops
                            .entry(format!("{}::{edge}", i.name))
                            .or_default()
                            .extend(ops);
                    }
                }
                if let Declaration::Link(l) = d {
                    let (from, pack) = (p.manifest.pack_id.as_str(), pack_of(&l.from));
                    let mut edge = l.edge.clone();
                    edge.event = edge.event.map(|e| graph.respelled(from, base(pack), &e));
                    if let Some(EdgeDeadline {
                        window: DeadlineWindow::Declared { deadline },
                        ..
                    }) = &mut edge.deadline
                    {
                        *deadline = graph.respell_from(from, base(pack), deadline);
                    }
                    links
                        .entry(l.from.clone())
                        .or_default()
                        .push((pack.to_owned(), edge));
                }
            }
        }
        graph.links = links;
        graph
    }

    /// The primary package's pack id.
    pub fn primary(&self) -> &'a str {
        self.packages
            .first()
            .map_or("", |p| p.manifest.pack_id.as_str())
    }

    /// Whether `id` names a used pack or a dependency of one, or an
    /// instance `base@origin` of one (a links package composes it; its
    /// nodes are `base`'s).
    fn is_pack(&self, id: &str) -> bool {
        self.vocabulary().any(|p| p.manifest.pack_id == base(id))
    }

    /// The used packages, then their dependencies.
    pub fn vocabulary(&self) -> impl Iterator<Item = &'a Package> + '_ {
        self.packages.iter().chain(&self.deps).copied()
    }

    /// `name` written in `pack`'s view, fully qualified (`pack::name`).
    pub fn qualify(&self, pack: &str, name: &str) -> String {
        match name.split_once("::") {
            Some((p, _)) if self.is_pack(p) => name.to_owned(),
            _ => format!("{pack}::{name}"),
        }
    }

    /// `name` (written in `from`'s view) respelled for `pack`'s view.
    fn respell_from(&self, from: &str, pack: &str, name: &str) -> String {
        let qualified = self.qualify(from, name);
        match qualified.split_once("::") {
            Some((p, local)) if p == pack => local.to_owned(),
            _ => qualified,
        }
    }

    /// `name` (in the primary view) respelled for `pack`'s view.
    pub fn respell(&self, pack: &str, name: &str) -> String {
        self.respell_from(self.primary(), pack, name)
    }

    /// `e` (in the primary view) respelled for `pack`'s view.
    pub fn view(&self, pack: &str, e: &Expr) -> Expr {
        self.respelled(self.primary(), pack, e)
    }

    /// `e` (written in `from`'s view) respelled for `pack`'s view.
    fn respelled(&self, from: &str, pack: &str, e: &Expr) -> Expr {
        let ty = match &e.ty {
            Type::Named(n) => Type::Named(self.respell_from(from, pack, n)),
            other => other.clone(),
        };
        let kind = match &e.kind {
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: self.respell_from(from, pack, callee),
                args: args.iter().map(|a| self.respelled(from, pack, a)).collect(),
            },
            ExprKind::Literal {
                value: Value::Variant(v),
            } => ExprKind::Literal {
                value: Value::Variant(self.respell_from(from, pack, v)),
            },
            ExprKind::Literal {
                value: Value::Edge(e),
            } => ExprKind::Literal {
                value: Value::Edge(self.respell_from(from, pack, e)),
            },
            other => other.clone(),
        };
        Expr {
            ty,
            kind,
            span: e.span.clone(),
        }
    }

    /// The temporal key of qualified `pack::id`; an instance
    /// `base@origin::id` keys to `base`'s declaration.
    pub fn key(qualified: &str) -> Key {
        let (pack, name) = qualified.split_once("::").unwrap_or(("", qualified));
        Key {
            pack: base(pack).to_owned(),
            name: name.to_owned(),
        }
    }

    /// Whether `pack` (a base pack id) is a used package's: a matter's
    /// stages and triggers are only theirs, never a dependency's.
    pub fn used(&self, pack: &str) -> bool {
        self.packages.iter().any(|p| p.manifest.pack_id == pack)
    }

    /// Every version of node `qualified` of a used package, in declaration
    /// order.
    pub fn node_versions(&self, qualified: &str) -> Vec<&'a Node> {
        let key = Self::key(qualified);
        if !self.used(&key.pack) {
            return Vec::new();
        }
        self.world
            .versions(&key)
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

    /// Whether some loaded package declares `qualified` (any version).
    pub fn declares(&self, qualified: &str) -> bool {
        !self
            .world
            .versions(&Self::key(qualified))
            .versions
            .is_empty()
    }

    /// The node version in force on `date` (SPEC §5.4, no transitions),
    /// unless a claimant displaces it (SPEC §5.5): lowering omits a
    /// displaced node, so a matter does not find it either.
    pub fn node_at(&self, qualified: &str, date: &str, known_at: Option<&str>) -> Option<&'a Node> {
        let key = Self::key(qualified);
        if !self.used(&key.pack) {
            return None;
        }
        precedence::select(&self.world, &key, date, known_at)
            .ok()
            .filter(|&w| self.world.key(w) == key)
            .and_then(|w| match self.world.decls[w].decl {
                Declaration::Node(n) => Some(n),
                _ => None,
            })
    }

    /// The declaration that governs `events[at]` for declaration
    /// `qualified` (precedence and transitions applied), with the
    /// qualified id of the logical declaration it is a version of: a
    /// claimant that displaces `qualified` has its own.
    pub fn governing(
        &self,
        qualified: &str,
        events: &[MatterEvent<'_>],
        at: usize,
        known_at: Option<&str>,
    ) -> Result<(String, &'a Declaration), temporal::Ungoverned> {
        temporal::governing(&self.world, &Self::key(qualified), events, at, known_at).map(|g| {
            let k = self.world.key(g.version);
            (
                format!("{}::{}", k.pack, k.name),
                self.world.decls[g.version].decl,
            )
        })
    }

    /// The declarations the version of `qualified` in force on `date`
    /// displaces by an `override` or `notwithstanding` claim (SPEC §5.5),
    /// qualified.
    pub fn claims(&self, qualified: &str, date: &str, known_at: Option<&str>) -> Vec<String> {
        let version = temporal::select(&self.world, &Self::key(qualified), date, known_at).ok();
        version
            .into_iter()
            .flat_map(|v| {
                let pack = self.world.pack_of(v);
                self.world.decls[v]
                    .meta
                    .relations
                    .iter()
                    .filter_map(move |r| match r {
                        Relation::Override(x) | Relation::Notwithstanding(x) => {
                            let k = self.world.target(pack, x).key;
                            Some(format!("{}::{}", k.pack, k.name))
                        }
                        _ => None,
                    })
            })
            .collect()
    }

    /// A note naming claimant pairs no relation or canon orders.
    pub fn unordered(&self, pairs: &[(usize, usize)]) -> String {
        format!(
            "{} (L3003 in `litigate check`)",
            precedence::select::unordered(&self.world, pairs, "")
        )
    }

    /// The outgoing edges of node version `node` at qualified id `at`: its
    /// own edges (in its pack), then link edges, with each edge's pack.
    pub fn edges<'s>(&'s self, at: &str, node: &'s Node) -> Vec<(String, &'s Edge)> {
        let pack = pack_of(at);
        node.edges
            .iter()
            .map(|e| (pack.to_owned(), e))
            .chain(
                self.links
                    .get(at)
                    .into_iter()
                    .flatten()
                    .map(|(p, e)| (p.clone(), e)),
            )
            .collect()
    }

    /// The guard of edge `edge` of node `posture` on `date` (SPEC §3.6):
    /// its own, with the guard operations of an instance's patch in force
    /// then composed over it, and the operations applied.
    pub fn guard(
        &self,
        posture: &str,
        edge: &'a Edge,
        date: &str,
        known_at: Option<&str>,
    ) -> (Option<std::borrow::Cow<'a, Expr>>, Vec<&GuardOp>) {
        let ops: Vec<&GuardOp> = edge
            .id
            .as_ref()
            .and_then(|id| self.guard_ops.get(&format!("{}::{id}", pack_of(posture))))
            .map(|ops| ops.iter().collect())
            .unwrap_or_default();
        if ops.is_empty() {
            return (edge.guard.as_ref().map(std::borrow::Cow::Borrowed), Vec::new());
        }
        let composed =
            crate::guard::compose(self.world.lattice(), edge.guard.as_ref(), &ops, date, known_at);
        let applied = composed.applied.iter().map(|&i| ops[i]).collect();
        (composed.guard.map(std::borrow::Cow::Owned), applied)
    }

    /// The qualified target of `edge` declared in `pack`.
    pub fn target(&self, pack: &str, edge: &Edge) -> String {
        self.qualify(pack, &edge.to)
    }

    /// Whether `id` is the pack id of a dependency that is not used.
    pub fn is_dependency(&self, id: &str) -> bool {
        self.deps.iter().any(|p| p.manifest.pack_id == id)
    }

    /// Every event type of the used packages, then of their dependencies
    /// (vocabulary: an event starting a dependency's period that a used
    /// package's chain follows): (pack id, declaration).
    pub fn event_types(&self) -> impl Iterator<Item = (&'a str, &'a EventType)> + '_ {
        self.vocabulary().flat_map(|p| {
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

    /// The variants of enum `name` declared in `pack` (a used package or a
    /// dependency of one).
    pub fn variants(&self, pack: &str, name: &str) -> Option<&'a [String]> {
        self.vocabulary()
            .filter(|p| p.manifest.pack_id == pack)
            .flat_map(declarations)
            .find_map(|d| match d {
                Declaration::Enum(e) if e.name == name => Some(e.variants.as_slice()),
                _ => None,
            })
    }

    /// Qualified ids of every logical deadline of the used packages (a
    /// dependency's deadlines are read, never replayed: SPEC §5.6).
    pub fn deadlines(&self) -> BTreeSet<String> {
        self.world
            .decls
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.decl, Declaration::Deadline(_)))
            .map(|(i, _)| self.world.key(i))
            .filter(|k| self.used(&k.pack))
            .map(|k| format!("{}::{}", k.pack, k.name))
            .collect()
    }

    /// The motion patterns of every `extend` grant (`on M()`, and its
    /// denial `unless D()`), with the pack each is written in: a motion to
    /// extend and its denial are matter history the grant reads (FRCP
    /// 6(b)(1)(B), FRAP 4(a)(5)(A)(i), Sup. Ct. R. 30.3), so an event
    /// matching one is legal without taking an edge.
    pub fn motion_patterns(&self) -> Vec<(&'a str, &'a Expr)> {
        let mut out = Vec::new();
        for p in &self.packages {
            for d in declarations(p) {
                if let Declaration::Deadline(x) = d {
                    for g in &x.extend {
                        if let Extension::Bounded {
                            motion: Some(m), ..
                        }
                        | Extension::Discretionary {
                            motion: Some(m), ..
                        } = g
                        {
                            let pack = p.manifest.pack_id.as_str();
                            out.push((pack, &m.event));
                            out.extend(m.unless.iter().map(|u| (pack, u)));
                        }
                    }
                }
            }
        }
        out
    }

    /// The `has_event` patterns of every transition arm, with the pack each
    /// is written in: matter history the transitions read (an order
    /// keeping the prior rules, FRCP 86(a)(2)(B)), so an event matching
    /// one is legal without taking an edge.
    pub fn transition_patterns(&self) -> Vec<(&'a str, &'a Expr)> {
        let mut out = Vec::new();
        for p in self.vocabulary() {
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

    /// Ids of every named edge declared at a node of `pack` (any version),
    /// including link edges leaving it.
    pub fn edge_ids(&self, pack: &str) -> BTreeSet<&str> {
        let own = self
            .packages
            .iter()
            .filter(|p| p.manifest.pack_id == pack)
            .flat_map(|p| declarations(p))
            .filter_map(|d| match d {
                Declaration::Node(n) => Some(n.edges.iter()),
                _ => None,
            })
            .flatten();
        let linked = self
            .links
            .iter()
            .filter(|(from, _)| from.split_once("::").is_some_and(|(p, _)| p == pack))
            .flat_map(|(_, edges)| edges.iter().map(|(_, e)| e));
        own.chain(linked).filter_map(|e| e.id.as_deref()).collect()
    }

    /// The other edges, link edges included, at every node of `pack` (any
    /// version) that offers edge `id`: the alternatives an election of `id`
    /// forecloses.
    pub fn alternatives(&self, pack: &str, id: &str) -> Vec<&Edge> {
        let mut out = Vec::new();
        let nodes = self
            .packages
            .iter()
            .filter(|p| p.manifest.pack_id == pack)
            .flat_map(|p| declarations(p))
            .filter_map(|d| match d {
                Declaration::Node(n) => Some(n),
                _ => None,
            });
        for n in nodes {
            let at = format!("{pack}::{}", n.meta.id);
            let edges = self.edges(&at, n);
            if edges.iter().any(|(_, e)| e.id.as_deref() == Some(id)) {
                out.extend(
                    edges
                        .into_iter()
                        .map(|(_, e)| e)
                        .filter(|e| e.id.as_deref() != Some(id)),
                );
            }
        }
        out
    }
}

/// The pack of qualified `pack::name` (empty when unqualified).
pub(super) fn pack_of(qualified: &str) -> &str {
    qualified.split_once("::").map_or("", |(p, _)| p)
}

/// The base pack of an instance `base@origin` (a pack is its own base).
pub(super) fn base(pack: &str) -> &str {
    pack.split_once('@').map_or(pack, |(b, _)| b)
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
pub fn display(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Call { callee, args } => format!(
            "{callee}({})",
            args.iter().map(display).collect::<Vec<_>>().join(", ")
        ),
        ExprKind::Literal { value } => match value {
            Value::Variant(v) => v.rsplit("::").next().unwrap_or(v).to_owned(),
            Value::Date(d) => d.clone(),
            Value::Hole(_) => "?".to_owned(),
            Value::String(s) => format!("{s:?}"),
            Value::Edge(e) => e.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(r) => crate::types::rational::display(*r),
            Value::Money(m) => format!("{} {} minor units", m.minor_units, m.currency),
            _ => "_".to_owned(),
        },
        ExprKind::Ref { target } => target.clone(),
        _ => "_".to_owned(),
    }
}

/// The transitive dependencies of `used` among `loaded`, by package name,
/// excluding `used` itself; each once, in discovery order.
fn dependencies<'a>(used: &[&'a Package], loaded: &[&'a Package]) -> Vec<&'a Package> {
    let mut seen: BTreeSet<&str> = used.iter().map(|p| p.manifest.name.as_str()).collect();
    let mut out: Vec<&'a Package> = Vec::new();
    let mut frontier: Vec<&'a Package> = used.to_vec();
    while let Some(p) = frontier.pop() {
        // `core` is importable undeclared (SPEC §4.3).
        let found = p
            .manifest
            .dependencies
            .keys()
            .map(String::as_str)
            .chain(["core"])
            .filter_map(|name| loaded.iter().find(|d| d.manifest.name == name));
        for &dep in found {
            if seen.insert(dep.manifest.name.as_str()) {
                out.push(dep);
                frontier.push(dep);
            }
        }
    }
    out
}
