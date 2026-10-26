//! HIR edges to `RawEdge` (SPEC §8.1), shared by packs and links.
use crate::diag::Sink;
use crate::select::Selection;
use litigate_base::{DiagnosticCode as C, Span};
use litigate_engine::model::{
    CauseStandard, DateBound, DateRef, Deadline, Designated, Duration, ExtensionBound,
    ExtensionWindow, Motion, RawEdge,
};
use litigate_sema::hir::{
    self, Authored, DayUnit, DeadlineWindow, Declaration, Direction, Extension, Rational, Valence,
};
use std::collections::BTreeMap;

/// What an edge lowering needs besides the edge.
pub(crate) struct EdgeCx<'s, 'a> {
    /// Version selection for `within` references.
    pub selection: &'s Selection<'a>,
    /// Pack id references are written in.
    pub pack: &'s str,
    /// The pack's `counting` rule, if it names one: a deadline counted by
    /// another rule names its own (`Deadline.counting`).
    pub counting: Option<&'s str>,
}

/// `valence` as written in litgraph JSON.
pub(crate) fn valence(v: Valence) -> String {
    match v {
        Valence::Good => "good",
        Valence::Caution => "caution",
        Valence::Bad => "bad",
        Valence::Neutral => "neutral",
    }
    .to_owned()
}

/// Exact attributes as binary64.
pub(crate) fn attrs(
    sink: &mut Sink,
    attrs: &BTreeMap<String, Rational>,
    span: Option<&Span>,
) -> BTreeMap<String, f64> {
    attrs
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), sink.number(*v, span)?)))
        .collect()
}

/// `e`, leaving `from` to `from`.
pub(crate) fn edge(cx: &EdgeCx<'_, '_>, sink: &mut Sink, from: String, e: &hir::Edge) -> RawEdge {
    let span = e.span.as_ref();
    if !e.spawn.is_empty() || !e.joins.is_empty() {
        sink.unsupported(
            span,
            "concurrent tracks (`spawn`, `joins`) have no lowering in the litgraph schema: the engine walks one track, so a package declaring them checks and replays but does not build",
        );
    }
    let probability = match &e.probability {
        Some(Authored::Known(p)) => sink.number(*p, span),
        Some(Authored::Hole(_)) | None => None,
    };
    let cost = match &e.cost {
        Some(Authored::Known(m)) => sink.money(m, span),
        Some(Authored::Hole(_)) | None => None,
    };
    RawEdge {
        id: e.id.clone(),
        from,
        to: e.to.clone(),
        label: e.label.clone(),
        actor: e.actor.clone(),
        authority: e.authority.clone(),
        deadline: e
            .deadline
            .as_ref()
            .and_then(|d| deadline(cx, sink, d, span)),
        duration: e.duration.as_ref().and_then(|d| {
            Some(Duration {
                min: d.min.and_then(|v| sink.number(v, span)),
                mode: sink.number(d.mode, span)?,
                max: d.max.and_then(|v| sink.number(v, span)),
            })
        }),
        cost,
        hours: e.hours.and_then(|h| sink.number(h, span)),
        probability,
        valence: e.valence.map(valence),
        court_listener_url: e.court_listener_url.clone(),
        action_id: e.action_id.clone(),
        note: e.note.clone(),
        tags: e.tags.clone(),
        attrs: attrs(sink, &e.attrs, span),
        replaces: Vec::new(),
        sets: e.sets.clone(),
        clears: e.clears.clone(),
        requires: e.requires.clone(),
        forbids: e.forbids.clone(),
    }
}

/// `Deadline.unit`: absent for calendar days, `"court"` for court days,
/// `"week"`, `"month"`, `"year"`; business days have no litgraph unit
/// (L6001).
fn unit(sink: &mut Sink, unit: DayUnit, span: Option<&Span>) -> Result<Option<String>, ()> {
    match unit {
        DayUnit::Calendar => Ok(None),
        DayUnit::Court => Ok(Some("court".to_owned())),
        DayUnit::Week => Ok(Some("week".to_owned())),
        DayUnit::Month => Ok(Some("month".to_owned())),
        DayUnit::Year => Ok(Some("year".to_owned())),
        DayUnit::Business => {
            sink.unsupported(
                span,
                "a business-day edge deadline has no litgraph unit (use court days)",
            );
            Err(())
        }
    }
}

/// `within d` (from d's version in force, edge options winning) or a
/// literal `deadline N.unit`. A designated `d` (SPEC §5.3) is a
/// `designated` window, its amount the least time and never a `length`:
/// the due date is each matter's notice's. `d`'s `extend` grants are its
/// `extensions`.
fn deadline(
    cx: &EdgeCx<'_, '_>,
    sink: &mut Sink,
    d: &hir::EdgeDeadline,
    span: Option<&Span>,
) -> Option<Deadline> {
    let (length, day_unit, declared, counting) = match &d.window {
        DeadlineWindow::Inline { length, unit } => (*length, *unit, None, d.counting.clone()),
        DeadlineWindow::Declared { deadline } => {
            let at = span.cloned().unwrap_or_else(|| sink.fallback.clone());
            match cx.selection.resolve(cx.pack, deadline, at) {
                Ok(Declaration::Deadline(decl)) => {
                    if decl.direction == Direction::Backward {
                        sink.unsupported(
                            span,
                            format!(
                                "`within {deadline}`: a backward-counted deadline has no \
                                 litgraph edge window"
                            ),
                        );
                        return None;
                    }
                    let at = span.cloned().unwrap_or_else(|| sink.fallback.clone());
                    // A displacing counting rule counts under its own name.
                    let rule = match cx.selection.resolve(cx.pack, &decl.counting, at) {
                        Ok(Declaration::Counting(c)) => c.meta.id.clone(),
                        _ => local(&decl.counting).to_owned(),
                    };
                    let own = (cx.counting != Some(rule.as_str())).then_some(rule);
                    (decl.amount, decl.unit, Some(decl), own)
                }
                Ok(_) => {
                    sink.error(
                        C::TypeMismatch,
                        span,
                        format!("`within {deadline}` does not name a deadline"),
                    );
                    return None;
                }
                Err(diagnostic) => {
                    sink.out.push(diagnostic);
                    return None;
                }
            }
        }
    };
    #[allow(clippy::cast_precision_loss)]
    let length = if length < 1 << 53 {
        length as f64
    } else {
        sink.error(
            C::NonFiniteNumber,
            span,
            format!("a {length}-day window has no exact binary64 value"),
        );
        return None;
    };
    let designated = declared
        .and_then(|x| x.designated.as_ref())
        .map(|x| Designated {
            min_period: length,
            source_event: x.event.clone(),
            source_event_arg: x.name.clone(),
            interpretation: x.interpretation.clone(),
        });
    Some(Deadline {
        length: designated.is_none().then_some(length),
        unit: unit(sink, day_unit, span).ok()?,
        designated,
        extensions: declared
            .map(|x| x.extend.iter().map(extension).collect())
            .unwrap_or_default(),
        extendable: d.extendable.or(declared.and_then(|x| x.extendable)),
        extension_authority: d
            .extension_authority
            .clone()
            .or_else(|| declared.and_then(|x| x.extension_authority.clone())),
        note: d
            .note
            .clone()
            .or_else(|| declared.and_then(|x| x.note.clone())),
        counting,
    })
}

/// One `extend` grant as a pack extension bound.
fn extension(x: &Extension) -> ExtensionBound {
    match x {
        Extension::Bounded {
            by,
            until,
            after_expiry,
            standard,
            motion: m,
            interpretation,
        } => ExtensionBound::Bounded {
            by: by.clone(),
            until: bound(until),
            after_expiry: *after_expiry,
            standard: standard.map(cause),
            motion: m.as_ref().map(motion),
            interpretation: interpretation.clone(),
        },
        Extension::Discretionary {
            by,
            standard,
            window,
            motion: m,
            interpretation,
        } => ExtensionBound::Discretionary {
            by: by.clone(),
            standard: cause(*standard),
            window: match window {
                hir::ExtensionWindow::BeforeExpiry => ExtensionWindow::BeforeExpiry,
                hir::ExtensionWindow::AfterExpiryOnMotion => ExtensionWindow::AfterExpiryOnMotion,
            },
            motion: m.as_ref().map(motion),
            interpretation: interpretation.clone(),
        },
        Extension::NonExtendable { authority } => ExtensionBound::NonExtendable {
            authority: authority.clone(),
        },
    }
}

fn cause(s: hir::CauseStandard) -> CauseStandard {
    match s {
        hir::CauseStandard::GoodCause => CauseStandard::GoodCause,
        hir::CauseStandard::ExcusableNeglect => CauseStandard::ExcusableNeglect,
        hir::CauseStandard::GoodCauseOrExcusableNeglect => {
            CauseStandard::GoodCauseOrExcusableNeglect
        }
        hir::CauseStandard::ExtraordinaryCircumstances => CauseStandard::ExtraordinaryCircumstances,
    }
}

fn date_ref(r: hir::DateRef) -> DateRef {
    match r {
        hir::DateRef::PrescribedDue => DateRef::PrescribedDue,
        hir::DateRef::GrantOrderEntered => DateRef::GrantOrderEntered,
        hir::DateRef::CurrentDue => DateRef::CurrentDue,
    }
}

fn bound(b: &hir::Bound) -> DateBound {
    match b {
        hir::Bound::Offset { from, days } => DateBound::Offset {
            from: date_ref(*from),
            days: *days,
        },
        hir::Bound::Before { from, days } => DateBound::Before {
            from: date_ref(*from),
            days: *days,
        },
        hir::Bound::Max(xs) => DateBound::Max(xs.iter().map(bound).collect()),
        hir::Bound::Min(xs) => DateBound::Min(xs.iter().map(bound).collect()),
    }
}

/// The motion's event pattern in source spelling (`ExtensionMoved(AnswerPeriod)`).
fn motion(m: &hir::Motion) -> Motion {
    Motion {
        event: litigate_sema::case::display_event(&m.event),
        within: m.within.as_ref().map(bound),
        unless: m.unless.as_ref().map(litigate_sema::case::display_event),
    }
}

/// The unqualified name of a qualified declaration id (`core.frap26::frap26`
/// is `frap26`): the engine's clock-library rule id.
fn local(name: &str) -> &str {
    let tail = name.rsplit("::").next().unwrap_or(name);
    tail.rsplit('.').next().unwrap_or(tail)
}
