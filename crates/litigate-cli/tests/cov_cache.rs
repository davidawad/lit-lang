//! The verification cache (SPEC §4.2) through the binary: `check` records
//! each package's verification in the build cache, keyed by the Merkle
//! root of its resolution, so a dependency's digest change misses its
//! dependents' entries; with `CI` set nothing is read or written.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::fmt::Write as _;
use std::path::Path;
use support::scratch;

fn manifest(name: &str, deps: &[&str]) -> String {
    let mut text = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\nforum = \"demo\"\nauthority = \"FederalRule\"\ntitle = \"T\"\nin_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\nCourt = \"nature\"\n"
    );
    if !deps.is_empty() {
        text.push_str("\n[dependencies]\n");
        for d in deps {
            let _ = writeln!(text, "{d} = {{ version = \"^1.0.0\", path = \"../{d}\" }}");
        }
    }
    text
}

/// `lit/dep` and `lit/app` (depending on `dep`), locked.
fn tree(name: &str) -> std::path::PathBuf {
    let dir = scratch(name);
    for (pkg, deps) in [("dep", &[][..]), ("app", &["dep"][..])] {
        let root = dir.join("lit").join(pkg);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Lit.toml"), manifest(pkg, deps)).unwrap();
        std::fs::write(
            root.join(format!("{pkg}.lit")),
            format!("module {pkg}\nstart s\nterminal s: \"S\"\n"),
        )
        .unwrap();
    }
    assert_eq!(litigate(&dir, None, &["lock", "lit/app"]).code, 0);
    dir
}

/// `litigate --format json ARGS` in `dir` with the cache in `cache/` and
/// `CI` as given.
fn litigate(dir: &Path, ci: Option<&str>, args: &[&str]) -> support::Run {
    litigate_as(&assert_cmd::cargo::cargo_bin("litigate"), dir, ci, args)
}

/// [`litigate`] running the executable at `bin`.
fn litigate_as(bin: &Path, dir: &Path, ci: Option<&str>, args: &[&str]) -> support::Run {
    let mut cmd = assert_cmd::Command::new(bin);
    cmd.current_dir(dir)
        .env("LITIGATE_CACHE", "cache")
        .env_remove("CI");
    if let Some(ci) = ci {
        cmd.env("CI", ci);
    }
    let out = cmd.args(["--format", "json"]).args(args).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();
    json["elapsed_ms"] = 0.into();
    support::Run {
        json,
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("cache"))
        .map(|d| {
            d.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_dependency_digest_change_misses_its_dependents_entries() {
    let dir = tree("cache-deps");
    let first = litigate(&dir, None, &["check", "lit/app"]);
    assert_eq!(first.code, 0, "{}", first.stdout);
    let cached = entries(&dir);
    assert_eq!(cached.len(), 2, "{cached:?}");
    // A rerun reads the entries: same answer, nothing new.
    let again = litigate(&dir, None, &["check", "lit/app"]);
    assert_eq!(again.json, first.json);
    assert_eq!(entries(&dir), cached);
    // `dep` changes: its own entry and `app`'s both miss; `app`'s lock is
    // stale (L1007) until relocked, never relocked silently.
    std::fs::write(
        dir.join("lit/dep/dep.lit"),
        "module dep\nstart s\nterminal s: \"Changed\"\n",
    )
    .unwrap();
    let stale = litigate(&dir, None, &["check", "lit/app"]);
    assert_eq!(stale.code, 2, "{}", stale.stdout);
    assert_eq!(stale.json["diagnostics"][0]["code"], "L1007");
    assert_eq!(entries(&dir).len(), 4);
    assert_eq!(litigate(&dir, None, &["lock", "lit/app"]).code, 0);
    assert_eq!(litigate(&dir, None, &["check", "lit/app"]).code, 0);
    assert_eq!(entries(&dir).len(), 4);
}

#[test]
fn ci_neither_reads_nor_writes_the_cache() {
    let dir = tree("cache-ci");
    let run = litigate(&dir, Some("true"), &["check", "lit/app"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(entries(&dir), Vec::<String>::new());
    // A planted entry would be read outside CI; CI ignores it.
    let local = litigate(&dir, None, &["check", "lit/app"]);
    for name in entries(&dir) {
        std::fs::write(
            dir.join("cache").join(name),
            r#"{"report":{"obligations":[],"profile":"dev"},"diagnostics":[]}"#,
        )
        .unwrap();
    }
    let planted = litigate(&dir, None, &["check", "lit/app"]);
    assert_ne!(
        planted.json["result"]["obligations"],
        local.json["result"]["obligations"]
    );
    let fresh = litigate(&dir, Some("1"), &["check", "lit/app"]);
    assert_eq!(fresh.json, local.json);
}

/// Writes `bytes` as the executable `bin` modified at `mtime`, as an
/// install over the previous binary does.
fn install(bin: &Path, bytes: &[u8], mtime: std::time::SystemTime) {
    use std::os::unix::fs::PermissionsExt as _;
    let tmp = bin.with_extension("new");
    std::fs::write(&tmp, bytes).unwrap();
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&tmp)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    std::fs::rename(&tmp, bin).unwrap();
}

#[test]
fn an_upgraded_compiler_never_reads_the_old_ones_entries() {
    let dir = tree("cache-upgrade");
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    let bin = dir.join("bin/litigate");
    let built = std::fs::read(assert_cmd::cargo::cargo_bin("litigate")).unwrap();
    let mtime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1);
    install(&bin, &built, mtime);
    let first = litigate_as(&bin, &dir, None, &["check", "lit/app"]);
    assert_eq!(first.code, 0, "{}", first.stdout);
    let cached = entries(&dir);
    assert_eq!(cached.len(), 2, "{cached:?}");
    // What the old compiler concluded, stored: an L1007 its successor no
    // longer finds. The old compiler reads its own entries back.
    let planted = r#"{"report":{"obligations":[],"profile":"dev"},"diagnostics":[{"code":"L1007","severity":"error","span":{"file":"lit/app/Lit.lock","line":1,"col":1,"end_line":1,"end_col":1},"message":"Lit.lock does not match the resolved package graph"}]}"#;
    for name in &cached {
        std::fs::write(dir.join("cache").join(name), planted).unwrap();
    }
    let old = litigate_as(&bin, &dir, None, &["check", "lit/app"]);
    assert_eq!(old.code, 2, "{}", old.stdout);
    assert_eq!(old.json["diagnostics"][0]["code"], "L1007");
    // The upgrade: other bytes (in the unloaded `.comment` section) of the
    // same size, installed with the same modification time over the same
    // tree. It verifies afresh: no L1007, and entries of its own.
    let mut upgraded = built.clone();
    let at = upgraded
        .windows(b"rustc version".len())
        .position(|w| w == b"rustc version")
        .unwrap();
    upgraded[at] = b'R';
    install(&bin, &upgraded, mtime);
    let new = litigate_as(&bin, &dir, None, &["check", "lit/app"]);
    assert_eq!(new.code, 0, "{}", new.stdout);
    assert_eq!(new.json, first.json);
    assert_eq!(entries(&dir).len(), 4);
    assert_eq!(
        litigate_as(&bin, &dir, None, &["lock", "--locked", "lit/app"]).code,
        0
    );
}
