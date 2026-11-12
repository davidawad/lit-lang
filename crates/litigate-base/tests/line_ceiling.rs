//! Enforces the repository source-file size ratchet and its growth behavior.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const CEILING: usize = 500;

fn parse_list(text: &str) -> Result<BTreeMap<String, usize>, String> {
    let mut entries = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (count, path) = line
            .split_once(' ')
            .ok_or_else(|| format!("line {}: expected COUNT PATH", index + 1))?;
        let count = count
            .parse::<usize>()
            .map_err(|_| format!("line {}: invalid count", index + 1))?;
        if count <= CEILING {
            return Err(format!(
                "line {}: listed count must exceed {CEILING}",
                index + 1
            ));
        }
        if entries.insert(path.to_owned(), count).is_some() {
            return Err(format!("line {}: duplicate path", index + 1));
        }
    }
    Ok(entries)
}

fn rust_sources(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            rust_sources(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn violations(root: &Path, list: &str) -> Result<Vec<String>, String> {
    let listed = parse_list(list)?;
    let mut sources = Vec::new();
    for package in fs::read_dir(root.join("crates")).map_err(|e| e.to_string())? {
        let source = package.map_err(|e| e.to_string())?.path().join("src");
        if source.is_dir() {
            rust_sources(&source, &mut sources)?;
        }
    }
    let mut errors = Vec::new();
    for path in sources {
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let actual = fs::read_to_string(&path)
            .map_err(|e| format!("{relative}: {e}"))?
            .lines()
            .count();
        match listed.get(&relative) {
            Some(maximum) if actual > *maximum => {
                errors.push(format!("{relative}: {actual} lines (ratchet {maximum})"));
            }
            None if actual > CEILING => {
                errors.push(format!("{relative}: {actual} lines (ceiling {CEILING})"));
            }
            _ => {}
        }
    }
    Ok(errors)
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn committed_tree_respects_line_ceiling_ratchet() {
    let root = root();
    let list = fs::read_to_string(root.join("crates/.line-ceiling")).unwrap();
    let errors = violations(&root, &list).unwrap();
    assert!(
        errors.is_empty(),
        "line ceiling violations:\n{}",
        errors.join("\n")
    );
}

#[test]
fn rejects_unlisted_501_line_file() {
    let directory = tempfile_root("unlisted");
    fs::create_dir_all(directory.join("crates/test/src")).unwrap();
    fs::write(
        directory.join("crates/test/src/too_long.rs"),
        "x\n".repeat(501),
    )
    .unwrap();
    let errors = violations(&directory, "").unwrap();
    assert_eq!(
        errors,
        ["crates/test/src/too_long.rs: 501 lines (ceiling 500)"]
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_growth_past_listed_count() {
    let directory = tempfile_root("growth");
    fs::create_dir_all(directory.join("crates/test/src")).unwrap();
    fs::write(directory.join("crates/test/src/long.rs"), "x\n".repeat(502)).unwrap();
    let errors = violations(&directory, "501 crates/test/src/long.rs\n").unwrap();
    assert_eq!(errors, ["crates/test/src/long.rs: 502 lines (ratchet 501)"]);
    fs::remove_dir_all(directory).unwrap();
}

fn tempfile_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "litgraph-line-ceiling-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}
