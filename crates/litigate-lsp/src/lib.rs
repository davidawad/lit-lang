//! The litigate language server (DESIGN §6, SPEC §9.3 `litigate lsp`).
//!
//! Every analysis runs the same pipeline as `litigate check`
//! ([`litigate_cli::check_session`]) over one salsa database kept across
//! edits ([`host`]): parse, sema, lowering, verification, inline tests, and
//! case replay under the engine clock. On top of that snapshot the server
//! offers:
//!
//! - `publishDiagnostics` after every change, for every checked file;
//! - hover: a declaration's kind, qualified id, type, every effective
//!   version, and the L0 text its `cite` resolves to (located through the
//!   lowering's source-map sidecar, narrowed to the pinpoint);
//! - go-to-definition and find-references across modules, packages, and
//!   the matters that use them;
//! - document and workspace symbols;
//! - formatting (`litigate fmt`);
//! - code actions from diagnostic fixes;
//! - inlay hints on `.case` events with the deadlines they trigger, due
//!   dates computed by the engine clock.
//!
//! The server is `litigate-lsp` (stdio); `litigate lsp --stdio` runs it.
pub mod analysis;
pub mod cite;
pub mod convert;
pub mod diagnostics;
pub mod host;
pub mod hover;
pub mod index;
pub mod inlay;
pub mod symbols;

use analysis::Analysis;
use host::{Host, Open, Settings};
use litigate_syntax::FileKind;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use tokio::io::{AsyncRead, AsyncWrite};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
    InlayHint, InlayHintParams, Location, MarkupContent, MarkupKind, MessageType, OneOf,
    ReferenceParams, ServerCapabilities, ServerInfo, SymbolInformation, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Url,
    WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer, LspService, Server as LspServer};

/// The language server.
pub struct Server {
    client: Client,
    host: Mutex<Host>,
    /// What each URI was last published with, so unchanged files are not
    /// re-sent and vanished diagnostics are cleared.
    published: Mutex<BTreeMap<Url, Vec<Diagnostic>>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Server {
    /// A server talking to `client`.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            host: Mutex::new(Host::default()),
            published: Mutex::new(BTreeMap::new()),
        }
    }

    /// `litigate/status`: how many times the database was built from
    /// scratch rather than updated in place, and the files under analysis.
    ///
    /// # Errors
    /// Never; the signature is the custom-method contract.
    #[allow(clippy::unused_async)]
    pub async fn status(&self) -> Result<serde_json::Value> {
        let host = lock(&self.host);
        let files: Vec<&String> = host
            .current()
            .map(|a| a.files.keys().collect())
            .unwrap_or_default();
        Ok(serde_json::json!({"rebuilds": host.rebuilds, "files": files}))
    }

    /// Re-checks and publishes what changed.
    async fn refresh(&self) {
        let (fresh, versions) = {
            let mut host = lock(&self.host);
            let versions: BTreeMap<PathBuf, i32> = host
                .open
                .iter()
                .map(|(p, o)| (p.clone(), o.version))
                .collect();
            let analysis = host.analyze();
            (diagnostics::publishable(analysis), versions)
        };
        let sends: Vec<(Url, Vec<Diagnostic>)> = {
            let mut published = lock(&self.published);
            let mut sends: Vec<(Url, Vec<Diagnostic>)> = published
                .keys()
                .filter(|u| !fresh.iter().any(|(f, _)| f == *u))
                .map(|u| (u.clone(), Vec::new()))
                .collect();
            sends.extend(
                fresh
                    .into_iter()
                    .filter(|(u, ds)| published.get(u) != Some(ds)),
            );
            for (u, ds) in &sends {
                if ds.is_empty() {
                    published.remove(u);
                } else {
                    published.insert(u.clone(), ds.clone());
                }
            }
            sends
        };
        for (uri, ds) in sends {
            let version = uri
                .to_file_path()
                .ok()
                .and_then(|p| versions.get(&p).copied());
            self.client.publish_diagnostics(uri, ds, version).await;
        }
    }

    /// Runs `f` over the current analysis and the path of `uri`.
    fn with<R>(&self, uri: &Url, f: impl FnOnce(&Analysis, &str) -> Option<R>) -> Option<R> {
        let host = lock(&self.host);
        let analysis = host.current()?;
        let file = analysis.path_of(uri)?;
        f(analysis, &file)
    }

    /// The definition under `position` of `uri`, as `(analysis-free)` index.
    fn definition_at(
        analysis: &Analysis,
        file: &str,
        position: tower_lsp::lsp_types::Position,
    ) -> Option<usize> {
        let text = analysis.text(file)?;
        analysis.index.at(file, convert::text_size(&text, position))
    }

    fn location(
        analysis: &Analysis,
        file: &str,
        range: litigate_syntax::TextRange,
    ) -> Option<Location> {
        let text = analysis.text(file)?;
        Some(Location {
            uri: analysis.uri_of(file)?,
            range: convert::range(&text, range),
        })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut host = lock(&self.host);
            #[allow(deprecated)]
            let root = params
                .workspace_folders
                .as_ref()
                .and_then(|f| f.first())
                .map(|f| f.uri.clone())
                .or(params.root_uri);
            host.root = root.and_then(|u| u.to_file_path().ok());
            host.settings = Settings::from_options(params.initialization_options.as_ref());
        }
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "litigate-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        ..TextDocumentSyncOptions::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "litigate language server ready")
            .await;
        self.refresh().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        if let Ok(path) = doc.uri.to_file_path() {
            lock(&self.host).open.insert(
                path,
                Open {
                    text: doc.text,
                    version: doc.version,
                },
            );
            self.refresh().await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Ok(path) = params.text_document.uri.to_file_path() else {
            return;
        };
        {
            let mut host = lock(&self.host);
            let Some(open) = host.open.get_mut(&path) else {
                return;
            };
            for change in params.content_changes {
                open.text = match change.range {
                    Some(range) => convert::apply_change(&open.text, range, &change.text),
                    None => change.text,
                };
            }
            open.version = params.text_document.version;
        }
        self.refresh().await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.refresh().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(path) = params.text_document.uri.to_file_path() {
            lock(&self.host).open.remove(&path);
        }
        self.refresh().await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let at = params.text_document_position_params;
        Ok(self.with(&at.text_document.uri, |analysis, file| {
            let text = analysis.text(file)?;
            let (value, range) = hover::at(
                analysis,
                file,
                &text,
                convert::text_size(&text, at.position),
            )?;
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: Some(convert::range(&text, range)),
            })
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let at = params.text_document_position_params;
        Ok(self.with(&at.text_document.uri, |analysis, file| {
            let def = &analysis.index.defs[Self::definition_at(analysis, file, at.position)?];
            Self::location(analysis, &def.file, def.name_range).map(GotoDefinitionResponse::Scalar)
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let at = params.text_document_position;
        let declaration = params.context.include_declaration;
        Ok(self.with(&at.text_document.uri, |analysis, file| {
            let i = Self::definition_at(analysis, file, at.position)?;
            let def = &analysis.index.defs[i];
            let own = declaration.then(|| Self::location(analysis, &def.file, def.name_range));
            Some(
                own.flatten()
                    .into_iter()
                    .chain(
                        analysis
                            .index
                            .references(i)
                            .filter_map(|u| Self::location(analysis, &u.file, u.range)),
                    )
                    .collect(),
            )
        }))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.with(&params.text_document.uri, |analysis, file| {
            Some(DocumentSymbolResponse::Nested(symbols::document(
                analysis, file,
            )))
        }))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let host = lock(&self.host);
        Ok(host.current().map(|a| symbols::workspace(a, &params.query)))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };
        let host = lock(&self.host);
        let (Some(open), Some(kind)) = (host.open.get(&path), FileKind::from_path(&path)) else {
            return Ok(None);
        };
        // A document that does not parse is left alone (as `litigate fmt`).
        Ok(litigate_syntax::format(&open.text, kind)
            .ok()
            .map(|formatted| {
                if formatted == open.text {
                    Vec::new()
                } else {
                    vec![TextEdit {
                        range: convert::full_range(&open.text),
                        new_text: formatted,
                    }]
                }
            }))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(self.with(&params.text_document.uri, |analysis, file| {
            Some(
                diagnostics::actions(analysis, file, params.range)
                    .into_iter()
                    .map(CodeActionOrCommand::CodeAction)
                    .collect(),
            )
        }))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        Ok(self.with(&params.text_document.uri, |analysis, file| {
            Some(inlay::hints(analysis, file, params.range))
        }))
    }
}

/// Serves one LSP session over `input`/`output` until the client exits.
pub async fn serve<I, O>(input: I, output: O)
where
    I: AsyncRead + Unpin,
    O: AsyncWrite,
{
    let (service, socket) = LspService::build(Server::new)
        .custom_method("litigate/status", Server::status)
        .finish();
    LspServer::new(input, output, socket).serve(service).await;
}

/// Serves one LSP session over stdio.
pub async fn run_stdio() {
    serve(tokio::io::stdin(), tokio::io::stdout()).await;
}

#[cfg(test)]
mod tests;
