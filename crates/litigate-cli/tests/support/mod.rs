//! Shared helpers: run the `litigate` binary in a directory and read its
//! JSON envelope with the wall-clock field zeroed and the compiler version
//! redacted.
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
        // Today before every fixture's `as_of`: each matter is judged at
        // its own, whatever the system date (lg-rpmf).
        .env("LITIGATE_TODAY", "2000-01-01")
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
    // Redact the compiler version so a release bump cannot stale snapshots.
    if let Some(version) = json.pointer_mut("/provenance/compiler_version") {
        assert_eq!(*version, env!("CARGO_PKG_VERSION"));
        *version = Value::from("[version]");
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

/// A scratch workspace holding the worked example: `lit/examples` with
/// `extra` appended to its module, and `cases/examples` with the matter
/// text replaced by `case` when given (its `Lit.lock` regenerated).
pub fn example(name: &str, extra: &str, case: Option<&str>) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&repo().join("lit/examples"), &dir.join("lit/examples"));
    copy_dir(&repo().join("cases/examples"), &dir.join("cases/examples"));
    let module = dir.join("lit/examples/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    std::fs::write(&module, format!("{text}\n{extra}\n")).unwrap();
    let package_lock = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(
        package_lock.code, 0,
        "{}{}",
        package_lock.stdout, package_lock.stderr
    );
    if let Some(case) = case {
        std::fs::write(dir.join("cases/examples/acme.case"), case).unwrap();
    }
    let lock = litigate_in(&dir, &["lock", "cases/examples"]);
    assert_eq!(lock.code, 0, "{}{}", lock.stdout, lock.stderr);
    dir
}

/// The worked example's matter text.
pub fn acme() -> String {
    std::fs::read_to_string(repo().join("cases/examples/acme.case")).unwrap()
}

/// Sets the Unix permission bits of `path`.
#[cfg(unix)]
pub fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// `(code, severity, file, line)` of every diagnostic.
pub fn located(json: &Value) -> Vec<(String, String, String, u64)> {
    json["diagnostics"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|d| {
                    (
                        d["code"].as_str().unwrap_or_default().to_owned(),
                        d["severity"].as_str().unwrap_or_default().to_owned(),
                        d["span"]["file"].as_str().unwrap_or_default().to_owned(),
                        d["span"]["line"].as_u64().unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A scratch workspace for filing limits set by layers (lg-962.10.3): the
/// `cov-cmds` fixture, whose `Answered` now files a `brief` (100 words, a
/// certificate, delegable to a local rule), `lit/core` and `lit/frap`, and
/// the `limit-layers` fixture's layer packages, matters and world, locked.
pub fn limit_layers(name: &str) -> PathBuf {
    let dir = scratch(name);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    copy_dir(&fixtures.join("cov-cmds"), &dir);
    copy_dir(&fixtures.join("limit-layers"), &dir);
    for pkg in ["core", "frap"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    // The copied locks pin the vendored L0 texts (SPEC §4.2).
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    let cov = dir.join("lit/cov/cov.lit");
    let text = std::fs::read_to_string(&cov).unwrap().replace(
        "event Answered(document: Document)\n",
        "event Answered(document: Document) paper brief\n",
    );
    let brief = "
limit brief delegable { to [LocalRule], op set } {
  words <= 100
  requires certificate compliance
}
";
    std::fs::write(&cov, format!("{text}{brief}")).unwrap();
    for target in [
        "lit/local",
        "lit/standing",
        "lit/frap-local",
        "lit/frap-standing",
        "cases",
    ] {
        let lock = litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{target}: {}{}", lock.stdout, lock.stderr);
    }
    dir
}
