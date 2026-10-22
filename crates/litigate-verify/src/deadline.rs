//! `deadline-satisfiable` (SPEC §7): at every reachable node, the due-date
//! windows of the mandatory (`rule duty`) deadlines on its outgoing edges
//! must intersect for some trigger date.
//!
//! A window exists only on days a version of its deadline is in force, so
//! the node and every duty bounding one of its edges need versions sharing
//! an in-force day. Within one such choice, deadlines with different
//! trigger patterns can be triggered on any dates, so only deadlines with
//! one exact trigger pattern constrain each other: a forward period closes
//! on or after its trigger day `t` (its window holds `t`), and a backward
//! period of `N > 0` units closes strictly before `t`. Sema types every
//! edge window forward, so that clash is a guard for linked or future
//! inputs, not a source-reachable case.
use crate::diag::{emit, note};
use crate::Context;
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_sema::hir::{self, Deadline, DeadlineWindow, Direction};
use std::collections::BTreeSet;

/// Most version combinations searched per node.
const MAX_CHOICES: usize = 4096;

/// L5005 for every reachable node whose duty deadlines cannot all be met.
pub(crate) fn unsatisfiable(cx: &Context<'_>, reached: &BTreeSet<String>) -> Vec<Diagnostic> {
    cx.nodes()
        .filter(|n| reached.contains(&n.meta.id))
        .filter_map(|n| check_node(cx, n))
        .collect()
}

/// A day range `[from, to)`; `None` is open-ended.
type Days<'a> = (&'a str, Option<&'a str>);

fn check_node(cx: &Context<'_>, node: &hir::Node) -> Option<Diagnostic> {
    let ids: BTreeSet<&str> = node
        .edges
        .iter()
        .filter_map(|e| match &e.deadline.as_ref()?.window {
            DeadlineWindow::Declared { deadline } => Some(deadline.as_str()),
            DeadlineWindow::Inline { .. } => None,
        })
        .filter(|id| cx.is_duty(id))
        .collect();
    let versions: Vec<Vec<&Deadline>> = ids.iter().map(|id| cx.deadline_versions(id)).collect();
    if versions.is_empty() || versions.iter().any(Vec::is_empty) {
        return None;
    }
    let start = days(&node.meta.effective);
    let mut clash = None;
    let mut budget = MAX_CHOICES;
    if search(&versions, start, &mut Vec::new(), &mut clash, &mut budget) {
        return None;
    }
    let span = cx.span_or_manifest(node.meta.span.as_ref());
    Some(match clash {
        Some((forward, backward)) => direction_clash(cx, node, span, forward, backward),
        None => never_together(cx, node, span, &versions),
    })
}

/// Whether some choice of one version per deadline shares a day with
/// `common` and has no direction clash. Records the last clash seen. An
/// exhausted search budget accepts: it bounds work, it is not a finding.
fn search<'a>(
    rest: &[Vec<&'a Deadline>],
    common: Days<'a>,
    chosen: &mut Vec<&'a Deadline>,
    clash: &mut Option<(&'a Deadline, &'a Deadline)>,
    budget: &mut usize,
) -> bool {
    let Some((first, rest)) = rest.split_first() else {
        return match directions(chosen) {
            None => true,
            found => {
                *clash = found;
                false
            }
        };
    };
    for version in first {
        if *budget == 0 {
            return true;
        }
        *budget -= 1;
        let Some(next) = intersect(common, days(&version.meta.effective)) else {
            continue;
        };
        chosen.push(version);
        let found = search(rest, next, chosen, clash, budget);
        chosen.pop();
        if found {
            return true;
        }
    }
    false
}

fn directions<'a>(chosen: &[&'a Deadline]) -> Option<(&'a Deadline, &'a Deadline)> {
    chosen.iter().find_map(|f| {
        let b = chosen.iter().find(|b| {
            f.direction == Direction::Forward
                && b.direction == Direction::Backward
                && b.amount > 0
                && crate::same_expr(&f.trigger, &b.trigger)
        })?;
        Some((*f, *b))
    })
}

fn days(i: &hir::Interval) -> Days<'_> {
    (i.from.as_str(), i.to.as_deref())
}

/// The days two ranges share, if any.
fn intersect<'a>(a: Days<'a>, b: Days<'a>) -> Option<Days<'a>> {
    let from = a.0.max(b.0);
    let to = match (a.1, b.1) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, y) => x.or(y),
    };
    to.is_none_or(|to| from < to).then_some((from, to))
}

fn never_together(
    cx: &Context<'_>,
    node: &hir::Node,
    span: litigate_base::Span,
    versions: &[Vec<&Deadline>],
) -> Diagnostic {
    let names: Vec<String> = versions
        .iter()
        .filter_map(|v| v.first().map(|d| format!("`{}`", d.meta.id)))
        .collect();
    let d = emit(
        C::DeadlineUnsatisfiable,
        cx.profile,
        span,
        format!(
            "at `{}`, duties {} are never in force on the same day as the node, so no trigger \
             date opens all their windows",
            node.meta.id,
            names.join(", ")
        ),
    );
    versions.iter().flatten().fold(d, |d, x| {
        let i = &x.meta.effective;
        note(
            d,
            x.meta.span.as_ref(),
            format!(
                "`{}` is in force [{}, {})",
                x.meta.id,
                i.from,
                i.to.as_deref().unwrap_or("..")
            ),
        )
    })
}

fn direction_clash(
    cx: &Context<'_>,
    node: &hir::Node,
    span: litigate_base::Span,
    forward: &Deadline,
    backward: &Deadline,
) -> Diagnostic {
    let d = emit(
        C::DeadlineUnsatisfiable,
        cx.profile,
        span,
        format!(
            "at `{}`, duty `{}` closes on or after its trigger but duty `{}` closes {} {} before \
             the same trigger, so no date meets both",
            node.meta.id,
            forward.meta.id,
            backward.meta.id,
            backward.amount,
            unit(backward.unit),
        ),
    );
    let d = note(
        d,
        forward.meta.span.as_ref(),
        format!("`{}` counts forward", forward.meta.id),
    );
    note(
        d,
        backward.meta.span.as_ref(),
        format!(
            "`{}` counts backward from the same trigger; give it a different trigger",
            backward.meta.id
        ),
    )
}

fn unit(u: hir::DayUnit) -> &'static str {
    match u {
        hir::DayUnit::Calendar => "days",
        hir::DayUnit::Court => "court days",
        hir::DayUnit::Business => "business days",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_open_ranges_share_a_day_only_when_they_overlap() {
        let early = ("2000-01-01", Some("2010-01-01"));
        let late = ("2010-01-01", None);
        let open = ("1990-01-01", None);
        assert_eq!(intersect(early, late), None);
        assert_eq!(intersect(late, early), None);
        assert_eq!(intersect(early, open), Some(early));
        assert_eq!(intersect(late, open), Some(late));
        assert_eq!(intersect(open, open), Some(open));
    }
}
