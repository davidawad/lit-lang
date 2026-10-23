//! A node renamed with `formerly` keeps its identity (SPEC §4.3): a matter
//! written against the old id still checks, with an L9007 warning whose
//! fix `litigate fix` applies.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{codes, copy_dir, litigate_in, repo, scratch};

#[test]
fn a_matter_written_against_a_renamed_node_is_fixed() {
    let dir = scratch("formerly");
    copy_dir(&repo().join("lit/examples"), &dir.join("lit/examples"));
    copy_dir(&repo().join("cases/examples"), &dir.join("cases"));
    let module = dir.join("lit/examples/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    let renamed = text.replace("mtd-ruling", "dismissal-ruling").replace(
        "  group motions\n  granted \"Motion granted\"",
        "  group motions\n  formerly [mtd-ruling]\n  granted \"Motion granted\"",
    );
    assert_ne!(renamed, text);
    std::fs::write(&module, renamed).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    let check = litigate_in(&dir, &["check", "cases/acme.case"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    let former = |json| {
        codes(json)
            .into_iter()
            .filter(|(code, _)| code == "L9007")
            .collect::<Vec<_>>()
    };
    assert_eq!(
        former(&check.json),
        [("L9007".to_owned(), "warning".to_owned())]
    );
    assert_eq!(
        check.json["result"]["cases"][0]["start"],
        "frcp-rule12-example::dismissal-ruling"
    );
    let fixed = litigate_in(&dir, &["fix", "cases/acme.case", "--code", "L9007"]);
    assert_eq!(fixed.json["result"]["changed_files"][0], "cases/acme.case");
    let case = std::fs::read_to_string(dir.join("cases/acme.case")).unwrap();
    assert!(case.contains("observe dismissal-ruling:"), "{case}");
    let after = litigate_in(&dir, &["check", "cases/acme.case"]);
    assert_eq!(
        (after.code, former(&after.json)),
        (0, Vec::new()),
        "{}",
        after.stdout
    );
}
