//! `check`, `build`, `run` and the op subcommands over an
//! in-memory copy of the repository's `lit/` and `cases/` print exactly
//! what the binary prints over the disk (the wall-clock `elapsed_ms`
//! aside), and `build` returns the files the binary writes.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::{LitigateRun, IN_MEMORY_COMMANDS};
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

fn example_files() -> BTreeMap<String, String> {
    let root = repo();
    let mut files = BTreeMap::new();
    tree(&root, &root.join("lit/examples"), &mut files);
    tree(&root, &root.join("cases/examples"), &mut files);
    files
}

/// `stdout` with the `elapsed_ms` of a JSON envelope or engine response,
/// and of the engine response an envelope carries as its result, zeroed.
fn untimed(stdout: &str) -> String {
    match serde_json::from_str::<Value>(stdout) {
        Ok(mut envelope) if envelope.get("elapsed_ms").is_some() => {
            envelope["elapsed_ms"] = Value::from(0);
            if envelope["result"].get("elapsed_ms").is_some() {
                envelope["result"]["elapsed_ms"] = Value::from(0);
            }
            envelope.to_string()
        }
        _ => stdout.to_owned(),
    }
}

/// The today both sides judge deadlines at (lg-2ie2): pinned, never the
/// wall clock, and later than the matters' `as_of`s, so `check` runs the
/// judge-at-today path (L4016, `judged_at`) on both.
const TODAY: &str = "2026-10-27";

/// The binary at the repository root, and the same arguments in memory,
/// both judging deadlines at [`TODAY`].
fn both_with(files: BTreeMap<String, String>, args: &[&str]) -> (LitigateRun, LitigateRun) {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .env("LITIGATE_TODAY", TODAY)
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
    let mut memory = litigate_cli::execute_in_memory(files, &args, Some(TODAY));
    memory.stdout = untimed(&memory.stdout);
    (native, memory)
}

fn both(args: &[&str]) -> (LitigateRun, LitigateRun) {
    both_with(repo_files(), args)
}

#[test]
fn check_prints_what_the_binary_prints() {
    for args in [
        &["--format", "json", "check", "cases/examples/acme.case"][..],
        &[
            "--format",
            "json",
            "check",
            "lit/examples/Lit.toml",
            "lit/examples/rule12.lit",
        ],
        &[
            "--format",
            "json",
            "check",
            "--lib",
            "lit/examples",
            "cases/examples",
        ],
        &["--format", "json", "check", "lit/examples/missing"],
        &["--format", "json", "check", "lit/examples/Lit.lock"],
    ] {
        let (native, memory) = both_with(example_files(), args);
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
fn ops_compile_the_matter_or_package_in_memory() {
    for args in [
        &["--format", "json", "solve", "cases/frcp-civil-case.case"][..],
        &[
            "--format",
            "json",
            "run",
            "cases/cofc-1498-patent-case.case",
            "--op",
            "chain",
            "--set",
            "rate=900",
        ],
        &["--format", "json", "chain", "lit/cofc"],
        &["--format", "json", "solve", "lit/links"],
        &["--format", "json", "solve", "lit/nope"],
    ] {
        let (native, memory) = both(args);
        assert_eq!(memory, native, "{args:?}");
    }
    let (native, memory) = both_with(
        example_files(),
        &["--format", "json", "solve", "cases/examples/acme.case"],
    );
    let envelope: Value = serde_json::from_str(&memory.stdout).unwrap();
    assert_eq!(
        (native.code, &envelope["result"]["ok"]),
        (0, &Value::from(true))
    );
}

#[test]
fn request_ops_run_on_compiled_source_in_memory() {
    // The ops the raw engine interface served (lg-dfv4.7): the manual,
    // the pack list and a solve over the whole library.
    for args in [
        &["run", "cases/examples/acme.case", "--op", "solve"][..],
        &["run", "lit/examples", "--op", "describe"],
        &["run", "cases/examples/acme.case", "--op", "packs"],
        &[
            "run",
            "cases/examples/acme.case",
            "--op",
            "validate",
            "--set",
            "rate=1",
        ],
    ] {
        let (native, memory) =
            both_with(example_files(), &[&["--format", "json"][..], args].concat());
        assert_eq!(memory, native, "{args:?}");
        assert_eq!(native.code, 0, "{args:?}: {}", native.stdout);
    }
    let gone =
        litigate_cli::execute_in_memory(repo_files(), &["engine".into(), "describe".into()], None);
    assert_eq!((gone.code, gone.stdout.as_str()), (1, ""));
}

#[test]
fn other_commands_do_not_run_in_memory() {
    let run =
        litigate_cli::execute_in_memory(repo_files(), &["explain".into(), "L3004".into()], None);
    assert_eq!(run.code, 1);
    assert_eq!(
        run.stderr,
        format!(
            "litigate explain: only {} and the op subcommands run over an in-memory source\n",
            IN_MEMORY_COMMANDS.join(", ")
        )
    );
    let usage = litigate_cli::execute_in_memory([], &["frobnicate".into()], None);
    assert_eq!((usage.code, usage.stdout.as_str()), (1, ""));
}
