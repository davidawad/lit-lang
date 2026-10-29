//! Filing requirements (SPEC §3.7): `limit` declarations type-check into
//! versioned `hir::Limit`s, events name the paper they file, and a paper's
//! counts are judged fail-closed. The FRAP and PTAB limits are the real
//! `lit/frap` and `lit/ptab` packages.
// A panic is the failed assertion.
#![allow(clippy::expect_used)]
mod common;

use common::{check_body, package, render, workspace, PRELUDE};
use litigate_sema::hir::{
    Authored, Counts, Declaration, EventType, Limit, Measure, Package, Verdict,
};
use std::path::Path;

fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The HIR of the repository package at `dir` (with its dependencies),
/// which must check without errors.
fn real(dir: &str) -> Package {
    let root = repo();
    let (db, ws) = litigate_sema::load(&root, &[root.join(dir)], None).expect("loads");
    let checks = litigate_sema::check_workspace(&db, ws);
    let check = checks
        .iter()
        .find(|c| {
            c.package
                .as_ref()
                .is_some_and(|p| root.join(dir).ends_with(p.manifest.forum.as_str()))
        })
        .expect("the package checks without errors");
    check.package.clone().expect("HIR")
}

/// The HIR of a one-module package `main` whose body is `body`.
fn hir(body: &str) -> Package {
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let rendered = render(&db, ws);
    assert!(rendered.is_empty(), "{rendered:#?}");
    let checks = litigate_sema::check_workspace(&db, ws);
    checks[0].package.clone().expect("HIR")
}

fn declarations(p: &Package) -> impl Iterator<Item = &Declaration> {
    p.modules.iter().flat_map(|m| m.declarations.iter())
}

fn limit<'p>(p: &'p Package, id: &str) -> &'p Limit {
    declarations(p)
        .find_map(|d| match d {
            Declaration::Limit(l) if l.meta.id == id => Some(l),
            _ => None,
        })
        .expect("the limit")
}

fn event<'p>(p: &'p Package, name: &str) -> &'p EventType {
    declarations(p)
        .find_map(|d| match d {
            Declaration::Event(e) if e.name == name => Some(e),
            _ => None,
        })
        .expect("the event")
}

fn counts(words: Option<u64>, pages: Option<u64>, lines: Option<u64>) -> Counts {
    Counts {
        words,
        pages,
        lines,
    }
}

#[test]
fn frap_32_a_7_is_three_alternative_bounds_and_a_type_volume_certificate() {
    let frap = real("lit/frap");
    let brief = limit(&frap, "principal-brief");
    let bounds: Vec<(Measure, Authored<u64>, Option<&str>)> = brief
        .bounds
        .iter()
        .map(|b| (b.measure, b.max.clone(), b.cite.as_deref()))
        .collect();
    assert_eq!(
        bounds,
        [
            (
                Measure::Pages,
                Authored::Known(30),
                Some("FRAP 32(a)(7)(A)")
            ),
            (
                Measure::Words,
                Authored::Known(13_000),
                Some("FRAP 32(a)(7)(B)(i)")
            ),
            (
                Measure::Lines,
                Authored::Known(1_300),
                Some("FRAP 32(a)(7)(B)(i)")
            ),
        ]
    );
    assert_eq!(brief.meta.cite.as_deref(), Some("FRAP 32(a)(7)"));
    assert_eq!(brief.meta.effective.from, "2016-12-01");
    assert_eq!(
        brief.meta.effective.unknown_before.as_deref(),
        Some("2016-12-01")
    );
    assert_eq!(brief.meta.authority, "FederalRule");
    assert_eq!(brief.meta.delegations.len(), 2, "FRAP 32(e)");
    let [cert] = brief.certificates.as_slice() else {
        panic!("one certificate: {:?}", brief.certificates);
    };
    assert_eq!(cert.kind, "compliance");
    assert_eq!(cert.measures, [Measure::Words, Measure::Lines]);
    assert_eq!(cert.cite.as_deref(), Some("FRAP 32(g)(1)"));
    // FRAP 32(f): nine parts named, and its catch-all is a hole.
    let excludes = brief.excludes.as_ref().expect("FRAP 32(f)");
    assert_eq!(excludes.cite.as_deref(), Some("FRAP 32(f)"));
    assert_eq!(excludes.items.len(), 10);
    assert!(excludes.open());
    assert_eq!(
        excludes.items.last(),
        Some(&Authored::Hole(Some(
            "any item specifically excluded by these rules or by local rule".to_owned()
        )))
    );
    assert!(brief.ecf.is_empty());
    assert_eq!(
        event(&frap, "PrincipalBriefFiled").paper.as_deref(),
        Some("principal-brief")
    );

    // 13,000 words is within, and relies on the type volume: certified.
    let words = counts(Some(13_000), None, None);
    assert_eq!(brief.length(&words), Some(Verdict::Within));
    assert_eq!(brief.requires(cert, &words), Some(true));
    // 25 pages is within FRAP 32(a)(7)(A) whatever the word count: no
    // certificate.
    let pages = counts(Some(20_000), Some(25), None);
    assert_eq!(brief.length(&pages), Some(Verdict::Within));
    assert_eq!(brief.requires(cert, &pages), Some(false));
    // Over every bound, but FRAP 32(f)'s open catch-all could exclude
    // more: unknown, never over.
    let over = counts(Some(13_001), Some(31), Some(1_301));
    assert_eq!(brief.length(&over), Some(Verdict::Unknown));
    assert_eq!(brief.requires(cert, &over), None);
    // Nothing counted decides nothing.
    assert_eq!(brief.length(&Counts::default()), Some(Verdict::Unknown));
}

#[test]
fn cfr_42_24_limits_an_ipr_petition_to_14000_words() {
    let ptab = real("lit/ptab");
    let petition = limit(&ptab, "ipr-petition");
    let [bound] = petition.bounds.as_slice() else {
        panic!("one bound: {:?}", petition.bounds);
    };
    assert_eq!(bound.measure, Measure::Words);
    assert_eq!(bound.max, Authored::Known(14_000));
    assert_eq!(bound.cite.as_deref(), Some("37 C.F.R. § 42.24(a)(1)(i)"));
    assert_eq!(petition.meta.effective.from, "2016-05-02");
    assert_eq!(petition.meta.authority, "Regulation");
    let [cert] = petition.certificates.as_slice() else {
        panic!("one certificate: {:?}", petition.certificates);
    };
    assert_eq!(cert.kind, "word-count");
    assert!(cert.measures.is_empty());
    let excludes = petition.excludes.as_ref().expect("§ 42.24(a)(1)");
    assert!(!excludes.open());
    assert!(excludes
        .items
        .contains(&Authored::Known("table-of-contents".to_owned())));
    assert_eq!(
        event(&ptab, "IprPetitionFiled").paper.as_deref(),
        Some("ipr-petition")
    );

    let within = counts(Some(14_000), None, None);
    assert_eq!(petition.length(&within), Some(Verdict::Within));
    // § 42.24(d): a type-volume paper always certifies its count.
    assert_eq!(petition.requires(cert, &within), Some(true));
    // A closed exclusion list decides over.
    let over = counts(Some(14_001), Some(1), None);
    assert_eq!(petition.bound(bound, &over), Verdict::Over);
    assert_eq!(petition.length(&over), Some(Verdict::Over));
}

#[test]
fn an_unvendored_exclusion_is_a_hole_and_leaves_over_unknown() {
    let body = "limit brief {
  words <= 100
  pages <= ?(\"local rule\")
  excludes [cover-page, ?(\"any item excluded by local rule\")]
  ecf \"brief\"
  ecf ?(\"district ECF event\")
}";
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let check = &litigate_sema::check_workspace(&db, ws)[0];
    let reasons: Vec<Option<&str>> = check.holes.iter().map(|h| h.reason.as_deref()).collect();
    assert_eq!(
        reasons,
        [
            Some("local rule"),
            Some("any item excluded by local rule"),
            Some("district ECF event"),
        ]
    );
    let p = check.package.clone().expect("HIR");
    let brief = limit(&p, "brief");
    assert!(brief
        .excludes
        .as_ref()
        .is_some_and(litigate_sema::hir::Exclusions::open));
    assert_eq!(
        brief.bounds[1].max,
        Authored::Hole(Some("local rule".into()))
    );
    assert_eq!(brief.ecf.len(), 2);
    assert_eq!(brief.ecf[0].code, Authored::Known("brief".into()));
    // Within the known bound stays within; over it is unknown, and the
    // hole bound judges nothing.
    assert_eq!(
        brief.length(&counts(Some(100), None, None)),
        Some(Verdict::Within)
    );
    assert_eq!(
        brief.length(&counts(Some(101), Some(1), None)),
        Some(Verdict::Unknown)
    );
    assert_eq!(
        brief.bound(&brief.bounds[1], &counts(None, Some(1), None)),
        Verdict::Unknown
    );
}

#[test]
fn a_limit_without_bounds_judges_no_length_and_its_entries_cite_alone() {
    let p = hir("limit served-paper {
  requires certificate service cite \"FRAP 25(d)(1)\"
  ecf \"proof\"
} cite ?(\"FRAP 25\")
event ProofFiled() paper served-paper
event Plain()
");
    let paper = limit(&p, "served-paper");
    assert_eq!(paper.length(&Counts::default()), None);
    assert_eq!(paper.certificates[0].cite.as_deref(), Some("FRAP 25(d)(1)"));
    assert_eq!(
        paper.requires(&paper.certificates[0], &Counts::default()),
        Some(true)
    );
    // The declaration's cite covers the entries without their own; it
    // may be a hole (its reason kept as the text, as for any declaration).
    assert_eq!(paper.ecf[0].cite, None);
    assert_eq!(paper.meta.cite.as_deref(), Some("FRAP 25"));
    assert_eq!(
        event(&p, "ProofFiled").paper.as_deref(),
        Some("served-paper")
    );
    assert_eq!(event(&p, "Plain").paper, None);
}

#[test]
fn limits_are_versioned_like_any_declaration() {
    let p = hir("limit brief effective [2009-12-01, 2016-12-01) {
  words <= 14_000
}
limit brief effective [2016-12-01, ..) {
  words <= 13_000
}");
    let versions: Vec<(&str, &Authored<u64>)> = declarations(&p)
        .filter_map(|d| match d {
            Declaration::Limit(l) => Some((l.meta.effective.from.as_str(), &l.bounds[0].max)),
            _ => None,
        })
        .collect();
    assert_eq!(
        versions,
        [
            ("2009-12-01", &Authored::Known(14_000)),
            ("2016-12-01", &Authored::Known(13_000)),
        ]
    );
    let overlap = check_body(
        "limit brief effective [2009-12-01, ..) { words <= 1 }\n\
         limit brief effective [2016-12-01, ..) { words <= 2 }",
    );
    assert!(overlap.iter().any(|d| d.contains("L3001")), "{overlap:#?}");
}

/// Each ill-typed `limit` (or `paper`) and the one code it reports.
#[test]
fn ill_typed_filing_requirements_are_rejected() {
    for (body, code, says) in [
        ("limit b { words <= 0 }", "L2001", "whole number above zero"),
        (
            "limit b { pages <= 1.5 }",
            "L2001",
            "whole number above zero",
        ),
        (
            "limit b { words <= 1, words <= 2 }",
            "L2007",
            "declared twice",
        ),
        (
            "limit b { words <= 1, requires certificate c for [pages] }",
            "L2007",
            "does not bound",
        ),
        (
            "limit b { words <= 1, requires certificate c for [words, words] }",
            "L2007",
            "named twice",
        ),
        (
            "limit b { requires certificate c, requires certificate c }",
            "L2007",
            "declared twice",
        ),
        (
            "limit b { requires certificate x::c }",
            "L2007",
            "plain identifier",
        ),
        (
            "limit b { excludes [a], excludes [b] }",
            "L2007",
            "declared twice",
        ),
        ("limit b { excludes [a, a] }", "L2007", "excluded twice"),
        ("limit b { excludes [x::a] }", "L2001", "plain identifier"),
        (
            "limit b { ecf \"x\", ecf \"x\" }",
            "L2007",
            "declared twice",
        ),
        (
            "limit b { words <= 1 cite \"X 1\" cite \"X 2\" }",
            "L2007",
            "at most one `cite`",
        ),
        ("limit x::b { words <= 1 }", "L2007", "plain identifier"),
        ("event E() paper nothing", "L1002", "nothing"),
        ("event E() paper s", "L2001", "expected a filing limit"),
        (
            "limit b {}\nstate x: \"X\" { _ \"go\" -> b }",
            "L2001",
            "is a filing limit",
        ),
        (
            "limit b {}\nevent E() paper b paper b",
            "L2007",
            "more than once",
        ),
    ] {
        let found = check_body(body);
        assert!(
            found.iter().any(|d| d.contains(code) && d.contains(says)),
            "{body}: {found:#?}"
        );
    }
}

/// A malformed `limit` (L0001) still checks: each part the parser could
/// not read is left out, never guessed.
#[test]
fn a_malformed_limit_reports_only_its_syntax_error() {
    for body in [
        "limit",
        "limit b",
        "limit b { words <= }",
        "limit b { requires certificate }",
        "limit b { excludes cover-page }",
        "limit b { ecf }",
        "limit b { note \"x\" }",
    ] {
        let found = check_body(body);
        assert!(
            found.iter().all(|d| d.contains("L0001")) && !found.is_empty(),
            "{body}: {found:#?}"
        );
    }
}
