//! The generated editor files (`just editor-queries`): they are current,
//! and two branches adding different keywords to the keyword table rebase
//! without a conflict, `.gitattributes` merging the table by `union` and
//! keeping ours of every generated file until regeneration.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_editors::support::{run, scratch_dir};
use litigate_editors::{generated, keywords, repo_root};
use std::path::Path;
use std::process::Command;

#[test]
fn generated_files_are_current() {
    let stale = generated::stale(&repo_root()).unwrap();
    assert!(
        stale.is_empty(),
        "stale generated editor files {stale:?}; run `just editor-queries`"
    );
}

/// `git` in `dir`, isolated from the user's and the system's config.
fn git(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args([
            "-c",
            "user.name=litigate-editors",
            "-c",
            "user.email=litigate-editors@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args);
    command
}

/// Copy `path` (relative) from the repository into `root`.
fn copy(root: &Path, path: &str) {
    let to = root.join(path);
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();
    std::fs::copy(repo_root().join(path), to).unwrap();
}

/// Append a keyword row to the table under `root`, regenerate, commit.
fn add_keyword(root: &Path, row: &str) {
    let table = root.join(keywords::TABLE);
    let mut text = std::fs::read_to_string(&table).unwrap();
    text.push_str(row);
    text.push('\n');
    std::fs::write(&table, text).unwrap();
    generated::write(root).unwrap();
    run(&mut git(root, &["commit", "-qam", row])).unwrap();
}

#[test]
fn two_branches_adding_keywords_rebase_without_a_conflict() {
    let root = scratch_dir(Path::new(env!("CARGO_TARGET_TMPDIR")), "keyword-rebase").unwrap();
    for path in [
        ".gitattributes",
        keywords::TABLE,
        keywords::TEMPLATE,
        "editors/tree-sitter-litigate/queries/indents.scm",
        "editors/tree-sitter-litigate/queries/locals.scm",
        "editors/tree-sitter-litigate/queries/injections.scm",
    ] {
        copy(&root, path);
    }
    std::fs::create_dir_all(root.join("editors/emacs")).unwrap();
    std::fs::create_dir_all(root.join("editors/helix/queries/litigate")).unwrap();
    let files = generated::write(&root).unwrap();
    assert_eq!(files.len(), 6, "{files:?}");
    run(&mut git(&root, &["init", "-q"])).unwrap();
    // What `just tree-sitter-merge-driver` registers.
    run(&mut git(
        &root,
        &["config", "merge.tsgenerate.driver", "true"],
    ))
    .unwrap();
    run(&mut git(&root, &["add", "-A"])).unwrap();
    run(&mut git(&root, &["commit", "-qm", "base"])).unwrap();

    // Both branches append to the same place in the table and so rewrite
    // the same lines of every generated file.
    run(&mut git(&root, &["checkout", "-qb", "a"])).unwrap();
    add_keyword(&root, "KwAlphaWord alpha_word clause");
    run(&mut git(&root, &["checkout", "-qb", "b", "main"])).unwrap();
    add_keyword(&root, "KwBravoWord bravo_word clause");

    let rebase = git(&root, &["rebase", "a"]).output().unwrap();
    assert!(
        rebase.status.success(),
        "rebase conflicted:\n{}{}",
        String::from_utf8_lossy(&rebase.stdout),
        String::from_utf8_lossy(&rebase.stderr)
    );
    let table = std::fs::read_to_string(root.join(keywords::TABLE)).unwrap();
    assert!(
        table.contains("KwAlphaWord alpha_word clause\n")
            && table.contains("KwBravoWord bravo_word clause\n"),
        "{table}"
    );

    // Regeneration (land's GENERATE step) brings in both keywords.
    generated::write(&root).unwrap();
    assert!(generated::stale(&root).unwrap().is_empty());
    let keyword_files = files.iter().filter(|path| {
        path.ends_with("highlights.scm") || path.extension().is_some_and(|ext| ext == "el")
    });
    for path in keyword_files {
        let text = std::fs::read_to_string(root.join(path)).unwrap();
        assert!(
            text.contains("\"alpha_word\"") && text.contains("\"bravo_word\""),
            "{}:\n{text}",
            path.display()
        );
    }
}
