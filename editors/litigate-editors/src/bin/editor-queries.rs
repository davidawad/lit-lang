//! Write (or with `--check`, verify) every generated editor file: the
//! keyword lists from `crates/litigate-syntax/keywords.txt` and Helix's
//! queries. See `litigate_editors::generated`.

use anyhow::Result;
use clap::Parser;
use litigate_editors::{generated, repo_root};
use std::process::ExitCode;

/// Render the editor keyword lists and Helix's litigate queries.
#[derive(Parser)]
struct Args {
    /// Exit 1 when a committed file is stale instead of writing.
    #[arg(long)]
    check: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();
    let root = repo_root();
    if args.check {
        let stale = generated::stale(&root)?;
        for path in &stale {
            eprintln!(
                "editor-queries: {} is stale; run `just editor-queries`",
                path.display()
            );
        }
        return Ok(if stale.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        });
    }
    for path in generated::write(&root)? {
        println!("{}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}
