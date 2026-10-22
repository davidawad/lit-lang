//! Build the tree-sitter-litigate shared library for an editor. See
//! `litigate_editors::grammar`.

use anyhow::Result;
use clap::Parser;
use litigate_editors::grammar::{self, Editor};
use litigate_editors::support::find_program;
use std::path::PathBuf;

/// Build libtree-sitter-litigate for an editor's tree-sitter ABI.
#[derive(Parser)]
struct Args {
    /// Output directory (default: ~/.emacs.d/tree-sitter, Emacs's search dir).
    #[arg(long, default_value_os_t = default_out())]
    out: PathBuf,
    /// tree-sitter ABI to build for (default: ask `emacs`, else the
    /// committed parser's ABI).
    #[arg(long)]
    abi: Option<u32>,
}

/// `~/.emacs.d/tree-sitter` (relative to `.` when `HOME` is unset).
fn default_out() -> PathBuf {
    std::env::var_os("HOME")
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join(".emacs.d/tree-sitter")
}

fn main() -> Result<()> {
    let args = Args::parse();
    let abi = match args.abi {
        Some(abi) => abi,
        None => find_program("EMACS", "emacs")
            .and_then(|emacs| grammar::editor_abi(Editor::Emacs, &emacs).ok())
            .map_or_else(grammar::committed_abi, Ok)?,
    };
    let library = grammar::build(&args.out, abi)?;
    println!("{} (ABI {abi})", library.display());
    Ok(())
}
