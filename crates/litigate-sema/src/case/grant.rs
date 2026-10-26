//! Judging an extension or a waiver against one grant (SPEC §5.3
//! `extend`): its window, the motion it reads, and its bound. Replay judges
//! a matter's `extends` and `waives` events with it, and a package's inline
//! tests their `given` orders ([`extensions`]), so a test extends a period
//! exactly as a matter would.
use crate::case::graph::display;
use crate::hir::{
    self, Bound, BoundDates, Capability, CauseStandard, Deadline, Declaration, EventEffect, Expr,
    ExprKind, Extension, ExtensionWindow, Motion, Value,
};
use crate::temporal::condition::matches;
use crate::temporal::{Key, World};
use std::fmt::Write as _;

/// What licenses a transform, read off one `extend` grant or the `waive`
/// capability of a deadline: who holds it, whether it reaches an instance
/// before and after its due date, its date bound, if any, and the motion
/// it requires, if any.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Grant<'g> {
    pub by: &'g str,
    before: bool,
    after: bool,
    /// The latest date an extension may reach; `None` under the court's
    /// discretion (and for a waiver, which has no extent).
    ceiling: Option<&'g Bound>,
    /// The motion the grant requires.
    pub motion: Option<&'g Motion>,
    /// When the motion must be made.
    moved: Moved,
    /// The cause standard the grant applies, if it states one.
    pub standard: Option<CauseStandard>,
    /// The interpretation the grant rests on, if any.
    pub interpretation: Option<&'g str>,
}

impl<'g> Grant<'g> {
    /// An `extend` grant; `None` for an `extend { never ... }` lock.
    pub fn extend(x: &'g Extension) -> Option<Self> {
        let interpretation = x.interpretation();
        match x {
            Extension::Bounded {
                by,
                until,
                after_expiry,
                motion,
                standard,
                ..
            } => Some(Self {
                by,
                before: true,
                after: *after_expiry,
                ceiling: Some(until),
                motion: motion.as_ref(),
                moved: Moved::Any,
                standard: *standard,
                interpretation,
            }),
            Extension::Discretionary {
                by,
                window,
                motion,
                standard,
                ..
            } => {
                let after = *window == ExtensionWindow::AfterExpiryOnMotion;
                Some(Self {
                    by,
                    before: !after,
                    after,
                    ceiling: None,
                    motion: motion.as_ref(),
                    moved: if after {
                        Moved::AfterExpiry
                    } else if motion.is_some() {
                        Moved::BeforeExpiry
                    } else {
                        Moved::Any
                    },
                    standard: Some(*standard),
                    interpretation,
                })
            }
            Extension::NonExtendable { .. } => None,
        }
    }

    /// The `waive` capability.
    pub fn waive(c: &'g Capability) -> Self {
        Self {
            by: &c.by,
            before: true,
            after: c.after_expiry,
            ceiling: None,
            motion: None,
            moved: Moved::Any,
            standard: None,
            interpretation: None,
        }
    }

    /// Whether it may be exercised on an instance that has (`expired`) or
    /// has not passed its due date.
    fn reaches(self, expired: bool) -> bool {
        if expired {
            self.after
        } else {
            self.before
        }
    }
}

/// When a grant's motion must be made, relative to the instance's due
/// date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Moved {
    /// At any time after the trigger.
    Any,
    /// After the period expired (FRCP 6(b)(1)(B)).
    AfterExpiry,
    /// On or before the due date: the grant is on request, and its window
    /// is judged on `request_made`, the motion's date, not on the order's,
    /// so a request made before expiry licenses an order entered after it
    /// (FRCP 6(b)(1)(A), "if a request is made, before the original time
    /// or its extension expires"; 37 C.F.R. §§ 90.3(c)(1)(i),
    /// 2.145(e)(1)(i)).
    BeforeExpiry,
}

impl Moved {
    /// The rule for an instance due `due`.
    fn at(self, due: &str) -> Made<'_> {
        match self {
            Self::Any => Made::Any,
            Self::AfterExpiry => Made::After(due),
            Self::BeforeExpiry => Made::Before(due),
        }
    }
}

/// [`Moved`], with the instance's due date.
#[derive(Debug, Clone, Copy)]
enum Made<'a> {
    /// At any time after the trigger.
    Any,
    /// After the period expired on the date (FRCP 6(b)(1)(B)).
    After(&'a str),
    /// On or before the date the period expires (FRCP 6(b)(1)(A)).
    Before(&'a str),
}

/// Why an event `action` on `date` is outside a grant's window for
/// `deadline`, due `current`: the period has (`expired`) or has not
/// passed.
fn outside(deadline: &str, action: &str, current: &str, date: &str, expired: bool) -> String {
    if expired {
        format!(
            "`{deadline}` expired on {current}, before `{action}` on {date}; the capability does not reach an expired period (no `after_expiry`)"
        )
    } else {
        format!(
            "`{deadline}` is due {current}, not expired when `{action}` comes on {date}; the grant reaches only an expired period, on motion"
        )
    }
}

/// One obligation instance of `deadline` as an order finds it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Instance<'a> {
    /// The deadline's qualified id.
    pub deadline: &'a str,
    /// The trigger's date.
    pub from: &'a str,
    /// The counted due date (`prescribed_due`).
    pub prescribed: &'a str,
    /// That date as last extended (`current_due`).
    pub current: &'a str,
    /// The date of the motion an earlier extension read, spent by it.
    pub spent: Option<&'a str>,
}

/// One event of a matter's history: its date, its action as written, and
/// that action in the grant's package view, which motions are matched in.
pub(crate) type Moment<'a> = (&'a str, &'a Expr, &'a Expr);

/// Whether order `history[order]` may transform `instance` under `grant`:
/// extend it to `new`, or waive it when `new` is `None`. `history` is the
/// matter's events in docket order. `Ok` carries the date of the motion the grant
/// read, if it reads one; `Err` the refusal, ranked by how specific it is:
/// the window (0), the motion (1), the bound (2).
pub(crate) fn judge(
    history: &[Moment<'_>],
    order: usize,
    instance: Instance<'_>,
    grant: Grant<'_>,
    new: Option<&str>,
) -> Result<Option<String>, (u8, String)> {
    let (date, action, _) = history[order];
    let Instance {
        deadline,
        prescribed,
        current,
        ..
    } = instance;
    let expired = date > current;
    if grant.moved != Moved::BeforeExpiry && !grant.reaches(expired) {
        return Err((
            0,
            outside(deadline, &display(action), current, date, expired),
        ));
    }
    let made = grant.moved.at(current);
    let request = match grant.motion {
        Some(m) => Some(requested(history, order, instance, m, made).map_err(|why| (1, why))?),
        None => None,
    };
    let Some(new) = new else {
        return Ok(request);
    };
    if new <= current {
        return Err((
            2,
            format!("an extension to {new} does not extend `{deadline}`, already due {current}"),
        ));
    }
    let Some(ceiling) = grant.ceiling else {
        return Ok(request);
    };
    let bound = ceiling
        .date(BoundDates {
            prescribed_due: prescribed,
            current_due: current,
            grant_order_entered: date,
        })
        .unwrap_or_default();
    if new > bound.as_str() {
        return Err((
            2,
            format!(
                "an extension to {new} exceeds the bound of `{deadline}`: {} = {bound} (prescribed_due {prescribed}, grant_order_entered {date})",
                ceiling.display(),
            ),
        ));
    }
    Ok(request)
}

/// The date of the motion `m` order `history[order]`'s grant requires for
/// `instance` (`made`: when the motion must be made relative to its due
/// date): the earliest event of `m`'s pattern on or after the trigger,
/// after any motion an earlier extension of the instance read (a motion is
/// spent by the extension it licenses), no later than the order and no
/// later than `m`'s bound, with no denial of `m`'s `unless` pattern from
/// the trigger to the order (Sup. Ct. R. 30.3); or why there is none. The
/// motion's date is its event's date as recorded: a request to the USPTO
/// Director is filed on its Eastern Time receipt date, "regardless of
/// whether that date is a Saturday, Sunday, or Federal holiday" (37 C.F.R.
/// §§ 90.3(c)(2), 2.145(e)(2)), and no counting rule moves it.
fn requested(
    history: &[Moment<'_>],
    order: usize,
    instance: Instance<'_>,
    m: &Motion,
    made: Made<'_>,
) -> Result<String, String> {
    let (entered, action, _) = history[order];
    let Instance {
        from,
        prescribed,
        current,
        spent,
        ..
    } = instance;
    let latest = m.within.as_ref().and_then(|w| {
        w.date(BoundDates {
            prescribed_due: prescribed,
            current_due: current,
            grant_order_entered: entered,
        })
    });
    let between = |j: usize| {
        let date = history[j].0;
        j != order && date >= from && date <= entered
    };
    let moved = (0..history.len()).find(|&j| {
        let date = history[j].0;
        between(j)
            && match made {
                Made::Any => true,
                Made::After(a) => date > a,
                Made::Before(a) => date <= a,
            }
            && latest.as_deref().is_none_or(|l| date <= l)
            && spent.is_none_or(|s| date > s)
            && matches(&m.event, history[j].2)
    });
    let denied = m.unless.as_ref().and_then(|u| {
        (0..history.len())
            .find(|&j| between(j) && matches(u, history[j].2))
            .map(|j| (u, history[j]))
    });
    if let Some((u, (date, denial, _))) = denied {
        return Err(format!(
            "`{}` on {date} bars `{}`: once denied, the motion `{}` may not be renewed (`unless {}`)",
            display(denial),
            display(action),
            display(&m.event),
            display(u)
        ));
    }
    if let Some(j) = moved {
        return Ok(history[j].0.to_owned());
    }
    let mut why = format!(
        "`{}` requires a motion `{}`",
        display(action),
        display(&m.event)
    );
    match made {
        Made::Any => {}
        Made::After(a) => {
            let _ = write!(why, " made after the period expired on {a}");
        }
        Made::Before(a) => {
            let _ = write!(why, " made before the period expired on {a}");
        }
    }
    if let (Some(w), Some(l)) = (&m.within, &latest) {
        let _ = write!(why, " no later than {} = {l}", w.display());
    }
    if let Some(s) = spent {
        let _ = write!(
            why,
            " after the one made on {s}, spent by an earlier extension"
        );
    }
    let _ = write!(why, ", and none precedes the order on {entered}");
    Err(why)
}

/// One event of an inline test: `given action [by actor] on date`, its
/// action in the tested deadline's package view.
#[derive(Debug, Clone, Copy)]
pub struct Given<'a> {
    /// Its date.
    pub date: &'a str,
    /// The event constructor call.
    pub action: &'a Expr,
    /// The actor doing it (`by`), if the test names one.
    pub actor: Option<&'a str>,
}

/// The extensions of the instance of deadline `d` (qualified id `name`)
/// that `history[trigger]` creates, counted due `counted`: each later
/// event in `history` (in date order) whose type extends `name` is an
/// order, licensed as replay licenses it, by one of `d`'s own `extend`
/// grants that its actor holds, within the grant's window, motion and
/// bound. `extends` gives the new-date argument of an event whose type
/// extends `name`. Each licensed order, by index, with the date it moves
/// the instance to; `Err` why the first refused order is refused.
///
/// # Errors
/// An order no grant of `d` licenses: the deadline grants no extension,
/// no grant is held by the order's actor (or it names none), it extends
/// to no known date, or it is outside every held grant's window, motion
/// or bound.
pub fn extensions<'a>(
    d: &Deadline,
    name: &str,
    history: &[Given<'a>],
    trigger: usize,
    counted: &str,
    extends: impl Fn(&'a Expr) -> Option<Option<&'a Expr>>,
) -> Result<Vec<(usize, String)>, String> {
    let actions: Vec<Moment<'_>> = history
        .iter()
        .map(|g| (g.date, g.action, g.action))
        .collect();
    let grants: Vec<Grant<'_>> = d.extend.iter().filter_map(Grant::extend).collect();
    let mut applied: Vec<(usize, String)> = Vec::new();
    let mut spent: Option<String> = None;
    for (k, order) in history.iter().enumerate().skip(trigger + 1) {
        let Some(arg) = extends(order.action) else {
            continue;
        };
        let action = display(order.action);
        if grants.is_empty() {
            return Err(match d.extend.iter().find_map(Extension::lock_source) {
                Some(lock) => format!(
                    "`{action}` cannot extend `{name}`: the deadline may not be extended (`{lock}`)"
                ),
                None => format!(
                    "`{action}` cannot extend `{name}`: the deadline grants no `extend` capability"
                ),
            });
        }
        let held: Vec<Grant<'_>> = grants
            .iter()
            .copied()
            .filter(|g| Some(g.by) == order.actor)
            .collect();
        if held.is_empty() {
            let mut by: Vec<&str> = grants.iter().map(|g| g.by).collect();
            by.sort_unstable();
            by.dedup();
            let given = order.actor.map_or_else(
                || "names no actor (`given ... by <actor> on <date>`)".to_owned(),
                |a| format!("is by `{a}`"),
            );
            return Err(format!(
                "only {} may extend `{name}`, but `{action}` on {} {given}",
                by.join(" or "),
                order.date
            ));
        }
        let Some(new) = arg.and_then(|e| match &e.kind {
            ExprKind::Literal {
                value: Value::Date(new),
            } => Some(new.as_str()),
            _ => None,
        }) else {
            return Err(format!(
                "`{action}` extends `{name}` to no known date, which moves nothing"
            ));
        };
        let current = applied.last().map_or(counted, |(_, c)| c.as_str());
        let instance = Instance {
            deadline: name,
            from: history[trigger].date,
            prescribed: counted,
            current,
            spent: spent.as_deref(),
        };
        let mut refusals = Vec::new();
        let mut licensed = None;
        for &grant in &held {
            match judge(&actions, k, instance, grant, Some(new)) {
                Ok(request) => {
                    licensed = Some(request);
                    break;
                }
                Err(refusal) => refusals.push(refusal),
            }
        }
        let Some(request) = licensed else {
            let refusal = refusals.into_iter().max_by_key(|(rank, _)| *rank);
            return Err(refusal.map(|(_, why)| why).unwrap_or_default());
        };
        spent = request;
        applied.push((k, new.to_owned()));
    }
    Ok(applied)
}

/// The new-date argument of `action` (an event of package `pkg`) when its
/// type, declared in `pkg`, `extends` deadline `target` (`Some(None)` when
/// the call omits it); `None` when it extends nothing, or another deadline.
#[must_use]
pub fn extends<'e>(
    world: &World<'_>,
    pkg: &hir::Package,
    target: &Key,
    action: &'e Expr,
) -> Option<Option<&'e Expr>> {
    let ExprKind::Call { callee, args } = &action.kind else {
        return None;
    };
    let effect = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Event(e) if e.name == *callee => e.effect.as_ref(),
            _ => None,
        })?;
    match effect {
        EventEffect::Extends { deadline, param }
            if world.target(&pkg.manifest.pack_id, deadline).key == *target =>
        {
            Some(args.get(*param))
        }
        _ => None,
    }
}
