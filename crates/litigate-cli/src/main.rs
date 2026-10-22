//! The `litigate` binary: a thin wrapper over [`litigate_cli::main`].
use std::process::ExitCode;

fn main() -> ExitCode {
    litigate_cli::main(std::env::args_os())
}
