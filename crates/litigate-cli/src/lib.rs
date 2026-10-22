//! The `litigate` toolchain (DESIGN §8, SPEC §9.3) as a library: every
//! subcommand is a function returning one envelope
//! `{ok, op, result, diagnostics, provenance, elapsed_ms}`, so the binary,
//! tests, and other front ends share one implementation.
//!
//! Exit codes mirror litgraph: `0` ok, `2` the input ran and failed (an
//! error diagnostic, or a litgraph `ok: false` from `run`), `1` usage or
//! I/O failure.
//!
//! ```
//! let printed = litigate_cli::execute(["litigate", "--format", "json", "explain", "L3004"]);
//! assert_eq!(printed.code, 0);
//! let envelope: serde_json::Value = serde_json::from_str(&printed.stdout).map_err(|e| e.to_string())?;
//! assert_eq!(envelope["result"]["code"], "L3004");
//! # Ok::<(), String>(())
//! ```
pub mod cli;
pub mod clock;
mod cmd;
pub mod describe;
pub mod hirx;
pub mod inline_tests;
pub mod matter;
pub mod out;
pub mod prov;
mod render;
pub mod report;
pub mod span;
pub mod verdict;
pub mod ws;

use clap::Parser;
use cli::{Cli, Format};
use out::Printed;
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::time::Instant;

/// Parses `args` (including the program name), runs the command, and
/// returns what to print. Never panics on bad input.
pub fn execute<I, T>(args: I) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render().to_string();
            let informational = matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            return Printed {
                stdout: if informational {
                    rendered.clone()
                } else {
                    String::new()
                },
                stderr: if informational {
                    String::new()
                } else {
                    rendered
                },
                code: u8::from(!informational),
            };
        }
    };
    let tty = std::io::stdout().is_terminal();
    let format = cli
        .format
        .unwrap_or(if tty { Format::Human } else { Format::Json });
    let op = cli.command.name();
    let started = Instant::now();
    let outcome = cmd::dispatch(cli.command);
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    out::print(op, outcome, format, tty, elapsed_ms)
}

/// The binary's `main`: [`execute`], print, exit.
pub fn main<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    if args.get(1).is_some_and(|arg| arg == "lsp") {
        if args.get(2).is_some_and(|arg| arg == "--stdio") && args.len() == 3 {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("failed to start LSP runtime: {error}");
                    return ExitCode::FAILURE;
                }
            };
            runtime.block_on(litigate_lsp::run_stdio());
            return ExitCode::SUCCESS;
        }
        eprintln!("usage: litigate lsp --stdio");
        return ExitCode::from(1);
    }
    let printed = execute(args);
    // A closed pipe cannot be reported anywhere; the exit code still is.
    let _ = std::io::stdout().write_all(printed.stdout.as_bytes());
    let _ = std::io::stderr().write_all(printed.stderr.as_bytes());
    ExitCode::from(printed.code)
}
