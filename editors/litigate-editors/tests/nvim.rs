//! Headless Neovim test for `editors/nvim`: builds the parser for the
//! installed Neovim's ABI and runs `editors/nvim/test/litigate_test.lua`
//! (filetype detection, ftplugin, every corpus file parses and highlights,
//! every query compiles, the tree-sitter highlight assertions hold, folds,
//! the LSP config, nvim-treesitter registration). Skipped, and reported,
//! when nvim is missing or older than 0.10.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_editors::grammar::{self, Editor};
use litigate_editors::support::{find_program, run, scratch_dir, skip};
use litigate_editors::{editors_dir, repo_root};
use std::path::Path;
use std::process::Command;

const TEST: &str = "nvim::headless_suite_passes";

#[test]
fn headless_suite_passes() {
    let Some(nvim) = find_program("NVIM", "nvim") else {
        skip(TEST, "nvim not on PATH");
        return;
    };
    let Ok(abi) = grammar::editor_abi(Editor::Neovim, &nvim) else {
        skip(TEST, "nvim older than 0.10");
        return;
    };
    let work = scratch_dir(Path::new(env!("CARGO_TARGET_TMPDIR")), "nvim").unwrap();
    let library = grammar::build(&work, abi).unwrap();
    let nvim_dir = editors_dir().join("nvim");
    run(Command::new(&nvim)
        .env("LITIGATE_REPO", repo_root())
        .env("LITIGATE_PARSER", &library)
        .env("XDG_STATE_HOME", work.join("state"))
        .env("XDG_DATA_HOME", work.join("data"))
        .args(["--headless", "--clean", "--cmd"])
        .arg(format!("set rtp^={}", nvim_dir.display()))
        .arg("-c")
        .arg(format!(
            "luafile {}",
            nvim_dir.join("test/litigate_test.lua").display()
        )))
    .unwrap();
}
