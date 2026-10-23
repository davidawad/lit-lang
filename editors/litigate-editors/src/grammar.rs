//! Build `tree-sitter-litigate` as a shared library editors can load.
//!
//! The committed parser (`editors/tree-sitter-litigate/src/parser.c`) is
//! ABI 15. Editors linked against libtree-sitter older than 0.25 (Emacs on
//! most distributions, Neovim before 0.12) only load ABI 14 and below, so for
//! them the parser is regenerated from `grammar.js` in a scratch copy with
//! `tree-sitter generate --abi N`, using the pinned CLI from `PATH` or else
//! `npx`. The committed `src/` is never touched.

use crate::support::run;
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The tree-sitter CLI version the grammar is pinned to (`package.json`).
pub const TS_CLI_VERSION: &str = "0.27.1";

/// The editors whose tree-sitter ABI can be queried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Editor {
    /// GNU Emacs 29+ (`treesit-library-abi-version`).
    Emacs,
    /// Neovim 0.10+ (`vim.treesitter.language_version`).
    Neovim,
}

/// `editors/tree-sitter-litigate`.
#[must_use]
pub fn grammar_dir() -> PathBuf {
    crate::editors_dir().join("tree-sitter-litigate")
}

/// The ABI of the committed parser (`#define LANGUAGE_VERSION`).
///
/// # Errors
/// When `parser.c` cannot be read or has no `LANGUAGE_VERSION`.
pub fn committed_abi() -> Result<u32> {
    let parser = grammar_dir().join("src/parser.c");
    let text = std::fs::read_to_string(&parser)
        .with_context(|| format!("reading {}", parser.display()))?;
    text.lines()
        .find_map(|line| line.strip_prefix("#define LANGUAGE_VERSION "))
        .and_then(|value| value.trim().parse().ok())
        .with_context(|| format!("no LANGUAGE_VERSION in {}", parser.display()))
}

/// The highest tree-sitter ABI `editor` (the program at `program`) loads.
///
/// # Errors
/// When the editor cannot run or prints no ABI (too old, or built without
/// tree-sitter).
pub fn editor_abi(editor: Editor, program: &Path) -> Result<u32> {
    let mut command = Command::new(program);
    match editor {
        Editor::Emacs => command.args([
            "--batch",
            "--eval",
            "(princ (if (and (fboundp 'treesit-available-p) (treesit-available-p)) (treesit-library-abi-version) \"\"))",
        ]),
        Editor::Neovim => command.args([
            "--headless",
            "--clean",
            "-c",
            "lua if vim.fn.has('nvim-0.10') == 1 then io.stdout:write(vim.treesitter.language_version) end",
            "-c",
            "qa!",
        ]),
    };
    let output = command
        .output()
        .with_context(|| format!("running {}", program.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.trim().parse().with_context(|| {
        format!(
            "{editor:?} at {} reports no tree-sitter ABI",
            program.display()
        )
    })
}

/// The shared-library file name for this platform.
#[must_use]
pub fn library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libtree-sitter-litigate.dylib"
    } else if cfg!(target_os = "windows") {
        "libtree-sitter-litigate.dll"
    } else {
        "libtree-sitter-litigate.so"
    }
}

/// Build the grammar for `abi` into `out` and return the library's path.
///
/// # Errors
/// When regeneration or compilation fails.
pub fn build(out: &Path, abi: u32) -> Result<PathBuf> {
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let scratch = out.join(".litigate-grammar-build");
    let src = if abi >= committed_abi()? {
        grammar_dir().join("src")
    } else {
        regenerate(&scratch, abi)?
    };
    let library = out.join(library_name());
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let compiled = run(Command::new(compiler)
        .args(["-shared", "-fPIC", "-O2"])
        .arg(format!("-I{}", src.display()))
        .arg(src.join("parser.c"))
        .arg("-o")
        .arg(&library));
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch)
            .with_context(|| format!("removing {}", scratch.display()))?;
    }
    compiled?;
    Ok(library)
}

/// Regenerate the parser for `abi` into `scratch/src` and return that dir.
fn regenerate(scratch: &Path, abi: u32) -> Result<PathBuf> {
    let grammar = grammar_dir();
    if scratch.exists() {
        std::fs::remove_dir_all(scratch)?;
    }
    std::fs::create_dir_all(scratch.join("src"))?;
    std::fs::create_dir_all(scratch.join("grammar"))?;
    for file in ["grammar.js", "tree-sitter.json", "package.json"] {
        std::fs::copy(grammar.join(file), scratch.join(file))
            .with_context(|| format!("copying {file}"))?;
    }
    for entry in std::fs::read_dir(grammar.join("grammar"))? {
        let entry = entry?;
        std::fs::copy(
            entry.path(),
            scratch.join("grammar").join(entry.file_name()),
        )?;
    }
    let mut command = ts_cli();
    run(command
        .current_dir(scratch)
        .args(["generate", "--abi", &abi.to_string()]))
    .context("regenerating the parser needs node and tree-sitter-cli (or npx)")?;
    Ok(scratch.join("src"))
}

/// The pinned tree-sitter CLI: `tree-sitter` on `PATH` when it is the
/// pinned version, else `npx --yes tree-sitter-cli@<pinned>`.
#[must_use]
pub fn ts_cli() -> Command {
    let pinned = Command::new("tree-sitter")
        .arg("--version")
        .output()
        .is_ok_and(|out| {
            String::from_utf8_lossy(&out.stdout).contains(&format!(" {TS_CLI_VERSION}"))
        });
    if pinned {
        Command::new("tree-sitter")
    } else {
        let mut npx = Command::new("npx");
        npx.args(["--yes", &format!("tree-sitter-cli@{TS_CLI_VERSION}")]);
        npx
    }
}
