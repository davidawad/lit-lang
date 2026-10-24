//! Typed guard operations in patches (SPEC §3.6): `when and C` narrows a
//! patched fact edge's guard, `when or C` widens it, `supersedes when C`
//! replaces it, each on its instrument's authority (`by Level`, `effective`,
//! `cite`).
//!
//! The operations on one edge are a set: [`order`] fixes the order they
//! apply in from their authorities and effective dates (lower authority
//! first, then the earlier `from`), never from source or file order, and
//! [`compose`] folds the ones in force on a date over the base guard.
//! [`check`] runs the package rules: placement (L2007), capability
//! (L3004), an order no authority or date fixes between operations that
//! do not commute (L3003), and disjointness of the composed guards (L2005).
use crate::hir::{
    Authored, BinaryOp, Declaration, Expr, ExprKind, GuardOp, GuardOpKind, Instance, Node,
    Relation, Type, UnaryOp, Value,
};
use crate::precedence::{Lattice, Order};
use crate::temporal::condition;
use crate::temporal::world::{error, DiagnosticExt as _, World};
use litigate_base::{Diagnostic, DiagnosticCode as C, Span};

/// Whether `a` applies before `b` in every canonical order: its authority
/// is lower, or equal with an earlier `from`. A strict partial order (the
/// lexicographic order over the authority lattice, then dates).
fn before(lattice: &Lattice, a: &GuardOp, b: &GuardOp) -> bool {
    match lattice.compare(&a.by, &b.by) {
        Order::Below => true,
        Order::Equal => a.effective.from < b.effective.from,
        _ => false,
    }
}

/// The canonical order of `ops` (indices): a linear extension of
/// [`before`], ties taken in the order of each operation's text (its kind,
/// level, dates, cite and condition), so neither source nor file order is
/// consulted. A higher authority applies last, so its operation prevails.
#[must_use]
pub fn order(lattice: &Lattice, ops: &[&GuardOp]) -> Vec<usize> {
    let mut rest: Vec<usize> = (0..ops.len()).collect();
    rest.sort_by_cached_key(|&i| sort_key(ops[i]));
    let mut out = Vec::with_capacity(ops.len());
    while !rest.is_empty() {
        // `before` is acyclic, so some remaining operation is minimal.
        let at = rest
            .iter()
            .position(|&i| !rest.iter().any(|&j| before(lattice, ops[j], ops[i])))
            .unwrap_or(0);
        out.push(rest.remove(at));
    }
    out
}

/// The text an operation is identified by when no authority or date
/// orders it.
fn sort_key(op: &GuardOp) -> (GuardOpKind, String, String, Option<String>, String, String) {
    (
        op.kind,
        op.by.clone(),
        op.effective.from.clone(),
        op.effective.to.clone(),
        cite_text(&op.cite).to_owned(),
        text(&op.condition),
    )
}

/// The cite's text (a hole's reason, or empty).
#[must_use]
pub fn cite_text(cite: &Authored<String>) -> &str {
    match cite {
        Authored::Known(s) => s,
        Authored::Hole(reason) => reason.as_deref().unwrap_or(""),
    }
}

/// Whether two operations give the same guard in either order: both
/// narrow or both widen.
fn commute(a: &GuardOp, b: &GuardOp) -> bool {
    a.kind == b.kind && a.kind != GuardOpKind::Replace
}

/// Whether the effective intervals of `a` and `b` share a day.
fn overlap(a: &GuardOp, b: &GuardOp) -> bool {
    let ends_after = |end: &Option<String>, from: &str| end.as_deref().is_none_or(|e| e > from);
    ends_after(&a.effective.to, &b.effective.from) && ends_after(&b.effective.to, &a.effective.from)
}

/// Pairs `(i, j)`, `i < j`, of operations in force together that no
/// authority or date orders and whose order changes the guard (L3003).
#[must_use]
pub fn ambiguous(lattice: &Lattice, ops: &[&GuardOp]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (j, b) in ops.iter().enumerate() {
        for (i, a) in ops[..j].iter().enumerate() {
            let ordered = before(lattice, a, b) || before(lattice, b, a);
            if !ordered && !commute(a, b) && overlap(a, b) {
                out.push((i, j));
            }
        }
    }
    out
}

/// Whether `op` is in force on `date` at corpus date `known_at` (every
/// recorded operation when `None`), with a verified instrument: an
/// operation whose `cite` is a hole does not apply.
#[must_use]
pub fn in_force(op: &GuardOp, date: &str, known_at: Option<&str>) -> bool {
    let e = &op.effective;
    matches!(op.cite, Authored::Known(_))
        && e.from.as_str() <= date
        && e.to.as_deref().is_none_or(|to| date < to)
        && known_at.is_none_or(|k| e.known_at.as_str() <= k)
}

/// One guard operation as the provenance trace records it: what it did,
/// on whose authority, in force when.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Applied {
    /// `narrow`, `widen` or `replace`.
    pub op: GuardOpKind,
    /// The condition, as text in the base pack's view.
    pub condition: String,
    /// The instrument's authority level.
    pub by: String,
    /// The instrument (a hole's reason when unverified).
    pub cite: String,
    /// First day in force.
    pub from: String,
    /// First day no longer in force; `None` when open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// When the instrument became known.
    pub known_at: String,
}

impl Applied {
    /// The record of `op`.
    #[must_use]
    pub fn of(op: &GuardOp) -> Self {
        Self {
            op: op.kind,
            condition: text(&op.condition),
            by: op.by.clone(),
            cite: cite_text(&op.cite).to_owned(),
            from: op.effective.from.clone(),
            to: op.effective.to.clone(),
            known_at: op.effective.known_at.clone(),
        }
    }
}

/// A guard and the operations that made it.
#[derive(Debug, Clone, PartialEq)]
pub struct Composed {
    /// The guard in force; `None` when the base has none and no operation
    /// applied.
    pub guard: Option<Expr>,
    /// The operations applied, in canonical order (indices into the ops
    /// passed to [`compose`]).
    pub applied: Vec<usize>,
}

/// The guard in force on `date` at `known_at`: the operations of `ops` in
/// force then, in canonical [`order`], folded over `base` (read as `true`
/// when absent): narrow is `g && C`, widen `g || C`, replace `C`.
#[must_use]
pub fn compose(
    lattice: &Lattice,
    base: Option<&Expr>,
    ops: &[&GuardOp],
    date: &str,
    known_at: Option<&str>,
) -> Composed {
    let applied: Vec<usize> = order(lattice, ops)
        .into_iter()
        .filter(|&i| in_force(ops[i], date, known_at))
        .collect();
    if applied.is_empty() {
        return Composed {
            guard: base.cloned(),
            applied,
        };
    }
    let mut g = base.cloned().unwrap_or_else(|| boolean(true));
    for &i in &applied {
        let c = ops[i].condition.clone();
        g = match ops[i].kind {
            GuardOpKind::Narrow => binary(BinaryOp::And, g, c),
            GuardOpKind::Widen => binary(BinaryOp::Or, g, c),
            GuardOpKind::Replace => c,
        };
    }
    Composed {
        guard: Some(g),
        applied,
    }
}

fn boolean(b: bool) -> Expr {
    Expr {
        ty: Type::Boolean,
        kind: ExprKind::Literal {
            value: Value::Boolean(b),
        },
        span: None,
    }
}

fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        ty: Type::Boolean,
        kind: ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        span: None,
    }
}

/// `name` (qualified as sema writes it in package `from`) as package
/// `base` spells it: `base::x` is `x`, a name local to `from` is
/// `from::x`, and any other qualified name is unchanged.
fn respell_name(name: &str, from: &str, base: &str) -> String {
    match name.split_once("::") {
        Some((p, local)) if p == base => local.to_owned(),
        Some(_) => name.to_owned(),
        None if from.is_empty() || from == base || is_builtin(name) => name.to_owned(),
        None => format!("{from}::{name}"),
    }
}

/// The guard builtins, spelled the same in every package.
fn is_builtin(name: &str) -> bool {
    matches!(name, "has_event" | "date_of" | "deadline")
}

/// A guard written in package `from` as package `base` reads it, so its
/// atoms compare with the base edge's guards (replay and L2005).
#[must_use]
pub fn respell(e: &Expr, from: &str, base: &str) -> Expr {
    let kind = match &e.kind {
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: respell_name(callee, from, base),
            args: args.iter().map(|a| respell(a, from, base)).collect(),
        },
        ExprKind::Ref { target } => ExprKind::Ref {
            target: respell_name(target, from, base),
        },
        ExprKind::Literal {
            value: Value::Variant(v),
        } => ExprKind::Literal {
            value: Value::Variant(respell_name(v, from, base)),
        },
        ExprKind::Unary { op, operand } => ExprKind::Unary {
            op: *op,
            operand: Box::new(respell(operand, from, base)),
        },
        ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
            op: *op,
            lhs: Box::new(respell(lhs, from, base)),
            rhs: Box::new(respell(rhs, from, base)),
        },
        other => other.clone(),
    };
    let ty = match &e.ty {
        Type::Named(n) => Type::Named(respell_name(n, from, base)),
        other => other.clone(),
    };
    Expr {
        ty,
        kind,
        span: e.span.clone(),
    }
}

/// A guard as text (`has_event(Served()) && (date_of(Filed()) <
/// 2026-01-01)`), for diagnostics and the provenance trace: every binary
/// operand that is itself binary is parenthesised, and atoms read as a
/// matter's events do ([`crate::case::display_event`]).
#[must_use]
pub fn text(e: &Expr) -> String {
    fn operand(e: &Expr) -> String {
        match &e.kind {
            ExprKind::Binary { .. } => format!("({})", text(e)),
            _ => text(e),
        }
    }
    match &e.kind {
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: x,
        } => format!("!{}", operand(x)),
        ExprKind::Binary { op, lhs, rhs } => format!(
            "{} {} {}",
            operand(lhs),
            crate::types::ops::symbol(*op),
            operand(rhs)
        ),
        ExprKind::Call { callee, args } => format!(
            "{callee}({})",
            args.iter().map(text).collect::<Vec<_>>().join(", ")
        ),
        _ => crate::case::display_event(e),
    }
}

/// The base pack id of instance name `base@origin`.
fn base_of(instance: &str) -> &str {
    instance.split('@').next().unwrap_or(instance)
}

/// Every declaration of node versions in pack `pack` that have an edge
/// `edge`, with the world index of each.
fn nodes_with_edge<'w>(world: &World<'w>, pack: &str, edge: &str) -> Vec<(usize, &'w Node)> {
    world
        .decls
        .iter()
        .enumerate()
        .filter(|(_, d)| world.packs[d.pack].pack == pack)
        .filter_map(|(i, d)| match d.decl {
            Declaration::Node(n) if n.edges.iter().any(|e| e.id.as_deref() == Some(edge)) => {
                Some((i, n))
            }
            _ => None,
        })
        .collect()
}

/// The span of `op` (a placeholder naming its instance for HIR without
/// source locations).
fn at(op: &GuardOp, instance: &Instance) -> Span {
    op.span
        .clone()
        .unwrap_or_else(|| crate::temporal::world::nowhere(&instance.name))
}

/// The SPEC §3.6 rules for the guard operations of pack `pack`'s
/// instances, over the base packages `world` loads (a base that is not
/// loaded, or an edge it lacks, is reported where the patch lowers).
#[must_use]
pub fn check(world: &World<'_>, pack: usize) -> Vec<Diagnostic> {
    let lattice = world.lattice();
    let mut out = Vec::new();
    for instance in world.instances_of(pack) {
        let base = base_of(&instance.pack);
        for (edge, patch) in &instance.patch_edges {
            if patch.guard_ops.is_empty() {
                continue;
            }
            let nodes = nodes_with_edge(world, base, edge);
            let ops: Vec<&GuardOp> = patch.guard_ops.iter().collect();
            placement(instance, edge, &nodes, &ops, &mut out);
            capability(world, lattice, instance, edge, &nodes, &ops, &mut out);
            for (i, j) in ambiguous(lattice, &ops) {
                out.push(
                    error(
                        C::UnresolvedPrecedence,
                        at(ops[j], instance),
                        format!(
                            "`{}` and `{}` on `{edge}` are in force together and neither \
                             authority nor effective date orders them",
                            ops[i].kind.keyword(),
                            ops[j].kind.keyword(),
                        ),
                    )
                    .with_note(
                        ops[i].span.clone(),
                        "guard operations apply in the order their authorities and dates fix \
                         (SPEC §3.6); give them comparable authorities or distinct dates",
                    ),
                );
            }
        }
        disjoint(world, lattice, instance, &mut out);
    }
    out
}

/// L2007: a guard operation only on an edge that may carry `when` (a
/// named edge of a `fact` node, SPEC §5.1).
fn placement(
    instance: &Instance,
    edge: &str,
    nodes: &[(usize, &Node)],
    ops: &[&GuardOp],
    out: &mut Vec<Diagnostic>,
) {
    let Some((_, node)) = nodes
        .iter()
        .find(|(_, n)| !n.tags.iter().any(|t| t == "fact"))
    else {
        return;
    };
    for op in ops {
        out.push(error(
            C::InvalidField,
            at(op, instance),
            format!(
                "`{}` is not allowed on `{edge}`: node `{}` of `{}` is not a `fact` chance node, \
                 so its edges carry no `when` guard",
                op.kind.keyword(),
                node.meta.id,
                instance.pack
            ),
        ));
    }
}

/// L3004: an operation's authority equals or outranks the base node's,
/// or the base delegates to it (`delegable_to`), on every version in
/// force during the operation.
fn capability(
    world: &World<'_>,
    lattice: &Lattice,
    instance: &Instance,
    edge: &str,
    nodes: &[(usize, &Node)],
    ops: &[&GuardOp],
    out: &mut Vec<Diagnostic>,
) {
    for op in ops {
        let unlicensed = nodes.iter().find(|&&(i, _)| {
            let meta = world.decls[i].meta;
            let window = &meta.effective;
            let overlaps = window.to.as_deref().is_none_or(|to| to > op.effective.from.as_str())
                && op.effective.to.as_deref().is_none_or(|to| to > window.from.as_str());
            let licensed = matches!(
                lattice.compare(&op.by, &meta.authority),
                Order::Above | Order::Equal
            ) || meta
                .relations
                .iter()
                .any(|r| matches!(r, Relation::DelegableTo(x) if *x == op.by));
            overlaps && !licensed
        });
        if let Some(&(i, node)) = unlicensed {
            out.push(
                error(
                    C::MissingDelegation,
                    at(op, instance),
                    format!(
                        "`{}` by `{}` cannot change the guard of `{edge}`: node `{}` is `{}` \
                         and does not delegate to `{}`",
                        op.kind.keyword(),
                        op.by,
                        node.meta.id,
                        world.decls[i].meta.authority,
                        op.by
                    ),
                )
                .with_note(
                    world.decls[i].meta.span.clone(),
                    format!(
                        "a guard operation displaces part of its target, so it needs \
                         `delegable_to [{}]` here or an authority at least `{}` (SPEC §3.6)",
                        op.by, world.decls[i].meta.authority
                    ),
                ),
            );
        }
    }
}

/// L2005: on every date an operation takes effect, the composed guards of
/// each patched fact node stay pairwise disjoint.
fn disjoint(world: &World<'_>, lattice: &Lattice, instance: &Instance, out: &mut Vec<Diagnostic>) {
    let base = base_of(&instance.pack);
    let from = world
        .instances
        .iter()
        .find(|(_, i)| std::ptr::eq(*i, instance))
        .map_or("", |(p, _)| world.packs[*p].pack.as_str());
    let mut seen: Vec<(String, String)> = Vec::new();
    for (edge, patch) in instance.patch_edges.iter().filter(|(_, p)| !p.guard_ops.is_empty()) {
        for (_, node) in nodes_with_edge(world, base, edge) {
            if !node.tags.iter().any(|t| t == "fact") {
                continue;
            }
            for op in &patch.guard_ops {
                let date = op.effective.from.as_str();
                let guards: Vec<(&str, Expr)> = node
                    .edges
                    .iter()
                    .filter_map(|e| {
                        let id = e.id.as_deref()?;
                        let ops: Vec<GuardOp> = instance
                            .patch_edges
                            .get(id)
                            .map(|p| p.guard_ops.iter().map(|o| respelled(o, from, base)).collect())
                            .unwrap_or_default();
                        let refs: Vec<&GuardOp> = ops.iter().collect();
                        compose(lattice, e.guard.as_ref(), &refs, date, None)
                            .guard
                            .map(|g| (id, g))
                    })
                    .collect();
                for (j, (b, gb)) in guards.iter().enumerate() {
                    for (a, ga) in &guards[..j] {
                        if !(a == edge || b == edge) {
                            continue;
                        }
                        let pair = ((*a).to_owned(), (*b).to_owned());
                        if seen.contains(&pair)
                            || condition::jointly_satisfiable(&[ga, gb]) == Some(false)
                        {
                            continue;
                        }
                        seen.push(pair);
                        out.push(
                            error(
                                C::NonExhaustiveMatch,
                                at(op, instance),
                                format!(
                                    "from {date} the guards of `{a}` and `{b}` on fact `{}` can \
                                     both hold",
                                    node.meta.id
                                ),
                            )
                            .with_note(
                                None,
                                format!(
                                    "composed: `{}` and `{}`; a fact has one value, so a changed \
                                     guard stays exclusive of its siblings (SPEC §3.6)",
                                    text(ga),
                                    text(gb)
                                ),
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// `op` with its condition read in base pack `base`'s view.
#[must_use]
pub fn respelled(op: &GuardOp, from: &str, base: &str) -> GuardOp {
    GuardOp {
        condition: respell(&op.condition, from, base),
        ..op.clone()
    }
}
