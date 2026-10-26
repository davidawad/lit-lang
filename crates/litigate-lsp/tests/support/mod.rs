//! An LSP client driving the server in-process over a duplex pipe, with
//! real `Content-Length` framing, so tests exercise the wire protocol.
#![allow(
    dead_code,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    clippy::expect_used
)]

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{
    AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf,
};
use tower_lsp::lsp_types::Url;

/// The repository root.
pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// The URI of a repository-relative path.
pub fn uri(path: &str) -> String {
    uri_in(&repo(), path)
}

/// The URI of a `root`-relative path.
pub fn uri_in(root: &Path, path: &str) -> String {
    Url::from_file_path(root.join(path)).unwrap().to_string()
}

/// A repository file's text.
pub fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap()
}

/// `(line, character)` of the `nth` occurrence of `needle` in `text`,
/// shifted `shift` characters into it.
pub fn at(text: &str, needle: &str, nth: usize, shift: u32) -> Value {
    let offset = text.match_indices(needle).nth(nth).unwrap().0;
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let col = before[before.rfind('\n').map_or(0, |i| i + 1)..]
        .encode_utf16()
        .count();
    json!({"line": line, "character": u32::try_from(col).unwrap() + shift})
}

/// A connected client.
pub struct Client {
    /// The workspace root the path helpers ([`Self::open`], [`Self::at`],
    /// ...) resolve against: the repository unless [`Self::start`] said.
    pub root: PathBuf,
    w: WriteHalf<DuplexStream>,
    r: BufReader<ReadHalf<DuplexStream>>,
    id: i64,
    /// Notifications not yet consumed.
    pub notes: Vec<Value>,
}

impl Client {
    /// Starts a server rooted at `root` and completes the handshake. Edits
    /// are checked at once (`debounceMs` 0) unless `options` say otherwise.
    pub async fn start(root: &Path, mut options: Value) -> (Self, Value) {
        if options.get("debounceMs").is_none() {
            options["debounceMs"] = json!(0);
        }
        let root_uri = Url::from_file_path(root).unwrap();
        let (mut client, init) = Self::initialize(
            json!({"processId": null, "rootUri": root_uri, "capabilities": {}, "initializationOptions": options}),
        )
        .await;
        client.root = root.to_path_buf();
        client.notify("initialized", json!({})).await;
        (client, init)
    }

    /// Starts a server and sends `initialize` with `params`, but not
    /// `initialized`.
    pub async fn initialize(params: Value) -> (Self, Value) {
        let (near, far) = tokio::io::duplex(1 << 22);
        let (far_r, far_w) = tokio::io::split(far);
        tokio::spawn(litigate_lsp::serve(far_r, far_w));
        let (r, w) = tokio::io::split(near);
        let mut client = Self {
            root: repo(),
            w,
            r: BufReader::new(r),
            id: 0,
            notes: Vec::new(),
        };
        let init = client.request("initialize", params).await;
        (client, init)
    }

    /// A client at the repository root, `dev` profile.
    pub async fn repo() -> Self {
        Self::start(&repo(), json!({})).await.0
    }

    /// A client over [`examples`]`(name, also)`, `dev` profile: the worked
    /// example at its repository paths, without the rest of the
    /// repository to check before each answer.
    pub async fn examples(name: &str, also: &[&str]) -> Self {
        Self::start(&examples(name, also), json!({})).await.0
    }

    /// The URI of a path under [`Self::root`].
    pub fn uri(&self, path: &str) -> String {
        uri_in(&self.root, path)
    }

    /// `(path, line)` of an LSP location, path relative to [`Self::root`].
    pub fn place(&self, location: &Value) -> (String, u64) {
        place_in(&self.root, location)
    }

    async fn send(&mut self, message: Value) {
        let body = message.to_string();
        let frame = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        self.w.write_all(frame.as_bytes()).await.unwrap();
        self.w.flush().await.unwrap();
    }

    async fn recv(&mut self) -> Value {
        let read = async {
            let mut length = 0;
            loop {
                let mut line = String::new();
                self.r.read_line(&mut line).await.unwrap();
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                if let Some(n) = line.strip_prefix("Content-Length: ") {
                    length = n.parse().unwrap();
                }
            }
            let mut body = vec![0; length];
            self.r.read_exact(&mut body).await.unwrap();
            serde_json::from_slice(&body).unwrap()
        };
        tokio::time::timeout(Duration::from_mins(1), read)
            .await
            .expect("the server answers within a minute")
    }

    /// Sends a request and returns its `error` (panics on a result).
    pub async fn request_error(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let id = self.id;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await;
        loop {
            let message = self.recv().await;
            if message.get("id") == Some(&json!(id)) && message.get("method").is_none() {
                assert!(message.get("result").is_none(), "{method}: {message}");
                return message["error"].clone();
            }
            self.notes.push(message);
        }
    }

    /// Sends a request and returns its `result` (panics on an error).
    pub async fn request(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let id = self.id;
        let mut message = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if !params.is_null() {
            message["params"] = params;
        }
        self.send(message).await;
        loop {
            let message = self.recv().await;
            if message.get("id") == Some(&json!(id)) && message.get("method").is_none() {
                assert!(message.get("error").is_none(), "{method}: {message}");
                return message["result"].clone();
            }
            self.notes.push(message);
        }
    }

    /// Sends a notification.
    pub async fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({"jsonrpc": "2.0", "method": method, "params": params}))
            .await;
    }

    /// The next `publishDiagnostics` for `uri` (consumed).
    pub async fn diagnostics(&mut self, uri: &str) -> Vec<Value> {
        let is = |m: &Value| {
            m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"] == uri
        };
        loop {
            if let Some(i) = self.notes.iter().position(is) {
                let m = self.notes.remove(i);
                return m["params"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
            }
            let m = self.recv().await;
            self.notes.push(m);
        }
    }

    /// The first `publishDiagnostics` for `uri` satisfying `want`; earlier
    /// ones for it are dropped.
    pub async fn diagnostics_where(
        &mut self,
        uri: &str,
        want: impl Fn(&[Value]) -> bool,
    ) -> Vec<Value> {
        loop {
            let ds = self.diagnostics(uri).await;
            if want(&ds) {
                return ds;
            }
        }
    }

    /// Opens `path` with `text`.
    pub async fn open(&mut self, path: &str, text: &str) {
        let language = if Path::new(path)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("case"))
        {
            "litigate-case"
        } else {
            "litigate"
        };
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": self.uri(path), "languageId": language, "version": 1, "text": text}}),
        )
        .await;
    }

    /// Opens the file at `uri` with `text`.
    pub async fn open_at(&mut self, uri: &str, text: &str) {
        let language = if Path::new(uri)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("case"))
        {
            "litigate-case"
        } else {
            "litigate"
        };
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri, "languageId": language, "version": 1, "text": text}}),
        )
        .await;
    }

    /// Replaces `range` of `path` with `text` (an incremental change).
    pub async fn edit(&mut self, path: &str, version: i32, range: Value, text: &str) {
        self.notify(
            "textDocument/didChange",
            json!({"textDocument": {"uri": self.uri(path), "version": version},
                   "contentChanges": [{"range": range, "text": text}]}),
        )
        .await;
    }

    /// `textDocument/<method>` at `position` of `path`.
    pub async fn at(&mut self, method: &str, path: &str, position: Value) -> Value {
        self.request(
            &format!("textDocument/{method}"),
            json!({"textDocument": {"uri": self.uri(path)}, "position": position,
                   "context": {"includeDeclaration": true}}),
        )
        .await
    }

    /// The server's `litigate/status`.
    pub async fn status(&mut self) -> Value {
        self.request("litigate/status", Value::Null).await
    }
}

/// `(path, line)` of an LSP location, path relative to the repository.
pub fn place(location: &Value) -> (String, u64) {
    place_in(&repo(), location)
}

/// `(path, line)` of an LSP location, path relative to `root`.
pub fn place_in(root: &Path, location: &Value) -> (String, u64) {
    let uri = Url::parse(location["uri"].as_str().unwrap()).unwrap();
    let path = uri.to_file_path().unwrap();
    let rel = path
        .strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    (rel, location["range"]["start"]["line"].as_u64().unwrap())
}

/// A fresh workspace under the temp dir holding `files` (`(path, text)`,
/// root-relative), unique to this process and `name`.
pub fn workspace(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("litigate-lsp-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    root
}

/// The worked example (SPEC §12) at its repository paths, `lit/examples`
/// and `cases/examples` with their locks, plus the repository files
/// `also`, in a fresh workspace (as [`workspace`]). A check of it takes a
/// fraction of the repository's (a dozen packages, two dozen matters), so
/// a test over the worked example alone does not wait on the rest.
pub fn examples(name: &str, also: &[&str]) -> PathBuf {
    let paths = [
        "lit/examples/Lit.toml",
        "lit/examples/Lit.lock",
        "lit/examples/rule12.lit",
        "cases/examples/acme.case",
        "cases/examples/Lit.lock",
    ];
    let texts: Vec<(&str, String)> = paths.iter().chain(also).map(|p| (*p, read(p))).collect();
    let files: Vec<(&str, &str)> = texts.iter().map(|(p, t)| (*p, t.as_str())).collect();
    workspace(name, &files)
}

/// The worked example (SPEC §12) as `lit/ex` and `cases/acme.case`, plus
/// `extra` files.
pub fn worked(name: &str, extra: &[(&str, &str)]) -> PathBuf {
    let manifest = read("lit/examples/Lit.toml");
    let module = read("lit/examples/rule12.lit");
    let case = read("cases/examples/acme.case");
    let mut files = vec![
        ("lit/ex/Lit.toml", manifest.as_str()),
        ("lit/ex/rule12.lit", module.as_str()),
        ("cases/acme.case", case.as_str()),
    ];
    files.extend_from_slice(extra);
    workspace(name, &files)
}

/// Writes `cases/Lit.lock` pinning every package `host` analysed at
/// `root` (as `litigate lock` would), so matters there replay.
pub fn pin(root: &Path, host: &mut litigate_lsp::host::Host) {
    let session = &host.analyze().checked.session;
    let lock = session.matter_lock(&session.cases).render();
    std::fs::write(root.join("cases/Lit.lock"), lock).unwrap();
}

/// A host rooted at `root` (as a client's `rootUri`).
pub fn host(root: &Path) -> litigate_lsp::host::Host {
    let mut host = litigate_lsp::host::Host::default();
    host.root = Some(root.to_path_buf());
    host
}

/// Codes of a diagnostic list, sorted.
pub fn codes(diagnostics: &[Value]) -> Vec<String> {
    let mut codes: Vec<String> = diagnostics
        .iter()
        .map(|d| d["code"].as_str().unwrap_or_default().to_owned())
        .collect();
    codes.sort();
    codes
}
