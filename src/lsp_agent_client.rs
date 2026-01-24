//! LspAgentClient - Library for agent-LSP communication
//!
//! Provides a simple, synchronous API for AI agents and CLI tools to interact
//! with the Pixelsrc language server functionality without the full LSP protocol.
//!
//! # Example
//!
//! ```rust,no_run
//! use pixelsrc::lsp_agent_client::LspAgentClient;
//!
//! let client = LspAgentClient::new();
//!
//! // Verify content
//! let result = client.verify_content(r#"{"type": "sprite", "name": "test", "grid": ["{a}"]}"#);
//! println!("Valid: {}", result.valid);
//!
//! // Get completions at position 2:45 (line 2, column 45)
//! let content = r##"{"type": "palette", "name": "p", "colors": {"{red}": "#FF0000"}}
//! {"type": "sprite", "name": "s", "grid": ["{red}"]}"##;
//! let completions = client.get_completions(content, 2, 45);
//! for c in &completions.items {
//!     println!("{}: {}", c.label, c.detail.clone().unwrap_or_default());
//! }
//!
//! // Resolve colors (CSS extensions)
//! let palette_content = r##"{"type": "palette", "name": "hero", "colors": {"--base": "#FF6347", "{skin}": "var(--base)"}}"##;
//! let colors = client.resolve_colors(palette_content);
//! for c in &colors.colors {
//!     println!("{}: {} -> {}", c.token, c.original, c.resolved);
//! }
//!
//! // Analyze timing functions
//! let anim_content = r#"{"type": "animation", "name": "walk", "timing_function": "ease-in-out"}"#;
//! let timing = client.analyze_timing(anim_content);
//! for t in &timing.animations {
//!     println!("{}: {} ({})", t.animation, t.timing_function, t.curve_type);
//! }
//! ```

use crate::color::parse_color;
use crate::motion::{parse_timing_function, Interpolation, StepPosition};
use crate::validate::{Severity, ValidationIssue, Validator};
use crate::variables::VariableRegistry;
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

/// Result of color resolution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorResolutionResult {
    /// List of resolved colors
    pub colors: Vec<ResolvedColor>,
    /// Number of colors that failed to resolve
    pub error_count: usize,
    /// Errors encountered during resolution
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// A resolved color with original and computed values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedColor {
    /// Token name (e.g., "{skin}") or variable name (e.g., "--primary")
    pub token: String,
    /// Original value before resolution (e.g., "var(--base)")
    pub original: String,
    /// Resolved hex value (e.g., "#FFCC99")
    pub resolved: String,
    /// Palette name where this color is defined
    pub palette: String,
    /// Whether this is a CSS variable (--name) rather than a token ({name})
    pub is_variable: bool,
}

/// Result of timing function analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysisResult {
    /// List of analyzed animations
    pub animations: Vec<TimingAnalysis>,
}

/// Timing function analysis for a single animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysis {
    /// Animation name
    pub animation: String,
    /// Original timing function string
    pub timing_function: String,
    /// Human-readable description of the timing effect
    pub description: String,
    /// Curve type classification
    pub curve_type: String,
    /// ASCII visualization of the easing curve (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascii_curve: Option<String>,
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

    /// Resolve all CSS colors in content to computed hex values.
    ///
    /// Parses all palettes in the content, resolves `var()` references and
    /// `color-mix()` functions, and returns the computed hex values for each
    /// color definition.
    ///
    /// # Arguments
    ///
    /// * `content` - The full document content
    ///
    /// # Returns
    ///
    /// A `ColorResolutionResult` containing all resolved colors and any errors.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pixelsrc::lsp_agent_client::LspAgentClient;
    ///
    /// let client = LspAgentClient::new();
    /// let content = r##"{"type": "palette", "name": "hero", "colors": {"--base": "#FF6347", "{skin}": "var(--base)"}}"##;
    ///
    /// let result = client.resolve_colors(content);
    /// assert!(result.colors.iter().any(|c| c.token == "{skin}"));
    /// ```
    pub fn resolve_colors(&self, content: &str) -> ColorResolutionResult {
        let mut colors = Vec::new();
        let mut errors = Vec::new();

        // First pass: build variable registry from all palettes
        let registry = Self::build_variable_registry(content);

        // Second pass: collect and resolve all colors
        for line in content.lines() {
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Only process palettes
            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) if t == "palette" => t,
                _ => continue,
            };
            let _ = obj_type;

            let palette_name =
                obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown").to_string();

            // Get the colors object
            let palette_colors = match obj.get("colors").and_then(|c| c.as_object()) {
                Some(c) => c,
                None => continue,
            };

            for (key, value) in palette_colors {
                let original_value = match value.as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                let is_variable = key.starts_with("--");
                let is_token = key.starts_with('{') && key.ends_with('}');

                // Skip non-color entries
                if !is_variable && !is_token {
                    continue;
                }

                // Resolve var() references first
                let resolved_value = match registry.resolve(&original_value) {
                    Ok(v) => v,
                    Err(e) => {
                        errors.push(format!("{}: {}", key, e));
                        continue;
                    }
                };

                // Parse the resolved value as a color and convert to hex
                let hex_value = match parse_color(&resolved_value) {
                    Ok(rgba) => {
                        if rgba.0[3] == 255 {
                            format!("#{:02X}{:02X}{:02X}", rgba.0[0], rgba.0[1], rgba.0[2])
                        } else {
                            format!(
                                "#{:02X}{:02X}{:02X}{:02X}",
                                rgba.0[0], rgba.0[1], rgba.0[2], rgba.0[3]
                            )
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", key, e));
                        continue;
                    }
                };

                colors.push(ResolvedColor {
                    token: key.clone(),
                    original: original_value,
                    resolved: hex_value,
                    palette: palette_name.clone(),
                    is_variable,
                });
            }
        }

        ColorResolutionResult { error_count: errors.len(), colors, errors }
    }

    /// Resolve colors and return JSON string
    ///
    /// Convenience method that returns the color resolution result as a JSON string.
    pub fn resolve_colors_json(&self, content: &str) -> String {
        let result = self.resolve_colors(content);
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize result: {}"}}"#, e))
    }

    /// Analyze timing functions in animations.
    ///
    /// Parses all animations in the content and extracts timing function
    /// information with human-readable descriptions and ASCII curve visualizations.
    ///
    /// # Arguments
    ///
    /// * `content` - The full document content
    ///
    /// # Returns
    ///
    /// A `TimingAnalysisResult` containing analysis for each animation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pixelsrc::lsp_agent_client::LspAgentClient;
    ///
    /// let client = LspAgentClient::new();
    /// let content = r#"{"type": "animation", "name": "walk", "timing_function": "ease-in-out"}"#;
    ///
    /// let result = client.analyze_timing(content);
    /// assert_eq!(result.animations.len(), 1);
    /// assert_eq!(result.animations[0].curve_type, "smooth");
    /// ```
    pub fn analyze_timing(&self, content: &str) -> TimingAnalysisResult {
        let mut animations = Vec::new();

        for line in content.lines() {
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Only process animations
            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) if t == "animation" => t,
                _ => continue,
            };
            let _ = obj_type;

            let anim_name =
                obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown").to_string();

            // Get timing function (default to "linear" if not specified)
            let timing_str =
                obj.get("timing_function").and_then(|t| t.as_str()).unwrap_or("linear").to_string();

            // Parse the timing function
            let (description, curve_type, ascii_curve) = match parse_timing_function(&timing_str) {
                Ok(interpolation) => {
                    let desc = Self::describe_interpolation(&interpolation);
                    let curve_type = Self::classify_curve_type(&interpolation);
                    let ascii = Self::render_ascii_curve(&interpolation, 20, 8);
                    (desc, curve_type, Some(ascii))
                }
                Err(_) => (
                    format!("Unknown timing function: {}", timing_str),
                    "unknown".to_string(),
                    None,
                ),
            };

            animations.push(TimingAnalysis {
                animation: anim_name,
                timing_function: timing_str,
                description,
                curve_type,
                ascii_curve,
            });
        }

        TimingAnalysisResult { animations }
    }

    /// Analyze timing and return JSON string
    ///
    /// Convenience method that returns the timing analysis result as a JSON string.
    pub fn analyze_timing_json(&self, content: &str) -> String {
        let result = self.analyze_timing(content);
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize result: {}"}}"#, e))
    }

    /// Build a VariableRegistry from document content
    fn build_variable_registry(content: &str) -> VariableRegistry {
        let mut registry = VariableRegistry::new();

        for line in content.lines() {
            let obj: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let obj = match obj.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Only process palettes
            if obj.get("type").and_then(|t| t.as_str()) != Some("palette") {
                continue;
            }

            // Get the colors object
            let colors = match obj.get("colors").and_then(|c| c.as_object()) {
                Some(c) => c,
                None => continue,
            };

            // Extract CSS variables (keys starting with '--')
            for (key, value) in colors {
                if key.starts_with("--") {
                    if let Some(value_str) = value.as_str() {
                        registry.define(key, value_str);
                    }
                }
            }
        }

        registry
    }

    /// Describe an interpolation/timing function in human-readable terms
    fn describe_interpolation(interpolation: &Interpolation) -> String {
        match interpolation {
            Interpolation::Linear => "Constant speed from start to end.".to_string(),
            Interpolation::EaseIn => "Starts slow, accelerates toward end.".to_string(),
            Interpolation::EaseOut => "Starts fast, decelerates toward end.".to_string(),
            Interpolation::EaseInOut => "Slow start and end, fast middle.".to_string(),
            Interpolation::Bounce => "Bouncy effect at the end, like a ball.".to_string(),
            Interpolation::Elastic => "Elastic overshoot effect, springs past target.".to_string(),
            Interpolation::Bezier { p1, p2 } => {
                format!(
                    "Custom cubic bezier curve ({:.2}, {:.2}, {:.2}, {:.2}).",
                    p1.0, p1.1, p2.0, p2.1
                )
            }
            Interpolation::Steps { count, position } => {
                let pos_desc = match position {
                    StepPosition::JumpStart => "starts immediately",
                    StepPosition::JumpEnd => "ends on final value",
                    StepPosition::JumpNone => "never sits on endpoints",
                    StepPosition::JumpBoth => "sits on both endpoints",
                };
                format!("Jumps in {} discrete step(s), {}.", count, pos_desc)
            }
        }
    }

    /// Classify a timing function into a curve type
    fn classify_curve_type(interpolation: &Interpolation) -> String {
        match interpolation {
            Interpolation::Linear => "linear".to_string(),
            Interpolation::EaseIn | Interpolation::EaseOut | Interpolation::EaseInOut => {
                "smooth".to_string()
            }
            Interpolation::Bounce => "bouncy".to_string(),
            Interpolation::Elastic => "elastic".to_string(),
            Interpolation::Bezier { .. } => "smooth".to_string(),
            Interpolation::Steps { .. } => "stepped".to_string(),
        }
    }

    /// Render an ASCII visualization of the easing curve
    fn render_ascii_curve(interpolation: &Interpolation, width: usize, height: usize) -> String {
        use crate::motion::ease;

        let mut grid = vec![vec![' '; width]; height];

        // Sample the curve
        for x in 0..width {
            let t = x as f64 / (width - 1) as f64;
            let y = ease(t, interpolation);
            let y_idx = ((1.0 - y) * (height - 1) as f64).round() as usize;
            let y_idx = y_idx.min(height - 1);
            grid[y_idx][x] = '█';
        }

        // Build the ASCII art with frame
        let mut result = String::new();
        result.push_str(&format!("┌{}┐\n", "─".repeat(width)));
        for row in &grid {
            result.push('│');
            result.extend(row.iter());
            result.push_str("│\n");
        }
        result.push_str(&format!("└{}┘", "─".repeat(width)));

        result
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
    /// Note: Grid format is deprecated - this always returns None now.
    #[allow(unused_variables)]
    fn parse_grid_context(_line: &str, _char_pos: u32) -> Option<GridPosition> {
        // Grid format is deprecated - use structured regions format
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
    #[ignore = "Grid format deprecated"]
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
    #[ignore = "Grid format deprecated"]
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
    #[ignore = "Grid format deprecated"]
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
    #[ignore = "Grid format deprecated"]
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

    // === CSS Color Resolution Tests ===

    #[test]
    fn test_resolve_colors_simple_hex() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000", "{blue}": "#0000FF"}}"##;

        let result = client.resolve_colors(content);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.colors.len(), 2);

        let red = result.colors.iter().find(|c| c.token == "{red}").unwrap();
        assert_eq!(red.resolved, "#FF0000");
        assert_eq!(red.palette, "test");
        assert!(!red.is_variable);
    }

    #[test]
    fn test_resolve_colors_css_variable() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "hero", "colors": {"--base": "#FF6347", "{skin}": "var(--base)"}}"##;

        let result = client.resolve_colors(content);
        assert_eq!(result.error_count, 0);

        let base = result.colors.iter().find(|c| c.token == "--base").unwrap();
        assert_eq!(base.resolved, "#FF6347");
        assert!(base.is_variable);

        let skin = result.colors.iter().find(|c| c.token == "{skin}").unwrap();
        assert_eq!(skin.original, "var(--base)");
        assert_eq!(skin.resolved, "#FF6347");
        assert!(!skin.is_variable);
    }

    #[test]
    fn test_resolve_colors_color_mix() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "hero", "colors": {"{shadow}": "color-mix(in srgb, red 50%, black)"}}"##;

        let result = client.resolve_colors(content);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.colors.len(), 1);

        let shadow = result.colors.iter().find(|c| c.token == "{shadow}").unwrap();
        assert!(shadow.original.contains("color-mix"));
        // Should be a darker red - not testing exact value due to color space differences
        assert!(shadow.resolved.starts_with('#'));
    }

    #[test]
    fn test_resolve_colors_chained_variables() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "hero", "colors": {"--primary": "#FF0000", "--accent": "var(--primary)", "{highlight}": "var(--accent)"}}"##;

        let result = client.resolve_colors(content);
        assert_eq!(result.error_count, 0);

        let highlight = result.colors.iter().find(|c| c.token == "{highlight}").unwrap();
        assert_eq!(highlight.resolved, "#FF0000");
    }

    #[test]
    fn test_resolve_colors_multiple_palettes() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "p1", "colors": {"--base": "#AA0000", "{a}": "var(--base)"}}
{"type": "palette", "name": "p2", "colors": {"{b}": "#00BB00"}}"##;

        let result = client.resolve_colors(content);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.colors.len(), 3);

        // Colors should reference their palette
        let a = result.colors.iter().find(|c| c.token == "{a}").unwrap();
        assert_eq!(a.palette, "p1");

        let b = result.colors.iter().find(|c| c.token == "{b}").unwrap();
        assert_eq!(b.palette, "p2");
    }

    #[test]
    fn test_resolve_colors_json() {
        let client = LspAgentClient::new();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##;
        let json = client.resolve_colors_json(content);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error_count"], 0);
        assert!(!parsed["colors"].as_array().unwrap().is_empty());
    }

    // === Timing Function Analysis Tests ===

    #[test]
    fn test_analyze_timing_linear() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "slide", "timing_function": "linear"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.animation, "slide");
        assert_eq!(anim.timing_function, "linear");
        assert_eq!(anim.curve_type, "linear");
        assert!(anim.description.contains("Constant speed"));
        assert!(anim.ascii_curve.is_some());
    }

    #[test]
    fn test_analyze_timing_ease_in_out() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "fade", "timing_function": "ease-in-out"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.curve_type, "smooth");
        assert!(anim.description.contains("Slow start and end"));
    }

    #[test]
    fn test_analyze_timing_cubic_bezier() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "custom", "timing_function": "cubic-bezier(0.25, 0.1, 0.25, 1.0)"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.curve_type, "smooth");
        assert!(anim.description.contains("cubic bezier"));
    }

    #[test]
    fn test_analyze_timing_steps() {
        let client = LspAgentClient::new();
        let content =
            r#"{"type": "animation", "name": "walk", "timing_function": "steps(4, jump-end)"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.curve_type, "stepped");
        assert!(anim.description.contains("4 discrete step"));
    }

    #[test]
    fn test_analyze_timing_bounce() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "drop", "timing_function": "bounce"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.curve_type, "bouncy");
        assert!(anim.description.contains("Bouncy effect"));
    }

    #[test]
    fn test_analyze_timing_default_linear() {
        let client = LspAgentClient::new();
        // No timing_function specified - should default to linear
        let content = r#"{"type": "animation", "name": "idle", "frames": ["f1", "f2"]}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 1);

        let anim = &result.animations[0];
        assert_eq!(anim.timing_function, "linear");
    }

    #[test]
    fn test_analyze_timing_multiple_animations() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "walk", "timing_function": "linear"}
{"type": "animation", "name": "run", "timing_function": "ease-in"}
{"type": "animation", "name": "jump", "timing_function": "bounce"}"#;

        let result = client.analyze_timing(content);
        assert_eq!(result.animations.len(), 3);

        let walk = result.animations.iter().find(|a| a.animation == "walk").unwrap();
        assert_eq!(walk.curve_type, "linear");

        let run = result.animations.iter().find(|a| a.animation == "run").unwrap();
        assert_eq!(run.curve_type, "smooth");

        let jump = result.animations.iter().find(|a| a.animation == "jump").unwrap();
        assert_eq!(jump.curve_type, "bouncy");
    }

    #[test]
    fn test_analyze_timing_json() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "test", "timing_function": "ease"}"#;
        let json = client.analyze_timing_json(content);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["animations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_ascii_curve_rendering() {
        let client = LspAgentClient::new();
        let content = r#"{"type": "animation", "name": "test", "timing_function": "ease-in-out"}"#;

        let result = client.analyze_timing(content);
        let anim = &result.animations[0];

        let curve = anim.ascii_curve.as_ref().unwrap();
        // Should have box-drawing characters
        assert!(curve.contains('┌'));
        assert!(curve.contains('┐'));
        assert!(curve.contains('└'));
        assert!(curve.contains('┘'));
        assert!(curve.contains('│'));
        assert!(curve.contains('█'));
    }
}
