//! Installation and workspace diagnostics (`litigate doctor`).
use serde_json::{json, Value};
use std::env;
use std::path::{Path, PathBuf};

/// Collects installation details and available source package files.
#[must_use]
pub fn doctor() -> Value {
    let cwd = workspace_root(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let packs = files_with_extension(&cwd.join("lit"), "lit");
    let cases = files_with_extension(&cwd.join("cases"), "case");
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "git_sha": match env!("LITIGATE_GIT_COMMIT") { "unknown" => Value::Null, sha => json!(sha) },
        "platform": env::consts::OS.to_owned() + "/" + env::consts::ARCH,
        "packs": packs,
        "cases": cases,
        "binaries": {
            "litigate": binary_on_path("litigate"),
            "litigate-lsp": binary_on_path("litigate-lsp"),
        }
    })
}

fn workspace_root(mut path: PathBuf) -> PathBuf {
    loop {
        if path.join("lit").is_dir() && path.join("cases").is_dir() {
            return path;
        }
        if !path.pop() {
            return PathBuf::from(".");
        }
    }
}

fn files_with_extension(root: &Path, extension: &str) -> Vec<String> {
    fn visit(dir: &Path, extension: &str, found: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, extension, found);
            } else if path.extension().is_some_and(|ext| ext == extension) {
                found.push(path.to_string_lossy().into_owned());
            }
        }
    }
    let mut found = Vec::new();
    visit(root, extension, &mut found);
    found.sort();
    found
}

fn binary_on_path(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else { return false };
    env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file() || (cfg!(windows) && dir.join(format!("{name}.exe")).is_file())
    })
}
