//! `litigate compat OLD NEW` (SPEC §4.4): the computed class of a release
//! and the version bump it needs (L1012).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{copy_dir, litigate_in, repo, scratch};

/// Copies of `lit/examples` as `old` and `new`, `new` edited by `edit`.
fn releases(name: &str, edit: impl Fn(&str) -> String, version: &str) -> std::path::PathBuf {
    let dir = scratch(name);
    for side in ["old", "new"] {
        copy_dir(&repo().join("lit/examples"), &dir.join(side));
    }
    let module = dir.join("new/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    std::fs::write(&module, edit(&text)).unwrap();
    let manifest = dir.join("new/Lit.toml");
    let toml = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        toml.replace("version = \"0.1.0\"", &format!("version = \"{version}\"")),
    )
    .unwrap();
    dir
}

fn compat(dir: &std::path::Path) -> support::Run {
    litigate_in(dir, &["compat", "old", "new"])
}

#[test]
fn an_unchanged_release_is_compatible_at_any_later_version() {
    let dir = releases("compat-same", str::to_owned, "0.1.0");
    let same = compat(&dir);
    assert_eq!(same.code, 0, "{}", same.stdout);
    assert_eq!(same.json["result"]["class"], "none");
    assert_eq!(same.json["result"]["old"], same.json["result"]["new"]);
    let comments = releases(
        "compat-comment",
        |t| format!("// reformatted\n{t}"),
        "0.1.1",
    );
    let run = compat(&comments);
    assert_eq!(
        (run.code, &run.json["result"]["declared"]),
        (0, &"patch".into())
    );
}

#[test]
fn a_value_change_needs_a_patch_bump_before_1_0() {
    let shorter = |t: &str| {
        t.replace("21.days", "14.days")
            .replace("2025-03-24", "2025-03-17")
    };
    let dir = releases("compat-value-same", shorter, "0.1.0");
    let run = compat(&dir);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_snapshot!(run.json["diagnostics"][0]["message"], @r#""`frcp-rule12-example` 0.1.0 is not later than 0.1.0: a release needs a later version (0.1.1 or later)""#);
    assert_eq!(
        run.json["result"]["changes"][0]["what"],
        "deadline answer-period"
    );
    let bumped = compat(&releases("compat-value-bumped", shorter, "0.1.1"));
    assert_eq!(
        (bumped.code, &bumped.json["result"]["class"]),
        (0, &"value".into())
    );
}

#[test]
fn an_interface_change_needs_a_minor_bump_before_1_0() {
    let dropped = |t: &str| {
        t.replace("  default \"Let the deadline pass\" -> default-risk { sets [defaulted], valence bad }\n", "")
    };
    let run = compat(&releases("compat-interface", dropped, "0.1.1"));
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_snapshot!(run.json["diagnostics"][0]["message"], @r#""`frcp-rule12-example` 0.1.1 bumps the patch version of 0.1.0, but its changes are `interface`: they need a minor bump (0.2.0 or later)""#);
    assert_eq!(
        run.json["diagnostics"][0]["fixes"][0]["edits"][0]["replacement"],
        "\"0.2.0\""
    );
    let ok = compat(&releases("compat-interface-major", dropped, "0.2.0"));
    assert_eq!(
        (ok.code, &ok.json["result"]["required"]),
        (0, &"minor".into())
    );
}

#[test]
fn an_earlier_version_or_a_pre_release_is_not_enough() {
    let dir = releases("compat-earlier", str::to_owned, "0.0.9");
    let run = compat(&dir);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_snapshot!(run.json["diagnostics"][0]["message"], @r#""`frcp-rule12-example` 0.0.9 is not later than 0.1.0: a release needs a later version (0.1.1 or later)""#);
    let pre = releases(
        "compat-pre",
        |t| {
            t.replace("21.days", "14.days")
                .replace("2025-03-24", "2025-03-17")
        },
        "0.1.0",
    );
    let toml = pre.join("old/Lit.toml");
    let text = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(
        &toml,
        text.replace("version = \"0.1.0\"", "version = \"0.1.0-rc.1\""),
    )
    .unwrap();
    let run = compat(&pre);
    insta::assert_snapshot!(run.json["diagnostics"][0]["message"], @r#""`frcp-rule12-example` 0.1.0 bumps the pre-release version of 0.1.0-rc.1, but its changes are `value`: they need a patch bump (0.1.1 or later)""#);
}

#[test]
fn both_sides_must_check_and_be_one_package() {
    let broken = releases("compat-broken", |t| format!("{t}\nstate oops\n"), "0.1.1");
    let run = compat(&broken);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json["result"].is_null());
    let other = scratch("compat-other");
    copy_dir(&repo().join("lit/examples"), &other.join("old"));
    copy_dir(&repo().join("lit/core"), &other.join("new"));
    let run = compat(&other);
    assert_eq!(run.code, 1);
    assert!(
        run.stderr
            .contains("compat compares two releases of one package"),
        "{}",
        run.stderr
    );
    std::fs::create_dir_all(other.join("empty")).unwrap();
    let run = litigate_in(&other, &["compat", "empty", "old"]);
    assert_eq!(
        (run.code, run.stderr.as_str()),
        (1, "litigate compat: empty: no Lit.toml\n")
    );
}

#[test]
fn from_1_0_0_an_interface_change_needs_a_major_bump() {
    let dropped = |t: &str| {
        t.replace("  default \"Let the deadline pass\" -> default-risk { sets [defaulted], valence bad }\n", "")
    };
    let dir = releases("compat-major", dropped, "1.1.0");
    let toml = dir.join("old/Lit.toml");
    let text = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(
        &toml,
        text.replace("version = \"0.1.0\"", "version = \"1.0.0\""),
    )
    .unwrap();
    let run = compat(&dir);
    insta::assert_snapshot!(run.json["diagnostics"][0]["message"], @r#""`frcp-rule12-example` 1.1.0 bumps the minor version of 1.0.0, but its changes are `interface`: they need a major bump (2.0.0 or later)""#);
}

#[test]
fn a_side_that_does_not_load_is_a_failure() {
    let dir = releases("compat-missing", str::to_owned, "0.1.0");
    for args in [["compat", "nope", "new"], ["compat", "old", "nope"]] {
        let run = litigate_in(&dir, &args);
        assert_eq!(run.code, 1, "{args:?}");
        assert!(run.stderr.contains("nope"), "{}", run.stderr);
    }
    #[cfg(unix)]
    {
        let module = dir.join("new/rule12.lit");
        support::chmod(&module, 0o000);
        let run = compat(&dir);
        support::chmod(&module, 0o644);
        assert_eq!(run.code, 1, "{}", run.stdout);
    }
}
