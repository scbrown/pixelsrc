//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.
//! Includes real-time diagnostics powered by the Validator.

use crate::validate::{Severity, ValidationIssue, Validator};
use std::collections::HashMap;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// The Pixelsrc Language Server
pub struct PixelsrcLanguageServer {
    client: Client,
    /// In-memory cache of open documents
    documents: RwLock<HashMap<Url, String>>,
}

impl PixelsrcLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Validate document content and publish diagnostics to the client
    async fn validate_and_publish(&self, uri: &Url, content: &str) {
        let mut validator = Validator::new();

        // Validate each line of the document
        for (line_idx, line) in content.lines().enumerate() {
            validator.validate_line(line_idx + 1, line);
        }

        // Convert ValidationIssues to LSP Diagnostics
        let diagnostics: Vec<Diagnostic> = validator
            .issues()
            .iter()
            .map(|issue| Self::issue_to_diagnostic(issue))
            .collect();

        // Publish diagnostics to the client
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Convert a ValidationIssue to an LSP Diagnostic
    fn issue_to_diagnostic(issue: &ValidationIssue) -> Diagnostic {
        let severity = match issue.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        // Build the message, including suggestion if available
        let message = if let Some(ref suggestion) = issue.suggestion {
            format!("{} ({})", issue.message, suggestion)
        } else {
            issue.message.clone()
        };

        Diagnostic {
            range: Range {
                // Lines are 0-indexed in LSP, but ValidationIssue uses 1-indexed
                start: Position {
                    line: (issue.line.saturating_sub(1)) as u32,
                    character: 0,
                },
                end: Position {
                    line: (issue.line.saturating_sub(1)) as u32,
                    // Highlight the entire line
                    character: u32::MAX,
                },
            },
            severity: Some(severity),
            code: Some(NumberOrString::String(issue.issue_type.to_string())),
            source: Some("pixelsrc".to_string()),
            message,
            ..Default::default()
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PixelsrcLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "pixelsrc-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Pixelsrc LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store the document content
        if let Ok(mut documents) = self.documents.write() {
            documents.insert(uri.clone(), text.clone());
        }

        // Validate and publish diagnostics
        self.validate_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // We use FULL sync mode, so take the full content from the first change
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Update the stored document content
            if let Ok(mut documents) = self.documents.write() {
                documents.insert(uri.clone(), text.clone());
            }

            // Validate and publish diagnostics
            self.validate_and_publish(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove the document from our cache
        if let Ok(mut documents) = self.documents.write() {
            documents.remove(&uri);
        }

        // Clear diagnostics for closed document
        self.client
            .publish_diagnostics(uri, Vec::new(), None)
            .await;
    }
}

/// Run the LSP server on stdin/stdout
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(PixelsrcLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
