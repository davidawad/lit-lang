//! `litigate lsp --stdio` hands the process to the `litigate-lsp` server;
//! any other arguments are a usage error.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::process::Output;

fn lsp(args: &[&str], path: &std::path::Path) -> Output {
    assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .arg("lsp")
        .args(args)
        .env("PATH", path)
        .output()
        .unwrap()
}

#[test]
fn lsp_without_stdio_is_a_usage_error() {
    let out = lsp(&["--tcp"], &support::scratch("lsp-usage"));
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    insta::assert_snapshot!(String::from_utf8_lossy(&out.stderr), @"usage: litigate lsp --stdio");
}

#[cfg(unix)]
#[test]
fn lsp_runs_the_server_and_passes_its_exit_code_through() {
    let dir = support::scratch("lsp-path");
    let server = dir.join("litigate-lsp");
    std::fs::write(
        &server,
        "#!/bin/sh\n[ \"$1\" = --stdio ] && exit 7\nexit 9\n",
    )
    .unwrap();
    support::chmod(&server, 0o755);
    let out = lsp(&["--stdio"], &dir);
    let beside = assert_cmd::cargo::cargo_bin("litigate").with_file_name("litigate-lsp");
    if !beside.is_file() {
        // No server was built beside the binary, so `PATH`'s answers.
        assert_eq!(out.status.code(), Some(7));
    }
}

#[test]
fn lsp_reports_a_server_it_cannot_start() {
    let out = lsp(&["--stdio"], &support::scratch("lsp-missing"));
    let beside = assert_cmd::cargo::cargo_bin("litigate").with_file_name("litigate-lsp");
    if !beside.is_file() {
        assert_eq!(out.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.starts_with("litigate lsp: cannot run `litigate-lsp`"),
            "{stderr}"
        );
    }
}
