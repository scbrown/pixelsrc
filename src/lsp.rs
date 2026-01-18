//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.

use crate::validate::{Severity, ValidationIssue, Validator};
use std::collections::HashMap;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// The Pixelsrc Language Server
pub struct PixelsrcLanguageServer {
    client: Client,
    /// Document state tracking for open files
    documents: RwLock<HashMap<Url, String>>,
}

impl PixelsrcLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Validate document content and publish diagnostics
    async fn validate_and_publish(&self, uri: &Url, content: &str) {
        let mut validator = Validator::new();
        for (line_num, line) in content.lines().enumerate() {
            validator.validate_line(line_num + 1, line);
        }

        let diagnostics: Vec<Diagnostic> = validator
            .issues()
            .iter()
            .map(|issue| Self::issue_to_diagnostic(issue))
            .collect();

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

        // Build the message with optional suggestion
        let message = if let Some(ref suggestion) = issue.suggestion {
            format!("{} ({})", issue.message, suggestion)
        } else {
            issue.message.clone()
        };

        Diagnostic {
            range: Range {
                start: Position {
                    line: (issue.line - 1) as u32,
                    character: 0,
                },
                end: Position {
                    line: (issue.line - 1) as u32,
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

        // Store document content
        self.documents
            .write()
            .unwrap()
            .insert(uri.clone(), text.clone());

        // Validate and publish diagnostics
        self.validate_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Get the full text from the first change (we use FULL sync)
        if let Some(change) = params.content_changes.into_iter().next() {
            // Store updated content
            self.documents
                .write()
                .unwrap()
                .insert(uri.clone(), change.text.clone());

            // Validate and publish diagnostics
            self.validate_and_publish(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from tracking
        self.documents.write().unwrap().remove(&uri);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::IssueType;

    #[test]
    fn test_issue_to_diagnostic_error() {
        let issue = ValidationIssue::error(5, IssueType::JsonSyntax, "Invalid JSON");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 4); // 0-indexed
        assert_eq!(diagnostic.range.end.line, 4);
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.message, "Invalid JSON");
        assert_eq!(diagnostic.source, Some("pixelsrc".to_string()));
        assert_eq!(
            diagnostic.code,
            Some(NumberOrString::String("json_syntax".to_string()))
        );
    }

    #[test]
    fn test_issue_to_diagnostic_warning() {
        let issue = ValidationIssue::warning(10, IssueType::UndefinedToken, "Undefined token {x}");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 9); // 0-indexed
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.message, "Undefined token {x}");
    }

    #[test]
    fn test_issue_to_diagnostic_with_suggestion() {
        let issue = ValidationIssue::warning(3, IssueType::UndefinedToken, "Undefined token {skni}")
            .with_suggestion("did you mean {skin}?");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 2); // 0-indexed
        assert_eq!(
            diagnostic.message,
            "Undefined token {skni} (did you mean {skin}?)"
        );
    }
}
