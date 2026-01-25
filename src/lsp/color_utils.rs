//! Color utility functions for LSP.

use crate::color::parse_color;
use crate::variables::VariableRegistry;
use serde_json::Value;

use super::types::ColorMatch;

/// Extract colors from a palette line.
///
/// Returns a vector of (ColorMatch, line_number) tuples.
pub fn extract_colors_from_line(
    line: &str,
    line_num: u32,
    var_registry: &VariableRegistry,
) -> Vec<(ColorMatch, u32)> {
    let mut matches = Vec::new();

    // First, try to parse as JSON to get structured color data
    let parsed: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return matches,
    };

    // Check if this is a palette object
    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return matches,
    };

    // Look for "colors" field
    let colors = match obj.get("colors").and_then(|v| v.as_object()) {
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
pub fn rgba_to_hex(r: f32, g: f32, b: f32, a: f32) -> String {
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
pub fn rgba_to_rgb_functional(r: f32, g: f32, b: f32, a: f32) -> String {
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
pub fn rgba_to_hsl(r: f32, g: f32, b: f32, a: f32) -> String {
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
