// SPDX-License-Identifier: MIT
// Backend implementation for Anvomidav LSP

use dashmap::DashMap;
use ropey::Rope;
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::DocumentState;
use crate::handlers;

pub struct Backend {
    pub client: Client,
    pub document_map: Arc<DashMap<Url, DocumentState>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: Arc::new(DashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: &Url) {
        if let Some(doc) = self.document_map.get(uri) {
            let source = doc.source();
            let diagnostics = handlers::diagnostics::generate_diagnostics(&source).await;
            self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![" ".to_string(), "{".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "anvomidav-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Anvomidav LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = Rope::from_str(&params.text_document.text);
        let version = params.text_document.version;

        self.document_map.insert(uri.clone(), DocumentState::new(content, version));
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(mut doc) = self.document_map.get_mut(&uri) {
            // Full sync - replace entire content
            if let Some(change) = params.content_changes.into_iter().next() {
                doc.content = Rope::from_str(&change.text);
                doc.version = params.text_document.version;
            }
        }

        self.publish_diagnostics(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_map.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;

        if let Some(doc) = self.document_map.get(uri) {
            let source = doc.source();
            Ok(handlers::hover::handle_hover(uri, &source, &params).await)
        } else {
            Ok(None)
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;

        if let Some(doc) = self.document_map.get(uri) {
            let source = doc.source();
            Ok(handlers::completion::handle_completion(uri, &source, &params).await)
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;

        if let Some(doc) = self.document_map.get(uri) {
            let source = doc.source();
            Ok(handlers::definition::handle_definition(uri, &source, &params).await)
        } else {
            Ok(None)
        }
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.document_map.get(uri) {
            let source = doc.source();
            Ok(Some(handlers::formatting::format_document(&source).await))
        } else {
            Ok(None)
        }
    }
}
