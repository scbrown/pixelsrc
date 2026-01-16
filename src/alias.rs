//! Alias extraction, expansion, and simple grid utilities
//!
//! This module provides utilities for:
//! - Extracting common tokens into single-letter aliases
//! - Expanding aliases back to full token names
//! - Column-aligned formatting for grid display
//! - Parsing simple space-separated grid input
//! - Converting simple grids to sprite definitions

use crate::tokenizer::tokenize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Extract common tokens into single-letter aliases.
///
/// Returns (alias_map, transformed_grid) where:
/// - alias_map maps single chars to full token names (without braces)
/// - transformed_grid has tokens replaced with `{letter}` format
///
/// Assignment is frequency-based: most common token gets 'a', next gets 'b', etc.
/// The special token `{_}` always maps to '_' (underscore).
///
/// # Examples
///
/// ```
/// use pixelsrc::alias::extract_aliases;
///
/// let grid = vec![
///     "{_}{a}{a}{_}".to_string(),
///     "{_}{b}{b}{_}".to_string(),
/// ];
/// let (aliases, transformed) = extract_aliases(&grid);
/// // Most frequent is {_}, then {a} and {b}
/// assert_eq!(aliases.get(&'_'), Some(&"_".to_string()));
/// ```
pub fn extract_aliases(grid: &[String]) -> (HashMap<char, String>, Vec<String>) {
    // Count token frequencies
    let mut freq: HashMap<String, usize> = HashMap::new();
    for row in grid {
        let (tokens, _) = tokenize(row);
        for token in tokens {
            *freq.entry(token).or_insert(0) += 1;
        }
    }

    // Sort by frequency (descending), then alphabetically for stability
    let mut sorted: Vec<_> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Assign aliases
    let mut aliases: HashMap<char, String> = HashMap::new();
    let mut token_to_alias: HashMap<String, char> = HashMap::new();
    let mut next_letter = 'a';

    for (token, _) in sorted {
        // Extract name without braces
        let name = token
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or(&token);

        let alias = if name == "_" {
            '_'
        } else {
            let a = next_letter;
            next_letter = (next_letter as u8 + 1) as char;
            // Skip underscore in the sequence
            if next_letter == '_' {
                next_letter = '`'; // after underscore in ASCII, but we should skip to next letter
                next_letter = (next_letter as u8 + 1) as char;
            }
            a
        };

        aliases.insert(alias, name.to_string());
        token_to_alias.insert(token, alias);
    }

    // Transform grid
    let transformed: Vec<String> = grid
        .iter()
        .map(|row| {
            let (tokens, _) = tokenize(row);
            tokens
                .iter()
                .map(|t| {
                    let alias = token_to_alias.get(t).unwrap_or(&'?');
                    format!("{{{}}}", alias)
                })
                .collect()
        })
        .collect();

    (aliases, transformed)
}

/// Expand aliases back to full token names.
///
/// Given a grid with single-letter aliases like `{a}{b}` and an alias map,
/// returns a grid with full token names like `{skin}{hair}`.
///
/// # Examples
///
/// ```
/// use pixelsrc::alias::expand_aliases;
/// use std::collections::HashMap;
///
/// let aliases: HashMap<char, String> = [
///     ('a', "skin".to_string()),
///     ('b', "hair".to_string()),
/// ].into_iter().collect();
///
/// let grid = vec!["{a}{b}".to_string()];
/// let expanded = expand_aliases(&grid, &aliases);
/// assert_eq!(expanded[0], "{skin}{hair}");
/// ```
pub fn expand_aliases(grid: &[String], aliases: &HashMap<char, String>) -> Vec<String> {
    grid.iter()
        .map(|row| {
            let (tokens, _) = tokenize(row);
            tokens
                .iter()
                .map(|t| {
                    // Extract the alias character
                    let alias_char = t
                        .strip_prefix('{')
                        .and_then(|s| s.strip_suffix('}'))
                        .and_then(|s| s.chars().next());

                    if let Some(c) = alias_char {
                        if let Some(name) = aliases.get(&c) {
                            return format!("{{{}}}", name);
                        }
                    }
                    t.clone()
                })
                .collect()
        })
        .collect()
}

/// Format grid with column-aligned spacing between cells.
///
/// Each row is a vector of token strings. The function finds the maximum
/// width for each column and adds padding to align all columns.
///
/// # Examples
///
/// ```
/// use pixelsrc::alias::format_columns;
///
/// let rows = vec![
///     vec!["{_}".to_string(), "{_}".to_string(), "{body_blue}".to_string()],
///     vec!["{_}".to_string(), "{skin_highlight}".to_string(), "{body_light}".to_string()],
/// ];
/// let formatted = format_columns(rows);
/// // Columns are aligned, with consistent spacing
/// ```
pub fn format_columns(rows: Vec<Vec<String>>) -> Vec<String> {
    if rows.is_empty() {
        return vec![];
    }

    // Find max width per column
    let mut col_widths: Vec<usize> = vec![];
    for row in &rows {
        for (i, token) in row.iter().enumerate() {
            if i >= col_widths.len() {
                col_widths.push(token.len());
            } else {
                col_widths[i] = col_widths[i].max(token.len());
            }
        }
    }

    // Join tokens with spacing to align columns
    rows.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, token)| {
                    let padding = col_widths.get(i).unwrap_or(&0).saturating_sub(token.len());
                    format!("{}{}", token, " ".repeat(padding + 2)) // +2 for gap
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Parse space-separated simple grid input.
///
/// Takes a string with newline-separated rows where each row has
/// space-separated single letters/tokens.
///
/// # Examples
///
/// ```
/// use pixelsrc::alias::parse_simple_grid;
///
/// let input = "_ _ b b\n_ b c b";
/// let grid = parse_simple_grid(input);
/// assert_eq!(grid.len(), 2);
/// assert_eq!(grid[0], vec!["_", "_", "b", "b"]);
/// ```
pub fn parse_simple_grid(input: &str) -> Vec<Vec<String>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .collect()
}

/// Convert simple grid to sprite definition JSON.
///
/// Takes a parsed simple grid (from `parse_simple_grid`) and produces
/// a complete sprite JSON object with auto-generated placeholder palette.
///
/// - `_` is automatically mapped to transparent (#00000000)
/// - Other tokens get placeholder black color (#000000)
/// - If `palette_ref` is Some, uses that as a palette reference instead of inline palette
///
/// # Examples
///
/// ```
/// use pixelsrc::alias::simple_grid_to_sprite;
///
/// let grid = vec![
///     vec!["_".to_string(), "b".to_string()],
///     vec!["b".to_string(), "_".to_string()],
/// ];
/// let sprite = simple_grid_to_sprite(grid, "test_sprite", None);
/// // Returns JSON with type, name, size, palette, and grid
/// ```
pub fn simple_grid_to_sprite(
    grid: Vec<Vec<String>>,
    name: &str,
    palette_ref: Option<&str>,
) -> Value {
    if grid.is_empty() {
        return json!({
            "type": "sprite",
            "name": name,
            "size": [0, 0],
            "palette": {},
            "grid": []
        });
    }

    // Calculate dimensions
    let height = grid.len();
    let width = grid.iter().map(|row| row.len()).max().unwrap_or(0);

    // Collect unique tokens and build palette
    let mut tokens: Vec<String> = vec![];
    for row in &grid {
        for token in row {
            if !tokens.contains(token) {
                tokens.push(token.clone());
            }
        }
    }

    // Build grid strings with {token} format
    let grid_strings: Vec<String> = grid
        .iter()
        .map(|row| row.iter().map(|t| format!("{{{}}}", t)).collect())
        .collect();

    // Build result based on palette_ref
    if let Some(palette_name) = palette_ref {
        json!({
            "type": "sprite",
            "name": name,
            "size": [width, height],
            "palette": palette_name,
            "grid": grid_strings
        })
    } else {
        // Build inline palette with placeholders
        let mut palette: HashMap<String, String> = HashMap::new();
        for token in tokens {
            let color = if token == "_" {
                "#00000000".to_string() // transparent
            } else {
                "#000000".to_string() // placeholder black
            };
            palette.insert(format!("{{{}}}", token), color);
        }

        json!({
            "type": "sprite",
            "name": name,
            "size": [width, height],
            "palette": palette,
            "grid": grid_strings
        })
    }
}

/// Parse a grid row string into individual tokens.
///
/// Convenience wrapper around tokenize that returns just the tokens.
pub fn parse_grid_row(row: &str) -> Vec<String> {
    let (tokens, _) = tokenize(row);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_aliases_basic() {
        let grid = vec![
            "{_}{a}{a}{_}".to_string(),
            "{_}{b}{b}{_}".to_string(),
        ];
        let (aliases, transformed) = extract_aliases(&grid);

        // {_} should always map to '_'
        assert_eq!(aliases.get(&'_'), Some(&"_".to_string()));

        // Most frequent non-underscore should get 'a'
        // {a} appears twice, {b} appears twice - alphabetical order determines tie
        assert!(aliases.contains_key(&'a'));
        assert!(aliases.contains_key(&'b'));

        // Transformed grid should have alias tokens
        assert_eq!(transformed.len(), 2);
        for row in &transformed {
            assert!(row.contains("{_}") || row.contains("{a}") || row.contains("{b}"));
        }
    }

    #[test]
    fn test_extract_aliases_underscore_preserved() {
        let grid = vec!["{_}{x}".to_string()];
        let (aliases, _) = extract_aliases(&grid);

        // Underscore is always '_', never gets a letter alias
        assert_eq!(aliases.get(&'_'), Some(&"_".to_string()));
    }

    #[test]
    fn test_expand_aliases() {
        let aliases: HashMap<char, String> = [
            ('a', "skin".to_string()),
            ('b', "hair".to_string()),
            ('_', "_".to_string()),
        ]
        .into_iter()
        .collect();

        let grid = vec!["{_}{a}{b}".to_string()];
        let expanded = expand_aliases(&grid, &aliases);

        assert_eq!(expanded[0], "{_}{skin}{hair}");
    }

    #[test]
    fn test_format_columns_basic() {
        let rows = vec![
            vec!["{_}".to_string(), "{_}".to_string(), "{body}".to_string()],
            vec![
                "{_}".to_string(),
                "{skin_highlight}".to_string(),
                "{body}".to_string(),
            ],
        ];
        let formatted = format_columns(rows);

        assert_eq!(formatted.len(), 2);
        // Second column should have padding to match {skin_highlight} width
        assert!(formatted[0].contains("  ")); // gap between columns
        // All rows should align
        let col1_start: Vec<_> = formatted.iter().map(|r| r.find("{_}")).collect();
        assert!(col1_start.iter().all(|&pos| pos == col1_start[0]));
    }

    #[test]
    fn test_format_columns_empty() {
        let rows: Vec<Vec<String>> = vec![];
        let formatted = format_columns(rows);
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_parse_simple_grid() {
        let input = "_ _ b b\n_ b c b";
        let grid = parse_simple_grid(input);

        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0], vec!["_", "_", "b", "b"]);
        assert_eq!(grid[1], vec!["_", "b", "c", "b"]);
    }

    #[test]
    fn test_parse_simple_grid_empty_lines() {
        let input = "a b\n\nc d\n";
        let grid = parse_simple_grid(input);

        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0], vec!["a", "b"]);
        assert_eq!(grid[1], vec!["c", "d"]);
    }

    #[test]
    fn test_simple_grid_to_sprite_inline_palette() {
        let grid = vec![
            vec!["_".to_string(), "b".to_string()],
            vec!["b".to_string(), "_".to_string()],
        ];
        let sprite = simple_grid_to_sprite(grid, "test_sprite", None);

        assert_eq!(sprite["type"], "sprite");
        assert_eq!(sprite["name"], "test_sprite");
        assert_eq!(sprite["size"], json!([2, 2]));

        // Palette should have transparent for {_}
        let palette = sprite["palette"].as_object().unwrap();
        assert_eq!(palette.get("{_}").unwrap(), "#00000000");
        assert_eq!(palette.get("{b}").unwrap(), "#000000");

        // Grid should have token format
        let grid_arr = sprite["grid"].as_array().unwrap();
        assert_eq!(grid_arr[0], "{_}{b}");
        assert_eq!(grid_arr[1], "{b}{_}");
    }

    #[test]
    fn test_simple_grid_to_sprite_named_palette() {
        let grid = vec![vec!["a".to_string(), "b".to_string()]];
        let sprite = simple_grid_to_sprite(grid, "test", Some("@synthwave"));

        assert_eq!(sprite["palette"], "@synthwave");
    }

    #[test]
    fn test_simple_grid_to_sprite_empty() {
        let grid: Vec<Vec<String>> = vec![];
        let sprite = simple_grid_to_sprite(grid, "empty", None);

        assert_eq!(sprite["size"], json!([0, 0]));
        assert!(sprite["grid"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_round_trip_alias_expand() {
        let original = vec![
            "{transparent}{skin}{skin}{transparent}".to_string(),
            "{transparent}{hair}{hair}{transparent}".to_string(),
        ];

        let (aliases, transformed) = extract_aliases(&original);
        let expanded = expand_aliases(&transformed, &aliases);

        // Expanded should match original
        assert_eq!(original, expanded);
    }

    #[test]
    fn test_parse_grid_row() {
        let row = "{a}{b}{c}";
        let tokens = parse_grid_row(row);
        assert_eq!(tokens, vec!["{a}", "{b}", "{c}"]);
    }
}
