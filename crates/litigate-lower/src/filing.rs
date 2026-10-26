//! Filing requirements (SPEC §3.7) to a pack's `filingLimits`: the version
//! of each `limit` in force at the build dates, field for field. A hole
//! (a bound, an excluded part or an ECF code the sources do not settle)
//! is left out of the document and listed in `lowered.holes`.
use crate::source_map::{Citations, Entry};
use litigate_engine::model::{
    EcfCode, FilingBound, FilingCertificate, FilingExclusions, FilingLimit, Hole, Measure,
};
use litigate_sema::hir::{self, Authored};

/// litgraph's measure for `m`.
const fn measure(m: hir::Measure) -> Measure {
    match m {
        hir::Measure::Words => Measure::Words,
        hir::Measure::Pages => Measure::Pages,
        hir::Measure::Lines => Measure::Lines,
    }
}

/// The known value of `a`, or the hole at `pointer` with its reason.
fn known<T: Clone>(a: &Authored<T>, pointer: String, holes: &mut Vec<Hole>) -> Option<T> {
    match a {
        Authored::Known(v) => Some(v.clone()),
        Authored::Hole(reason) => {
            holes.push(Hole {
                pointer,
                reason: reason.clone(),
            });
            None
        }
    }
}

/// `l` as the `filingLimits` entry at `base`, with its holes.
pub(crate) fn limit(l: &hir::Limit, base: &str, holes: &mut Vec<Hole>) -> FilingLimit {
    let bounds = l
        .bounds
        .iter()
        .enumerate()
        .map(|(i, b)| FilingBound {
            measure: measure(b.measure),
            max: known(&b.max, format!("{base}/bounds/{i}/max"), holes),
            cite: b.cite.clone(),
        })
        .collect();
    let certificates = l
        .certificates
        .iter()
        .map(|c| FilingCertificate {
            kind: c.kind.clone(),
            measures: c.measures.iter().map(|m| measure(*m)).collect(),
            cite: c.cite.clone(),
        })
        .collect();
    let excludes = l.excludes.as_ref().map(|x| {
        let mut open = 0;
        let items = x
            .items
            .iter()
            .filter_map(|item| {
                let pointer = format!("{base}/excludes/open/{open}");
                if matches!(item, Authored::Hole(_)) {
                    open += 1;
                }
                known(item, pointer, holes)
            })
            .collect();
        FilingExclusions {
            items,
            cite: x.cite.clone(),
        }
    });
    let ecf = l
        .ecf
        .iter()
        .enumerate()
        .map(|(i, e)| EcfCode {
            code: known(&e.code, format!("{base}/ecf/{i}/code"), holes),
            cite: e.cite.clone(),
        })
        .collect();
    FilingLimit {
        id: l.meta.id.clone(),
        cite: l.meta.cite.clone(),
        bounds,
        certificates,
        excludes,
        ecf,
    }
}

/// The source-map entry of the `filingLimits` entry at `base`.
pub(crate) fn entry(citations: &Citations<'_>, l: &hir::Limit, base: String) -> Entry {
    Entry {
        pointer: base,
        id: l.meta.id.clone(),
        span: l.meta.span.clone(),
        decl: Some(crate::select::Selection::versioned(&l.meta)),
        cite: citations.map_at(l.meta.cite.as_ref(), Some(&l.meta.effective.from)),
    }
}
