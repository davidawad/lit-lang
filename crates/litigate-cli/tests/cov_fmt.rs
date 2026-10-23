//! `fmt` over modules and matters: files that do not parse are reported,
//! not rewritten; unreadable and unwritable files fail.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{example, litigate_in, located};

#[test]
fn fmt_formats_matters_and_reports_files_that_do_not_parse() {
    let dir = example("fmt-parse", "", None);
    let case = dir.join("cases/examples/acme.case");
    let text = std::fs::read_to_string(&case).unwrap();
    std::fs::write(
        &case,
        text.replace("perspective widget", "perspective   widget"),
    )
    .unwrap();
    std::fs::write(
        dir.join("lit/examples/broken.lit"),
        "module m\nstate s: \"S\" {\n",
    )
    .unwrap();
    let run = litigate_in(&dir, &["fmt", "lit/examples", "cases/examples"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_debug_snapshot!(located(&run.json));
    assert_eq!(std::fs::read_to_string(&case).unwrap(), text);
}

#[cfg(unix)]
#[test]
fn unreadable_and_unwritable_files_fail_fmt() {
    let dir = example("fmt-io", "", None);
    let module = dir.join("lit/examples/rule12.lit");
    support::chmod(&module, 0o000);
    let unreadable = litigate_in(&dir, &["fmt", "lit/examples"]);
    support::chmod(&module, 0o644);
    assert_eq!(unreadable.code, 1);
    insta::assert_snapshot!(unreadable.stderr, @"litigate fmt: lit/examples/rule12.lit: Permission denied (os error 13)");
    let text = std::fs::read_to_string(&module).unwrap();
    std::fs::write(
        &module,
        text.replace("start time-barred", "start   time-barred"),
    )
    .unwrap();
    support::chmod(&dir.join("lit/examples"), 0o555);
    let unwritable = litigate_in(&dir, &["fmt", "lit/examples"]);
    support::chmod(&dir.join("lit/examples"), 0o755);
    assert_eq!(unwritable.code, 1);
    insta::assert_snapshot!(unwritable.stderr, @"litigate fmt: lit/examples/rule12.litigate-tmp: Permission denied (os error 13)");
}
