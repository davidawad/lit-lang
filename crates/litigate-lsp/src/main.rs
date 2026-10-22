//! `litigate-lsp [--stdio]`: the language server over stdio (also run by
//! `litigate lsp --stdio`).
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(args.is_empty() || args == ["--stdio"]) {
        eprintln!("usage: litigate-lsp [--stdio]");
        return ExitCode::from(1);
    }
    litigate_lsp::run_stdio().await;
    ExitCode::SUCCESS
}
