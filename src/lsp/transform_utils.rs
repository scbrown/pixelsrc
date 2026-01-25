//! Transform parsing utilities for LSP hover.

use crate::transforms::parse_transform_str;
use serde_json::Value;

use super::types::TransformInfo;

/// Parse transform context from a JSON line at a specific character position
///
/// Returns TransformInfo if the cursor is positioned within a transform string.
pub fn parse_transform_context(line: &str, char_pos: u32) -> Option<TransformInfo> {
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
