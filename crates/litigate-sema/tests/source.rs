//! The [`Source`] implementations, and loading package trees from memory.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
use litigate_sema::pkg::load::load_tree_in;
use litigate_sema::pkg::source::{Disk, MemoryTree, Source};
use std::io;
use std::path::{Path, PathBuf};

fn tree() -> MemoryTree {
    MemoryTree::new([
        ("lit/a/Lit.toml", "a".to_owned()),
        ("./lit/a/x.lit", "x".to_owned()),
        ("/cases/c.case", "c".to_owned()),
        ("lit/b/../ab", "ab".to_owned()),
    ])
}

#[test]
fn paths_normalize_against_the_root() {
    let t = tree();
    assert_eq!(t.root().ok(), Some(PathBuf::from("/")));
    for path in [
        "lit/a/x.lit",
        "/lit/a/x.lit",
        "lit/./a/x.lit",
        "../lit/a/x.lit",
    ] {
        assert_eq!(t.read_to_string(Path::new(path)).ok().as_deref(), Some("x"));
    }
    assert_eq!(t.read(Path::new("lit/ab")).ok(), Some(b"ab".to_vec()));
    assert_eq!(
        t.canonicalize(Path::new("lit/./a")).ok(),
        Some(PathBuf::from("/lit/a"))
    );
    assert_eq!(t.canonicalize(Path::new("")).ok(), Some(PathBuf::from("/")));
}

#[test]
fn directories_are_implied_by_their_files() {
    let t = tree();
    for dir in ["", "/", "lit", "lit/a", "cases"] {
        assert!(
            t.is_dir(Path::new(dir)) && t.exists(Path::new(dir)),
            "{dir}"
        );
    }
    assert!(!t.is_dir(Path::new("lit/a/x.lit")));
    assert!(!t.is_dir(Path::new("li")));
    assert!(t.is_file(Path::new("cases/c.case")) && !t.is_file(Path::new("cases")));
    assert!(!t.exists(Path::new("nope")));
    let mut entries = t.entries(Path::new("lit")).unwrap_or_default();
    entries.sort();
    assert_eq!(entries, [PathBuf::from("lit/a"), PathBuf::from("lit/ab")]);
    assert_eq!(
        t.entries(Path::new("/")).unwrap_or_default(),
        [PathBuf::from("/cases"), PathBuf::from("/lit")]
    );
}

#[test]
fn missing_paths_fail_with_their_name() {
    let t = tree();
    let message = |r: io::Result<()>| r.err().map(|e| (e.kind(), e.to_string()));
    assert_eq!(
        message(t.read(Path::new("lit/none")).map(drop)),
        Some((
            io::ErrorKind::NotFound,
            "lit/none: no such file or directory".to_owned()
        ))
    );
    assert!(t.entries(Path::new("cases/c.case")).is_err());
    assert!(t.canonicalize(Path::new("nope")).is_err());
}

#[test]
fn writes_are_collected_apart_from_the_tree() {
    let t = tree();
    assert!(t.write_atomic(Path::new("/out/./p.json"), "1").is_ok());
    assert!(t.write_atomic(Path::new("out/p.json"), "2").is_ok());
    assert_eq!(
        t.written().into_iter().collect::<Vec<_>>(),
        [("out/p.json".to_owned(), "2".to_owned())]
    );
    assert!(!t.exists(Path::new("out")));
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litigate-source-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn disk_is_the_working_directory() {
    let dir = scratch("disk");
    assert_eq!(Disk.root().ok(), std::env::current_dir().ok());
    assert!(Disk.on_disk());
    assert!(!MemoryTree::new(Vec::<(String, String)>::new()).on_disk());
    let file = dir.join("a/b.txt");
    assert!(Disk.write_atomic(&file, "text").is_ok());
    assert_eq!(Disk.read(&file).ok(), Some(b"text".to_vec()));
    assert_eq!(Disk.read_to_string(&file).ok().as_deref(), Some("text"));
    assert_eq!(Disk.entries(&dir).ok(), Some(vec![dir.join("a")]));
    assert!(Disk.is_dir(&dir) && Disk.is_file(&file) && Disk.exists(&file));
    assert_eq!(Disk.canonicalize(&file).ok(), file.canonicalize().ok());
}

#[test]
fn disk_writes_name_each_failing_step() {
    let dir = scratch("fail");
    let failure = |path: &Path| {
        Disk.write_atomic(path, "text")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    };
    assert!(Disk
        .write_atomic(Path::new("relative-name.litigate-test"), "")
        .is_ok());
    let _ = std::fs::remove_file("relative-name.litigate-test");
    std::fs::write(dir.join("file"), "").unwrap();
    let shown = dir.join("file").display().to_string();
    assert!(failure(&dir.join("file/x.json")).starts_with(&format!("{shown}: ")));
    std::fs::create_dir_all(dir.join("t.litigate-tmp")).unwrap();
    let shown = dir.join("t.litigate-tmp").display().to_string();
    assert!(failure(&dir.join("t.json")).starts_with(&format!("{shown}: ")));
    std::fs::create_dir_all(dir.join("d/inner")).unwrap();
    let shown = dir.join("d").display().to_string();
    assert!(failure(&dir.join("d")).starts_with(&format!("{shown}: ")));
}

#[test]
fn loads_from_memory() {
    let example = include_str!("../../../lit/examples/Lit.toml");
    let manifest = |name: &str, deps: &str| {
        let text = example.replace("\"frcp-rule12-example\"", &format!("\"{name}\""));
        format!("{text}\n[dependencies]\n{deps}")
    };
    let tree = MemoryTree::new([
        (
            "app/Lit.toml",
            manifest(
                "app",
                "dep = { version = \"^1.0.0\", path = \"../dep\" }\nghost = { version = \"^1.0.0\", path = \"../ghost\" }\n",
            ),
        ),
        ("app/Lit.lock", "version = 1\n".to_owned()),
        ("app/b.lit", "b".to_owned()),
        ("app/sub/a.lit", "a".to_owned()),
        ("app/sub/notes.txt", String::new()),
        ("app/.hidden/h.lit", String::new()),
        ("app/nested/Lit.toml", manifest("nested", "")),
        ("dep/Lit.toml", manifest("dep", "")),
    ]);
    let pkgs =
        load_tree_in(&tree, Path::new("/"), &[PathBuf::from("/app")], None).unwrap_or_default();
    let summary: Vec<(String, Vec<String>, bool)> = pkgs
        .into_iter()
        .map(|p| {
            (
                p.root,
                p.files.into_iter().map(|f| f.relative).collect(),
                p.lock.is_some(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "app".to_owned(),
                vec!["b.lit".to_owned(), "sub/a.lit".to_owned()],
                true
            ),
            ("app/../dep".to_owned(), vec![], false),
        ]
    );
}
