//! Command-line entry point for the litigate language toolchain.
use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "litigate",
    version,
    about = "Check and compile legal procedure modules"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the language and CLI contract as JSON.
    Describe,
    /// Check source files (implementation supplied by the compiler bead).
    Check {
        #[arg(required = true)]
        paths: Vec<String>,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Describe => {
            println!("{{\"ok\":true,\"op\":\"describe\",\"result\":{{}} ,\"diagnostics\":[],\"provenance\":{{}},\"elapsed_ms\":0.0}}");
            ExitCode::SUCCESS
        }
        Command::Check { .. } => {
            eprintln!("litigate check is not implemented yet");
            ExitCode::from(1)
        }
    }
}
