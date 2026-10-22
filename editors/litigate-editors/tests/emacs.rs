//! Emacs batch test for `editors/emacs/litigate-ts-mode.el`: builds the
//! grammar for the installed Emacs's ABI, byte-compiles the mode with
//! warnings as errors, then runs the ERT suite
//! (`editors/emacs/test/litigate-ts-mode-test.el`), which loads every
//! corpus file in the mode. Skipped, and reported, when Emacs is missing
//! or lacks tree-sitter.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_editors::grammar::{self, Editor};
use litigate_editors::support::{find_program, run, scratch_dir, skip};
use litigate_editors::{editors_dir, repo_root};
use std::path::Path;
use std::process::Command;

const TEST: &str = "emacs::ert_suite_passes_on_every_corpus_file";

#[test]
fn ert_suite_passes_on_every_corpus_file() {
    let Some(emacs) = find_program("EMACS", "emacs") else {
        skip(TEST, "emacs not on PATH");
        return;
    };
    let Ok(abi) = grammar::editor_abi(Editor::Emacs, &emacs) else {
        skip(
            TEST,
            "emacs lacks tree-sitter support (needs 29.1+ built with tree-sitter)",
        );
        return;
    };
    let work = scratch_dir(Path::new(env!("CARGO_TARGET_TMPDIR")), "emacs").unwrap();
    let library = grammar::build(&work, abi).unwrap();
    let mode_dir = editors_dir().join("emacs");
    let test_dir = mode_dir.join("test");
    let elc = work.join("litigate-ts-mode.elc");
    let emacs_command = || {
        let mut command = Command::new(&emacs);
        command
            .env("LITIGATE_REPO", repo_root())
            .env("LITIGATE_TS_GRAMMAR_DIR", library.parent().unwrap())
            .args(["--batch", "-Q", "-L"])
            .arg(&mode_dir)
            .arg("-L")
            .arg(&test_dir);
        command
    };
    run(emacs_command()
        .arg("--eval")
        .arg(format!(
            "(setq byte-compile-error-on-warn t byte-compile-dest-file-function (lambda (_) {:?}))",
            elc.display().to_string()
        ))
        .args(["-f", "batch-byte-compile"])
        .arg(mode_dir.join("litigate-ts-mode.el")))
    .unwrap();
    assert!(elc.is_file(), "byte-compilation wrote no {}", elc.display());
    run(emacs_command().args([
        "-l",
        "litigate-ts-mode-test",
        "-f",
        "ert-run-tests-batch-and-exit",
    ]))
    .unwrap();
}
