//! Command-line entry point for the litigate language toolchain.
use clap::{Parser, Subcommand};
use litigate_base::{DiagnosticCode, Envelope};
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

/// The `describe` result: the stable parts of the SPEC §9.3 contract.
fn describe() -> Envelope<serde_json::Value> {
    let codes: Vec<serde_json::Value> = DiagnosticCode::ALL
        .iter()
        .map(|code| serde_json::json!({"code": code.as_str(), "explain": code.explain()}))
        .collect();
    Envelope::success(
        "describe",
        serde_json::json!({
            "language_version": 1,
            "diagnostic_codes": codes,
            "commands": [
                "describe", "lock", "check", "build", "fmt", "explain", "fix", "where",
                "explain-value", "diff", "query", "import", "lsp"
            ],
        }),
    )
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Describe => match serde_json::to_string(&describe()) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("litigate describe: {error}");
                ExitCode::from(1)
            }
        },
        Command::Check { .. } => {
            eprintln!("litigate check is not implemented yet");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn describe_lists_every_registered_code() {
        let envelope = super::describe();
        assert!(envelope.ok);
        let codes = envelope
            .result
            .as_ref()
            .and_then(|r| r["diagnostic_codes"].as_array())
            .map_or(0, Vec::len);
        assert_eq!(codes, litigate_base::DiagnosticCode::ALL.len());
    }
}
