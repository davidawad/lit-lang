//! Alternative-characterization deadline modeling stays executable and strict.
#![allow(clippy::unwrap_used)]
mod support;

use support::{copy_dir, litigate_in, repo, scratch};

#[test]
fn competing_deadline_clocks_are_a_costed_choice_and_typed_chance_branch() {
    let root = scratch("alternative-characterization");
    let fixture = root.join("alternative-characterization");
    copy_dir(
        &repo().join("crates/litigate-cli/tests/fixtures/alternative-characterization"),
        &fixture,
    );
    copy_dir(&repo().join("lit/core"), &root.join("lit/core"));
    copy_dir(&repo().join("sources"), &root.join("sources"));
    let core_lock = litigate_in(&root, &["lock", "lit/core"]);
    assert_eq!(core_lock.code, 0, "{}", core_lock.stdout);
    let fixture = fixture.to_str().unwrap();
    let lock = litigate_in(&root, &["lock", fixture]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let run = litigate_in(&root, &["check", fixture]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["ok"], true);
    assert_eq!(run.json["result"]["packages"][0]["tests"]["passed"], 2);
    assert_eq!(run.json["result"]["packages"][0]["holes"], 0);
}
