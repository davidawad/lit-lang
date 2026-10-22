//! The `litigate-lsp` binary speaks LSP on stdio: initialize, shutdown,
//! exit.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

fn read_message(r: &mut impl BufRead) -> serde_json::Value {
    let mut length = 0;
    loop {
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(n) = line.strip_prefix("Content-Length: ") {
            length = n.parse().unwrap();
        }
    }
    let mut body = vec![0; length];
    r.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[test]
fn serves_a_session_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_litigate-lsp"))
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#;
    stdin.write_all(frame(init).as_bytes()).unwrap();
    stdin.flush().unwrap();
    let reply = read_message(&mut stdout);
    assert_eq!(reply["id"], 1);
    assert_eq!(reply["result"]["serverInfo"]["name"], "litigate-lsp");
    let shutdown = r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#;
    stdin.write_all(frame(shutdown).as_bytes()).unwrap();
    stdin.flush().unwrap();
    assert_eq!(read_message(&mut stdout)["id"], 2);
    let exit = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    stdin.write_all(frame(exit).as_bytes()).unwrap();
    drop(stdin);
    assert!(child.wait().unwrap().success());
}

#[test]
fn rejects_unknown_arguments() {
    let out = Command::new(env!("CARGO_BIN_EXE_litigate-lsp"))
        .arg("--tcp")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage: litigate-lsp [--stdio]"));
}
