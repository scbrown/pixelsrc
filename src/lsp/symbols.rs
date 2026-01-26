//! Document symbol extraction and analysis.

use crate::variables::VariableRegistry;
use serde_json::Value;
use tower_lsp::lsp_types::SymbolKind;

/// Extract document symbols from content
///
/// Returns a list of (name, type, line_number) tuples for all defined objects.
pub fn extract_symbols(content: &str) -> Vec<(String, String, usize)> {
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
pub fn type_to_symbol_kind(obj_type: &str) -> SymbolKind {
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
/// Returns a list of (token, color, optional_role) tuples from all palette definitions.
pub fn collect_defined_tokens(content: &str) -> Vec<(String, String, Option<String>)> {
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

        // Get the roles object (if present)
        let roles = obj.get("roles").and_then(|r| r.as_object());

        // Extract tokens (keys starting with '{' and ending with '}')
        for (key, value) in colors {
            if key.starts_with('{') && key.ends_with('}') {
                let color_str = match value.as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                // Look up role for this token
                let role =
                    roles.and_then(|r| r.get(key)).and_then(|v| v.as_str()).map(|s| s.to_string());
                tokens.push((key.clone(), color_str, role));
            }
        }
    }

    tokens
}

/// Collect all CSS variables from palettes in the document
///
/// Returns a list of (variable_name, raw_value, line_number, palette_name) tuples.
pub fn collect_css_variables(content: &str) -> Vec<(String, String, usize, String)> {
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
pub fn build_variable_registry(content: &str) -> VariableRegistry {
    let mut registry = VariableRegistry::new();

    for (name, value, _, _) in collect_css_variables(content) {
        registry.define(&name, &value);
    }

    registry
}

/// Find the position of a CSS variable definition in content
///
/// Returns (line_number, start_char, end_char) if found.
pub fn find_variable_definition(content: &str, var_name: &str) -> Option<(usize, u32, u32)> {
    let normalized_name =
        if var_name.starts_with("--") { var_name.to_string() } else { format!("--{}", var_name) };

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

/// Extract CSS variable name at position, including var() references
///
/// Returns the variable name (with -- prefix) if cursor is on a var() reference.
pub fn extract_variable_at_position(line: &str, char_pos: u32) -> Option<String> {
    let pos = char_pos as usize;

    // Look for var(--name) pattern around cursor
    // First, find if we're inside a var() reference
    let before = &line[..pos.min(line.len())];
    let after = &line[pos.min(line.len())..];

    // Find the most recent "var(" before cursor
    let var_start = before.rfind("var(")?;

    // Check if we're still inside this var() (no closing paren between var_start and cursor)
    let between = &line[var_start..pos.min(line.len())];
    if between.contains(')') {
        return None;
    }

    // Find the closing paren after cursor
    let close_offset = after.find(')')?;
    let close_pos = pos + close_offset;

    // Extract the content between var( and )
    let var_content = &line[var_start + 4..close_pos];

    // Strip leading/trailing whitespace and quotes
    let var_content = var_content.trim().trim_matches('"').trim_matches('\'');

    // Extract just the variable name (handle fallback: var(--name, fallback))
    let var_name = if let Some(comma_pos) = var_content.find(',') {
        var_content[..comma_pos].trim()
    } else {
        var_content
    };

    if var_name.is_empty() {
        None
    } else {
        Some(var_name.to_string())
    }
}

/// Check if the cursor is in a position where CSS variable completion should be offered
pub fn is_css_variable_completion_context(line: &str, char_pos: u32) -> bool {
    let pos = char_pos as usize;

    if pos > line.len() {
        return false;
    }

    // Look backwards from cursor for var( that isn't closed yet
    let before = &line[..pos.min(line.len())];

    // Find the most recent "var(" before cursor
    if let Some(var_start) = before.rfind("var(") {
        // Check if there's a closing paren between var( and cursor
        let between = &line[var_start..pos.min(line.len())];
        if !between.contains(')') {
            return true;
        }
    }

    // Also check if we just typed "var(" (cursor right after opening paren)
    if pos >= 4 {
        let last_four = &line[pos - 4..pos];
        if last_four == "var(" {
            return true;
        }
    }

    false
}
