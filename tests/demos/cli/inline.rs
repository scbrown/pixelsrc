//! Inline CLI Demos
//!
//! Demo tests for the `pxl inline` command that displays sprite grids
//! with column-aligned spacing for improved readability.

use pixelsrc::alias::{format_columns, parse_grid_row};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::io::Cursor;

/// @demo cli/inline#basic
/// @title Basic Inline Grid Display
/// @description Parses sprite grid and displays tokens with column alignment.
#[test]
fn test_inline_basic() {
    let input = include_str!("../../../examples/demos/cli/format/inline_sprite.jsonl");

    // Parse to get sprite
    let reader = Cursor::new(input);
    let result = parse_stream(reader);

    let sprite = result
        .objects
        .iter()
        .find_map(|o| match o {
            TtpObject::Sprite(s) if s.name == "face" => Some(s),
            _ => None,
        })
        .expect("Sprite 'face' should exist");

    // Parse each grid row into tokens
    let rows: Vec<Vec<String>> = sprite.grid.iter().map(|row| parse_grid_row(row)).collect();

    // Verify grid dimensions
    assert_eq!(rows.len(), 5, "Grid should have 5 rows");
    assert!(rows.iter().all(|r| r.len() == 5), "Each row should have 5 tokens");
}

/// @demo cli/inline#column_alignment
/// @title Column-Aligned Output
/// @description Tokens are padded to align columns for visual clarity.
#[test]
fn test_inline_column_alignment() {
    // Create rows with varying token lengths
    let rows = vec![
        vec!["{_}".to_string(), "{_}".to_string(), "{body}".to_string()],
        vec!["{_}".to_string(), "{skin_highlight}".to_string(), "{body}".to_string()],
        vec!["{eye}".to_string(), "{skin_shadow}".to_string(), "{body_dark}".to_string()],
    ];

    let formatted = format_columns(rows);

    assert_eq!(formatted.len(), 3, "Should have 3 formatted rows");

    // Second column should be padded to match {skin_highlight} width
    // All rows should have consistent column positions
    let first_body_pos = formatted[0].find("{body}").expect("{{body}} should be in first row");
    let second_body_pos = formatted[1].find("{body}").expect("{{body}} should be in second row");
    let third_body_pos =
        formatted[2].find("{body_dark}").expect("{{body_dark}} should be in third row");

    // All third column entries should start at the same position
    assert_eq!(first_body_pos, second_body_pos, "Third column should be aligned across rows");
    assert_eq!(first_body_pos, third_body_pos, "Third column should be aligned with body_dark");
}

/// @demo cli/inline#token_preservation
/// @title Token Names Preserved
/// @description Full token names are preserved in inline display (not aliased).
#[test]
fn test_inline_token_preservation() {
    let input = include_str!("../../../examples/demos/cli/format/inline_sprite.jsonl");

    // Parse to get sprite
    let reader = Cursor::new(input);
    let result = parse_stream(reader);

    let sprite = result
        .objects
        .iter()
        .find_map(|o| match o {
            TtpObject::Sprite(s) if s.name == "face" => Some(s),
            _ => None,
        })
        .expect("Sprite 'face' should exist");

    // Parse grid rows
    let rows: Vec<Vec<String>> = sprite.grid.iter().map(|row| parse_grid_row(row)).collect();

    // Format with columns
    let formatted = format_columns(rows);

    // Full token names should be preserved
    assert!(formatted.iter().any(|row| row.contains("{hair}")), "Should preserve {{hair}} token");
    assert!(formatted.iter().any(|row| row.contains("{skin}")), "Should preserve {{skin}} token");
    assert!(formatted.iter().any(|row| row.contains("{eye}")), "Should preserve {{eye}} token");
    assert!(formatted.iter().any(|row| row.contains("{shirt}")), "Should preserve {{shirt}} token");
}

/// @demo cli/inline#empty_grid
/// @title Empty Grid Handling
/// @description Empty grids are handled gracefully.
#[test]
fn test_inline_empty_grid() {
    let rows: Vec<Vec<String>> = vec![];
    let formatted = format_columns(rows);

    assert!(formatted.is_empty(), "Empty input should produce empty output");
}

/// @demo cli/inline#single_row
/// @title Single Row Grid
/// @description Single-row grids format correctly without extra padding.
#[test]
fn test_inline_single_row() {
    let rows = vec![vec!["{a}".to_string(), "{bb}".to_string(), "{ccc}".to_string()]];

    let formatted = format_columns(rows);

    assert_eq!(formatted.len(), 1, "Should have 1 formatted row");
    assert!(formatted[0].contains("{a}"), "Should contain first token");
    assert!(formatted[0].contains("{ccc}"), "Should contain last token");
}

/// @demo cli/inline#wide_tokens
/// @title Wide Token Handling
/// @description Long token names don't break column alignment.
#[test]
fn test_inline_wide_tokens() {
    let rows = vec![
        vec!["{short}".to_string(), "{x}".to_string()],
        vec!["{very_long_token_name}".to_string(), "{y}".to_string()],
    ];

    let formatted = format_columns(rows);

    // Second column should be aligned despite different first column widths
    let first_x_pos = formatted[0].find("{x}").expect("{{x}} should be in first row");
    let second_y_pos = formatted[1].find("{y}").expect("{{y}} should be in second row");

    assert_eq!(first_x_pos, second_y_pos, "Second column should be aligned");
}
