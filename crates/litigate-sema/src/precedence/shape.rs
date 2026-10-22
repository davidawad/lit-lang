//! Rule kinds (Hart; SPEC §5.5) and direction-dependent counting typing
//! (DESIGN §4.5).
//!
//! - `rule duty` states what must be done in time: it contains a deadline.
//! - `rule power` grants a choice: it contains a `decision` node.
//! - `rule liability` is the consequence of a missed duty: it contains the
//!   consequence nodes and no deadline of its own.
//!
//! A deadline's direction is part of its type, `Deadline<Forward>` or
//! `Deadline<Backward>`: every version shares it, only a forward deadline
//! can bound an edge (`within`), and a backward one cannot use a counting
//! rule anchored on the next business day.
use crate::hir::{
    Anchor, Deadline, DeadlineWindow, Declaration, Direction, Node, NodeKind, Rule, RuleKind,
};
use crate::temporal::world::{error, span_of, Decl, DiagnosticExt, Key, Locate, World};
use litigate_base::{Diagnostic, DiagnosticCode as C};

/// The kind of the `rule` enclosing declaration `i`, if any.
#[must_use]
pub fn rule_kind(world: &World<'_>, i: usize) -> Option<RuleKind> {
    world.decls[i].rule.map(|(_, k)| k)
}

/// True when deadline `key` is declared inside a `rule duty` (a mandatory
/// deadline for `deadline-satisfiable` and case replay L4002).
#[must_use]
pub fn is_duty(world: &World<'_>, key: &Key) -> bool {
    world.versions(key).versions.iter().any(|v| {
        matches!(world.decls[v.decl].decl, Declaration::Deadline(_))
            && rule_kind(world, v.decl) == Some(RuleKind::Duty)
    })
}

fn word(k: RuleKind) -> &'static str {
    match k {
        RuleKind::Duty => "duty",
        RuleKind::Power => "power",
        RuleKind::Liability => "liability",
    }
}

/// L2007 for ill-shaped rules, L2001/L3009 for direction errors, in pack
/// `pack`.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (i, d) in world
        .decls
        .iter()
        .enumerate()
        .filter(|(_, d)| d.pack == pack)
    {
        match d.decl {
            Declaration::Rule(r) => out.extend(rule_shape(world, loc, d, r)),
            Declaration::Node(n) => out.extend(windows(world, loc, i, d, n)),
            Declaration::Deadline(x) if x.direction == Direction::Backward => {
                out.extend(backward_anchor(world, loc, i, d, x));
            }
            _ => {}
        }
    }
    out.extend(directions(world, loc, pack));
    out
}

/// A rule contains what its kind requires.
fn rule_shape(world: &World<'_>, loc: &dyn Locate, d: &Decl<'_>, r: &Rule) -> Option<Diagnostic> {
    let inner: Vec<&Declaration> = world
        .decls
        .iter()
        .filter(|x| x.pack == d.pack && x.rule.is_some_and(|(id, _)| id == r.meta.id))
        .map(|x| x.decl)
        .collect();
    let deadline = inner.iter().any(|x| matches!(x, Declaration::Deadline(_)));
    let node = |want: fn(&NodeKind) -> bool| {
        inner
            .iter()
            .any(|x| matches!(x, Declaration::Node(n) if want(&n.kind)))
    };
    let message = match r.rule_kind {
        RuleKind::Duty if !deadline => "a `duty` rule must contain the deadline it imposes",
        RuleKind::Power if !node(|k| matches!(k, NodeKind::Decision)) => "a `power` rule must contain the `decision` it grants",
        RuleKind::Liability if !node(|_| true) => "a `liability` rule must contain its consequence nodes",
        RuleKind::Liability if deadline => {
            "a `liability` is a consequence and imposes no deadline; move the deadline into a `duty`"
        }
        _ => return None,
    };
    Some(
        error(C::InvalidField, span_of(loc, d, ""), message).with_note(
            None,
            format!("rule `{}` is a `{}`", r.meta.id, word(r.rule_kind)),
        ),
    )
}

/// `within d` needs a forward `d`.
fn windows(
    world: &World<'_>,
    loc: &dyn Locate,
    i: usize,
    d: &Decl<'_>,
    n: &Node,
) -> Vec<Diagnostic> {
    n.edges
        .iter()
        .filter_map(|e| {
            let Some(DeadlineWindow::Declared { deadline }) = e.deadline.as_ref().map(|x| &x.window) else {
                return None;
            };
            let key = world.target(world.pack_of(i), deadline).key;
            let backward = world.versions(&key).versions.iter().any(|v| {
                matches!(world.decls[v.decl].decl, Declaration::Deadline(x) if x.direction == Direction::Backward)
            });
            backward.then(|| {
                error(
                    C::TypeMismatch,
                    e.span.clone().unwrap_or_else(|| span_of(loc, d, "")),
                    format!("`within {deadline}` needs a forward deadline, but `{deadline}` counts backward (`before`)"),
                )
                .with_note(None, "a deadline's direction is part of its type: an edge window runs forward from the trigger")
            })
        })
        .collect()
}

/// A backward deadline cannot anchor on the next business day.
fn backward_anchor(
    world: &World<'_>,
    loc: &dyn Locate,
    i: usize,
    d: &Decl<'_>,
    x: &Deadline,
) -> Option<Diagnostic> {
    let counting = world.target(world.pack_of(i), &x.counting);
    let window = crate::temporal::interval::Window::of(&d.meta.effective);
    let anchored = world.versions(&counting.key).versions.iter().any(|v| {
        v.in_force.overlaps(&window)
            && matches!(world.decls[v.decl].decl, Declaration::Counting(c) if c.anchor == Anchor::NextBusinessDay)
    });
    anchored.then(|| {
        error(
            C::InvalidCounting,
            span_of(loc, d, "counted_by"),
            format!(
                "`{}` counts backward, but `{}` anchors on the next business day, which only counts forward",
                d.meta.id, x.counting
            ),
        )
        .with_note(None, "count backward deadlines with a rule anchored on the trigger day")
    })
}

/// Versions of one deadline must share its direction.
fn directions(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (_, set) in world.sets() {
        let dirs: Vec<(usize, Direction)> = set
            .versions
            .iter()
            .filter(|v| world.decls[v.decl].pack == pack)
            .filter_map(|v| match world.decls[v.decl].decl {
                Declaration::Deadline(x) => Some((v.decl, x.direction)),
                _ => None,
            })
            .collect();
        if let Some(&(first, dir)) = dirs.first() {
            for &(other, d) in dirs.iter().skip(1).filter(|(_, d)| *d != dir) {
                let say = |d: Direction| {
                    if d == Direction::Forward {
                        "forward"
                    } else {
                        "backward"
                    }
                };
                out.push(
                    error(
                        C::TypeMismatch,
                        span_of(loc, &world.decls[other], ""),
                        format!(
                            "version `{}` counts {}, but `{}` counts {}",
                            world.versioned(other, other),
                            say(d),
                            world.versioned(first, other),
                            say(dir)
                        ),
                    )
                    .with_note(
                        world.decls[first].meta.span.clone(),
                        "versions of a deadline share its type, direction included",
                    ),
                );
            }
        }
    }
    out
}
