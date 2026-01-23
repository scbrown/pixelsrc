//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.

use crate::color::parse_color;
use crate::motion::{ease, parse_timing_function, Interpolation, StepPosition};
use crate::tokenizer::tokenize;
use crate::transforms::{explain_transform, parse_transform_str, Transform};
use crate::validate::{Severity, ValidationIssue, Validator};
use crate::variables::VariableRegistry;
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

/// Information about a timing function at cursor position
#[derive(Debug, Clone)]
struct TimingFunctionInfo {
    /// Raw timing function string from JSON
    function_str: String,
    /// Parsed interpolation
    interpolation: Interpolation,
}

/// Information about a color found in the document
#[derive(Debug, Clone)]
struct ColorMatch {
    /// The original color string as it appears in the document
    #[allow(dead_code)]
    original: String,
    /// The resolved RGBA values (0.0-1.0 range)
    rgba: (f32, f32, f32, f32),
    /// Start position in the line
    start: u32,
    /// End position in the line
    end: u32,
}

/// Information about a transform at a cursor position
#[derive(Debug, Clone)]
struct TransformInfo {
    /// The parsed transform
    transform: Transform,
    /// The raw transform string
    raw: String,
    /// Object type (sprite, animation, composition, etc.)
    object_type: String,
    /// Object name
    object_name: String,
    /// Index in the transform array (0-indexed)
    index: usize,
    /// Total number of transforms in the array
    total: usize,
}

/// The Pixelsrc Language Server
pub struct PixelsrcLanguageServer {
    client: Client,
    /// Document state tracking for open files
    documents: RwLock<HashMap<Url, String>>,
}

impl PixelsrcLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client, documents: RwLock::new(HashMap::new()) }
    }

    /// Validate document content and publish diagnostics
    async fn validate_and_publish(&self, uri: &Url, content: &str) {
        let mut validator = Validator::new();
        for (line_num, line) in content.lines().enumerate() {
            validator.validate_line(line_num + 1, line);
        }

        let diagnostics: Vec<Diagnostic> =
            validator.issues().iter().map(Self::issue_to_diagnostic).collect();

        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
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
                start: Position { line: (issue.line - 1) as u32, character: 0 },
                end: Position { line: (issue.line - 1) as u32, character: u32::MAX },
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

    /// Parse transform context from a JSON line at a specific character position
    ///
    /// Returns TransformInfo if the cursor is positioned within a transform string.
    fn parse_transform_context(line: &str, char_pos: u32) -> Option<TransformInfo> {
        // Parse the JSON line
        let obj: Value = serde_json::from_str(line).ok()?;
        let obj = obj.as_object()?;

        // Get the type and name
        let obj_type = obj.get("type")?.as_str()?.to_string();
        let obj_name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();

        // Get the transform array
        let transform_array = obj.get("transform")?.as_array()?;
        if transform_array.is_empty() {
            return None;
        }

        // Find the "transform" key position in the raw JSON
        let transform_key_pos = line.find("\"transform\"")?;

        // Find the opening bracket of the transform array
        let after_key = &line[transform_key_pos..];
        let bracket_offset = after_key.find('[')?;
        let array_start = transform_key_pos + bracket_offset;

        // If cursor is before the transform array, no hover
        if (char_pos as usize) <= array_start {
            return None;
        }

        // Now we need to find which array element contains the cursor
        // Walk through the array portion of the line
        let array_portion = &line[array_start..];
        let char_in_array = (char_pos as usize) - array_start;

        // Parse through the array manually to find string positions
        let mut pos = 0;
        let chars: Vec<char> = array_portion.chars().collect();
        let total = transform_array.len();

        for (idx, transform_val) in transform_array.iter().enumerate() {
            // Get the transform string value
            let transform_str = transform_val.as_str()?;

            // Find the opening quote for this string
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

            // Check if cursor is within this string (including quotes for better UX)
            if char_in_array >= string_start.saturating_sub(1) && char_in_array <= string_end {
                // Parse the transform string
                if let Ok(transform) = parse_transform_str(transform_str) {
                    return Some(TransformInfo {
                        transform,
                        raw: transform_str.to_string(),
                        object_type: obj_type,
                        object_name: obj_name,
                        index: idx,
                        total,
                    });
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

    /// Collect all CSS variables from palettes in the document
    ///
    /// Returns a list of (variable_name, raw_value, line_number, palette_name) tuples.
    fn collect_css_variables(content: &str) -> Vec<(String, String, usize, String)> {
        let mut variables = Vec::new();

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

            // Check if it's a palette
            let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };

            if obj_type != "palette" {
                continue;
            }

            let palette_name =
                obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown").to_string();

            // Get the colors object
            let colors = match obj.get("colors").and_then(|c| c.as_object()) {
                Some(c) => c,
                None => continue,
            };

            // Extract CSS variables (keys starting with '--')
            for (key, value) in colors {
                if key.starts_with("--") {
                    let value_str = match value.as_str() {
                        Some(s) => s.to_string(),
                        None => continue,
                    };
                    variables.push((key.clone(), value_str, line_num, palette_name.clone()));
                }
            }
        }

        variables
    }

    /// Build a VariableRegistry from document content
    fn build_variable_registry(content: &str) -> VariableRegistry {
        let mut registry = VariableRegistry::new();

        for (name, value, _, _) in Self::collect_css_variables(content) {
            registry.define(&name, &value);
        }

        registry
    }

    /// Find the position of a CSS variable definition in content
    ///
    /// Returns (line_number, start_char, end_char) if found.
    fn find_variable_definition(content: &str, var_name: &str) -> Option<(usize, u32, u32)> {
        let normalized_name = if var_name.starts_with("--") {
            var_name.to_string()
        } else {
            format!("--{}", var_name)
        };

        // Search pattern: "var_name": or "var_name" : (with possible spaces)
        let search_key = format!("\"{}\"", normalized_name);

        for (line_num, line) in content.lines().enumerate() {
            if let Some(pos) = line.find(&search_key) {
                // Verify this is in a palette's colors object
                if let Ok(obj) = serde_json::from_str::<Value>(line) {
                    if let Some(obj) = obj.as_object() {
                        if obj.get("type").and_then(|t| t.as_str()) == Some("palette") {
                            if let Some(colors) = obj.get("colors").and_then(|c| c.as_object()) {
                                if colors.contains_key(&normalized_name) {
                                    let start = pos as u32;
                                    let end = start + search_key.len() as u32;
                                    return Some((line_num, start, end));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract CSS variable reference from text at a position
    ///
    /// Handles:
    /// - `var(--name)` - returns Some("--name")
    /// - `var(--name, fallback)` - returns Some("--name")
    /// - Direct definition `"--name": "value"` - returns Some("--name")
    fn extract_variable_at_position(line: &str, char_pos: u32) -> Option<String> {
        let pos = char_pos as usize;
        if pos >= line.len() {
            return None;
        }

        // Check if we're in a var() reference
        if let Some(var_start) = line[..=pos.min(line.len() - 1)].rfind("var(") {
            // Find the end of the var() call
            let after_var = &line[var_start..];
            let mut paren_depth = 0;
            let mut var_end = var_start;

            for (i, c) in after_var.char_indices() {
                match c {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            var_end = var_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // Check if cursor is within this var() call
            if pos >= var_start && pos <= var_end {
                // Extract the variable name from var(--name) or var(--name, fallback)
                let content = &line[var_start + 4..var_end];
                let var_name = if let Some(comma) = content.find(',') {
                    content[..comma].trim()
                } else {
                    content.trim()
                };

                if var_name.starts_with("--") || !var_name.contains('(') {
                    return Some(if var_name.starts_with("--") {
                        var_name.to_string()
                    } else {
                        format!("--{}", var_name)
                    });
                }
            }
        }

        // Check if we're on a variable definition (e.g., "--name": "value")
        // Look for "--" before the cursor position
        let before_pos = &line[..=pos.min(line.len() - 1)];
        if let Some(quote_start) = before_pos.rfind('"') {
            let after_quote = &line[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                let potential_var = &after_quote[..quote_end];
                if potential_var.starts_with("--") {
                    // Check if cursor is within or right after this string
                    let string_end = quote_start + 1 + quote_end;
                    if pos >= quote_start && pos <= string_end {
                        return Some(potential_var.to_string());
                    }
                }
            }
        }

        None
    }

    /// Check if cursor is in a context where CSS variable completions should be offered
    ///
    /// Returns true if cursor is after "var(" or "var(--"
    fn is_css_variable_completion_context(line: &str, char_pos: u32) -> bool {
        let pos = char_pos as usize;
        if pos < 4 {
            return false;
        }

        let before = &line[..pos];

        // Check for "var(" or "var(--" patterns
        if before.ends_with("var(") || before.ends_with("var(--") {
            return true;
        }

        // Check if we're inside a var() that's still being typed
        if let Some(var_start) = before.rfind("var(") {
            let in_var = &before[var_start + 4..];
            // We're in a var() context if there's no closing paren yet
            // and we haven't hit a comma (fallback)
            if !in_var.contains(')') && !in_var.contains(',') {
                return true;
            }
        }

        false
    }

    /// Render an ASCII visualization of an easing curve
    ///
    /// Creates a simple ASCII graph showing the easing function's shape.
    fn render_easing_curve(interpolation: &Interpolation, width: usize, height: usize) -> String {
        let mut grid = vec![vec![' '; width]; height];

        // Sample the easing function
        let samples: Vec<f64> = (0..=width)
            .map(|i| {
                let t = i as f64 / width as f64;
                ease(t, interpolation)
            })
            .collect();

        // Find min/max for scaling (handle overshoot)
        let min_val = samples.iter().cloned().fold(f64::INFINITY, f64::min).min(0.0);
        let max_val = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1.0);
        let range = max_val - min_val;

        // Plot the curve
        for (x, &value) in samples.iter().enumerate().take(width) {
            // Scale value to grid height
            let normalized = if range > 0.0 { (value - min_val) / range } else { 0.5 };
            let y = ((1.0 - normalized) * (height - 1) as f64).round() as usize;
            let y = y.min(height - 1);
            if x < width {
                grid[y][x] = '█';
            }
        }

        // Build the output with axis labels
        let mut output = String::new();

        // Top label (1.0 or max)
        let top_label = if max_val > 1.0 { format!("{:.1}", max_val) } else { "1.0".to_string() };
        output.push_str(&format!("{:>4}│", top_label));
        output.push_str(&grid[0].iter().collect::<String>());
        output.push('\n');

        // Middle rows
        for row in grid.iter().skip(1).take(height - 2) {
            output.push_str("    │");
            output.push_str(&row.iter().collect::<String>());
            output.push('\n');
        }

        // Bottom row with 0.0 label
        let bottom_label =
            if min_val < 0.0 { format!("{:.1}", min_val) } else { "0.0".to_string() };
        output.push_str(&format!("{:>4}│", bottom_label));
        output.push_str(&grid[height - 1].iter().collect::<String>());
        output.push('\n');

        // X-axis
        output.push_str("    └");
        output.push_str(&"─".repeat(width));
        output.push('\n');
        output.push_str("     0");
        output.push_str(&" ".repeat(width - 3));
        output.push_str("→ 1");

        output
    }

    /// Get a human-readable description of an interpolation type
    fn describe_interpolation(interpolation: &Interpolation) -> &'static str {
        match interpolation {
            Interpolation::Linear => "Constant speed (no easing)",
            Interpolation::EaseIn => "Slow start, fast end (acceleration)",
            Interpolation::EaseOut => "Fast start, slow end (deceleration)",
            Interpolation::EaseInOut => "Smooth S-curve (slow start and end)",
            Interpolation::Bounce => "Overshoot and settle back",
            Interpolation::Elastic => "Spring-like oscillation",
            Interpolation::Bezier { .. } => "Custom cubic bezier curve",
            Interpolation::Steps { .. } => "Discrete step function",
        }
    }

    /// Get the CSS-canonical form of an interpolation
    fn interpolation_to_css(interpolation: &Interpolation) -> String {
        match interpolation {
            Interpolation::Linear => "linear".to_string(),
            Interpolation::EaseIn => "ease-in".to_string(),
            Interpolation::EaseOut => "ease-out".to_string(),
            Interpolation::EaseInOut => "ease-in-out".to_string(),
            Interpolation::Bounce => "bounce".to_string(),
            Interpolation::Elastic => "elastic".to_string(),
            Interpolation::Bezier { p1, p2 } => {
                format!("cubic-bezier({}, {}, {}, {})", p1.0, p1.1, p2.0, p2.1)
            }
            Interpolation::Steps { count, position } => match position {
                StepPosition::JumpEnd => {
                    if *count == 1 {
                        "step-end".to_string()
                    } else {
                        format!("steps({})", count)
                    }
                }
                StepPosition::JumpStart => {
                    if *count == 1 {
                        "step-start".to_string()
                    } else {
                        format!("steps({}, jump-start)", count)
                    }
                }
                _ => format!("steps({}, {})", count, position),
            },
        }
    }

    /// Parse timing function context from a JSON line at cursor position
    ///
    /// Returns TimingFunctionInfo if the cursor is within a timing_function value.
    fn parse_timing_function_context(line: &str, char_pos: u32) -> Option<TimingFunctionInfo> {
        // Parse the JSON line
        let obj: Value = serde_json::from_str(line).ok()?;
        let obj = obj.as_object()?;

        // Check if this is an animation type
        let obj_type = obj.get("type")?.as_str()?;
        if obj_type != "animation" {
            return None;
        }

        // Look for timing_function field
        let timing_str = obj.get("timing_function")?.as_str()?;

        // Find the timing_function key position in the raw JSON
        let key_pos = line.find("\"timing_function\"")?;

        // Find the colon after the key
        let after_key = &line[key_pos..];
        let colon_offset = after_key.find(':')?;

        // Find the opening quote of the value
        let after_colon = &after_key[colon_offset..];
        let quote_offset = after_colon.find('"')?;
        let value_start = key_pos + colon_offset + quote_offset + 1;

        // Find the closing quote
        let value_end = value_start + timing_str.len();

        // Check if cursor is within the value
        let char_pos = char_pos as usize;
        if char_pos < value_start || char_pos > value_end {
            return None;
        }

        // Parse the timing function
        let interpolation = parse_timing_function(timing_str).ok()?;

        Some(TimingFunctionInfo { function_str: timing_str.to_string(), interpolation })
    }

    /// Extract all colors from a palette line
    ///
    /// Finds color values in palette definitions and resolves var() references.
    fn extract_colors_from_line(
        line: &str,
        line_num: u32,
        var_registry: &VariableRegistry,
    ) -> Vec<(ColorMatch, u32)> {
        let mut matches = Vec::new();

        // Try to parse as JSON
        let obj: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return matches,
        };

        let obj = match obj.as_object() {
            Some(o) => o,
            None => return matches,
        };

        // Check if it's a palette
        let obj_type = match obj.get("type").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return matches,
        };

        if obj_type != "palette" {
            return matches;
        }

        // Get the colors object
        let colors = match obj.get("colors").and_then(|c| c.as_object()) {
            Some(c) => c,
            None => return matches,
        };

        // Find the "colors" key position in the line
        let colors_key_pos = match line.find("\"colors\"") {
            Some(pos) => pos,
            None => return matches,
        };

        // Find each color value in the colors object
        for (key, value) in colors {
            let color_str = match value.as_str() {
                Some(s) => s,
                None => continue,
            };

            // Skip CSS variable definitions (they're not colors themselves)
            if key.starts_with("--") {
                continue;
            }

            // Find the position of this color value in the line
            // We search for the pattern "key": "value"
            let search_pattern = format!("\"{}\": \"{}\"", key, color_str);
            let alt_pattern = format!("\"{}\":\"{}\"", key, color_str);

            let value_start = if let Some(pos) = line[colors_key_pos..].find(&search_pattern) {
                let key_start = colors_key_pos + pos;
                // Find the start of the value string (after ": ")
                key_start + key.len() + 5 // ": " + opening quote
            } else if let Some(pos) = line[colors_key_pos..].find(&alt_pattern) {
                let key_start = colors_key_pos + pos;
                key_start + key.len() + 4 // ":" + opening quote
            } else {
                continue;
            };

            let value_end = value_start + color_str.len();

            // Resolve var() references if present
            let resolved_value = if color_str.contains("var(") {
                match var_registry.resolve(color_str) {
                    Ok(resolved) => resolved,
                    Err(_) => color_str.to_string(),
                }
            } else {
                color_str.to_string()
            };

            // Try to parse the resolved color
            if let Ok(rgba) = parse_color(&resolved_value) {
                matches.push((
                    ColorMatch {
                        original: color_str.to_string(),
                        rgba: (
                            rgba.0[0] as f32 / 255.0,
                            rgba.0[1] as f32 / 255.0,
                            rgba.0[2] as f32 / 255.0,
                            rgba.0[3] as f32 / 255.0,
                        ),
                        start: value_start as u32,
                        end: value_end as u32,
                    },
                    line_num,
                ));
            }
        }

        matches
    }

    /// Convert RGBA values (0.0-1.0) to hex string
    fn rgba_to_hex(r: f32, g: f32, b: f32, a: f32) -> String {
        let r = (r * 255.0).round() as u8;
        let g = (g * 255.0).round() as u8;
        let b = (b * 255.0).round() as u8;
        let a = (a * 255.0).round() as u8;

        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    /// Convert RGBA values (0.0-1.0) to rgb() or rgba() string
    fn rgba_to_rgb_functional(r: f32, g: f32, b: f32, a: f32) -> String {
        let r = (r * 255.0).round() as u8;
        let g = (g * 255.0).round() as u8;
        let b = (b * 255.0).round() as u8;

        if a >= 0.999 {
            format!("rgb({}, {}, {})", r, g, b)
        } else {
            format!("rgba({}, {}, {}, {:.2})", r, g, b, a)
        }
    }

    /// Convert RGBA values (0.0-1.0) to hsl() or hsla() string
    fn rgba_to_hsl(r: f32, g: f32, b: f32, a: f32) -> String {
        // Convert RGB to HSL
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < 0.0001 {
            // Achromatic
            if a >= 0.999 {
                format!("hsl(0, 0%, {}%)", (l * 100.0).round() as u32)
            } else {
                format!("hsla(0, 0%, {}%, {:.2})", (l * 100.0).round() as u32, a)
            }
        } else {
            let d = max - min;
            let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

            let h = if (max - r).abs() < 0.0001 {
                ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
            } else if (max - g).abs() < 0.0001 {
                ((b - r) / d + 2.0) / 6.0
            } else {
                ((r - g) / d + 4.0) / 6.0
            };

            let h_deg = (h * 360.0).round() as u32;
            let s_pct = (s * 100.0).round() as u32;
            let l_pct = (l * 100.0).round() as u32;

            if a >= 0.999 {
                format!("hsl({}, {}%, {}%)", h_deg, s_pct, l_pct)
            } else {
                format!("hsla({}, {}%, {}%, {:.2})", h_deg, s_pct, l_pct, a)
            }
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        "-".to_string(),
                        "(".to_string(),
                    ]),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Pixelsrc LSP initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store document content
        self.documents.write().unwrap().insert(uri.clone(), text.clone());

        // Validate and publish diagnostics
        self.validate_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Get the full text from the first change (we use FULL sync)
        if let Some(change) = params.content_changes.into_iter().next() {
            // Store updated content
            self.documents.write().unwrap().insert(uri.clone(), change.text.clone());

            // Validate and publish diagnostics
            self.validate_and_publish(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from tracking
        self.documents.write().unwrap().remove(&uri);

        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
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
            use std::cmp::Ordering;
            let alignment_status = match grid_info.row_width.cmp(&grid_info.expected_width) {
                Ordering::Equal => "✓ Aligned".to_string(),
                Ordering::Less => format!("⚠ Short by {} token(s)", grid_info.expected_width - grid_info.row_width),
                Ordering::Greater => format!("⚠ Long by {} token(s)", grid_info.row_width - grid_info.expected_width),
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

        // Check for CSS variable reference
        if let Some(var_name) = Self::extract_variable_at_position(line, pos.character) {
            let css_variables = Self::collect_css_variables(&content);
            let registry = Self::build_variable_registry(&content);

            // Find the variable's info
            let var_info = css_variables.iter().find(|(name, _, _, _)| name == &var_name);

            if let Some((_, raw_value, _, palette_name)) = var_info {
                // Try to resolve the variable
                let resolved = registry.resolve_var(&var_name);

                let hover_text = match resolved {
                    Ok(resolved_value) => {
                        if &resolved_value == raw_value {
                            format!(
                                "**CSS Variable**: `{}`\n\n\
                                 **Value**: `{}`\n\n\
                                 **Palette**: `{}`",
                                var_name, raw_value, palette_name
                            )
                        } else {
                            format!(
                                "**CSS Variable**: `{}`\n\n\
                                 **Raw Value**: `{}`\n\n\
                                 **Resolved**: `{}`\n\n\
                                 **Palette**: `{}`",
                                var_name, raw_value, resolved_value, palette_name
                            )
                        }
                    }
                    Err(e) => {
                        format!(
                            "**CSS Variable**: `{}`\n\n\
                             **Raw Value**: `{}`\n\n\
                             **Error**: {}\n\n\
                             **Palette**: `{}`",
                            var_name, raw_value, e, palette_name
                        )
                    }
                };

                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                }));
            } else {
                // Variable referenced but not defined
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**CSS Variable**: `{}`\n\n\
                             ⚠ **Undefined** - This variable is not defined in any palette",
                            var_name
                        ),
                    }),
                    range: None,
                }));
            }
        }

        // Try to parse timing function context at the cursor position
        if let Some(timing_info) = Self::parse_timing_function_context(line, pos.character) {
            // Render the ASCII easing curve (25 chars wide, 8 rows tall)
            let curve = Self::render_easing_curve(&timing_info.interpolation, 25, 8);
            let description = Self::describe_interpolation(&timing_info.interpolation);
            let css_form = Self::interpolation_to_css(&timing_info.interpolation);

            let hover_text = format!(
                "**Timing Function**: `{}`\n\n\
                 **Type**: {}\n\n\
                 **CSS**: `{}`\n\n\
                 ```\n{}\n```",
                timing_info.function_str, description, css_form, curve,
            );

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: None,
            }));
        }

        // Try to parse transform context at the cursor position
        if let Some(transform_info) = Self::parse_transform_context(line, pos.character) {
            let explanation = explain_transform(&transform_info.transform);

            // Build the hover text with context
            let position_text = if transform_info.total == 1 {
                String::new()
            } else {
                format!(
                    "\n\n**Position**: {} of {} transforms",
                    transform_info.index + 1,
                    transform_info.total
                )
            };

            let hover_text = format!(
                "**Transform**: `{}`\n\n\
                 **Effect**: {}\n\n\
                 **Applied to**: {} `{}`{}",
                transform_info.raw,
                explanation,
                transform_info.object_type,
                transform_info.object_name,
                position_text,
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Get the current line
        let current_line = content.lines().nth(pos.line as usize).unwrap_or("");

        // Check if we're in a CSS variable completion context (inside var())
        if Self::is_css_variable_completion_context(current_line, pos.character) {
            let css_variables = Self::collect_css_variables(&content);
            let registry = Self::build_variable_registry(&content);

            let mut completions: Vec<CompletionItem> = Vec::new();

            for (var_name, raw_value, _, palette_name) in css_variables {
                // Try to resolve the variable for the detail
                let resolved = registry.resolve_var(&var_name);
                let detail = match resolved {
                    Ok(resolved_value) => {
                        if resolved_value == raw_value {
                            format!("{} ({})", raw_value, palette_name)
                        } else {
                            format!("{} → {} ({})", raw_value, resolved_value, palette_name)
                        }
                    }
                    Err(_) => format!("{} ({})", raw_value, palette_name),
                };

                completions.push(CompletionItem {
                    label: var_name.clone(),
                    detail: Some(detail),
                    kind: Some(CompletionItemKind::VARIABLE),
                    insert_text: Some(var_name),
                    ..Default::default()
                });
            }

            return Ok(Some(CompletionResponse::Array(completions)));
        }

        // Standard token completions (for grid context)
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
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                    },
                    container_name: None,
                }
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = &params.text_document.uri;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(Vec::new()),
        };
        drop(documents);

        // Build variable registry from all palettes for var() resolution
        let var_registry = Self::build_variable_registry(&content);

        // Extract all colors from palette definitions
        let mut colors = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_colors = Self::extract_colors_from_line(line, line_num as u32, &var_registry);
            for (color_match, line_idx) in line_colors {
                colors.push(ColorInformation {
                    range: Range {
                        start: Position { line: line_idx, character: color_match.start },
                        end: Position { line: line_idx, character: color_match.end },
                    },
                    color: Color {
                        red: color_match.rgba.0,
                        green: color_match.rgba.1,
                        blue: color_match.rgba.2,
                        alpha: color_match.rgba.3,
                    },
                });
            }
        }

        Ok(colors)
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let color = params.color;
        let r = color.red;
        let g = color.green;
        let b = color.blue;
        let a = color.alpha;

        // Provide multiple format options when user picks a color
        let mut presentations = Vec::new();

        // Hex format (most common for pixel art)
        let hex = Self::rgba_to_hex(r, g, b, a);
        presentations.push(ColorPresentation {
            label: hex.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: hex }),
            additional_text_edits: None,
        });

        // RGB functional format
        let rgb = Self::rgba_to_rgb_functional(r, g, b, a);
        presentations.push(ColorPresentation {
            label: rgb.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: rgb }),
            additional_text_edits: None,
        });

        // HSL format
        let hsl = Self::rgba_to_hsl(r, g, b, a);
        presentations.push(ColorPresentation {
            label: hsl.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: hsl }),
            additional_text_edits: None,
        });

        Ok(presentations)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Get the line at the cursor position
        let line = match content.lines().nth(pos.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };

        // Check if cursor is on a CSS variable reference
        if let Some(var_name) = Self::extract_variable_at_position(line, pos.character) {
            // Find where this variable is defined
            if let Some((def_line, start_char, end_char)) =
                Self::find_variable_definition(&content, &var_name)
            {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: def_line as u32, character: start_char },
                        end: Position { line: def_line as u32, character: end_char },
                    },
                })));
            }
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
        assert_eq!(diagnostic.code, Some(NumberOrString::String("json_syntax".to_string())));
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
        let issue =
            ValidationIssue::warning(3, IssueType::UndefinedToken, "Undefined token {skni}")
                .with_suggestion("did you mean {skin}?");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 2); // 0-indexed
        assert_eq!(diagnostic.message, "Undefined token {skni} (did you mean {skin}?)");
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
        assert_eq!(PixelsrcLanguageServer::type_to_symbol_kind("palette"), SymbolKind::CONSTANT);
    }

    #[test]
    fn test_type_to_symbol_kind_sprite() {
        assert_eq!(PixelsrcLanguageServer::type_to_symbol_kind("sprite"), SymbolKind::CLASS);
    }

    #[test]
    fn test_type_to_symbol_kind_animation() {
        assert_eq!(PixelsrcLanguageServer::type_to_symbol_kind("animation"), SymbolKind::FUNCTION);
    }

    #[test]
    fn test_type_to_symbol_kind_composition() {
        assert_eq!(PixelsrcLanguageServer::type_to_symbol_kind("composition"), SymbolKind::MODULE);
    }

    #[test]
    fn test_type_to_symbol_kind_unknown() {
        assert_eq!(PixelsrcLanguageServer::type_to_symbol_kind("unknown"), SymbolKind::OBJECT);
    }

    // === CSS Variable Tests ===

    #[test]
    fn test_collect_css_variables_single_palette() {
        let content = r##"{"type": "palette", "name": "hero", "colors": {"--primary": "#FF0000", "{body}": "var(--primary)"}}"##;
        let variables = PixelsrcLanguageServer::collect_css_variables(content);

        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].0, "--primary");
        assert_eq!(variables[0].1, "#FF0000");
        assert_eq!(variables[0].2, 0); // Line 0
        assert_eq!(variables[0].3, "hero"); // Palette name
    }

    #[test]
    fn test_collect_css_variables_multiple_palettes() {
        let content = r##"{"type": "palette", "name": "p1", "colors": {"--color1": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"--color2": "#00FF00", "--color3": "var(--color1)"}}"##;
        let variables = PixelsrcLanguageServer::collect_css_variables(content);

        assert_eq!(variables.len(), 3);

        // Check that we have all variables
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color1"));
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color2"));
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color3"));
    }

    #[test]
    fn test_collect_css_variables_no_variables() {
        let content = r##"{"type": "palette", "name": "simple", "colors": {"{red}": "#FF0000"}}"##;
        let variables = PixelsrcLanguageServer::collect_css_variables(content);
        assert!(variables.is_empty());
    }

    #[test]
    fn test_build_variable_registry() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000", "--secondary": "var(--primary)"}}"##;
        let registry = PixelsrcLanguageServer::build_variable_registry(content);

        assert!(registry.contains("--primary"));
        assert!(registry.contains("--secondary"));
        assert_eq!(registry.resolve_var("--primary").unwrap(), "#FF0000");
        assert_eq!(registry.resolve_var("--secondary").unwrap(), "#FF0000");
    }

    #[test]
    fn test_find_variable_definition_exists() {
        let content =
            r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        let result = PixelsrcLanguageServer::find_variable_definition(content, "--primary");

        assert!(result.is_some());
        let (line, start, end) = result.unwrap();
        assert_eq!(line, 0);
        assert!(start > 0);
        assert!(end > start);
    }

    #[test]
    fn test_find_variable_definition_without_dashes() {
        let content =
            r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        // Should work even without the -- prefix
        let result = PixelsrcLanguageServer::find_variable_definition(content, "primary");

        assert!(result.is_some());
    }

    #[test]
    fn test_find_variable_definition_not_found() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000"}}"##;
        let result = PixelsrcLanguageServer::find_variable_definition(content, "--missing");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_variable_at_position_var_reference() {
        let line = r#""{body}": "var(--primary)""#;
        // Position inside var(--primary)
        let pos = line.find("--primary").unwrap() as u32;
        let result = PixelsrcLanguageServer::extract_variable_at_position(line, pos);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "--primary");
    }

    #[test]
    fn test_extract_variable_at_position_var_with_fallback() {
        let line = r#""{body}": "var(--primary, #FF0000)""#;
        let pos = line.find("--primary").unwrap() as u32;
        let result = PixelsrcLanguageServer::extract_variable_at_position(line, pos);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "--primary");
    }

    #[test]
    fn test_extract_variable_at_position_definition() {
        let line = r##""--primary": "#FF0000""##;
        // Position on the variable name in the definition
        let pos = line.find("--primary").unwrap() as u32;
        let result = PixelsrcLanguageServer::extract_variable_at_position(line, pos);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "--primary");
    }

    #[test]
    fn test_extract_variable_at_position_not_on_variable() {
        let line = r##""{body}": "#FF0000""##;
        let result = PixelsrcLanguageServer::extract_variable_at_position(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_css_variable_completion_context_after_var_open() {
        let line = r#""{body}": "var("#;
        assert!(PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    // === Timing Function Visualization Tests ===

    #[test]
    fn test_parse_timing_function_context_ease_in() {
        let line = r#"{"type": "animation", "name": "bounce", "timing_function": "ease-in", "frames": []}"#;
        // Find position within "ease-in" value
        let value_start = line.find("\"ease-in\"").unwrap() + 1;
        let info = PixelsrcLanguageServer::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "ease-in");
        assert!(matches!(info.interpolation, Interpolation::EaseIn));
    }

    #[test]
    fn test_parse_timing_function_context_cubic_bezier() {
        let line = r#"{"type": "animation", "name": "custom", "timing_function": "cubic-bezier(0.25, 0.1, 0.25, 1.0)", "frames": []}"#;
        let value_start = line.find("\"cubic-bezier").unwrap() + 1;
        let info = PixelsrcLanguageServer::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "cubic-bezier(0.25, 0.1, 0.25, 1.0)");
        assert!(matches!(info.interpolation, Interpolation::Bezier { .. }));
    }

    #[test]
    fn test_parse_timing_function_context_steps() {
        let line = r#"{"type": "animation", "name": "step", "timing_function": "steps(4, jump-end)", "frames": []}"#;
        let value_start = line.find("\"steps").unwrap() + 1;
        let info = PixelsrcLanguageServer::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "steps(4, jump-end)");
        assert!(matches!(info.interpolation, Interpolation::Steps { count: 4, .. }));
    }

    #[test]
    fn test_is_css_variable_completion_context_after_var_dashes() {
        let line = r#""{body}": "var(--"#;
        assert!(PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    #[test]
    fn test_is_css_variable_completion_context_typing_name() {
        let line = r#""{body}": "var(--pri"#;
        assert!(PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    #[test]
    fn test_is_css_variable_completion_context_not_in_var() {
        let line = r##""{body}": "#FF0000""##;
        assert!(!PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    #[test]
    fn test_is_css_variable_completion_context_after_close() {
        let line = r#""{body}": "var(--primary)"#;
        // After the closing paren, should not be in context
        assert!(!PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    #[test]
    fn test_is_css_variable_completion_context_in_fallback() {
        let line = r#""{body}": "var(--primary, "#;
        // After the comma (in fallback), should not be in var completion context
        assert!(!PixelsrcLanguageServer::is_css_variable_completion_context(
            line,
            line.len() as u32
        ));
    }

    #[test]
    fn test_parse_timing_function_context_not_animation() {
        let line = r#"{"type": "sprite", "name": "test", "timing_function": "ease"}"#;
        let info = PixelsrcLanguageServer::parse_timing_function_context(line, 50);
        assert!(info.is_none());
    }

    // === Transform Context Tests (LSP-12) ===

    #[test]
    fn test_parse_transform_context_single_transform() {
        let line = r#"{"type": "sprite", "name": "flipped", "source": "original", "transform": ["mirror-h"]}"#;
        // Find position within the "mirror-h" string
        let transform_start = line.find("[\"mirror-h\"]").unwrap() + 2;
        let info = PixelsrcLanguageServer::parse_transform_context(line, transform_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "mirror-h");
        assert_eq!(info.object_type, "sprite");
        assert_eq!(info.object_name, "flipped");
        assert_eq!(info.index, 0);
        assert_eq!(info.total, 1);
    }

    #[test]
    fn test_parse_transform_context_multiple_transforms_first() {
        let line = r#"{"type": "animation", "name": "walk_left", "source": "walk", "transform": ["mirror-h", "rotate:90"]}"#;
        // Position within first transform
        let first_transform_pos = line.find("[\"mirror-h\"").unwrap() + 2;
        let info =
            PixelsrcLanguageServer::parse_transform_context(line, first_transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "mirror-h");
        assert_eq!(info.index, 0);
        assert_eq!(info.total, 2);
    }

    #[test]
    fn test_parse_transform_context_multiple_transforms_second() {
        let line = r#"{"type": "animation", "name": "walk_left", "source": "walk", "transform": ["mirror-h", "rotate:90"]}"#;
        // Position within second transform
        let second_transform_pos = line.find("\"rotate:90\"").unwrap() + 1;
        let info =
            PixelsrcLanguageServer::parse_transform_context(line, second_transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "rotate:90");
        assert_eq!(info.index, 1);
        assert_eq!(info.total, 2);
    }

    #[test]
    fn test_parse_transform_context_not_a_transform() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}{b}"]}"#;
        let info = PixelsrcLanguageServer::parse_transform_context(line, 30);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_transform_context_before_array() {
        let line = r#"{"type": "sprite", "name": "test", "transform": ["mirror-h"]}"#;
        // Position before the transform array
        let info = PixelsrcLanguageServer::parse_transform_context(line, 10);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_timing_function_context_cursor_outside_value() {
        let line =
            r#"{"type": "animation", "name": "test", "timing_function": "ease", "frames": []}"#;
        // Position in "name" field, not timing_function
        let info = PixelsrcLanguageServer::parse_timing_function_context(line, 20);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_transform_context_composition() {
        let line = r#"{"type": "composition", "name": "scene", "layers": [], "transform": ["scale:2.0,2.0"]}"#;
        let transform_pos = line.find("\"scale:2.0,2.0\"").unwrap() + 1;
        let info = PixelsrcLanguageServer::parse_transform_context(line, transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "scale:2.0,2.0");
        assert_eq!(info.object_type, "composition");
        assert_eq!(info.object_name, "scene");
    }

    #[test]
    fn test_render_easing_curve_linear() {
        let curve = PixelsrcLanguageServer::render_easing_curve(&Interpolation::Linear, 10, 5);
        // Linear should produce a diagonal line
        assert!(curve.contains("█"));
        assert!(curve.contains("1.0"));
        assert!(curve.contains("0.0"));
        assert!(curve.contains("→ 1"));
    }

    #[test]
    fn test_render_easing_curve_ease_in() {
        let curve = PixelsrcLanguageServer::render_easing_curve(&Interpolation::EaseIn, 10, 5);
        // Ease-in starts slow, should have more blocks in lower rows initially
        assert!(curve.contains("█"));
    }

    #[test]
    fn test_render_easing_curve_steps() {
        let curve = PixelsrcLanguageServer::render_easing_curve(
            &Interpolation::Steps { count: 4, position: StepPosition::JumpEnd },
            20,
            6,
        );
        // Steps should produce a staircase pattern
        assert!(curve.contains("█"));
    }

    #[test]
    fn test_describe_interpolation_all_types() {
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::Linear),
            "Constant speed (no easing)"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::EaseIn),
            "Slow start, fast end (acceleration)"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::EaseOut),
            "Fast start, slow end (deceleration)"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::EaseInOut),
            "Smooth S-curve (slow start and end)"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::Bounce),
            "Overshoot and settle back"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::Elastic),
            "Spring-like oscillation"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::Bezier {
                p1: (0.0, 0.0),
                p2: (1.0, 1.0)
            }),
            "Custom cubic bezier curve"
        );
        assert_eq!(
            PixelsrcLanguageServer::describe_interpolation(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpEnd
            }),
            "Discrete step function"
        );
    }

    #[test]
    fn test_interpolation_to_css_named() {
        assert_eq!(PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Linear), "linear");
        assert_eq!(PixelsrcLanguageServer::interpolation_to_css(&Interpolation::EaseIn), "ease-in");
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::EaseOut),
            "ease-out"
        );
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::EaseInOut),
            "ease-in-out"
        );
    }

    #[test]
    fn test_interpolation_to_css_bezier() {
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Bezier {
                p1: (0.25, 0.1),
                p2: (0.25, 1.0)
            }),
            "cubic-bezier(0.25, 0.1, 0.25, 1)"
        );
    }

    #[test]
    fn test_interpolation_to_css_steps() {
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Steps {
                count: 1,
                position: StepPosition::JumpEnd
            }),
            "step-end"
        );
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Steps {
                count: 1,
                position: StepPosition::JumpStart
            }),
            "step-start"
        );
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpEnd
            }),
            "steps(4)"
        );
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpStart
            }),
            "steps(4, jump-start)"
        );
        assert_eq!(
            PixelsrcLanguageServer::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpBoth
            }),
            "steps(4, jump-both)"
        );
    }

    // === Color Provider Tests ===

    #[test]
    fn test_extract_colors_from_line_hex() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000", "{blue}": "#0000FF"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = PixelsrcLanguageServer::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 2);
        // Check that we found the colors
        let has_red = colors.iter().any(|(c, _)| {
            (c.rgba.0 - 1.0).abs() < 0.01 && c.rgba.1.abs() < 0.01 && c.rgba.2.abs() < 0.01
        });
        let has_blue = colors.iter().any(|(c, _)| {
            c.rgba.0.abs() < 0.01 && c.rgba.1.abs() < 0.01 && (c.rgba.2 - 1.0).abs() < 0.01
        });
        assert!(has_red, "Should find red color");
        assert!(has_blue, "Should find blue color");
    }

    #[test]
    fn test_extract_colors_from_line_css_functions() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{red}": "rgb(255, 0, 0)", "{green}": "hsl(120, 100%, 50%)"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = PixelsrcLanguageServer::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_extract_colors_from_line_with_vars() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000", "{red}": "var(--primary)"}}"##;
        let registry = PixelsrcLanguageServer::build_variable_registry(content);
        let colors = PixelsrcLanguageServer::extract_colors_from_line(content, 0, &registry);

        // Should have 1 color (the {red} token, --primary is skipped as a variable definition)
        assert_eq!(colors.len(), 1);
        // The resolved color should be red
        let (color, _) = &colors[0];
        assert!((color.rgba.0 - 1.0).abs() < 0.01, "Red component should be 1.0");
        assert!(color.rgba.1.abs() < 0.01, "Green component should be 0.0");
        assert!(color.rgba.2.abs() < 0.01, "Blue component should be 0.0");
    }

    #[test]
    fn test_extract_colors_from_line_color_mix() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{purple}": "color-mix(in srgb, red 50%, blue)"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = PixelsrcLanguageServer::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 1);
        // Color should be a purple-ish mix
        let (color, _) = &colors[0];
        assert!(color.rgba.0 > 0.4, "Should have red component");
        assert!(color.rgba.2 > 0.4, "Should have blue component");
    }

    #[test]
    fn test_extract_colors_from_line_not_palette() {
        let line = r#"{"type": "sprite", "name": "test", "grid": ["{a}"]}"#;
        let registry = crate::variables::VariableRegistry::new();
        let colors = PixelsrcLanguageServer::extract_colors_from_line(line, 0, &registry);

        assert!(colors.is_empty(), "Should not extract colors from sprites");
    }

    #[test]
    fn test_extract_colors_skips_css_vars() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = PixelsrcLanguageServer::extract_colors_from_line(line, 0, &registry);

        // CSS variable definitions should be skipped
        assert!(colors.is_empty(), "Should skip CSS variable definitions");
    }

    #[test]
    fn test_rgba_to_hex_no_alpha() {
        let hex = PixelsrcLanguageServer::rgba_to_hex(1.0, 0.0, 0.0, 1.0);
        assert_eq!(hex, "#FF0000");

        let hex = PixelsrcLanguageServer::rgba_to_hex(0.0, 1.0, 0.0, 1.0);
        assert_eq!(hex, "#00FF00");

        let hex = PixelsrcLanguageServer::rgba_to_hex(0.0, 0.0, 1.0, 1.0);
        assert_eq!(hex, "#0000FF");
    }

    #[test]
    fn test_rgba_to_hex_with_alpha() {
        let hex = PixelsrcLanguageServer::rgba_to_hex(1.0, 0.0, 0.0, 0.5);
        assert_eq!(hex, "#FF000080");

        let hex = PixelsrcLanguageServer::rgba_to_hex(1.0, 1.0, 1.0, 0.0);
        assert_eq!(hex, "#FFFFFF00");
    }

    #[test]
    fn test_rgba_to_rgb_functional() {
        let rgb = PixelsrcLanguageServer::rgba_to_rgb_functional(1.0, 0.0, 0.0, 1.0);
        assert_eq!(rgb, "rgb(255, 0, 0)");

        let rgba = PixelsrcLanguageServer::rgba_to_rgb_functional(1.0, 0.0, 0.0, 0.5);
        assert_eq!(rgba, "rgba(255, 0, 0, 0.50)");
    }

    #[test]
    fn test_rgba_to_hsl() {
        // Pure red
        let hsl = PixelsrcLanguageServer::rgba_to_hsl(1.0, 0.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(0, 100%, 50%)");

        // Pure green
        let hsl = PixelsrcLanguageServer::rgba_to_hsl(0.0, 1.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(120, 100%, 50%)");

        // Pure blue
        let hsl = PixelsrcLanguageServer::rgba_to_hsl(0.0, 0.0, 1.0, 1.0);
        assert_eq!(hsl, "hsl(240, 100%, 50%)");

        // White
        let hsl = PixelsrcLanguageServer::rgba_to_hsl(1.0, 1.0, 1.0, 1.0);
        assert_eq!(hsl, "hsl(0, 0%, 100%)");

        // Black
        let hsl = PixelsrcLanguageServer::rgba_to_hsl(0.0, 0.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(0, 0%, 0%)");
    }

    #[test]
    fn test_rgba_to_hsl_with_alpha() {
        let hsla = PixelsrcLanguageServer::rgba_to_hsl(1.0, 0.0, 0.0, 0.5);
        assert_eq!(hsla, "hsla(0, 100%, 50%, 0.50)");
    }
}
