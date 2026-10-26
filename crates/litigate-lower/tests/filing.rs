//! Filing limits lower to `Pack.filingLimits` (SPEC §3.7): the version in
//! force at the build dates, field for field, each hole absent from the
//! document and listed in `lowered.holes`, and one source-map entry per
//! limit.
mod support;

use litigate_engine::model::{Measure, Pack};
use litigate_sema::hir::{self, Authored, CertificateReq, Ecf, Exclusions, LengthBound, Limit};
use support::{decls_mut, example, lowered, pack_of};

/// A limit with every kind of entry, a hole of each kind among them.
fn brief(meta: hir::Metadata) -> Limit {
    let bound = |measure, max| LengthBound {
        measure,
        max,
        cite: Some("FRCP 12(a)".into()),
        span: None,
    };
    Limit {
        meta,
        bounds: vec![
            bound(hir::Measure::Words, Authored::Known(13_000)),
            bound(
                hir::Measure::Pages,
                Authored::Hole(Some("local rule".into())),
            ),
            bound(hir::Measure::Lines, Authored::Known(1_300)),
        ],
        certificates: vec![CertificateReq {
            kind: "compliance".into(),
            measures: vec![hir::Measure::Words, hir::Measure::Lines],
            cite: None,
            span: None,
        }],
        excludes: Some(Exclusions {
            items: vec![
                Authored::Hole(None),
                Authored::Known("cover-page".into()),
                Authored::Hole(Some("any item excluded by local rule".into())),
            ],
            cite: Some("FRCP 12(a)".into()),
            span: None,
        }),
        ecf: vec![
            Ecf {
                code: Authored::Known("brief".into()),
                cite: None,
                span: None,
            },
            Ecf {
                code: Authored::Hole(None),
                cite: None,
                span: None,
            },
        ],
    }
}

#[test]
fn a_limit_lowers_field_for_field_with_its_holes_listed() {
    let mut pkg = example();
    let mut meta = decls_mut(&mut pkg)
        .find_map(|d| match d {
            hir::Declaration::Node(n) => Some(n.meta.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("a node"));
    meta.id = "brief".into();
    meta.relations.clear();
    meta.delegations.clear();
    pkg.modules[0]
        .declarations
        .push(hir::Declaration::Limit(brief(meta.clone())));
    let lowered = lowered(&pkg);
    let pack = pack_of(&lowered);
    let [limit] = pack.filing_limits.as_slice() else {
        panic!("one limit: {:?}", pack.filing_limits);
    };
    assert_eq!(limit.id, "brief");
    assert_eq!(limit.cite, meta.cite);
    let bounds: Vec<(Measure, Option<u64>)> =
        limit.bounds.iter().map(|b| (b.measure, b.max)).collect();
    assert_eq!(
        bounds,
        [
            (Measure::Words, Some(13_000)),
            (Measure::Pages, None),
            (Measure::Lines, Some(1_300)),
        ]
    );
    assert_eq!(
        limit.certificates[0].measures,
        [Measure::Words, Measure::Lines]
    );
    let excludes = limit
        .excludes
        .as_ref()
        .unwrap_or_else(|| panic!("excludes"));
    assert_eq!(excludes.items, ["cover-page"]);
    let codes: Vec<Option<&str>> = limit.ecf.iter().map(|e| e.code.as_deref()).collect();
    assert_eq!(codes, [Some("brief"), None]);
    let holes: Vec<(&str, Option<&str>)> = pack
        .lowered
        .iter()
        .flat_map(|l| &l.holes)
        .filter(|h| h.pointer.starts_with("/filingLimits"))
        .map(|h| (h.pointer.as_str(), h.reason.as_deref()))
        .collect();
    assert_eq!(
        holes,
        [
            ("/filingLimits/0/bounds/1/max", Some("local rule")),
            ("/filingLimits/0/ecf/1/code", None),
            ("/filingLimits/0/excludes/open/0", None),
            (
                "/filingLimits/0/excludes/open/1",
                Some("any item excluded by local rule")
            ),
        ]
    );
    let entry = lowered
        .source_map
        .entries
        .iter()
        .find(|e| e.pointer == "/filingLimits/0")
        .unwrap_or_else(|| panic!("a source-map entry"));
    assert_eq!(entry.id, "brief");
    assert!(entry
        .decl
        .as_deref()
        .is_some_and(|d| d.starts_with("brief@")));
    // The engine's strict loader takes the document, holes and all.
    let json = serde_json::to_string(pack).unwrap_or_default();
    let back = Pack::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(back.filing_limits, pack.filing_limits);
}

#[test]
fn a_limit_not_in_force_at_the_build_dates_is_not_lowered() {
    let mut pkg = example();
    let mut meta = decls_mut(&mut pkg)
        .find_map(|d| match d {
            hir::Declaration::Node(n) => Some(n.meta.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("a node"));
    meta.id = "brief".into();
    meta.relations.clear();
    meta.delegations.clear();
    meta.effective.from = "2999-01-01".into();
    meta.effective.to = None;
    pkg.modules[0]
        .declarations
        .push(hir::Declaration::Limit(brief(meta)));
    assert!(pack_of(&lowered(&pkg)).filing_limits.is_empty());
}
