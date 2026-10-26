//! A vendored text dated D is the text of a cite in force from E < D only
//! when the section's vendored amendment history shows no amendment in
//! (E, D] (SPEC §4.3, §7); otherwise the cite is a hole, and the hole's
//! diagnostic names the amendment that broke the proof.
mod support;

use litigate_base::Profile;
use litigate_engine::cite::SourceCorpus;
use litigate_lower::source_map::sha256_hex;
use litigate_verify::{Checked, Obligation, ObligationStatus};
use support::{pkg_with_sources, verify_in, BASE, FRCP};

/// The vendored FRCP text's date, D.
const D: &str = "2024-12-01";
/// The effective date of the declaration under test, E.
const E: &str = "2017-01-14";
const HISTORY: &str = "sources/rule-history.txt";

/// A `## versions Rule <n>` block listing versions effective on `dates`.
fn block(n: u32, dates: &[&str]) -> String {
    let versions: Vec<String> = dates
        .iter()
        .map(|d| format!("{{\"date\":\"{d}\",\"amendment_date\":\"{d}\",\"issue_date\":\"{d}\"}}"))
        .collect();
    format!(
        "## versions Rule {n}\nas_of 2026-10-24\n{{\"content_versions\":[{}]}}\n",
        versions.join(",")
    )
}

/// [`BASE`] plus a default in force from E citing `cite`, checked
/// strictly with the FRCP text dated D and a pinned history holding
/// Rules 6 and 12 (unamended since 1990, BASE's effective date) and Rule 8
/// as `rule8` lists it (absent when `None`).
fn checked(cite: &str, rule8: Option<&[&str]>) -> Checked {
    let mut history = block(6, &["1990-01-01"]) + &block(12, &["1990-01-01"]);
    if let Some(dates) = rule8 {
        history += &block(8, dates);
    }
    let sources = format!(
        "{}\n[sources.history]\npath = \"{HISTORY}\"\npath_sha256 = \"{}\"\n",
        FRCP.replacen(
            "section_heading = \"Rule {n}\"\n",
            &format!("section_heading = \"Rule {{n}}\"\nas_of = \"{D}\"\n"),
            1
        ),
        sha256_hex(&history),
    );
    let lit = format!("{BASE}\ndefault cited effective [{E}, ..) {cite} when true => 1\n");
    let embedded = SourceCorpus::embedded_ref().files.clone();
    let corpus = SourceCorpus::from_files(embedded.into_iter().chain([(HISTORY.into(), history)]));
    let pkg = pkg_with_sources("demo", "FederalRule", &[], &sources, &lit);
    verify_in(&[pkg], Profile::Strict, &corpus)
}

/// The messages of `c`'s holes.
fn holes(c: &Checked) -> Vec<String> {
    c.verification
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "L5002")
        .map(|d| d.message.clone())
        .collect()
}

/// `c` accepts with no hole and no unresolved cite.
fn proved(c: &Checked) {
    assert_eq!(holes(c), Vec::<String>::new());
    assert!(c.verification.is_accepted(), "{:#?}", c.verification.report);
}

#[test]
fn no_amendment_in_the_window_proves_the_text() {
    proved(&checked("cite \"FRCP 8(c)\"", Some(&["2000-01-01"])));
}

#[test]
fn an_amendment_exactly_at_the_effective_date_holds() {
    proved(&checked("cite \"FRCP 8(c)\"", Some(&["2000-01-01", E])));
}

#[test]
fn an_amendment_exactly_at_the_text_date_breaks() {
    let c = checked("cite \"FRCP 8(c)\"", Some(&["2000-01-01", D]));
    assert_eq!(
        holes(&c),
        [format!(
            "strict does not permit holes: `FRCP 8(c)` is not proved the text in force from \
             {E}: Rule 8 (source `frcp` as of {D}): amended effective {D} (version {D}, issued \
             {D}), per `{HISTORY}`"
        )]
    );
}

#[test]
fn an_amendment_strictly_inside_is_a_hole_naming_it() {
    let c = checked("cite \"FRCP 8(c)\"", Some(&["2000-01-01", "2020-03-01"]));
    let found = holes(&c);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("amended effective 2020-03-01 (version 2020-03-01"),
        "{found:?}"
    );
    let report = &c.verification.report;
    let finding = report.finding(Obligation::Holes);
    assert!(finding.is_some_and(|f| f.status == ObligationStatus::Failed && f.count == 1));
    assert_eq!(
        finding.and_then(|f| f.evidence.as_ref()).map(|e| e.checked),
        Some(1)
    );
    // The cite still resolves: only its expression is unproved.
    let cites = report.finding(Obligation::CiteResolves);
    assert!(cites.is_some_and(|f| f.status == ObligationStatus::Passed));
}

#[test]
fn a_missing_history_is_a_hole() {
    let c = checked("cite \"FRCP 8(c)\"", None);
    assert_eq!(
        holes(&c),
        [format!(
            "strict does not permit holes: `FRCP 8(c)` is not proved the text in force from \
             {E}: Rule 8 (source `frcp` as of {D}): no source binding pins a `## versions Rule 8` \
             amendment history"
        )]
    );
}

#[test]
fn a_historical_or_written_hole_cite_is_not_held_to_the_history() {
    proved(&checked("cite \"FRCP 8(c)\" historical", None));
    // The written hole is the only hole.
    let c = checked("cite ?(\"FRCP 8(c)\")", None);
    assert_eq!(holes(&c), ["strict does not permit holes: FRCP 8(c)"]);
}
