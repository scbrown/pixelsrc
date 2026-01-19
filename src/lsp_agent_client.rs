//! LspAgentClient - Library for agent-LSP communication
//!
//! Provides a simple, synchronous API for AI agents and CLI tools to interact
//! with the Pixelsrc language server functionality without the full LSP protocol.
//!
//! # Example
//!
//! ```rust,ignore
//! use ttp::lsp_agent_client::LspAgentClient;
//!
//! let client = LspAgentClient::new();
//!
//! // Verify content
//! let result = client.verify_content(r#"{"type": "sprite", "name": "test", "grid": ["{a}"]}"#);
//! println!("Valid: {}", result.valid);
//!
//! // Get completions at position 2:45 (line 2, column 45)
//! let content = r#"{"type": "palette", "name": "p", "colors": {"{red}": "#FF0000"}}
//! {"type": "sprite", "name": "s", "grid": ["{red}"]}"#;
//! let completions = client.get_completions(content, 2, 45);
//! for c in &completions.items {
//!     println!("{}: {}", c.label, c.detail.clone().unwrap_or_default());
//! }
//! ```

use crate::tokenizer::tokenize;
use crate::validate::{Severity, ValidationIssue, Validator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of content verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the content is valid (no errors, optionally no warnings in strict mode)
    pub valid: bool,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// List of errors
    pub errors: Vec<Diagnostic>,
    /// List of warnings
    pub warnings: Vec<Diagnostic>,
}

/// A diagnostic message (error or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Line number (1-indexed)
    pub line: usize,
    /// Issue type identifier
    pub issue_type: String,
    /// Human-readable message
    pub message: String,
    /// Optional context (e.g., sprite name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional suggestion for fixing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Result of completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult {
    /// List of completion items
    pub items: Vec<CompletionItem>,
}

/// A single completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display label
    pub label: String,
    /// Text to insert (if different from label)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
    /// Additional detail (e.g., color value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Kind of completion (e.g., "color", "token")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Grid position information for hover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPosition {
    /// Column index (0-indexed)
    pub x: usize,
    /// Row index (0-indexed)
    pub y: usize,
    /// The token at this position
    pub token: String,
    /// Width of this row in tokens
    pub row_width: usize,
    /// Expected width (from first row or size field)
    pub expected_width: usize,
    /// Name of the sprite
    pub sprite_name: String,
    /// Whether the row is properly aligned
    pub aligned: bool,
}

/// LspAgentClient provides programmatic access to LSP functionality
///
/// This client is designed for AI agents and CLI tools that need to
/// validate content or get completions without running a full LSP server.
#[derive(Debug, Default)]
pub struct LspAgentClient {
    /// Whether to treat warnings as errors in validation
    strict: bool,
}

impl LspAgentClient {
    /// Create a new LspAgentClient
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a new LspAgentClient with strict mode enabled
    ///
    /// In strict mode, warnings are treated as validation failures.
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Verify content and return diagnostics
    ///
    /// Validates the provided content line by line and returns a structured
    /// result containing all errors and warnings found.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to validate (multi-line string)
    ///
    /// # Returns
    ///
    /// A `VerificationResult` containing validity status and all diagnostics.
    pub fn verify_content(&self, content: &str) -> VerificationResult {
        let mut validator = Validator::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_number = line_idx + 1;
            validator.validate_line(line_number, line);
        }

        let issues = validator.into_issues();

        let errors: Vec<Diagnostic> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .map(Self::issue_to_diagnostic)
            .collect();

        let warnings: Vec<Diagnostic> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Warning))
            .map(Self::issue_to_diagnostic)
            .collect();

        let error_count = errors.len();
        let warning_count = warnings.len();

        // Determine validity based on strict mode
        let valid =
            if self.strict { error_count == 0 && warning_count == 0 } else { error_count == 0 };

        VerificationResult { valid, error_count, warning_count, errors, warnings }
    }

    /// Get completions at a specific position
    ///
    /// Returns token completions available at the given position in the content.
    /// This is useful for autocomplete functionality in editors or agent tooling.
    ///
    /// # Arguments
    ///
    /// * `content` - The full document content
    /// * `line` - Line number (1-indexed)
    /// * `character` - Character position within the line (0-indexed)
    ///
    /// # Returns
    ///
    /// A `CompletionResult` containing all available completions.
    pub fn get_completions(
        &self,
        content: &str,
        line: usize,
        _character: usize,
    ) -> CompletionResult {
        // Collect defined tokens from palettes
        let defined_tokens = Self::collect_defined_tokens(content);

        // Build completion items
        let mut items: Vec<CompletionItem> = Vec::new();

        // Add built-in transparent token
        items.push(CompletionItem {
            label: "{_}".to_string(),
            insert_text: Some("{_}".to_string()),
            detail: Some("Transparent (built-in)".to_string()),
            kind: Some("color".to_string()),
        });

        // Add the standard dot token for transparent
        items.push(CompletionItem {
            label: ".".to_string(),
            insert_text: Some(".".to_string()),
            detail: Some("Transparent (shorthand)".to_string()),
            kind: Some("color".to_string()),
        });

        // Check if we're in a sprite's grid context on the specified line
        let is_in_grid = if line > 0 {
            content.lines().nth(line - 1).map(Self::is_grid_context).unwrap_or(false)
        } else {
            false
        };

        // Add defined tokens from palettes
        for (token, color) in defined_tokens {
            items.push(CompletionItem {
                label: token.clone(),
                insert_text: Some(token),
                detail: Some(color),
                kind: Some("color".to_string()),
            });
        }

        // If we're in a grid context, prioritize tokens
        if is_in_grid {
            // Sort to put tokens first (items starting with '{')
            items.sort_by(|a, b| {
                let a_is_token = a.label.starts_with('{');
                let b_is_token = b.label.starts_with('{');
                b_is_token.cmp(&a_is_token)
            });
        }

        CompletionResult { items }
    }

    /// Get grid position information at a specific location
    ///
    /// Returns information about the grid position if the cursor is within
    /// a sprite's grid array.
    ///
    /// # Arguments
    ///
    /// * `content` - The full document content
    /// * `line` - Line number (1-indexed)
    /// * `character` - Character position within the line (0-indexed)
    ///
    /// # Returns
    ///
    /// `Some(GridPosition)` if the cursor is in a grid, `None` otherwise.
    pub fn get_grid_position(
        &self,
        content: &str,
        line: usize,
        character: usize,
    ) -> Option<GridPosition> {
        if line == 0 {
            return None;
        }

        let line_content = content.lines().nth(line - 1)?;
        Self::parse_grid_context(line_content, character as u32)
    }

    /// Verify content and return JSON string
    ///
    /// Convenience method that returns the verification result as a JSON string.
    pub fn verify_content_json(&self, content: &str) -> String {
        let result = self.verify_content(content);
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize result: {}"}}"#, e))
    }

    /// Get completions and return JSON string
    ///
    /// Convenience method that returns completions as a JSON string.
    pub fn get_completions_json(&self, content: &str, line: usize, character: usize) -> String {
        let result = self.get_completions(content, line, character);
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize result: {}"}}"#, e))
    }

    /// Convert a ValidationIssue to a Diagnostic
    fn issue_to_diagnostic(issue: &ValidationIssue) -> Diagnostic {
        Diagnostic {
            line: issue.line,
            issue_type: issue.issue_type.to_string(),
            message: issue.message.clone(),
            context: issue.context.clone(),
            suggestion: issue.suggestion.clone(),
        }
    }

    /// Check if a line appears to be within a grid context
    fn is_grid_context(line: &str) -> bool {
        // Quick check: does this line look like a sprite with a grid?
        if let Ok(obj) = serde_json::from_str::<Value>(line) {
            if let Some(obj) = obj.as_object() {
                return obj.get("type").and_then(|t| t.as_str()) == Some("sprite")
                    && obj.contains_key("grid");
            }
        }
        false
    }

    /// Collect all defined tokens from palettes in the document
    fn collect_defined_tokens(content: &str) -> Vec<(String, String)> {
        let mut tokens = Vec::new();

        for line in content.lines() {
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };

            if obj_type != "palette" {
                continue;
            }

            let colors = match obj.get("colors").and_then(|c| c.as_object()) {
                Some(c) => c,
                None => continue,
            };

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

    /// Parse grid context from a JSON line at a specific character position
    fn parse_grid_context(line: &str, char_pos: u32) -> Option<GridPosition> {
        let obj: Value = serde_json::from_str(line).ok()?;
        let obj = obj.as_object()?;

        if obj.get("type")?.as_str()? != "sprite" {
            return None;
        }

        let sprite_name = obj.get("name")?.as_str()?.to_string();
        let grid = obj.get("grid")?.as_array()?;

        if grid.is_empty() {
            return None;
        }

        // Get expected width from size field or first row
        let expected_width = if let Some(size) = obj.get("size").and_then(|s| s.as_array()) {
            size.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize
        } else {
            let first_row = grid.first()?.as_str()?;
            let (tokens, _) = tokenize(first_row);
            tokens.len()
        };

        // Find the "grid" key position
        let grid_key_pos = line.find("\"grid\"")?;
        let after_key = &line[grid_key_pos..];
        let bracket_offset = after_key.find('[')?;
        let grid_array_start = grid_key_pos + bracket_offset;

        if (char_pos as usize) <= grid_array_start {
            return None;
        }

        let grid_portion = &line[grid_array_start..];
        let char_in_grid = (char_pos as usize) - grid_array_start;

        let mut pos = 0;
        let chars: Vec<char> = grid_portion.chars().collect();

        for (row_idx, grid_row) in grid.iter().enumerate() {
            let row_str = grid_row.as_str()?;

            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            if pos >= chars.len() {
                return None;
            }

            let string_start = pos + 1;

            pos += 1;
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 2;
                    continue;
                }
                pos += 1;
            }

            let string_end = pos;

            if char_in_grid >= string_start && char_in_grid < string_end {
                let char_in_string = char_in_grid - string_start;
                let (tokens, _) = tokenize(row_str);
                let row_width = tokens.len();

                let mut string_pos = 0;
                for (token_idx, token) in tokens.iter().enumerate() {
                    let token_start = string_pos;
                    let token_end = string_pos + token.len();

                    if char_in_string >= token_start && char_in_string < token_end {
                        return Some(GridPosition {
                            x: token_idx,
                            y: row_idx,
                            token: token.clone(),
                            row_width,
                            expected_width,
                            sprite_name,
                            aligned: row_width == expected_width,
                        });
                    }

                    string_pos = token_end;
                }
            }

            pos += 1;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_valid_content() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##;
        let result = client.verify_content(content);

        assert!(result.valid);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_verify_invalid_json() {
        let client = LspAgentClient::new();
        let content = "{not valid json}";
        let result = client.verify_content(content);

        assert!(!result.valid);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.errors[0].issue_type, "json_syntax");
    }

    #[test]
    fn test_verify_missing_type() {
        let client = LspAgentClient::new();
        let content = r#"{"name": "test"}"#;
        let result = client.verify_content(content);

        assert!(!result.valid);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.errors[0].issue_type, "missing_type");
    }

    #[test]
    fn test_verify_warning_not_error() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "unknown", "name": "test"}"#;
        let result = client.verify_content(content);

        // Warning doesn't make it invalid in non-strict mode
        assert!(result.valid);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_verify_strict_mode() {
        let client = LspAgentClient::strict();
        let content = r#"{"type": "unknown", "name": "test"}"#;
        let result = client.verify_content(content);

        // Warning makes it invalid in strict mode
        assert!(!result.valid);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_verify_undefined_token() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "s", "palette": "test", "grid": ["{a}{b}"]}"##;
        let result = client.verify_content(content);

        // Undefined token {b} should be a warning
        assert!(result.valid); // Still valid (only warning)
        assert_eq!(result.warning_count, 1);
        assert!(result.warnings[0].message.contains("{b}"));
    }

    #[test]
    fn test_get_completions_basic() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000", "{blue}": "#0000FF"}}"##;
        let result = client.get_completions(content, 1, 0);

        // Should have built-in tokens + defined tokens
        assert!(result.items.len() >= 4); // {_}, ., {red}, {blue}

        let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"{_}"));
        assert!(labels.contains(&"."));
        assert!(labels.contains(&"{red}"));
        assert!(labels.contains(&"{blue}"));
    }

    #[test]
    fn test_get_completions_with_palette() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "p", "colors": {"{skin}": "#FFE0BD", "{hair}": "#4A3C31"}}
{"type": "sprite", "name": "s", "palette": "p", "grid": ["{"]}"##;
        let result = client.get_completions(content, 2, 50);

        let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"{skin}"));
        assert!(labels.contains(&"{hair}"));
    }

    #[test]
    fn test_get_grid_position() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "sprite", "name": "test", "grid": ["{a}{b}{c}"]}"#;

        // Position within {b}
        let grid_start = content.find("[\"").unwrap() + 2 + 3; // After [" and {a}
        let pos = client.get_grid_position(content, 1, grid_start);

        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);
        assert_eq!(pos.token, "{b}");
        assert_eq!(pos.sprite_name, "test");
        assert!(pos.aligned);
    }

    #[test]
    fn test_verify_content_json() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##;
        let json = client.verify_content_json(content);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["valid"], true);
        assert_eq!(parsed["error_count"], 0);
    }

    #[test]
    fn test_get_completions_json() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##;
        let json = client.get_completions_json(content, 1, 0);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["items"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn test_multiline_verification() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "p1", "colors": {"{a}": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"{b}": "#00FF00"}}
{"type": "sprite", "name": "s1", "palette": "p1", "grid": ["{a}"]}
{"type": "sprite", "name": "s2", "palette": "p2", "grid": ["{b}"]}"##;

        let result = client.verify_content(content);
        assert!(result.valid);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_suggestion_in_diagnostic() {
        let client = LspAgentClient::new();
        // Typo: {skni} instead of {skin}
        let content = r##"{"type": "palette", "name": "p", "colors": {"{skin}": "#FFE0BD"}}
{"type": "sprite", "name": "s", "palette": "p", "grid": ["{skni}"]}"##;

        let result = client.verify_content(content);
        assert_eq!(result.warning_count, 1);
        assert!(result.warnings[0].suggestion.is_some());
        assert!(result.warnings[0].suggestion.as_ref().unwrap().contains("{skin}"));
    }

    #[test]
    fn test_empty_content() {
        let client = LspAgentClient::new();
        let result = client.verify_content("");
        assert!(result.valid);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_whitespace_only_content() {
        let client = LspAgentClient::new();
        let result = client.verify_content("   \n\n   \n");
        assert!(result.valid);
        assert_eq!(result.error_count, 0);
    }
}
