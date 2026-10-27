//! `calibration` declarations to the engine's calibration sets
//! (`CalibrationSet`, one per declaration): each entry's target, value,
//! source and sample size. A source's `vintage [from, to)` is half-open
//! like every interval; the engine's `vintageEnd` is its last day, `to`
//! minus one.
use crate::diag::Sink;
use litigate_engine::api::{
    CalibrationEntry, CalibrationSet, CalibrationSource, CalibrationTarget,
};
use litigate_engine::clock::Date;
use litigate_engine::model::Duration;
use litigate_sema::hir::{self, CalibratedValue, Declaration};

/// Every calibration set `pkg` declares, in module order.
pub(crate) fn sets(pkg: &hir::Package, sink: &mut Sink) -> Vec<CalibrationSet> {
    pkg.modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(|d| match d {
            Declaration::Calibration(c) => Some(set(c, sink)),
            _ => None,
        })
        .collect()
}

fn set(c: &hir::Calibration, sink: &mut Sink) -> CalibrationSet {
    let entries = c.entries.iter().filter_map(|e| entry(c, e, sink)).collect();
    CalibrationSet {
        id: c.meta.id.clone(),
        title: c.meta.label.clone(),
        description: c.description.clone(),
        entries,
    }
}

fn entry(
    c: &hir::Calibration,
    e: &hir::CalibrationEntry,
    sink: &mut Sink,
) -> Option<CalibrationEntry> {
    let span = e.span.as_ref();
    // Sema keeps only entries naming a declared source.
    let s = c.sources.iter().find(|s| s.id == e.source)?;
    // The lexer only admits valid dates.
    let vintage_end =
        s.to.as_deref()
            .and_then(|to| Date::parse_iso(to).ok())
            .map(|to| to.add_days(-1).to_iso());
    let (target, attr, value, distribution) = match &e.value {
        CalibratedValue::Probability { value } => (
            CalibrationTarget::EdgeProbability,
            None,
            match value {
                Some(p) => Some(sink.number(*p, span)?),
                None => None,
            },
            None,
        ),
        CalibratedValue::Duration { value } => (
            CalibrationTarget::EdgeDuration,
            None,
            None,
            Some(Duration {
                min: value.min.and_then(|v| sink.number(v, span)),
                mode: sink.number(value.mode, span)?,
                max: value.max.and_then(|v| sink.number(v, span)),
            }),
        ),
        CalibratedValue::Attr { name, value } => (
            CalibrationTarget::NodeAttr,
            Some(name.clone()),
            Some(sink.number(*value, span)?),
            None,
        ),
    };
    Some(CalibrationEntry {
        target,
        target_ref: e.target.clone(),
        attr,
        value,
        distribution,
        source: CalibrationSource {
            title: s.title.clone(),
            url: s.url.clone(),
            vintage_start: s.from.clone(),
            vintage_end,
            retrieved: s.retrieved.clone(),
        },
        n: e.n,
        method: e.method.clone(),
    })
}
