//! Write (or with `--check`, verify) `editors/helix/queries/litigate`'s
//! generated queries from the canonical ones. See `litigate_editors::helix`.

use anyhow::Result;
use clap::Parser;
use litigate_editors::helix;
use std::process::ExitCode;

/// Render Helix's litigate queries from editors/tree-sitter-litigate/queries.
#[derive(Parser)]
struct Args {
    /// Exit 1 when a committed copy is stale instead of writing.
    #[arg(long)]
    check: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();
    if args.check {
        let stale = helix::stale()?;
        for file in &stale {
            eprintln!("helix-queries: {file} is stale; run `cargo run -p litigate-editors --bin helix-queries`");
        }
        return Ok(if stale.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        });
    }
    let dir = helix::queries_dir();
    for (file, text) in helix::expected()? {
        std::fs::write(dir.join(file), text)?;
        println!("{}", dir.join(file).display());
    }
    Ok(ExitCode::SUCCESS)
}
