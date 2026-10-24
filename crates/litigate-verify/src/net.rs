//! `workflow-sound` (SPEC §7; design review §2.7, §2.8, §6 item 27): the
//! lowered pack read as a bounded workflow net (van der Aalst 1998) and
//! checked for soundness.
//!
//! The net has one control place per node and one 1-safe place per state
//! flag. Each edge is a transition moving the control token from its node
//! to its target, marking the flags it `sets`, unmarking those it
//! `clears`, and enabled only when every flag it `requires` is marked and
//! none it `forbids` is. Every transition has one control place in and one
//! out, so the control places carry the P-invariant "exactly one token":
//! the matter is at one node. A marking is therefore a node and a flag
//! set, the net is bounded by `nodes x 2^flags`, and exploring every
//! marking reachable from `start` or an `entry` node decides each property
//! below exactly, with a firing sequence as its counterexample. Every edge
//! of a rule package targets one of its own nodes (L1002 otherwise); a
//! matter leaves it only through a terminal, which links route onward.
//!
//! - Option to complete (L5007): from every reachable marking a terminal
//!   is reachable. This finds a node whose every edge the reached flags
//!   disable, and a cycle with no way out.
//! - No dead transitions (L5008): every edge out of a reachable node is
//!   enabled in some reachable marking.
//! - Obligations can be consumed (L5009). A `rule duty` deadline is an
//!   obligation place (§2.7), marked by an edge whose `event` matches the
//!   deadline's trigger. Its token is consumed by the act, an enabled
//!   `within` edge of it, or by the breach at its due date, a clock
//!   transition outside this net; a terminal ends the proceeding and moots
//!   it. The token is therefore not conserved, and what the net must
//!   guarantee is that the act can happen: from the marking every creating
//!   firing reaches, some marking enabling a `within` edge of the duty is
//!   reachable. Otherwise the bearer can only ever breach it.
use crate::diag::{emit, note};
use crate::evidence;
use crate::report::Builder;
use crate::Context;
use litgraph::model::{NodeKind, Pack, RawEdge};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_lower::SourceMap;
use litigate_sema::hir::{self, DeadlineWindow, Declaration, RuleKind};
use litigate_sema::temporal::condition;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

/// The `entry` tag: an extra root (SPEC §5.1).
const ENTRY: &str = "entry";

/// Most markings explored; a larger net is undecided, which fails.
const MAX_MARKINGS: usize = 1 << 16;

/// Most edges of a counterexample firing sequence shown.
const MAX_PATH: usize = 12;

/// The soundness findings of `pack`; records the markings explored as
/// evidence when the net is within the bound.
pub(crate) fn check(
    b: &mut Builder,
    cx: &Context<'_>,
    pack: &Pack,
    map: &SourceMap,
) -> Vec<Diagnostic> {
    let net = Net::new(cx, pack, map);
    let Some(graph) = Reach::explore(&net, MAX_MARKINGS) else {
        let d = emit(
            C::NoOptionToComplete,
            cx.profile,
            cx.node_span(&pack.start_node_id),
            format!(
                "the procedure net of `{}` has more than {MAX_MARKINGS} reachable markings, so its soundness is undecided",
                pack.id
            ),
        );
        return vec![note(d, None, "split the package, or use fewer state flags")];
    };
    evidence::net(b, graph.markings.len());
    let mut out = incomplete(cx, &net, &graph);
    out.extend(dead(cx, &net, &graph));
    out.extend(unconsumed(cx, &net, &graph));
    out
}

/// The net: the pack's edges with their obligation effects.
struct Net<'p> {
    pack: &'p Pack,
    /// Node index by id.
    index: BTreeMap<&'p str, usize>,
    /// Edge indices out of each node, in pack order.
    out: Vec<Vec<usize>>,
    /// Tracked duties, `pack::id`.
    duties: Vec<String>,
    /// Per edge: the duties it creates.
    creates: Vec<BTreeSet<usize>>,
    /// Per edge: the duties it is a `within` act of.
    within: Vec<BTreeSet<usize>>,
}

impl<'p> Net<'p> {
    fn new(cx: &Context<'_>, pack: &'p Pack, map: &SourceMap) -> Self {
        let index: BTreeMap<&str, usize> = pack
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();
        let mut out = vec![Vec::new(); pack.nodes.len()];
        for (i, from) in pack
            .edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Some((i, *index.get(e.from.as_str())?)))
        {
            out[from].push(i);
        }
        let hir = edges_of(cx, map);
        let duties = duties(cx, &hir);
        let key = |target: &str| {
            let (pkg, id) = cx.owner(target);
            duties
                .iter()
                .position(|d| *d == format!("{}::{id}", pkg.manifest.pack_id))
        };
        let within = (0..pack.edges.len())
            .map(|i| {
                hir.get(i)
                    .and_then(|e| match &e.deadline.as_ref()?.window {
                        DeadlineWindow::Declared { deadline } => key(deadline),
                        DeadlineWindow::Inline { .. } => None,
                    })
                    .into_iter()
                    .collect()
            })
            .collect();
        let creates = (0..pack.edges.len())
            .map(|i| {
                let Some(event) = hir.get(i).and_then(|e| e.event.as_ref()) else {
                    return BTreeSet::new();
                };
                (0..duties.len())
                    .filter(|&d| {
                        cx.deadline_versions(&duties[d])
                            .iter()
                            .any(|v| condition::matches(&v.trigger, event))
                    })
                    .collect()
            })
            .collect();
        Self {
            pack,
            index,
            out,
            duties,
            creates,
            within,
        }
    }

    fn edge(&self, i: usize) -> &'p RawEdge {
        &self.pack.edges[i]
    }

    fn is_terminal(&self, node: usize) -> bool {
        self.pack.nodes[node].kind == Some(NodeKind::Terminal)
    }

    /// Whether edge `i` is enabled under `flags`.
    fn enabled(&self, i: usize, flags: &BTreeSet<String>) -> bool {
        let e = self.edge(i);
        e.requires.iter().all(|f| flags.contains(f)) && !e.forbids.iter().any(|f| flags.contains(f))
    }

    /// Whether a marking at `node` under `flags` enables a `within` act of
    /// `duty`.
    fn offers(&self, node: usize, flags: &BTreeSet<String>, duty: usize) -> bool {
        self.out[node]
            .iter()
            .any(|&i| self.within[i].contains(&duty) && self.enabled(i, flags))
    }

    /// `id`, or `from -> to` for an anonymous edge.
    fn name(&self, i: usize) -> String {
        let e = self.edge(i);
        e.id.clone()
            .unwrap_or_else(|| format!("{} -> {}", e.from, e.to))
    }
}

/// The HIR edge of each pack edge, in pack order: the emitted version of
/// each node (the source map's `decl`) and its edges.
fn edges_of<'a>(cx: &Context<'a>, map: &SourceMap) -> Vec<&'a hir::Edge> {
    map.entries
        .iter()
        .filter(|e| e.pointer.starts_with("/nodes/"))
        .filter_map(|e| {
            cx.nodes().find(|n| {
                e.decl.as_deref() == Some(&format!("{}@{}", n.meta.id, n.meta.effective.from))
            })
        })
        .flat_map(|n| n.edges.iter())
        .collect()
}

/// The duties tracked: every `rule duty` deadline of the package, and every
/// one a `within` edge names, as `pack::id`.
fn duties(cx: &Context<'_>, edges: &[&hir::Edge]) -> Vec<String> {
    let own = cx
        .declarations()
        .filter_map(|d| match d {
            Declaration::Rule(r) if r.rule_kind == RuleKind::Duty => Some(&r.declarations),
            _ => None,
        })
        .flatten()
        .filter(|id| !cx.deadline_versions(id).is_empty())
        .map(|id| format!("{}::{id}", cx.package.manifest.pack_id));
    let named = edges
        .iter()
        .filter_map(|e| match &e.deadline.as_ref()?.window {
            DeadlineWindow::Declared { deadline } => Some(deadline.as_str()),
            DeadlineWindow::Inline { .. } => None,
        })
        .filter(|d| cx.is_duty(d))
        .map(|d| {
            let (pkg, id) = cx.owner(d);
            format!("{}::{id}", pkg.manifest.pack_id)
        });
    let set: BTreeSet<String> = own.chain(named).collect();
    set.into_iter().collect()
}

/// A reachable marking: where the control token is, and which flags are
/// set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Marking {
    node: usize,
    flags: BTreeSet<String>,
}

/// One firing: the edge, and the marking it reaches.
type Firing = (usize, usize);

/// The reachability graph.
struct Reach {
    markings: Vec<Marking>,
    /// Firings out of each marking.
    next: Vec<Vec<Firing>>,
    /// The firing first reaching each marking (`None` for a root).
    parent: Vec<Option<(usize, usize)>>,
}

impl Reach {
    /// Every marking reachable from the roots, breadth first; `None` past
    /// `cap` markings.
    fn explore(net: &Net<'_>, cap: usize) -> Option<Self> {
        let mut graph = Self {
            markings: Vec::new(),
            next: Vec::new(),
            parent: Vec::new(),
        };
        let mut seen: HashMap<Marking, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let roots = std::iter::once(net.pack.start_node_id.as_str()).chain(
            net.pack
                .nodes
                .iter()
                .filter(|n| n.tags.iter().any(|t| t == ENTRY))
                .map(|n| n.id.as_str()),
        );
        for &node in roots.filter_map(|root| net.index.get(root)) {
            let m = Marking {
                node,
                flags: BTreeSet::new(),
            };
            graph.insert(&mut seen, &mut queue, m, None);
        }
        while let Some(at) = queue.pop_front() {
            let m = graph.markings[at].clone();
            let enabled = net.out[m.node]
                .iter()
                .filter(|&&i| net.enabled(i, &m.flags))
                .filter_map(|&i| Some((i, *net.index.get(net.edge(i).to.as_str())?)));
            for (i, to) in enabled {
                let e = net.edge(i);
                let mut flags = m.flags.clone();
                flags.extend(e.sets.iter().cloned());
                flags.retain(|f| !e.clears.contains(f));
                let reached = Marking { node: to, flags };
                let ix = graph.add(&mut seen, &mut queue, reached, Some((at, i)), cap)?;
                graph.next[at].push((i, ix));
            }
        }
        Some(graph)
    }

    fn add(
        &mut self,
        seen: &mut HashMap<Marking, usize>,
        queue: &mut VecDeque<usize>,
        m: Marking,
        parent: Option<(usize, usize)>,
        cap: usize,
    ) -> Option<usize> {
        if let Some(&ix) = seen.get(&m) {
            return Some(ix);
        }
        if self.markings.len() == cap {
            return None;
        }
        Some(self.insert(seen, queue, m, parent))
    }

    /// Records the new marking `m`, or finds it, and returns its index.
    fn insert(
        &mut self,
        seen: &mut HashMap<Marking, usize>,
        queue: &mut VecDeque<usize>,
        m: Marking,
        parent: Option<(usize, usize)>,
    ) -> usize {
        if let Some(&ix) = seen.get(&m) {
            return ix;
        }
        let ix = self.markings.len();
        seen.insert(m.clone(), ix);
        self.markings.push(m);
        self.next.push(Vec::new());
        self.parent.push(parent);
        queue.push_back(ix);
        ix
    }

    /// Which markings reach one where `goal` holds (itself included).
    fn backward(&self, goal: impl Fn(usize) -> bool) -> Vec<bool> {
        let n = self.markings.len();
        let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (at, firings) in self.next.iter().enumerate() {
            for &(_, to) in firings {
                preds[to].push(at);
            }
        }
        let mut done: Vec<bool> = (0..n).map(&goal).collect();
        let mut queue: VecDeque<usize> = (0..n).filter(|&at| done[at]).collect();
        while let Some(at) = queue.pop_front() {
            for &p in &preds[at] {
                if !done[p] {
                    done[p] = true;
                    queue.push_back(p);
                }
            }
        }
        done
    }

    /// The edges firing from a root to marking `at`, the last
    /// [`MAX_PATH`] of them.
    fn path(&self, net: &Net<'_>, mut at: usize) -> String {
        let mut edges = Vec::new();
        while let Some((from, edge)) = self.parent[at] {
            edges.push(net.name(edge));
            at = from;
        }
        let root = &net.pack.nodes[self.markings[at].node].id;
        edges.reverse();
        let shown = edges.len().saturating_sub(MAX_PATH);
        let mut steps = vec![format!("`{root}`")];
        if shown > 0 {
            steps.push(format!("... {shown} more"));
        }
        steps.extend(edges[shown..].iter().map(|e| format!("`{e}`")));
        steps.join(" -> ")
    }
}

/// `{a, b}`, or `no flags`.
fn flags(set: &BTreeSet<String>) -> String {
    if set.is_empty() {
        "no flags".to_owned()
    } else {
        let names: Vec<&str> = set.iter().map(String::as_str).collect();
        format!("flags {{{}}}", names.join(", "))
    }
}

/// L5007 once per node where the matter loses its option to complete: a
/// marking from which no terminal is reachable that is a root, is entered
/// from a marking that could still complete, or enables no edge. A node
/// with no edge at all is L5004's.
fn incomplete(cx: &Context<'_>, net: &Net<'_>, graph: &Reach) -> Vec<Diagnostic> {
    let done = graph.backward(|at| net.is_terminal(graph.markings[at].node));
    let mut entered: Vec<bool> = graph.parent.iter().map(Option::is_none).collect();
    for (at, firings) in graph.next.iter().enumerate() {
        for &(_, to) in firings.iter().filter(|_| done[at]) {
            entered[to] = true;
        }
    }
    let n = graph.markings.len();
    let mut reported = BTreeSet::new();
    let mut out = Vec::new();
    let lost = (0..n).filter(|&at| !done[at] && (entered[at] || graph.next[at].is_empty()));
    for at in lost {
        let m = &graph.markings[at];
        if net.out[m.node].is_empty() || !reported.insert(m.node) {
            continue;
        }
        let id = &net.pack.nodes[m.node].id;
        let enabled: Vec<String> = graph.next[at]
            .iter()
            .map(|&(i, _)| format!("`{}`", net.name(i)))
            .collect();
        let (message, fix) = if enabled.is_empty() {
            (
                format!(
                    "`{id}` with {} enables no edge: every edge out of it requires or forbids a flag, so the matter is stuck there",
                    flags(&m.flags)
                ),
                "add an edge enabled under these flags, or change the guards",
            )
        } else {
            (
                format!(
                    "from `{id}` with {} no terminal is reachable through its enabled edges {}",
                    flags(&m.flags),
                    enabled.join(", ")
                ),
                "route a path from here to a terminal, or declare the node where the matter ends `terminal`",
            )
        };
        let d = emit(C::NoOptionToComplete, cx.profile, cx.node_span(id), message);
        let d = note(d, None, format!("reached by {}", graph.path(net, at)));
        out.push(note(d, None, fix));
    }
    out
}

/// L5008 for every edge out of a reachable node that no reachable marking
/// enables.
fn dead(cx: &Context<'_>, net: &Net<'_>, graph: &Reach) -> Vec<Diagnostic> {
    let mut nodes: BTreeMap<usize, Vec<&BTreeSet<String>>> = BTreeMap::new();
    for m in &graph.markings {
        nodes.entry(m.node).or_default().push(&m.flags);
    }
    let mut out = Vec::new();
    for (node, seen) in nodes {
        let dead = net.out[node]
            .iter()
            .filter(|&&i| !seen.iter().any(|f| net.enabled(i, f)));
        for &i in dead {
            let e = net.edge(i);
            let sets: Vec<String> = seen.iter().map(|f| flags(f)).collect();
            let guards: Vec<String> = [("requires", &e.requires), ("forbids", &e.forbids)]
                .into_iter()
                .filter(|(_, names)| !names.is_empty())
                .map(|(word, names)| format!("{word} [{}]", names.join(", ")))
                .collect();
            let d = emit(
                C::DeadTransition,
                cx.profile,
                cx.node_span(&e.from),
                format!(
                    "edge `{}` out of `{}` is never enabled: it {}, and `{}` is reached only with {}",
                    net.name(i),
                    e.from,
                    guards.join(" and "),
                    e.from,
                    sets.join(" or ")
                ),
            );
            out.push(note(
                d,
                None,
                "set the flags it requires on a path into the node, or remove the edge",
            ));
        }
    }
    out
}

/// L5009 once per duty and creating edge whose firing reaches a marking
/// from which no marking offering the duty's `within` act is reachable:
/// the obligation can never be consumed.
fn unconsumed(cx: &Context<'_>, net: &Net<'_>, graph: &Reach) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (d, duty) in net.duties.iter().enumerate() {
        let can = graph.backward(|at| {
            let m = &graph.markings[at];
            net.offers(m.node, &m.flags, d)
        });
        let mut reported = BTreeSet::new();
        for firings in &graph.next {
            let creating = firings
                .iter()
                .filter(|&&(i, to)| !can[to] && net.creates[i].contains(&d));
            for &(i, to) in creating {
                if !reported.insert(i) {
                    continue;
                }
                let diag = emit(
                    C::ObligationUnconsumed,
                    cx.profile,
                    cx.node_span(&net.edge(i).from),
                    format!(
                        "obligation `{duty}`, created by edge `{}`, can never be consumed: no marking reachable from `{}` with {} offers a `within {}` edge",
                        net.name(i),
                        net.pack.nodes[graph.markings[to].node].id,
                        flags(&graph.markings[to].flags),
                        cx.local(duty)
                    ),
                );
                let diag = note(diag, None, format!("created by {}", graph.path(net, to)));
                out.push(note(
                    diag,
                    None,
                    "a duty is consumed by a `within` act or a breach at its due date: route a path to a node offering its `within` edge, or make the deadline a `power`",
                ));
            }
        }
    }
    out
}
