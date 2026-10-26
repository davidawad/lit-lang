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
//!   dates computed by the engine clock;
//! - rename of a declaration (every version) and its references across
//!   files, and completion of the names a reference can resolve to;
//! - a world picker: `litigate/worlds` lists the worlds the workspace
//!   names (the packages' own dates, and each matter's), and
//!   `litigate/selectWorld` checks packages in another `as_of`/`known_at`
//!   (hover then shows what governs each declaration there).
//!
//! Edits are debounced (`debounceMs`): a check runs once edits pause, on a
//! blocking thread so the server keeps reading messages, and a newer edit
//! cancels it between packages and matters. Requests wait for the check
//! running when they arrive and answer from the last complete analysis.
//!
//! The server is `litigate-lsp` (stdio); `litigate lsp --stdio` runs it.
pub mod analysis;
pub mod cite;
pub mod completion;
pub mod convert;
pub mod diagnostics;
pub mod host;
pub mod hover;
pub mod index;
pub mod inlay;
pub mod rename;
pub mod symbols;

use analysis::Analysis;
use host::{Host, Open, Settings};
use litigate_syntax::FileKind;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
    InlayHint, InlayHintParams, Location, MarkupContent, MarkupKind, MessageType, OneOf,
    PrepareRenameResponse, ReferenceParams, RenameOptions, RenameParams, ServerCapabilities,
    ServerInfo, SymbolInformation, TextDocumentPositionParams, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Url,
    WorkDoneProgressOptions, WorkspaceEdit, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer, LspService, Server as LspServer};

/// What the handlers share with the checks they start.
#[derive(Default)]
struct State {
    /// The analysis and its session, held for a whole check. Fair: a
    /// request waits for the check started before it.
    host: Arc<tokio::sync::Mutex<Host>>,
    /// Open documents, current as of the last notification (locked only
    /// briefly, so edits never wait for a check).
    docs: Mutex<BTreeMap<PathBuf, Open>>,
    /// Bumped by every change: a check started before the latest change is
    /// stale and cancels itself.
    generation: Arc<AtomicU64>,
    /// [`Settings::debounce_ms`].
    debounce_ms: AtomicU64,
    /// The last generation whose analysis completed. Several notifications
    /// can enqueue refresh handlers while one check owns `host`; once the
    /// first handler checks the newest generation, the queued handlers must
    /// not replay that same (potentially expensive) workspace check. Set
    /// before the check lets go of `host`: the next handler in the queue
    /// takes `host` before the one that checked is scheduled again.
    completed_generation: Arc<Mutex<Option<u64>>>,
}

/// The language server.
pub struct Server {
    client: Client,
    state: State,
    /// What each URI was last published with, so unchanged files are not
    /// re-sent and vanished diagnostics are cleared.
    published: Mutex<BTreeMap<Url, Vec<Diagnostic>>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A world as the picker shows it.
fn world(label: &str, as_of: Option<&str>, known_at: Option<&str>) -> Value {
    json!({"label": label, "asOf": as_of, "knownAt": known_at})
}

impl Server {
    /// A server talking to `client`.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: State::default(),
            published: Mutex::new(BTreeMap::new()),
        }
    }

    /// `litigate/status`: how many times the database was built from
    /// scratch rather than updated in place, how many checks ran and how
    /// many of them a newer change cancelled, and the files under analysis.
    ///
    /// # Errors
    /// Never; the signature is the custom-method contract.
    pub async fn status(&self) -> Result<Value> {
        let host = self.state.host.lock().await;
        let files: Vec<&String> = host
            .current()
            .map(|a| a.files.keys().collect())
            .unwrap_or_default();
        Ok(json!({
            "rebuilds": host.rebuilds,
            "checks": host.checks,
            "cancelled": host.cancelled,
            "files": files,
        }))
    }

    /// `litigate/worlds`: the selected world and the worlds the workspace
    /// names: the packages' own dates, and each matter's `as_of` and
    /// `known_at`.
    ///
    /// # Errors
    /// Never; the signature is the custom-method contract.
    pub async fn worlds(&self) -> Result<Value> {
        let host = self.state.host.lock().await;
        let s = &host.settings;
        let mut worlds = vec![world("package dates", None, None)];
        worlds.extend(host.current().into_iter().flat_map(|a| {
            a.checked.cases.iter().filter_map(|c| {
                let case = c.case.as_ref()?;
                Some(world(&c.path, Some(&case.as_of), case.known_at.as_deref()))
            })
        }));
        Ok(json!({
            "current": world("selected", s.as_of.as_deref(), s.known_at.as_deref()),
            "worlds": worlds,
        }))
    }

    /// `litigate/selectWorld {asOf, knownAt}`: checks packages at these
    /// dates from now on (`null`: each package's own), and returns
    /// [`Self::worlds`]. Only verification and lowering re-run; parsing,
    /// typing and the matters' checks are world-independent.
    ///
    /// # Errors
    /// `InvalidParams` for a date that is not `YYYY-MM-DD`.
    pub async fn select_world(&self, params: Value) -> Result<Value> {
        let date = |key: &str| -> Result<Option<String>> {
            match params.get(key) {
                None | Some(Value::Null) => Ok(None),
                Some(Value::String(d)) if litigate_sema::types::date::is_valid(d) => {
                    Ok(Some(d.clone()))
                }
                Some(other) => Err(Error::invalid_params(format!(
                    "`{key}` must be a YYYY-MM-DD date or null, not {other}"
                ))),
            }
        };
        let (as_of, known_at) = (date("asOf")?, date("knownAt")?);
        {
            let mut host = self.state.host.lock().await;
            host.settings.as_of = as_of;
            host.settings.known_at = known_at;
        }
        self.state.generation.fetch_add(1, Ordering::SeqCst);
        self.refresh().await;
        self.worlds().await
    }

    /// Re-checks and publishes what changed. The check runs on a blocking
    /// thread holding the analysis, so requests wait for it while changes
    /// still arrive, and a newer change cancels it.
    async fn refresh(&self) {
        let mut host = Arc::clone(&self.state.host).lock_owned().await;
        let (generation, versions) = {
            let docs = lock(&self.state.docs);
            host.open = docs.clone();
            let versions: BTreeMap<PathBuf, i32> =
                docs.iter().map(|(p, o)| (p.clone(), o.version)).collect();
            let generation = self.state.generation.load(Ordering::SeqCst);
            if *lock(&self.state.completed_generation) == Some(generation) {
                return;
            }
            (generation, versions)
        };
        let latest = Arc::clone(&self.state.generation);
        let completed = Arc::clone(&self.state.completed_generation);
        let checked = tokio::task::spawn_blocking(move || {
            let cancelled = || latest.load(Ordering::SeqCst) != generation;
            let (complete, analysis) = host.run(&cancelled);
            let fresh = complete.then(|| diagnostics::publishable(analysis));
            if complete {
                *lock(&completed) = Some(generation);
            }
            drop(host);
            fresh
        })
        .await;
        // A cancelled check publishes nothing: the change that cancelled it
        // checks again.
        let Ok(Some(fresh)) = checked else {
            return;
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
    async fn with<R>(&self, uri: &Url, f: impl FnOnce(&Analysis, &str) -> Option<R>) -> Option<R> {
        let host = self.state.host.lock().await;
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
            let mut host = self.state.host.lock().await;
            #[allow(deprecated)]
            let root = params
                .workspace_folders
                .as_ref()
                .and_then(|f| f.first())
                .map(|f| f.uri.clone())
                .or(params.root_uri);
            host.root = root.and_then(|u| u.to_file_path().ok());
            host.settings = Settings::from_options(params.initialization_options.as_ref());
            self.state
                .debounce_ms
                .store(host.settings.debounce_ms, Ordering::SeqCst);
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
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".into()]),
                    ..CompletionOptions::default()
                }),
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
            lock(&self.state.docs).insert(
                path,
                Open {
                    text: doc.text,
                    version: doc.version,
                },
            );
            self.state.generation.fetch_add(1, Ordering::SeqCst);
            self.refresh().await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Ok(path) = params.text_document.uri.to_file_path() else {
            return;
        };
        let generation = {
            let mut docs = lock(&self.state.docs);
            let Some(open) = docs.get_mut(&path) else {
                return;
            };
            for change in params.content_changes {
                open.text = match change.range {
                    Some(range) => convert::apply_change(&open.text, range, &change.text),
                    None => change.text,
                };
            }
            open.version = params.text_document.version;
            self.state.generation.fetch_add(1, Ordering::SeqCst) + 1
        };
        // Debounce: only the last of a burst of edits checks.
        let delay = self.state.debounce_ms.load(Ordering::SeqCst);
        if delay > 0 {
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
        if self.state.generation.load(Ordering::SeqCst) == generation {
            self.refresh().await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.state.generation.fetch_add(1, Ordering::SeqCst);
        self.refresh().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(path) = params.text_document.uri.to_file_path() {
            lock(&self.state.docs).remove(&path);
        }
        self.state.generation.fetch_add(1, Ordering::SeqCst);
        self.refresh().await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let at = params.text_document_position_params;
        Ok(self
            .with(&at.text_document.uri, |analysis, file| {
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
            })
            .await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let at = params.text_document_position_params;
        Ok(self
            .with(&at.text_document.uri, |analysis, file| {
                let def = &analysis.index.defs[Self::definition_at(analysis, file, at.position)?];
                Self::location(analysis, &def.file, def.name_range)
                    .map(GotoDefinitionResponse::Scalar)
            })
            .await)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let at = params.text_document_position;
        let declaration = params.context.include_declaration;
        Ok(self
            .with(&at.text_document.uri, |analysis, file| {
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
            })
            .await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self
            .with(&params.text_document.uri, |analysis, file| {
                Some(DocumentSymbolResponse::Nested(symbols::document(
                    analysis, file,
                )))
            })
            .await)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let host = self.state.host.lock().await;
        Ok(host.current().map(|a| symbols::workspace(a, &params.query)))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };
        let docs = lock(&self.state.docs);
        let (Some(open), Some(kind)) = (docs.get(&path), FileKind::from_path(&path)) else {
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
        Ok(self
            .with(&params.text_document.uri, |analysis, file| {
                Some(
                    diagnostics::actions(analysis, file, params.range)
                        .into_iter()
                        .map(CodeActionOrCommand::CodeAction)
                        .collect(),
                )
            })
            .await)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        Ok(self
            .with(&params.text_document.uri, |analysis, file| {
                Some(inlay::hints(analysis, file, params.range))
            })
            .await)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        Ok(self
            .with(&params.text_document.uri, |analysis, file| {
                let text = analysis.text(file)?;
                let (range, placeholder) =
                    rename::prepare(analysis, file, convert::text_size(&text, params.position))?;
                Some(PrepareRenameResponse::RangeWithPlaceholder {
                    range: convert::range(&text, range),
                    placeholder,
                })
            })
            .await)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        rename::valid(&params.new_name).map_err(Error::invalid_params)?;
        let at = params.text_document_position;
        Ok(self
            .with(&at.text_document.uri, |analysis, file| {
                let text = analysis.text(file)?;
                let edits = rename::ranges(analysis, file, convert::text_size(&text, at.position));
                let changes: std::collections::HashMap<Url, Vec<TextEdit>> = edits
                    .into_iter()
                    .filter_map(|(file, ranges)| {
                        let text = analysis.text(&file).unwrap_or_default();
                        analysis.uri_of(&file).map(|uri| {
                            let edits = ranges
                                .into_iter()
                                .map(|r| TextEdit {
                                    range: convert::range(&text, r),
                                    new_text: params.new_name.clone(),
                                })
                                .collect();
                            (uri, edits)
                        })
                    })
                    .collect();
                (!changes.is_empty()).then(|| WorkspaceEdit {
                    changes: Some(changes),
                    ..WorkspaceEdit::default()
                })
            })
            .await)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let at = params.text_document_position;
        let docs = lock(&self.state.docs).clone();
        Ok(self
            .with(&at.text_document.uri, |analysis, file| {
                // Complete against what is typed, not what was last checked.
                let typed = at.text_document.uri.to_file_path().ok();
                let text = typed
                    .and_then(|p| docs.get(&p).map(|o| o.text.clone()))
                    .or_else(|| analysis.text(file))?;
                let offset = usize::from(convert::text_size(&text, at.position));
                Some(CompletionResponse::Array(completion::at(
                    analysis, file, &text, offset,
                )))
            })
            .await)
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
        .custom_method("litigate/worlds", Server::worlds)
        .custom_method("litigate/selectWorld", Server::select_world)
        .finish();
    LspServer::new(input, output, socket).serve(service).await;
}

/// Serves one LSP session over stdio.
pub async fn run_stdio() {
    serve(tokio::io::stdin(), tokio::io::stdout()).await;
}

#[cfg(test)]
mod tests;
