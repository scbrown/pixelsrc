//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.

use crate::tokenizer::tokenize;
use crate::validate::{Severity, ValidationIssue, Validator};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// Information about a token's position in a grid
#[derive(Debug, Clone)]
struct GridInfo {
    /// Column index (0-indexed)
    x: usize,
    /// Row index (0-indexed within grid array)
    y: usize,
    /// The token at this position
    token: String,
    /// Width of this row in tokens
    row_width: usize,
    /// Expected width (from first row or size field)
    expected_width: usize,
    /// Name of the sprite
    sprite_name: String,
}

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

    /// Parse grid context from a JSON line at a specific character position
    ///
    /// Returns GridInfo if the cursor is positioned within a grid token.
    fn parse_grid_context(line: &str, char_pos: u32) -> Option<GridInfo> {
        // Parse the JSON line
        let obj: Value = serde_json::from_str(line).ok()?;
        let obj = obj.as_object()?;

        // Must be a sprite type
        let obj_type = obj.get("type")?.as_str()?;
        if obj_type != "sprite" {
            return None;
        }

        let sprite_name = obj.get("name")?.as_str()?.to_string();

        // Get the grid array
        let grid = obj.get("grid")?.as_array()?;
        if grid.is_empty() {
            return None;
        }

        // Get expected width from size field or first row
        let expected_width = if let Some(size) = obj.get("size").and_then(|s| s.as_array()) {
            size.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize
        } else {
            // Use first row width as expected
            let first_row = grid.first()?.as_str()?;
            let (tokens, _) = tokenize(first_row);
            tokens.len()
        };

        // Find the "grid" key position in the raw JSON
        // We need to locate where the grid array starts in the line
        let grid_key_pos = line.find("\"grid\"")?;

        // Find the opening bracket of the grid array
        let after_key = &line[grid_key_pos..];
        let bracket_offset = after_key.find('[')?;
        let grid_array_start = grid_key_pos + bracket_offset;

        // If cursor is before the grid array, no hover
        if (char_pos as usize) <= grid_array_start {
            return None;
        }

        // Now we need to find which row string contains the cursor
        // Walk through the grid array portion of the line
        let grid_portion = &line[grid_array_start..];
        let char_in_grid = (char_pos as usize) - grid_array_start;

        // Parse through the grid array manually to find string positions
        let mut pos = 0;
        let chars: Vec<char> = grid_portion.chars().collect();

        for (row_idx, grid_row) in grid.iter().enumerate() {
            let row_str = grid_row.as_str()?;

            // Find the opening quote for this row string
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            if pos >= chars.len() {
                return None;
            }

            let string_start = pos + 1; // Position after opening quote

            // Find the closing quote
            pos += 1; // Move past opening quote
            while pos < chars.len() && chars[pos] != '"' {
                // Handle escaped quotes
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 2;
                    continue;
                }
                pos += 1;
            }

            let string_end = pos; // Position of closing quote

            // Check if cursor is within this string
            if char_in_grid >= string_start && char_in_grid < string_end {
                // Cursor is in this row string
                let char_in_string = char_in_grid - string_start;

                // Tokenize the row and find which token the cursor is in
                let (tokens, _) = tokenize(row_str);
                let row_width = tokens.len();

                // Track position within the string to map to token index
                let mut string_pos = 0;
                for (token_idx, token) in tokens.iter().enumerate() {
                    let token_start = string_pos;
                    let token_end = string_pos + token.len();

                    if char_in_string >= token_start && char_in_string < token_end {
                        return Some(GridInfo {
                            x: token_idx,
                            y: row_idx,
                            token: token.clone(),
                            row_width,
                            expected_width,
                            sprite_name,
                        });
                    }

                    string_pos = token_end;
                }
            }

            pos += 1; // Move past closing quote
        }

        None
    }

    /// Extract document symbols from content
    ///
    /// Returns a list of (name, type, line_number) tuples for all defined objects.
    fn extract_symbols(content: &str) -> Vec<(String, String, usize)> {
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // Try to parse as JSON
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Get type and name fields
            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };

            let name = match obj.get("name").and_then(|n| n.as_str()) {
                Some(n) => n,
                None => continue,
            };

            symbols.push((name.to_string(), obj_type.to_string(), line_num));
        }

        symbols
    }

    /// Map pixelsrc type to LSP SymbolKind
    fn type_to_symbol_kind(obj_type: &str) -> SymbolKind {
        match obj_type {
            "palette" => SymbolKind::CONSTANT,
            "sprite" => SymbolKind::CLASS,
            "animation" => SymbolKind::FUNCTION,
            "composition" => SymbolKind::MODULE,
            _ => SymbolKind::OBJECT,
        }
    }

    /// Collect all defined tokens from palettes in the document
    ///
    /// Returns a list of (token, color) pairs from all palette definitions.
    fn collect_defined_tokens(content: &str) -> Vec<(String, String)> {
        let mut tokens = Vec::new();

        for line in content.lines() {
            // Try to parse as JSON
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Check if it's a palette
            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };

            if obj_type != "palette" {
                continue;
            }

            // Get the colors object
            let colors = match obj.get("colors").and_then(|c| c.as_object()) {
                Some(c) => c,
                None => continue,
            };

            // Extract tokens (keys starting with '{' and ending with '}')
            for (key, value) in colors {
                if key.starts_with('{') && key.ends_with('}') {
                    let color_str = match value.as_str() {
                        Some(s) => s.to_string(),
                        None => continue,
                    };
                    tokens.push((key.clone(), color_str));
                }
            }
        }

        tokens
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["{".to_string()]),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents); // Release lock before async work

        // Get the line at the cursor position
        let line = match content.lines().nth(pos.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };

        // Try to parse grid context at the cursor position
        if let Some(grid_info) = Self::parse_grid_context(line, pos.character) {
            // Format alignment status
            let alignment_status = if grid_info.row_width == grid_info.expected_width {
                "✓ Aligned".to_string()
            } else if grid_info.row_width < grid_info.expected_width {
                format!(
                    "⚠ Short by {} token(s)",
                    grid_info.expected_width - grid_info.row_width
                )
            } else {
                format!(
                    "⚠ Long by {} token(s)",
                    grid_info.row_width - grid_info.expected_width
                )
            };

            let hover_text = format!(
                "**Grid Position**: ({}, {})\n\n\
                 **Token**: `{}`\n\n\
                 **Row Width**: {} tokens\n\n\
                 **Expected Width**: {} tokens\n\n\
                 **Status**: {}\n\n\
                 **Sprite**: `{}`",
                grid_info.x,
                grid_info.y,
                grid_info.token,
                grid_info.row_width,
                grid_info.expected_width,
                alignment_status,
                grid_info.sprite_name,
            );

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Collect defined tokens from the document
        let defined_tokens = Self::collect_defined_tokens(&content);

        // Build completion items
        let mut completions: Vec<CompletionItem> = Vec::new();

        // Add built-in transparent token
        completions.push(CompletionItem {
            label: "{_}".to_string(),
            detail: Some("Transparent (built-in)".to_string()),
            kind: Some(CompletionItemKind::COLOR),
            insert_text: Some("{_}".to_string()),
            ..Default::default()
        });

        // Add the standard dot token for transparent
        completions.push(CompletionItem {
            label: ".".to_string(),
            detail: Some("Transparent (shorthand)".to_string()),
            kind: Some(CompletionItemKind::COLOR),
            insert_text: Some(".".to_string()),
            ..Default::default()
        });

        // Add defined tokens from palettes
        for (token, color) in defined_tokens {
            completions.push(CompletionItem {
                label: token.clone(),
                detail: Some(color),
                kind: Some(CompletionItemKind::COLOR),
                insert_text: Some(token),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Extract symbols using helper method
        let extracted = Self::extract_symbols(&content);

        // Convert to SymbolInformation
        let symbols: Vec<SymbolInformation> = extracted
            .into_iter()
            .map(|(name, obj_type, line_num)| {
                let line = content.lines().nth(line_num).unwrap_or("");
                #[allow(deprecated)]
                SymbolInformation {
                    name,
                    kind: Self::type_to_symbol_kind(&obj_type),
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: line.len() as u32,
                            },
                        },
                    },
                    container_name: None,
                }
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
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

    #[test]
    fn test_parse_grid_context_first_row_first_token() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}{b}{c}"]}"#;
        // Find position of first {a} - after the opening quote of the first grid row
        let grid_start = line.find("[\"").unwrap() + 2; // Position after ["
        let info = PixelsrcLanguageServer::parse_grid_context(line, grid_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.x, 0);
        assert_eq!(info.y, 0);
        assert_eq!(info.token, "{a}");
        assert_eq!(info.row_width, 3);
        assert_eq!(info.expected_width, 3);
        assert_eq!(info.sprite_name, "test");
    }

    #[test]
    fn test_parse_grid_context_second_token() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}{b}{c}"]}"#;
        // Position within {b}
        let grid_start = line.find("[\"").unwrap() + 2 + 3; // After [" and {a}
        let info = PixelsrcLanguageServer::parse_grid_context(line, grid_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.x, 1);
        assert_eq!(info.y, 0);
        assert_eq!(info.token, "{b}");
    }

    #[test]
    fn test_parse_grid_context_second_row() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}{a}", "{b}{b}"]}"#;
        // Find position within second row
        let second_row_start = line.rfind("\"{b}").unwrap() + 1; // Position after the quote
        let info = PixelsrcLanguageServer::parse_grid_context(line, second_row_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.x, 0);
        assert_eq!(info.y, 1);
        assert_eq!(info.token, "{b}");
    }

    #[test]
    fn test_parse_grid_context_with_size() {
        let line = r#"{"type": "sprite", "name": "sized", "size": [4, 2], "grid": ["{a}{a}"]}"#;
        let grid_start = line.find("[\"").unwrap() + 2;
        let info = PixelsrcLanguageServer::parse_grid_context(line, grid_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.row_width, 2);
        assert_eq!(info.expected_width, 4); // From size field
    }

    #[test]
    fn test_parse_grid_context_not_sprite() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##;
        let info = PixelsrcLanguageServer::parse_grid_context(line, 50);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_grid_context_outside_grid() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}{b}"]}"#;
        // Position before the grid array
        let info = PixelsrcLanguageServer::parse_grid_context(line, 10);
        assert!(info.is_none());
    }

    #[test]
    fn test_collect_defined_tokens_single_palette() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "--var": "100"}}"##;
        let tokens = PixelsrcLanguageServer::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2); // Only {a} and {b}, not --var
        assert!(tokens.iter().any(|(t, c)| t == "{a}" && c == "#FF0000"));
        assert!(tokens.iter().any(|(t, c)| t == "{b}" && c == "#00FF00"));
    }

    #[test]
    fn test_collect_defined_tokens_multiple_palettes() {
        let content = r##"{"type": "palette", "name": "p1", "colors": {"{red}": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"{blue}": "#0000FF"}}
{"type": "sprite", "name": "s", "grid": ["{red}{blue}"]}"##;
        let tokens = PixelsrcLanguageServer::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().any(|(t, _)| t == "{red}"));
        assert!(tokens.iter().any(|(t, _)| t == "{blue}"));
    }

    #[test]
    fn test_collect_defined_tokens_no_palettes() {
        let content = r#"{"type": "sprite", "name": "s", "grid": ["{a}{b}"]}"#;
        let tokens = PixelsrcLanguageServer::collect_defined_tokens(content);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_collect_defined_tokens_empty_content() {
        let tokens = PixelsrcLanguageServer::collect_defined_tokens("");
        assert!(tokens.is_empty());
    }

    // === Document Symbol Tests ===

    #[test]
    fn test_extract_symbols_single_palette() {
        let content = r##"{"type": "palette", "name": "hero", "colors": {"{a}": "#FF0000"}}"##;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "hero");
        assert_eq!(symbols[0].1, "palette");
        assert_eq!(symbols[0].2, 0);
    }

    #[test]
    fn test_extract_symbols_single_sprite() {
        let content = r#"{"type": "sprite", "name": "player", "grid": ["{a}"]}"#;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "player");
        assert_eq!(symbols[0].1, "sprite");
        assert_eq!(symbols[0].2, 0);
    }

    #[test]
    fn test_extract_symbols_animation() {
        let content = r#"{"type": "animation", "name": "walk_cycle", "frames": ["frame1"]}"#;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "walk_cycle");
        assert_eq!(symbols[0].1, "animation");
    }

    #[test]
    fn test_extract_symbols_composition() {
        let content = r#"{"type": "composition", "name": "scene1", "layers": []}"#;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "scene1");
        assert_eq!(symbols[0].1, "composition");
    }

    #[test]
    fn test_extract_symbols_multiple_objects() {
        let content = r##"{"type": "palette", "name": "colors", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "hero", "grid": ["{a}"]}
{"type": "sprite", "name": "enemy", "grid": ["{a}"]}
{"type": "animation", "name": "idle", "frames": ["hero"]}"##;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 4);

        // Check names in order
        assert_eq!(symbols[0].0, "colors");
        assert_eq!(symbols[0].1, "palette");
        assert_eq!(symbols[0].2, 0);

        assert_eq!(symbols[1].0, "hero");
        assert_eq!(symbols[1].1, "sprite");
        assert_eq!(symbols[1].2, 1);

        assert_eq!(symbols[2].0, "enemy");
        assert_eq!(symbols[2].1, "sprite");
        assert_eq!(symbols[2].2, 2);

        assert_eq!(symbols[3].0, "idle");
        assert_eq!(symbols[3].1, "animation");
        assert_eq!(symbols[3].2, 3);
    }

    #[test]
    fn test_extract_symbols_skips_invalid_json() {
        let content = r##"this is not json
{"type": "palette", "name": "valid", "colors": {}}
also not json"##;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "valid");
        assert_eq!(symbols[0].2, 1); // Line 1 (0-indexed)
    }

    #[test]
    fn test_extract_symbols_skips_missing_type() {
        let content = r#"{"name": "no_type", "colors": {}}"#;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_extract_symbols_skips_missing_name() {
        let content = r#"{"type": "palette", "colors": {}}"#;
        let symbols = PixelsrcLanguageServer::extract_symbols(content);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_extract_symbols_empty_content() {
        let symbols = PixelsrcLanguageServer::extract_symbols("");
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_type_to_symbol_kind_palette() {
        assert_eq!(
            PixelsrcLanguageServer::type_to_symbol_kind("palette"),
            SymbolKind::CONSTANT
        );
    }

    #[test]
    fn test_type_to_symbol_kind_sprite() {
        assert_eq!(
            PixelsrcLanguageServer::type_to_symbol_kind("sprite"),
            SymbolKind::CLASS
        );
    }

    #[test]
    fn test_type_to_symbol_kind_animation() {
        assert_eq!(
            PixelsrcLanguageServer::type_to_symbol_kind("animation"),
            SymbolKind::FUNCTION
        );
    }

    #[test]
    fn test_type_to_symbol_kind_composition() {
        assert_eq!(
            PixelsrcLanguageServer::type_to_symbol_kind("composition"),
            SymbolKind::MODULE
        );
    }

    #[test]
    fn test_type_to_symbol_kind_unknown() {
        assert_eq!(
            PixelsrcLanguageServer::type_to_symbol_kind("unknown"),
            SymbolKind::OBJECT
        );
    }
}
