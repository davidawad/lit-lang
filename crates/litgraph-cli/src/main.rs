// SPDX-License-Identifier: GPL-3.0-or-later
//! `litgraph`: deprecated. `litigate` is the one command line: `litigate
//! <op> CASE` runs an engine op on a compiled matter or package, and
//! `litigate engine ...` is this binary's raw JSON-in/JSON-out interface.
//! This shim forwards its arguments to `litigate engine`
//! ([`litigate_cli::engine::execute`]) with the packs and named scenarios
//! built into it as the default catalog, so its output and exit codes are
//! unchanged; it notes the deprecation on standard error and goes away
//! with lg-dfv4.7.

use std::io::Write as _;
use std::process::ExitCode;

use litigate_cli::engine::{execute, Builtin};

/// The note every run prints on standard error.
const DEPRECATED: &str =
    "litgraph: deprecated; use `litigate engine ...` (raw requests) or `litigate <op> CASE` (compiled matters)\n";

fn main() -> ExitCode {
    let args = std::iter::once("litgraph".into()).chain(std::env::args_os().skip(1));
    let printed = execute(
        args,
        Builtin::Embedded(litgraph_packs::PACKS, litgraph_packs::SCENARIOS),
    );
    // A closed pipe cannot be reported anywhere; the exit code still is.
    let _ = std::io::stdout().write_all(printed.stdout.as_bytes());
    let _ = std::io::stderr().write_all(DEPRECATED.as_bytes());
    let _ = std::io::stderr().write_all(printed.stderr.as_bytes());
    ExitCode::from(printed.code)
}
