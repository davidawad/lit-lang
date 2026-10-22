//! Shared helpers: run the `litigate` binary in a directory and read its
//! JSON envelope with the wall-clock field zeroed.
// Test helpers: a panic here is the failed assertion.
#![allow(dead_code, clippy::unwrap_used)]
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The repository root.
pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// A fresh, empty scratch directory for `name`.
pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litigate-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Output of one run.
pub struct Run {
    /// The envelope (`Value::Null` when stdout is not JSON).
    pub json: Value,
    /// Raw stdout.
    pub stdout: String,
    /// Raw stderr.
    pub stderr: String,
    /// Exit code.
    pub code: i32,
}

/// Runs `litigate --format json ARGS` in `dir`.
pub fn litigate_in(dir: &Path, args: &[&str]) -> Run {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(dir)
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut json: Value = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    if json.get("elapsed_ms").is_some() {
        json["elapsed_ms"] = Value::from(0);
    }
    Run {
        json,
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

/// Runs `litigate --format json ARGS` at the repository root.
pub fn litigate(args: &[&str]) -> Run {
    litigate_in(&repo(), args)
}

/// Copies directory `from` to `to` recursively.
pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// `(code, severity)` of every diagnostic.
pub fn codes(json: &Value) -> Vec<(String, String)> {
    json["diagnostics"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|d| {
                    (
                        d["code"].as_str().unwrap_or_default().to_owned(),
                        d["severity"].as_str().unwrap_or_default().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
