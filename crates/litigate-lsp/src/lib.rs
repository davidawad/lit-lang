//! Language server for `.lit` and `.case` documents.
//!
//! The server keeps document snapshots in memory and publishes compiler parser
//! diagnostics after every change. Its handlers operate on the same Rowan
//! syntax and formatter used by `litigate`.
#![allow(clippy::redundant_closure_for_method_calls)]

use std::collections::HashMap;

use litigate_base::Applicability;
use litigate_syntax::{format, parse, FileKind};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
        DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
        InitializeParams, InitializeResult, InitializedParams, InlayHint, InlayHintParams,
        Location, MarkedString, MessageType, OneOf, ReferenceParams, ServerCapabilities,
        ServerInfo, SymbolInformation, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
        Url, WorkspaceEdit, WorkspaceSymbolParams,
    },
    Client, LanguageServer, LspService, Server as LspServer,
};

/// Language server state.
#[derive(Debug)]
pub struct Server {
    client: Client,
    documents: tokio::sync::RwLock<HashMap<Url, Document>>,
    root: tokio::sync::RwLock<Option<std::path::PathBuf>>,
}

#[derive(Debug, Clone)]
struct Document {
    text: String,
    version: i32,
    kind: FileKind,
}

impl Server {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: tokio::sync::RwLock::new(HashMap::new()),
            root: tokio::sync::RwLock::new(None),
        }
    }

    async fn publish(&self, uri: Url) {
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&uri) else {
            return;
        };
        let diagnostics = diagnostics(&uri, doc);
        self.client
            .publish_diagnostics(uri, diagnostics, Some(doc.version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        *self.root.write().await = params.root_uri.and_then(|uri| uri.to_file_path().ok());
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "litigate-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
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
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let item = params.text_document;
        if let Some(kind) = kind(&item.uri) {
            self.documents.write().await.insert(
                item.uri.clone(),
                Document {
                    text: item.text,
                    version: item.version,
                    kind,
                },
            );
            self.publish(item.uri).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(&params.text_document.uri) {
            for change in params.content_changes {
                doc.text = match change.range {
                    Some(range) => replace_range(&doc.text, range, &change.text),
                    None => change.text,
                };
            }
            doc.version = params.text_document.version;
        }
        drop(docs);
        self.publish(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&uri) else {
            return Ok(None);
        };
        let root_path = self.root.read().await.clone();
        let parsed = parse(&doc.text, doc.kind);
        let Some(token) = token_at(&parsed.syntax(), position) else {
            return Ok(None);
        };
        let word = token.text().trim();
        let token_kind = token.kind();
        let token_range = to_range(token.text_range(), &doc.text);
        let word = word.to_owned();
        drop(token);
        let source = cite_source(&word, root_path.as_deref());
        let detail = if let Some(source_text) = source {
            format!("`{word}`\n\nSource text: {source_text}")
        } else {
            format!("`{word}`\n\nLitigate syntax token ({token_kind:?}).")
        };
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(detail)),
            range: Some(token_range),
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let tdp = params.text_document_position_params;
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&tdp.text_document.uri) else {
            return Ok(None);
        };
        let parsed = parse(&doc.text, doc.kind);
        let Some(token) = token_at(&parsed.syntax(), tdp.position) else {
            return Ok(None);
        };
        let name = token.text();
        for (uri, other) in docs.iter() {
            let tree = parse(&other.text, other.kind);
            for node in tree.syntax().descendants() {
                if is_declaration(node.kind())
                    && node
                        .children_with_tokens()
                        .find_map(|it| it.into_token())
                        .is_some_and(|t| t.text() == name)
                {
                    let range = node
                        .children_with_tokens()
                        .find_map(|it| it.into_token())
                        .map(|t| to_range(t.text_range(), &other.text))
                        .unwrap_or_default();
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    })));
                }
            }
        }
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let tdp = params.text_document_position;
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&tdp.text_document.uri) else {
            return Ok(None);
        };
        let parsed = parse(&doc.text, doc.kind);
        let Some(token) = token_at(&parsed.syntax(), tdp.position) else {
            return Ok(None);
        };
        let needle = token.text().to_owned();
        let mut found = Vec::new();
        for (uri, other) in docs.iter() {
            let tree = parse(&other.text, other.kind);
            for tok in tree
                .syntax()
                .descendants_with_tokens()
                .filter_map(|it| it.into_token())
            {
                if tok.text() == needle {
                    found.push(Location {
                        uri: uri.clone(),
                        range: to_range(tok.text_range(), &other.text),
                    });
                }
            }
        }
        Ok(Some(found))
    }

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let parsed = parse(&doc.text, doc.kind);
        let symbols = parsed
            .syntax()
            .descendants()
            .filter(|node| is_declaration(node.kind()))
            .filter_map(|node| {
                let token = node.children_with_tokens().find_map(|it| it.into_token())?;
                let range = to_range(node.text_range(), &doc.text);
                let selection_range = to_range(token.text_range(), &doc.text);
                Some(DocumentSymbol {
                    name: token.text().to_owned(),
                    detail: Some(format!("{:?}", node.kind())),
                    kind: symbol_kind(node.kind()),
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: None,
                })
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    #[allow(deprecated)]
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let docs = self.documents.read().await;
        let mut out = Vec::new();
        for (uri, doc) in docs.iter() {
            let parsed = parse(&doc.text, doc.kind);
            for node in parsed
                .syntax()
                .descendants()
                .filter(|node| is_declaration(node.kind()))
            {
                if let Some(token) = node.children_with_tokens().find_map(|it| it.into_token()) {
                    if token
                        .text()
                        .to_lowercase()
                        .contains(&params.query.to_lowercase())
                    {
                        out.push(SymbolInformation {
                            name: token.text().into(),
                            kind: symbol_kind(node.kind()),
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: to_range(token.text_range(), &doc.text),
                            },
                            container_name: None,
                        });
                    }
                }
            }
        }
        Ok(Some(out))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&params.text_document.uri) else {
            return Ok(None);
        };
        Ok(format::format(&doc.text, doc.kind)
            .ok()
            .filter(|formatted| formatted != &doc.text)
            .map(|new_text| {
                vec![TextEdit {
                    range: full_range(&doc.text),
                    new_text,
                }]
            }))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let docs = self.documents.read().await;
        let Some(doc) = docs.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let diagnostics = parse(&doc.text, doc.kind).diagnostics(&litigate_base::FileId(
            params.text_document.uri.path().to_owned(),
        ));
        let actions = diagnostics
            .iter()
            .flat_map(|diag| {
                diag.fixes
                    .iter()
                    .filter(|fix| fix.applicability == Applicability::MachineApplicable)
                    .map(|fix| {
                        CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Fix {}", diag.code.as_str()),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![lsp_diagnostic(diag, &doc.text)]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(
                                    [(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: span_range(&fix.span, &doc.text),
                                            new_text: fix.replacement.clone(),
                                        }],
                                    )]
                                    .into_iter()
                                    .collect(),
                                ),
                                ..WorkspaceEdit::default()
                            }),
                            ..CodeAction::default()
                        })
                    })
            })
            .collect();
        Ok(Some(actions))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let _ = params;
        Ok(Some(Vec::new()))
    }
}

/// Runs the language server over stdio.
pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Server::new);
    LspServer::new(stdin, stdout, socket).serve(service).await;
}

mod helpers;
use helpers::{
    cite_source, diagnostics, full_range, is_declaration, kind, lsp_diagnostic, replace_range,
    span_range, symbol_kind, to_range, token_at,
};
