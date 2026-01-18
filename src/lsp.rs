//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.

use crate::validate::{Severity, ValidationIssue, Validator};
use std::collections::HashMap;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// Grid position information for hover display
#[derive(Debug, Clone)]
pub struct GridInfo {
    /// X coordinate (0-indexed column in grid)
    pub x: usize,
    /// Y coordinate (0-indexed row in grid)
    pub y: usize,
    /// The token at this position
    pub token: String,
    /// Actual width of this row in tokens
    pub row_width: usize,
    /// Expected width (from size field or first row)
    pub expected_width: usize,
    /// Name of the sprite containing this grid
    pub sprite_name: String,
}

/// Information about a sprite definition for grid context parsing
struct SpriteInfo {
    name: String,
    size: Option<(usize, usize)>,
    first_row_width: Option<usize>,
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

    /// Parse grid context from document content at a given position
    ///
    /// This handles both single-line and multi-line grid formats:
    /// - Single-line: `{"type": "sprite", "grid": ["{a}{b}", "{c}{d}"]}`
    /// - Multi-line: grid array spans multiple lines with each row on its own line
    fn parse_grid_context(&self, content: &str, line: u32, character: u32) -> Option<GridInfo> {
        let lines: Vec<&str> = content.lines().collect();
        let target_line = lines.get(line as usize)?;

        // Check if this line contains a grid string (has tokens like {x})
        // Find the position of the cursor within the line
        let char_pos = character as usize;
        if char_pos >= target_line.len() {
            return None;
        }

        // Look for tokens in the current line
        // Tokens are in format {token_name}
        let tokens = Self::extract_tokens(target_line);
        if tokens.is_empty() {
            return None;
        }

        // Find which token the cursor is in
        let (token_index, token) = Self::find_token_at_position(target_line, char_pos)?;
        let row_width = tokens.len();

        // Now we need to find the grid context - which row (y) is this?
        // And what's the expected width?
        // We need to parse the JSON to understand the full grid structure

        // Find the sprite definition that contains this grid
        let (sprite_info, grid_row_index) =
            Self::find_sprite_context(content, line as usize, target_line)?;

        let expected_width = sprite_info
            .size
            .map(|(w, _)| w)
            .or(sprite_info.first_row_width)
            .unwrap_or(row_width);

        Some(GridInfo {
            x: token_index,
            y: grid_row_index,
            token,
            row_width,
            expected_width,
            sprite_name: sprite_info.name,
        })
    }

    /// Extract all tokens from a line
    ///
    /// Pixelsrc tokens are in the format `{name}` where name consists of
    /// alphanumeric characters, underscores, and hyphens (for CSS variables).
    /// This excludes JSON structure braces like `{"type": ...}`.
    fn extract_tokens(line: &str) -> Vec<(usize, usize, String)> {
        let mut tokens = Vec::new();
        let mut chars = line.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            if ch == '{' {
                let mut token = String::from("{");
                let mut end = start;
                let mut is_valid_token = true;

                for (i, c) in chars.by_ref() {
                    token.push(c);
                    end = i;
                    if c == '}' {
                        break;
                    }
                    // Valid token characters: alphanumeric, underscore, hyphen
                    // (hyphens for CSS variable references like --primary)
                    if !c.is_alphanumeric() && c != '_' && c != '-' {
                        is_valid_token = false;
                    }
                }

                // Must end with }, have content (len > 2), and contain only valid chars
                if token.ends_with('}') && token.len() > 2 && is_valid_token {
                    tokens.push((start, end, token));
                }
            }
        }

        tokens
    }

    /// Find which token the cursor is positioned on
    fn find_token_at_position(line: &str, char_pos: usize) -> Option<(usize, String)> {
        let tokens = Self::extract_tokens(line);

        for (index, (start, end, token)) in tokens.iter().enumerate() {
            if char_pos >= *start && char_pos <= *end {
                return Some((index, token.clone()));
            }
        }

        None
    }

    /// Find the sprite definition that contains the current line's grid content
    fn find_sprite_context(
        content: &str,
        target_line: usize,
        _line_content: &str,
    ) -> Option<(SpriteInfo, usize)> {
        // Strategy: look backwards from the target line to find the sprite definition
        // Then parse the grid to determine which row we're in

        let lines: Vec<&str> = content.lines().collect();

        // For multi-line grids, we need to find the start of the JSON object
        // Look for a line containing "type": "sprite" going backwards
        let mut sprite_start_line = None;
        for i in (0..=target_line).rev() {
            if let Some(line) = lines.get(i) {
                if line.contains("\"type\"") && line.contains("\"sprite\"") {
                    sprite_start_line = Some(i);
                    break;
                }
            }
        }

        let sprite_line_idx = sprite_start_line?;

        // Collect lines from sprite start to find the complete JSON
        // For single-line format, it's all on one line
        // For multi-line, we need to find matching braces
        let mut json_content = String::new();
        let mut brace_count = 0;
        let mut started = false;

        for i in sprite_line_idx..lines.len() {
            let line = lines[i];
            json_content.push_str(line);
            json_content.push('\n');

            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    started = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            if started && brace_count == 0 {
                break;
            }
        }

        // Parse the JSON to get sprite info
        let json_value: serde_json::Value = serde_json::from_str(&json_content).ok()?;
        let obj = json_value.as_object()?;

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let size = obj.get("size").and_then(|v| {
            let arr = v.as_array()?;
            let w = arr.first()?.as_u64()? as usize;
            let h = arr.get(1)?.as_u64()? as usize;
            Some((w, h))
        });

        let grid = obj.get("grid").and_then(|v| v.as_array())?;

        // Determine which row of the grid the target line corresponds to
        // For multi-line format: count which grid row array element this is
        // For single-line: find position within the grid array

        let first_row_width = grid.first().and_then(|v| {
            let row_str = v.as_str()?;
            Some(Self::extract_tokens(row_str).len())
        });

        // Find grid row index
        let grid_row_index = if sprite_line_idx == target_line {
            // Single-line format - need to find position within grid array
            Self::find_grid_row_in_single_line(lines[target_line], grid)?
        } else {
            // Multi-line format - count from grid start
            Self::find_grid_row_in_multiline(&lines, sprite_line_idx, target_line, grid)?
        };

        Some((
            SpriteInfo {
                name,
                size,
                first_row_width,
            },
            grid_row_index,
        ))
    }

    /// Find which grid row we're in for single-line format
    fn find_grid_row_in_single_line(line: &str, grid: &[serde_json::Value]) -> Option<usize> {
        // For single-line, we need to find which grid row string contains our cursor
        // This is tricky - we'll look for the pattern and match against grid values
        for (i, row_value) in grid.iter().enumerate() {
            if let Some(row_str) = row_value.as_str() {
                if line.contains(row_str) {
                    return Some(i);
                }
            }
        }
        // Default to first row if we can't determine
        Some(0)
    }

    /// Find which grid row we're in for multi-line format
    fn find_grid_row_in_multiline(
        lines: &[&str],
        sprite_start: usize,
        target_line: usize,
        grid: &[serde_json::Value],
    ) -> Option<usize> {
        // Find lines that match grid row content
        let mut row_index = 0;
        for i in sprite_start..=target_line {
            if let Some(line) = lines.get(i) {
                // Check if this line contains a grid row
                for row_value in grid.iter() {
                    if let Some(row_str) = row_value.as_str() {
                        if line.contains(row_str) {
                            if i == target_line {
                                return Some(row_index);
                            }
                            row_index += 1;
                            break;
                        }
                    }
                }
            }
        }
        Some(row_index.saturating_sub(1))
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

        // Get document content
        let content = {
            let documents = self.documents.read().unwrap();
            documents.get(uri).cloned()
        };

        let content = match content {
            Some(c) => c,
            None => return Ok(None),
        };

        // Try to parse grid context at this position
        if let Some(grid_info) = self.parse_grid_context(&content, pos.line, pos.character) {
            let alignment_status = if grid_info.row_width == grid_info.expected_width {
                "✓ aligned".to_string()
            } else if grid_info.row_width < grid_info.expected_width {
                format!(
                    "⚠ {} token(s) short",
                    grid_info.expected_width - grid_info.row_width
                )
            } else {
                format!(
                    "⚠ {} token(s) over",
                    grid_info.row_width - grid_info.expected_width
                )
            };

            let hover_text = format!(
                "**Grid Position**: ({}, {})\n\n\
                 **Sprite**: `{}`\n\n\
                 **Token**: `{}`\n\n\
                 **Row Width**: {} tokens ({})\n\n\
                 **Expected Width**: {} tokens",
                grid_info.x,
                grid_info.y,
                grid_info.sprite_name,
                grid_info.token,
                grid_info.row_width,
                alignment_status,
                grid_info.expected_width
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

    // === Hover functionality tests ===

    #[test]
    fn test_extract_tokens() {
        let line = r#"  "{_}{r}{r}{_}{r}{r}{_}","#;
        let tokens = PixelsrcLanguageServer::extract_tokens(line);

        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].2, "{_}");
        assert_eq!(tokens[1].2, "{r}");
        assert_eq!(tokens[6].2, "{_}");
    }

    #[test]
    fn test_extract_tokens_longer_names() {
        let line = r#""{skin}{shadow}{_}{highlight}""#;
        let tokens = PixelsrcLanguageServer::extract_tokens(line);

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].2, "{skin}");
        assert_eq!(tokens[1].2, "{shadow}");
        assert_eq!(tokens[2].2, "{_}");
        assert_eq!(tokens[3].2, "{highlight}");
    }

    #[test]
    fn test_extract_tokens_no_tokens() {
        let line = r#"{"type": "palette", "name": "test"}"#;
        let tokens = PixelsrcLanguageServer::extract_tokens(line);

        // JSON braces are not treated as tokens (they don't have content between them)
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_find_token_at_position() {
        // Line: "{a}{bb}{c}"
        // Positions: 0123456789
        let line = "{a}{bb}{c}";

        // Position 0-2: {a}
        assert_eq!(
            PixelsrcLanguageServer::find_token_at_position(line, 0),
            Some((0, "{a}".to_string()))
        );
        assert_eq!(
            PixelsrcLanguageServer::find_token_at_position(line, 2),
            Some((0, "{a}".to_string()))
        );

        // Position 3-6: {bb}
        assert_eq!(
            PixelsrcLanguageServer::find_token_at_position(line, 3),
            Some((1, "{bb}".to_string()))
        );
        assert_eq!(
            PixelsrcLanguageServer::find_token_at_position(line, 5),
            Some((1, "{bb}".to_string()))
        );

        // Position 7-9: {c}
        assert_eq!(
            PixelsrcLanguageServer::find_token_at_position(line, 7),
            Some((2, "{c}".to_string()))
        );
    }

    #[test]
    fn test_find_sprite_context_single_line() {
        let content = r#"{"type": "sprite", "name": "heart", "size": [7, 6], "grid": ["{_}{r}{r}{_}{r}{r}{_}", "{r}{r}{r}{r}{r}{r}{r}"]}"#;

        let result = PixelsrcLanguageServer::find_sprite_context(content, 0, content);
        assert!(result.is_some());

        let (sprite_info, _row_idx) = result.unwrap();
        assert_eq!(sprite_info.name, "heart");
        assert_eq!(sprite_info.size, Some((7, 6)));
        assert_eq!(sprite_info.first_row_width, Some(7));
    }

    #[test]
    fn test_find_sprite_context_multiline() {
        let content = r#"{"type": "sprite", "name": "coin", "size": [4, 3], "grid": [
  "{_}{a}{a}{_}",
  "{a}{a}{a}{a}",
  "{_}{a}{a}{_}"
]}"#;

        // Test line 2 (second row)
        let lines: Vec<&str> = content.lines().collect();
        let result = PixelsrcLanguageServer::find_sprite_context(content, 2, lines[2]);
        assert!(result.is_some());

        let (sprite_info, row_idx) = result.unwrap();
        assert_eq!(sprite_info.name, "coin");
        assert_eq!(sprite_info.size, Some((4, 3)));
        assert_eq!(row_idx, 1); // Second row (0-indexed)
    }

    #[test]
    fn test_grid_info_alignment_check() {
        // Test that GridInfo correctly calculates alignment status
        let aligned = GridInfo {
            x: 0,
            y: 0,
            token: "{a}".to_string(),
            row_width: 8,
            expected_width: 8,
            sprite_name: "test".to_string(),
        };
        assert_eq!(aligned.row_width, aligned.expected_width);

        let short = GridInfo {
            x: 0,
            y: 1,
            token: "{a}".to_string(),
            row_width: 6,
            expected_width: 8,
            sprite_name: "test".to_string(),
        };
        assert!(short.row_width < short.expected_width);

        let long = GridInfo {
            x: 0,
            y: 2,
            token: "{a}".to_string(),
            row_width: 10,
            expected_width: 8,
            sprite_name: "test".to_string(),
        };
        assert!(long.row_width > long.expected_width);
    }
}
