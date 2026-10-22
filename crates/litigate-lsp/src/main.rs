//! Stdio language-server executable.
#[tokio::main]
async fn main() {
    litigate_lsp::run_stdio().await;
}
