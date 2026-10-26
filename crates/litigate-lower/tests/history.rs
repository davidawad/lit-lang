//! Amendment-history proofs (SPEC §4.3, §7): a vendored text dated D is
//! the text of a cite in force from E < D only when the section's vendored
//! history records no amendment in (E, D]. An amendment exactly at E holds,
//! one exactly at D or strictly between breaks, and a missing, unpinned,
//! stale or unreadable history proves nothing.
use litigate_engine::cite::SourceCorpus;
use litigate_engine::model::CiteSyntax;
use litigate_lower::source_map::{sha256_hex, Amendment, Citations, L0Section, Proof};
use litigate_sema::hir::SourceBinding;

const TEXT: &str = "## Rule 1\nOne.\n\n## Rule 2\nTwo.\n";
const D: &str = "2026-10-19";

/// One version entry as the eCFR versioner writes it.
fn version(date: &str) -> String {
    format!(
        "{{\"date\":\"{date}\",\"amendment_date\":\"{date}\",\"issue_date\":\"{date}\",\
         \"identifier\":\"1\",\"substantive\":true,\"removed\":false}}"
    )
}

/// A `## versions Rule <n>` block read on `as_of`, listing `dates`.
fn block(n: &str, as_of: &str, dates: &[&str]) -> String {
    let versions: Vec<String> = dates.iter().map(|d| version(d)).collect();
    format!(
        "## versions Rule {n}\nas_of {as_of}\n{{\"content_versions\":[{}],\"meta\":{{}}}}\n",
        versions.join(",")
    )
}

fn binding(id: &str, path: Option<&str>, text: &str, as_of: Option<&str>) -> SourceBinding {
    SourceBinding {
        id: id.into(),
        title: None,
        url: None,
        path: path.map(str::to_owned),
        sha256: None,
        path_sha256: Some(sha256_hex(text)),
        as_of: as_of.map(str::to_owned),
        cite_prefixes: Vec::new(),
        section_heading: None,
    }
}

/// The rules text (dated `as_of`) and a history file holding `history`,
/// pinned when `pinned`.
struct Fixture {
    corpus: SourceCorpus,
    sources: Vec<SourceBinding>,
}

impl Fixture {
    fn new(history: &str, as_of: Option<&str>, pinned: bool) -> Self {
        let corpus = SourceCorpus::from_files([
            ("sources/rules.txt".to_owned(), TEXT.to_owned()),
            ("sources/hist.txt".to_owned(), history.to_owned()),
        ]);
        let rules = SourceBinding {
            cite_prefixes: vec!["R.".into()],
            section_heading: Some("Rule {n}".into()),
            ..binding("rules", Some("sources/rules.txt"), TEXT, as_of)
        };
        let pin = if pinned { history } else { "something else" };
        let hist = binding("hist", Some("sources/hist.txt"), pin, None);
        Self {
            corpus,
            sources: vec![rules, hist],
        }
    }

    fn citations(&self) -> Citations<'_> {
        Citations::new(CiteSyntax::default(), &self.sources, Some(&self.corpus))
    }
}

fn section(c: &Citations<'_>, text: &str) -> L0Section {
    c.map(Some(&text.to_owned()))
        .and_then(|m| m.sections.into_iter().next())
        .unwrap_or_else(|| panic!("no section in {text}"))
}

fn prove(history: &str, effective: &str) -> Proof {
    let f = Fixture::new(history, Some(D), true);
    let c = f.citations();
    c.prove(&section(&c, "R. 1(a)"), effective)
}

#[test]
fn no_amendment_after_the_effective_date_holds() {
    let history = block("1", "2026-10-24", &["2017-01-01", "2017-01-14"]);
    let Proof::Holds(h) = prove(&history, "2017-01-20") else {
        panic!("not proved");
    };
    assert_eq!(h.version, "2017-01-14");
    assert_eq!((h.path.as_str(), h.line), ("sources/hist.txt", 1));
    assert_eq!(h.as_of, "2026-10-24");
    assert_eq!(h.effective, "2017-01-20");
    assert_eq!(h.sha256.len(), 64);
}

#[test]
fn an_amendment_exactly_at_the_effective_date_holds() {
    // Out of order on purpose: the history is read by effective date.
    let history = block("1", "2026-10-24", &["2017-01-14", "2017-01-01"]);
    let Proof::Holds(h) = prove(&history, "2017-01-14") else {
        panic!("not proved");
    };
    assert_eq!(h.version, "2017-01-14");
}

#[test]
fn an_amendment_exactly_at_the_text_date_breaks() {
    let history = block("1", "2026-10-24", &["2017-01-01", D]);
    let Proof::Broken { first, more, .. } = prove(&history, "2017-01-14") else {
        panic!("proved");
    };
    assert_eq!((first.effective(), more), (D, 0));
}

#[test]
fn amendments_strictly_inside_break_and_are_named() {
    let history = block(
        "1",
        "2026-10-24",
        &["2017-01-01", "2020-05-01", "2021-06-01", "2026-10-20"],
    );
    let Proof::Broken {
        history: h,
        first,
        more,
    } = prove(&history, "2017-01-14")
    else {
        panic!("proved");
    };
    // The amendment after D is not in the window.
    assert_eq!(
        (first.date.as_str(), more, h.version.as_str()),
        ("2020-05-01", 1, "2017-01-01")
    );
    let f = Fixture::new(&history, Some(D), true);
    let why = f.citations().unproven("R. 1(a)", "2017-01-14");
    assert_eq!(
        why.as_deref(),
        Some(
            "`R. 1(a)` is not proved the text in force from 2017-01-14: Rule 1 (source `rules` \
             as of 2026-10-19): amended effective 2020-05-01 (version 2020-05-01, issued \
             2020-05-01) and 1 later, per `sources/hist.txt`"
        )
    );
    let once = block("1", "2026-10-24", &["2017-01-01", "2020-05-01"]);
    let f = Fixture::new(&once, Some(D), true);
    let why = f
        .citations()
        .unproven("R. 1(a)", "2017-01-14")
        .unwrap_or_default();
    assert!(
        why.ends_with("2020-05-01), per `sources/hist.txt`"),
        "{why}"
    );
}

#[test]
fn a_removal_and_an_undated_issue_are_described() {
    let removed = Amendment {
        date: "2020-01-01".into(),
        amendment_date: None,
        issue_date: None,
        removed: true,
    };
    assert_eq!(removed.effective(), "2020-01-01");
    assert_eq!(
        removed.describe(),
        "removed effective 2020-01-01 (version 2020-01-01, issued on an unrecorded date)"
    );
}

#[test]
fn a_missing_or_unusable_history_proves_nothing() {
    let missing = |history: &str, pinned: bool| {
        let f = Fixture::new(history, Some(D), pinned);
        let c = f.citations();
        match c.prove(&section(&c, "R. 1"), "2017-01-14") {
            Proof::Missing(why) => why,
            other => panic!("{other:?}"),
        }
    };
    let ok = block("1", "2026-10-24", &["2017-01-01"]);
    // No block for the section; a block in a file no binding pins.
    let other = block("2", "2026-10-24", &["2017-01-01"]);
    assert!(missing(&other, true).starts_with("no source binding pins a `## versions Rule 1`"));
    assert!(missing(&ok, false).starts_with("no source binding pins"));
    // Read before D, or not dated at all.
    let stale = block("1", "2026-10-18", &["2017-01-01"]);
    assert!(missing(&stale, true).contains("`as_of` date on or after 2026-10-19"));
    let undated = ok.replace("as_of 2026-10-24\n", "");
    assert!(missing(&undated, true).contains("does not open with an `as_of` date"));
    let bare = "## versions Rule 1\nas_of 2026-10-24";
    assert!(missing(bare, true).contains("is not a versions list"));
    let garbled = ok.replace("content_versions", "versions");
    assert!(missing(&garbled, true).contains("is not a versions list"));
    // The record starts after E: nothing shows the text E made.
    let late = block("1", "2026-10-24", &["2018-01-01"]);
    assert!(missing(&late, true).contains("records no version on or before 2017-01-14"));
    let empty = block("1", "2026-10-24", &[]);
    assert!(missing(&empty, true).contains("records no version on or before"));
    let f = Fixture::new(&other, Some(D), true);
    let why = f
        .citations()
        .unproven("R. 1", "2017-01-14")
        .unwrap_or_default();
    assert!(
        why.contains("Rule 1 (source `rules` as of 2026-10-19): no source"),
        "{why}"
    );
}

#[test]
fn an_undated_unvendored_or_older_text_needs_no_proof() {
    let history = block("1", "2026-10-24", &["2017-01-01"]);
    assert_eq!(prove(&history, D), Proof::Moot);
    assert_eq!(prove(&history, "2027-01-01"), Proof::Moot);
    let undated = Fixture::new(&history, None, true);
    let c = undated.citations();
    assert_eq!(c.prove(&section(&c, "R. 1"), "2017-01-14"), Proof::Moot);
    // Rule 3 is in no vendored file: there is no text to prove.
    let f = Fixture::new(&history, Some(D), true);
    let c = f.citations();
    assert_eq!(c.prove(&section(&c, "R. 3"), "2017-01-14"), Proof::Moot);
    assert_eq!(c.unproven("R. 3; R. 1", "2017-01-14"), None);
}

#[test]
fn a_repeated_section_is_named_once() {
    let history = block("2", "2026-10-24", &["2017-01-01"]);
    let f = Fixture::new(&history, Some(D), true);
    let why = f
        .citations()
        .unproven("R. 1(a); R. 1(b)", "2017-01-14")
        .unwrap_or_default();
    assert_eq!(why.matches("Rule 1 (source").count(), 1, "{why}");
}

#[test]
fn the_proving_history_goes_into_the_source_map() {
    let history = block("1", "2026-10-24", &["2017-01-01"]);
    let f = Fixture::new(&history, Some(D), true);
    let c = f.citations();
    let text = "R. 1(a); R. 2".to_owned();
    let map = c
        .map_at(Some(&text), Some("2017-01-14"))
        .unwrap_or_else(|| panic!());
    let h = map.sections[0].history.as_ref().map(|h| h.version.as_str());
    assert_eq!(h, Some("2017-01-01"));
    // Rule 2 has no history: no evidence, and the cite is unproven.
    assert_eq!(map.sections[1].history, None);
    insta::assert_json_snapshot!(map);
    let undated = c.map_at(Some(&text), None).unwrap_or_else(|| panic!());
    assert!(undated.sections.iter().all(|s| s.history.is_none()));
    assert_eq!(c.map_at(None, Some("2017-01-14")), None);
}

#[test]
fn each_proving_history_is_named_once_as_the_evidence() {
    let history = block("1", "2026-10-24", &["2017-01-01"]);
    let f = Fixture::new(&history, Some(D), true);
    let c = f.citations();
    // Rule 1 twice is one proof; Rule 2 has no history, so none.
    let proofs = c.proofs("R. 1(a); R. 1(b); R. 2", "2017-01-14");
    assert_eq!(proofs.len(), 1, "{proofs:?}");
    let sha = sha256_hex(history.trim_end().split_once('\n').map_or("", |(_, b)| b));
    let expected = format!(
        "`R. 1(a); R. 1(b); R. 2`: Rule 1 (source `rules` as of {D}) is the text in force \
         from 2017-01-14: `## versions Rule 1` in `sources/hist.txt` line 1 (sha256 {sha}, \
         read 2026-10-24) records version 2017-01-01 and no amendment in (2017-01-14, {D}]"
    );
    assert_eq!(proofs[0], expected);
    // A text not after the version needs no proof; an unparsed one has none.
    assert!(c.proofs("R. 1(a)", D).is_empty());
    assert!(c.proofs("no rule here", "2017-01-14").is_empty());
}
