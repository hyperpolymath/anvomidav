// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

//! Anvomidav Language Server Protocol implementation.

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use anv_core::source::FileId;
use anv_semantics::Discipline;
use anv_syntax::parse;

/// Document state stored by the server.
struct Document {
    /// Document content as rope for efficient editing.
    content: Rope,
    /// Current version.
    version: i32,
}

/// The Anvomidav language server.
struct AnvomidavServer {
    /// LSP client handle.
    client: Client,
    /// Open documents.
    documents: DashMap<Url, Document>,
}

impl AnvomidavServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Validate a document and publish diagnostics.
    async fn validate_document(&self, uri: &Url) {
        let Some(doc) = self.documents.get(uri) else {
            return;
        };

        let source = doc.content.to_string();
        let mut diagnostics = Vec::new();

        // Parse the document
        match parse(&source, FileId(0)) {
            Ok(program) => {
                // Type check
                if let Err(type_diags) = anv_types::check(&program, FileId(0)) {
                    for diag in type_diags.iter() {
                        // Get the first label's span for the range, or default to start
                        let range = if let Some(label) = diag.labels.first() {
                            self.span_to_range(&source, label.span.start as usize, label.span.end as usize)
                        } else {
                            Range::new(Position::new(0, 0), Position::new(0, 1))
                        };

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("type-error".into())),
                            source: Some("anvomidav".into()),
                            message: diag.message.clone(),
                            ..Default::default()
                        });
                    }
                }

                // Semantic validation (ISU rules)
                let result = anv_semantics::validate_program(&program, Discipline::MenSingles);
                for err in result.errors {
                    diagnostics.push(Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("isu-rule".into())),
                        source: Some("anvomidav".into()),
                        message: format!("{}", err),
                        ..Default::default()
                    });
                }
                for warn in result.warnings {
                    diagnostics.push(Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("isu-warning".into())),
                        source: Some("anvomidav".into()),
                        message: format!("{}", warn),
                        ..Default::default()
                    });
                }
            }
            Err(parse_errors) => {
                for err in parse_errors {
                    let range = self.span_to_range(&source, err.span.start, err.span.end);
                    let mut diagnostic = Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("parse-error".into())),
                        source: Some("anvomidav".into()),
                        message: err.message,
                        ..Default::default()
                    };

                    // Add related information for help text
                    if let Some(help) = err.help {
                        diagnostic.related_information = Some(vec![DiagnosticRelatedInformation {
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            message: help,
                        }]);
                    }

                    diagnostics.push(diagnostic);
                }
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Convert byte offsets to LSP range.
    fn span_to_range(&self, source: &str, start: usize, end: usize) -> Range {
        let start_pos = self.offset_to_position(source, start);
        let end_pos = self.offset_to_position(source, end);
        Range::new(start_pos, end_pos)
    }

    /// Convert byte offset to LSP position.
    fn offset_to_position(&self, source: &str, offset: usize) -> Position {
        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Position::new(line, col)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AnvomidavServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![" ".into(), ":".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "anvomidav-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Anvomidav LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = Rope::from_str(&params.text_document.text);
        let version = params.text_document.version;

        self.documents.insert(uri.clone(), Document { content, version });
        self.validate_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(mut doc) = self.documents.get_mut(&uri) {
            // Full sync - replace entire content
            if let Some(change) = params.content_changes.into_iter().next() {
                doc.content = Rope::from_str(&change.text);
                doc.version = params.text_document.version;
            }
        }

        self.validate_document(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.validate_document(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };

        let source = doc.content.to_string();

        // Find the word at the cursor position
        let offset = self.position_to_offset(&source, position);
        let word = self.word_at_offset(&source, offset);

        // Provide hover info for keywords and elements
        let info = match word.as_str() {
            // Element keywords
            "jump" => Some("**Jump Element**\n\nSyntax: `jump <rotation> <kind>`\n\nRotations: single, double, triple, quad\nKinds: axel, lutz, flip, loop, salchow, toe_loop, euler"),
            "spin" => Some("**Spin Element**\n\nSyntax: `spin <positions> [level]`\n\nPositions: upright, sit, camel, layback, biellmann\nLevels: B, L1, L2, L3, L4"),
            "step" => Some("**Step Sequence**\n\nSyntax: `step <pattern> [level]`\n\nPatterns: straight, circular, serpentine, diagonal, midline"),
            "lift" => Some("**Lift Element** (Pairs)\n\nSyntax: `lift <group> [level]`\n\nGroups: Gr1, Gr2, Gr3, Gr4, Gr5"),
            "throw" => Some("**Throw Jump** (Pairs)\n\nSyntax: `throw <rotation> <kind>`"),
            "twist" => Some("**Twist Lift** (Pairs)\n\nSyntax: `twist <rotation> [level]`"),
            "death_spiral" => Some("**Death Spiral** (Pairs)\n\nSyntax: `death_spiral <edge> [level]`\n\nEdges: LFO, LFI, LBO, LBI, RFO, RFI, RBO, RBI"),
            "choreographic" => Some("**Choreographic Element**\n\nSyntax: `choreographic <kind>`\n\nKinds: spiral, spread, ina, hydroblading, pivot"),

            // Segment kinds
            "short" => Some("**Short Program**\n\n2:40 duration (+/- 10 seconds)\nRequired elements based on discipline"),
            "free" => Some("**Free Skating**\n\n4:00 duration (+/- 10 seconds)\nWell-balanced program requirements"),
            "rhythm" => Some("**Rhythm Dance** (Ice Dance)\n\n2:50 duration (+/- 10 seconds)\nRequired pattern dance sequences"),
            "exhibition" => Some("**Exhibition/Gala**\n\nNo time limits or element requirements\nPure artistic expression"),

            // Rotations
            "triple" => Some("**Triple** (3 rotations)"),
            "quad" => Some("**Quadruple** (4 rotations)\n\nHighest difficulty rotation count"),

            _ => None,
        };

        Ok(info.map(|text| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text.to_string(),
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };

        let source = doc.content.to_string();
        let offset = self.position_to_offset(&source, position);

        // Get context for completion
        let line_start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..offset];

        let mut items = Vec::new();

        // Context-aware completions
        if line.trim().is_empty() || line.ends_with('{') {
            // Top-level or block start
            items.extend(vec![
                self.completion_item("sequence", CompletionItemKind::KEYWORD, "sequence { }"),
                self.completion_item("jump", CompletionItemKind::KEYWORD, "jump triple "),
                self.completion_item("spin", CompletionItemKind::KEYWORD, "spin camel "),
                self.completion_item("step", CompletionItemKind::KEYWORD, "step circular "),
                self.completion_item("choreographic", CompletionItemKind::KEYWORD, "choreographic spiral"),
            ]);
        }

        if line.contains("jump") || line.contains("throw") {
            // After jump/throw keyword
            items.extend(vec![
                self.completion_item("single", CompletionItemKind::ENUM_MEMBER, "single"),
                self.completion_item("double", CompletionItemKind::ENUM_MEMBER, "double"),
                self.completion_item("triple", CompletionItemKind::ENUM_MEMBER, "triple"),
                self.completion_item("quad", CompletionItemKind::ENUM_MEMBER, "quad"),
            ]);
        }

        if line.contains("triple") || line.contains("quad") || line.contains("double") || line.contains("single") {
            // After rotation
            items.extend(vec![
                self.completion_item("axel", CompletionItemKind::ENUM_MEMBER, "axel"),
                self.completion_item("lutz", CompletionItemKind::ENUM_MEMBER, "lutz"),
                self.completion_item("flip", CompletionItemKind::ENUM_MEMBER, "flip"),
                self.completion_item("loop", CompletionItemKind::ENUM_MEMBER, "loop"),
                self.completion_item("salchow", CompletionItemKind::ENUM_MEMBER, "salchow"),
                self.completion_item("toe_loop", CompletionItemKind::ENUM_MEMBER, "toe_loop"),
            ]);
        }

        if line.contains("spin") {
            // Spin positions
            items.extend(vec![
                self.completion_item("upright", CompletionItemKind::ENUM_MEMBER, "upright"),
                self.completion_item("sit", CompletionItemKind::ENUM_MEMBER, "sit"),
                self.completion_item("camel", CompletionItemKind::ENUM_MEMBER, "camel"),
                self.completion_item("layback", CompletionItemKind::ENUM_MEMBER, "layback"),
            ]);
        }

        if line.contains("segment") && line.contains(":") {
            // Segment kinds
            items.extend(vec![
                self.completion_item("short", CompletionItemKind::ENUM_MEMBER, "short"),
                self.completion_item("free", CompletionItemKind::ENUM_MEMBER, "free"),
                self.completion_item("rhythm", CompletionItemKind::ENUM_MEMBER, "rhythm"),
                self.completion_item("exhibition", CompletionItemKind::ENUM_MEMBER, "exhibition"),
            ]);
        }

        // Level completions
        items.extend(vec![
            self.completion_item("L1", CompletionItemKind::ENUM_MEMBER, "L1"),
            self.completion_item("L2", CompletionItemKind::ENUM_MEMBER, "L2"),
            self.completion_item("L3", CompletionItemKind::ENUM_MEMBER, "L3"),
            self.completion_item("L4", CompletionItemKind::ENUM_MEMBER, "L4"),
        ]);

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let Some(doc) = self.documents.get(uri) else {
            return Ok(None);
        };

        let source = doc.content.to_string();

        match parse(&source, FileId(0)) {
            Ok(program) => {
                let mut symbols = Vec::new();

                // Program symbol
                let program_range = Range::new(Position::new(0, 0), Position::new(0, 10));

                let mut children = Vec::new();

                // Add segments as children
                for segment in &program.segments {
                    let segment_range = self.span_to_range(
                        &source,
                        segment.span.start as usize,
                        segment.span.end as usize,
                    );

                    let mut seq_children = Vec::new();
                    for seq in &segment.sequences {
                        let seq_range = self.span_to_range(
                            &source,
                            seq.span.start as usize,
                            seq.span.end as usize,
                        );
                        let seq_name = seq.name.as_ref()
                            .map(|n| n.node.clone())
                            .unwrap_or_else(|| "sequence".into());

                        #[allow(deprecated)]
                        seq_children.push(DocumentSymbol {
                            name: seq_name,
                            detail: Some(format!("{} elements", seq.elements.len())),
                            kind: SymbolKind::FUNCTION,
                            range: seq_range,
                            selection_range: seq_range,
                            children: None,
                            tags: None,
                            deprecated: None,
                        });
                    }

                    #[allow(deprecated)]
                    children.push(DocumentSymbol {
                        name: segment.name.node.clone(),
                        detail: Some(format!("{:?}", segment.kind)),
                        kind: SymbolKind::CLASS,
                        range: segment_range,
                        selection_range: segment_range,
                        children: Some(seq_children),
                        tags: None,
                        deprecated: None,
                    });
                }

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: program.name.node.clone(),
                    detail: Some("program".into()),
                    kind: SymbolKind::MODULE,
                    range: program_range,
                    selection_range: program_range,
                    children: Some(children),
                    tags: None,
                    deprecated: None,
                });

                Ok(Some(DocumentSymbolResponse::Nested(symbols)))
            }
            Err(_) => Ok(None),
        }
    }
}

impl AnvomidavServer {
    fn position_to_offset(&self, source: &str, position: Position) -> usize {
        let mut offset = 0;
        for (line_num, line) in source.lines().enumerate() {
            if line_num == position.line as usize {
                offset += position.character as usize;
                break;
            }
            offset += line.len() + 1; // +1 for newline
        }
        offset.min(source.len())
    }

    fn word_at_offset(&self, source: &str, offset: usize) -> String {
        let bytes = source.as_bytes();

        // Find word start
        let mut start = offset;
        while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            start -= 1;
        }

        // Find word end
        let mut end = offset;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }

        source[start..end].to_string()
    }

    fn completion_item(&self, label: &str, kind: CompletionItemKind, insert: &str) -> CompletionItem {
        CompletionItem {
            label: label.into(),
            kind: Some(kind),
            insert_text: Some(insert.into()),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(AnvomidavServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
