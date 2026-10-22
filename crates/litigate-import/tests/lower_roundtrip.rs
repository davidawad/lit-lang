//! Bead acceptance, second half: `import` then `litigate build` (lowering)
//! gives JSON that `litigate diff` finds semantically equal to every
//! original pack, `links.json` and scenario (SPEC §8.6, §9.3).
//!
//! It drives the `litigate` binary through its frozen CLI contract, so it
//! needs no lowering API. Until `build` and `diff` exist (the lowering bead)
//! it skips with a message, logged to `TEST_SKIP_LOG` when that is set;
//! `roundtrip.rs` meanwhile proves the same equality through a CST oracle.
mod support;

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use support::{corpus, import_ok, repo_root};

/// `target/<profile>/litigate`, built alongside this test binary.
fn litigate() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.parent()?;
    let bin = dir.join(format!("litigate{}", std::env::consts::EXE_SUFFIX));
    bin.is_file().then_some(bin)
}

fn skip(reason: &str) {
    let line = format!("SKIP lower_roundtrip::import_then_lower_is_semantically_equal: {reason}\n");
    eprint!("{line}");
    if let Ok(log) = std::env::var("TEST_SKIP_LOG") {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
    }
}

/// Whether the binary implements `command` (its `--help` succeeds).
fn supports(bin: &Path, command: &str) -> bool {
    Command::new(bin)
        .args([command, "--help"])
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Runs `litigate ARGS --format json` and returns the envelope's `result`.
fn run(bin: &Path, args: &[&str]) -> Value {
    let out = Command::new(bin)
        .args(args)
        .args(["--format", "json"])
        .output()
        .unwrap_or_else(|e| panic!("litigate {args:?}: {e}"));
    let envelope: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    assert!(
        envelope["ok"] == Value::Bool(true),
        "litigate {args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    envelope["result"].clone()
}

#[test]
fn import_then_lower_is_semantically_equal() {
    let Some(bin) = litigate() else {
        return skip("the `litigate` binary is not built (cargo build -p litigate-cli)");
    };
    if !["lock", "build", "diff"].iter().all(|c| supports(&bin, c)) {
        return skip(
            "`litigate lock/build/diff` are not implemented yet (litigate-lower lands them)",
        );
    }
    let inputs = corpus();
    let root = std::env::temp_dir().join(format!("litigate-import-lower-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for file in import_ok(&inputs) {
        let path = root.join(&file.path);
        std::fs::create_dir_all(path.parent().unwrap_or(&root)).unwrap_or_default();
        std::fs::write(&path, &file.text).unwrap_or_default();
    }
    let built = root.join("build");
    let built_str = built.to_string_lossy().into_owned();
    let mut sources: Vec<PathBuf> = std::fs::read_dir(root.join("lit"))
        .map(|r| r.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    sources.sort();
    for dir in &sources {
        run(&bin, &["lock", &dir.to_string_lossy()]);
    }
    let mut cases: Vec<PathBuf> = std::fs::read_dir(root.join("cases"))
        .map(|r| r.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    cases.sort();
    let mut generated: Vec<String> = Vec::new();
    for target in sources.iter().chain(&cases) {
        let result = run(
            &bin,
            &["build", &target.to_string_lossy(), "--out", &built_str],
        );
        for kind in ["packs", "links", "scenarios"] {
            let paths = result["outputs"][kind]
                .as_array()
                .cloned()
                .unwrap_or_default();
            generated.extend(paths.iter().filter_map(Value::as_str).map(str::to_owned));
        }
    }
    for input in &inputs {
        let name = Path::new(&input.path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let output = generated
            .iter()
            .find(|g| {
                Path::new(g)
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy() == name)
            })
            .unwrap_or_else(|| panic!("{}: no lowered output among {generated:?}", input.path));
        let original = repo_root().join(&input.path);
        let diff = run(&bin, &["diff", output, &original.to_string_lossy()]);
        assert_eq!(
            diff["equal"],
            Value::Bool(true),
            "{}: {}",
            input.path,
            diff["changes"]
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}
