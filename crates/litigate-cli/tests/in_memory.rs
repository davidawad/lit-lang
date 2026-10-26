//! `check` and `build` over an in-memory copy of the repository's `lit/`
//! and `cases/` print exactly what the binary prints over the disk (the
//! wall-clock `elapsed_ms` aside), and `build` returns the files the
//! binary writes.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::LitigateRun;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use support::{repo, scratch};

/// Every file under `dir`, keyed by its path relative to `base`.
fn tree(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            tree(base, &path, out);
        } else {
            let rel = path.strip_prefix(base).unwrap().to_string_lossy();
            out.insert(
                rel.replace('\\', "/"),
                std::fs::read_to_string(&path).unwrap(),
            );
        }
    }
}

fn repo_files() -> BTreeMap<String, String> {
    let root = repo();
    let mut files = BTreeMap::new();
    tree(&root, &root.join("lit"), &mut files);
    tree(&root, &root.join("cases"), &mut files);
    files
}

/// `stdout` with a JSON envelope's `elapsed_ms` zeroed.
fn untimed(stdout: &str) -> String {
    match serde_json::from_str::<Value>(stdout) {
        Ok(mut envelope) if envelope.get("elapsed_ms").is_some() => {
            envelope["elapsed_ms"] = Value::from(0);
            envelope.to_string()
        }
        _ => stdout.to_owned(),
    }
}

/// The binary at the repository root, and the same arguments in memory.
fn both(args: &[&str]) -> (LitigateRun, LitigateRun) {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(repo())
        .args(args)
        .output()
        .unwrap();
    let native = LitigateRun {
        code: u8::try_from(out.status.code().unwrap()).unwrap(),
        stdout: untimed(&String::from_utf8(out.stdout).unwrap()),
        stderr: String::from_utf8(out.stderr).unwrap(),
        files: BTreeMap::new(),
    };
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let mut memory = litigate_cli::execute_in_memory(repo_files(), &args);
    memory.stdout = untimed(&memory.stdout);
    (native, memory)
}

#[test]
fn check_prints_what_the_binary_prints() {
    for args in [
        &["--format", "json", "check", "lit", "cases"][..],
        &[
            "--format",
            "json",
            "check",
            "--profile",
            "strict",
            "lit",
            "cases",
        ],
        &["--format", "human", "check", "lit", "cases"],
        &["--format", "json", "check", "cases/frcp-civil-case.case"],
        &[
            "--format",
            "json",
            "check",
            "lit/frcp/Lit.toml",
            "lit/examples/rule12.lit",
        ],
        &[
            "--format",
            "json",
            "check",
            "--lib",
            "lit",
            "cases/examples",
        ],
        &["--format", "json", "check", "lit/nope"],
        &["--format", "json", "check", "lit/frcp/Lit.lock"],
    ] {
        let (native, memory) = both(args);
        assert_eq!(memory, native, "{args:?}");
    }
}

#[test]
fn build_returns_the_files_the_binary_writes() {
    let out = scratch("in-memory-build");
    let out_arg = out.to_str().unwrap();
    let mut args = vec!["--format", "json", "build", "--out", out_arg];
    args.extend([
        "lit/cafc",
        "lit/cofc",
        "lit/frcp",
        "lit/links",
        "cases/cofc-1498-patent-case.case",
        "cases/frcp-civil-case.case",
        "--pack-file",
        "cafc=cafc-federal-circuit",
    ]);
    let (native, mut memory) = both(&args);
    assert_eq!(native.code, 0, "{}", native.stderr);
    let mut written = BTreeMap::new();
    tree(&out, &out, &mut written);
    let prefix = format!("{}/", out_arg.trim_start_matches('/'));
    let files = std::mem::take(&mut memory.files);
    let files: BTreeMap<String, String> = files
        .into_iter()
        .map(|(path, text)| (path.strip_prefix(&prefix).unwrap().to_owned(), text))
        .collect();
    assert!(files.contains_key("packs/cafc-federal-circuit.json"));
    assert_eq!(files, written);
    assert_eq!(memory, native);
}

#[test]
fn other_commands_do_not_run_in_memory() {
    let run = litigate_cli::execute_in_memory(repo_files(), &["explain".into(), "L3004".into()]);
    assert_eq!(run.code, 1);
    assert_eq!(
        run.stderr,
        "litigate explain: only check, build and worlds run over an in-memory source\n"
    );
    let usage = litigate_cli::execute_in_memory([], &["frobnicate".into()]);
    assert_eq!((usage.code, usage.stdout.as_str()), (1, ""));
}
