//! `cite-resolves` over filing limits (SPEC §3.7, §7): every bound's
//! number and measure and every excluded part named must be in the cited
//! vendored text, so a value the sources do not settle can only be a hole.
// A panic is the failed assertion.
#![allow(clippy::expect_used)]
mod support;

use litigate_base::Profile;
use litigate_engine::model::Measure;
use litigate_lower::Document;
use litigate_verify::Checked;
use support::{pkg_with_sources, verify, with, FRCP};

/// The vendored FRAP text, bound as `sources/frap.txt` is in `lit/frap`.
const FRAP: &str = "[sources.frap]\ntitle = \"Federal Rules of Appellate Procedure\"\n\
     path = \"sources/frap.txt\"\n\
     path_sha256 = \"87d71a1f443c5c084cc9f11932a66436ab300f83146ee2c5ce2654a8a4824479\"\n\
     cite_prefixes = [\"FRAP\"]\nsection_heading = \"FRAP {n}\"\n";

/// [`support::BASE`] plus `limit`, checked, verified in `dev` and
/// lowered (so sema accepted it).
fn checked(limit: &str) -> Checked {
    let checked = verify(
        &[pkg_with_sources(
            "demo",
            "FederalRule",
            &[],
            &format!("{FRCP}\n{FRAP}"),
            &with(limit),
        )],
        Profile::Dev,
    );
    assert!(checked.lowered.is_some(), "{limit}");
    checked
}

/// The L1010 messages of [`checked`].
fn unresolved(limit: &str) -> Vec<String> {
    checked(limit)
        .verification
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "L1010")
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn frap_32s_numbers_and_parts_are_in_its_vendored_text() {
    let limit = "limit principal-brief {
  pages <= 30 cite \"FRAP 32(a)(7)(A)\"
  words <= 13_000 cite \"FRAP 32(a)(7)(B)(i)\"
  lines <= 1_300 cite \"FRAP 32(a)(7)(B)(i)\"
  requires certificate compliance for [words, lines] cite \"FRAP 32(g)(1)\"
  excludes [cover-page, statement-regarding-oral-argument, proof-of-service, ?(\"local rule\")] cite \"FRAP 32(f)\"
  ecf ?(\"the circuit's event\") cite \"FRAP 25(a)\"
} cite \"FRAP 32(a)(7)\"";
    assert_eq!(unresolved(limit), Vec::<String>::new());
    // Lowered field for field; the holes are listed, not filled.
    let lowered = checked(limit).lowered.expect("lowered");
    let Document::Pack(pack) = &lowered.document else {
        panic!("a pack");
    };
    let [brief] = pack.filing_limits.as_slice() else {
        panic!("one limit: {:?}", pack.filing_limits);
    };
    assert_eq!(brief.id, "principal-brief");
    assert_eq!(brief.cite.as_deref(), Some("FRAP 32(a)(7)"));
    let bounds: Vec<(Measure, Option<u64>)> =
        brief.bounds.iter().map(|b| (b.measure, b.max)).collect();
    assert_eq!(
        bounds,
        [
            (Measure::Pages, Some(30)),
            (Measure::Words, Some(13_000)),
            (Measure::Lines, Some(1_300)),
        ]
    );
    assert_eq!(
        brief.certificates[0].measures,
        [Measure::Words, Measure::Lines]
    );
    let excludes = brief.excludes.as_ref().expect("excludes");
    assert_eq!(
        excludes.items,
        [
            "cover-page",
            "statement-regarding-oral-argument",
            "proof-of-service"
        ]
    );
    assert_eq!(brief.ecf[0].code, None);
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
            ("/filingLimits/0/ecf/0/code", Some("the circuit's event")),
            ("/filingLimits/0/excludes/open/0", Some("local rule")),
        ]
    );
    let entry = lowered
        .source_map
        .entries
        .iter()
        .find(|e| e.pointer == "/filingLimits/0")
        .expect("a source-map entry");
    assert_eq!(entry.id, "principal-brief");
    assert!(entry.span.is_some());
    // The pack round-trips through the engine's strict loader.
    let json = serde_json::to_string(pack).expect("serializes");
    let back = litigate_engine::model::Pack::from_json(&json).expect("loads");
    assert_eq!(back.filing_limits, pack.filing_limits);
}

#[test]
fn a_bound_the_rule_spells_in_words_is_in_its_vendored_text() {
    // 37 C.F.R. § 1.943(c): "shall not exceed thirty pages or 14,000 words".
    let cfr = "[sources.cfr]\ntitle = \"37 C.F.R. Part 1\"\npath = \"sources/cfr-37-part1.txt\"\n\
         path_sha256 = \"e5765fec0e8c5f295d4f230ed9743b7972dbd41340693833abf990fe1918d1ec\"\n\
         cite_prefixes = [\"37 C.F.R.\"]\nsection_heading = \"37 C.F.R. § {n}\"\n";
    let resolved = |limit: &str| -> Vec<String> {
        verify(
            &[pkg_with_sources(
                "demo",
                "FederalRule",
                &[],
                &format!("{FRCP}\n{cfr}"),
                &with(limit),
            )],
            Profile::Dev,
        )
        .verification
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "L1010")
        .map(|d| d.message.clone())
        .collect()
    };
    assert_eq!(
        resolved("limit brief { pages <= 30 words <= 14_000 } cite \"37 C.F.R. § 1.943\""),
        Vec::<String>::new()
    );
    assert_eq!(
        resolved("limit brief { pages <= 31 } cite \"37 C.F.R. § 1.943\""),
        ["`37 C.F.R. § 1.943` is cited for \"31 pages\" or \"thirty-one pages\" or \"thirty-one (31) pages\", which its vendored text does not hold"]
    );
}

#[test]
fn a_bound_or_part_the_vendored_text_does_not_hold_is_unresolved() {
    for (limit, message) in [
        (
            "limit b { words <= 14_000 cite \"FRAP 32(a)(7)(B)(i)\" }",
            "`FRAP 32(a)(7)(B)(i)` is cited for \"14,000 words\" or \"fourteen thousand words\" or \"fourteen thousand (14,000) words\", which its vendored text does not hold",
        ),
        (
            "limit b { pages <= 30 } cite \"FRAP 31\"",
            "`FRAP 31` is cited for \"30 pages\" or \"thirty pages\" or \"thirty (30) pages\", which its vendored text does not hold",
        ),
        (
            "limit b { excludes [cover-page, oral-argument-statement] cite \"FRAP 32(f)\" }",
            "`FRAP 32(f)` is cited for \"oral argument statement\", which its vendored text does not hold",
        ),
        (
            "limit b { requires certificate compliance cite \"FRAP 99\" }",
            "has no `## FRAP 99` heading",
        ),
        ("limit b { ecf \"brief\" cite \"FRAP 98\" }", "has no `## FRAP 98` heading"),
    ] {
        let found = unresolved(limit);
        assert!(
            found.len() == 1 && found[0].contains(message),
            "{limit}: {found:#?}"
        );
    }
}

#[test]
fn a_hole_quotes_nothing_and_an_uncited_entry_quotes_nothing() {
    // Holes name no number and no part; without any cite there is nothing
    // to hold an entry to (`cite-resolves` checks cites, `HolePresent` and
    // `strict` the rest).
    for limit in [
        "limit b { words <= ? excludes [?(\"local rule\")] } cite \"FRAP 32(f)\"",
        "limit b { words <= 13_000 excludes [anything] requires certificate c ecf \"x\" }",
    ] {
        assert_eq!(unresolved(limit), Vec::<String>::new(), "{limit}");
    }
}
