//! `cite-resolves` against pinned source text and dated expressions (SPEC
//! §4.3, §7): a vendored file must be the text its binding's `path_sha256`
//! pins, and a cite's expression (the binding's `as_of`) must not postdate
//! the citing version's window unless the cite is `historical` or a hole.
mod support;

use litigate_base::Profile;
use litigate_sema::pkg::load::LoadedFile;
use litigate_verify::{Checked, Obligation};
use support::{pkg_with_sources, verify, BASE, FRCP};

/// The L1010 messages of `c`.
fn unresolved(c: &Checked) -> Vec<String> {
    c.verification
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "L1010")
        .map(|d| d.message.clone())
        .collect()
}

/// `lit` checked strictly under the FRCP binding edited by `from -> to`.
fn under(from: &str, to: &str, lit: &str) -> Checked {
    assert!(FRCP.contains(from), "FRCP has no {from:?}");
    let sources = FRCP.replacen(from, to, 1);
    verify(
        &[pkg_with_sources("demo", "FederalRule", &[], &sources, lit)],
        Profile::Strict,
    )
}

const PIN: &str =
    "path_sha256 = \"cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44\"\n";

#[test]
fn an_unpinned_or_mismatched_file_is_unverified_text() {
    let base = BASE.to_owned();
    let unpinned = under(PIN, "", &base);
    let messages = unresolved(&unpinned);
    assert_eq!(messages.len(), 6, "{messages:?}");
    assert!(
        messages[0].contains("source `frcp` pins no `path_sha256`"),
        "{messages:?}"
    );
    let note = &unpinned.verification.diagnostics[0].notes[0].message;
    assert!(note.contains(PIN.trim()), "{note}");
    let mismatched = under(
        "cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44",
        &"0".repeat(64),
        &base,
    );
    let messages = unresolved(&mismatched);
    assert_eq!(messages.len(), 6, "{messages:?}");
    assert!(
        messages[0].contains("is not the text source `frcp` pins"),
        "{messages:?}"
    );
}

/// [`BASE`] with a default whose first version is in force until 2010
/// and cited as `cite`; the open-ended second version cites the rule.
fn dated(cite: &str) -> String {
    format!(
        "{BASE}\ndefault dated effective [1990-01-01, 2010-01-01) {cite} when true => 1\n\
         default dated effective [2010-01-01, ..) supersedes dated@1990-01-01 \
         cite \"FRCP 12(a)\" when true => 2\n"
    )
}

const AS_OF: &str = "section_heading = \"Rule {n}\"\n";
const DATED: &str = "section_heading = \"Rule {n}\"\nas_of = \"2024-12-01\"\n";

#[test]
fn an_expression_after_the_window_does_not_resolve() {
    let late = under(AS_OF, DATED, &dated("cite \"FRCP 12(a)\""));
    assert_eq!(
        unresolved(&late),
        ["`FRCP 12(a)` cites source `frcp` as of 2024-12-01, but this version left force on 2010-01-01"]
    );
    let finding = late.verification.report.finding(Obligation::CiteResolves);
    assert!(finding.is_some_and(|f| f.count == 1));
    // `historical` says the later text is cited on purpose.
    let historical = under(AS_OF, DATED, &dated("cite \"FRCP 12(a)\" historical"));
    assert_eq!(unresolved(&historical), Vec::<String>::new());
    // A binding without a date has no expression to compare.
    let undated = under(AS_OF, AS_OF, &dated("cite \"FRCP 12(a)\""));
    assert_eq!(unresolved(&undated), Vec::<String>::new());
    // An expression dated inside the window is fine.
    let early = under(
        AS_OF,
        "section_heading = \"Rule {n}\"\nas_of = \"2000-01-01\"\n",
        &dated("cite \"FRCP 12(a)\""),
    );
    assert_eq!(unresolved(&early), Vec::<String>::new());
}

#[test]
fn a_cite_hole_cites_no_expression() {
    let holed = under(AS_OF, DATED, &dated("cite ?(\"FRCP 12(a)\")"));
    assert_eq!(unresolved(&holed), Vec::<String>::new());
    // A hole with the same reason elsewhere does not excuse the cite: not
    // before it, not after it, and not in another file.
    let lit = format!(
        "{}\ndefault after cite ?(\"FRCP 12(a)\") when true => 1\n",
        dated("cite \"FRCP 12(a)\"")
    )
    .replacen(
        "start served",
        "start served\n\ndefault before cite ?(\"FRCP 12(a)\") when true => 1",
        1,
    );
    let sources = FRCP.replacen(AS_OF, DATED, 1);
    let mut pkg = pkg_with_sources("demo", "FederalRule", &[], &sources, &lit);
    pkg.files.push(LoadedFile {
        relative: "other.lit".to_owned(),
        text: "module demo.other\n\ndefault elsewhere cite ?(\"FRCP 12(a)\") when true => 1\n"
            .to_owned(),
    });
    let c = verify(&[pkg], Profile::Strict);
    assert_eq!(unresolved(&c).len(), 1, "{:?}", unresolved(&c));
}
