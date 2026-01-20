//! Grid CLI Demos
//!
//! Demo tests for the `pxl grid` command that displays sprite grids
//! with coordinate headers for easy position reference.

use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use pixelsrc::terminal::render_coordinate_grid;
use std::io::Cursor;

/// @demo cli/grid#basic
/// @title Basic Coordinate Grid
/// @description Renders sprite grid with row and column coordinate headers.
#[test]
fn test_grid_basic() {
    let input = include_str!("../../../examples/demos/cli/format/grid_sprite.jsonl");

    // Parse to get sprite
    let reader = Cursor::new(input);
    let result = parse_stream(reader);

    let sprite = result
        .objects
        .iter()
        .find_map(|o| match o {
            TtpObject::Sprite(s) if s.name == "arrow" => Some(s),
            _ => None,
        })
        .expect("Sprite 'arrow' should exist");

    // Render coordinate grid
    let output = render_coordinate_grid(&sprite.grid, false);

    // Should have column headers (0-4 for a 5-wide sprite)
    assert!(output.contains(" 0"), "Should have column 0 header");
    assert!(output.contains(" 1"), "Should have column 1 header");
    assert!(output.contains(" 2"), "Should have column 2 header");
    assert!(output.contains(" 3"), "Should have column 3 header");
    assert!(output.contains(" 4"), "Should have column 4 header");

    // Should have row indicators
    assert!(output.contains("0 │"), "Should have row 0 indicator");
    assert!(output.contains("1 │"), "Should have row 1 indicator");

    // Should have grid border
    assert!(output.contains("┌"), "Should have top-left corner");
    assert!(output.contains("─"), "Should have horizontal border");
}

/// @demo cli/grid#abbreviated
/// @title Abbreviated Token Names
/// @description By default, shows abbreviated single-letter token names.
#[test]
fn test_grid_abbreviated() {
    let grid = vec!["{skin}{hair}{eye}".to_string()];

    let output = render_coordinate_grid(&grid, false);

    // Should show abbreviated names (first letter)
    assert!(output.contains(" s"), "Should show 's' for {{skin}}");
    assert!(output.contains(" h"), "Should show 'h' for {{hair}}");
    assert!(output.contains(" e"), "Should show 'e' for {{eye}}");

    // Should NOT show full names
    assert!(!output.contains("{skin}"), "Should not show full {{skin}}");
}

/// @demo cli/grid#full_names
/// @title Full Token Names
/// @description With --full flag, shows complete token names.
#[test]
fn test_grid_full_names() {
    let grid = vec!["{skin}{hair}".to_string()];

    let output = render_coordinate_grid(&grid, true);

    // Should show full token names
    assert!(output.contains("{skin}"), "Should show full {{skin}}");
    assert!(output.contains("{hair}"), "Should show full {{hair}}");
}

/// @demo cli/grid#underscore
/// @title Underscore Preserved
/// @description The {_} transparent token shows as underscore.
#[test]
fn test_grid_underscore() {
    let grid = vec!["{_}{a}{_}".to_string()];

    let output = render_coordinate_grid(&grid, false);

    // Underscore should be preserved (not abbreviated to something else)
    assert!(output.contains(" _"), "Should show underscore for {{_}}");
}

/// @demo cli/grid#empty
/// @title Empty Grid Handling
/// @description Empty grids produce empty output.
#[test]
fn test_grid_empty() {
    let grid: Vec<String> = vec![];

    let output = render_coordinate_grid(&grid, false);

    assert!(output.is_empty(), "Empty grid should produce empty output");
}

/// @demo cli/grid#single_pixel
/// @title Single Pixel Sprite
/// @description Single-pixel sprites render correctly with minimal grid.
#[test]
fn test_grid_single_pixel() {
    let grid = vec!["{x}".to_string()];

    let output = render_coordinate_grid(&grid, false);

    // Should have column 0 header
    assert!(output.contains(" 0"), "Should have column 0 header");

    // Should have row 0 indicator
    assert!(output.contains("0 │"), "Should have row 0 indicator");

    // Should have the token
    assert!(output.contains(" x"), "Should show 'x' for {{x}}");
}

/// @demo cli/grid#wide_sprite
/// @title Wide Sprite Grid
/// @description Wide sprites show all column numbers in header.
#[test]
fn test_grid_wide_sprite() {
    // Create a 10-wide sprite
    let grid = vec!["{a}{b}{c}{d}{e}{f}{g}{h}{i}{j}".to_string()];

    let output = render_coordinate_grid(&grid, false);

    // Should have double-digit column header
    assert!(output.contains(" 9"), "Should have column 9 header");
}

/// @demo cli/grid#tall_sprite
/// @title Tall Sprite Grid
/// @description Tall sprites show all row numbers.
#[test]
fn test_grid_tall_sprite() {
    // Create a 10-tall sprite
    let grid: Vec<String> = (0..10).map(|_| "{x}".to_string()).collect();

    let output = render_coordinate_grid(&grid, false);

    // Should have double-digit row indicator
    assert!(output.contains("9 │"), "Should have row 9 indicator");
}

/// @demo cli/grid#alignment
/// @title Column Alignment
/// @description Tokens are aligned in columns for easy reading.
#[test]
fn test_grid_alignment() {
    let grid = vec![
        "{a}{b}{c}".to_string(),
        "{d}{e}{f}".to_string(),
        "{g}{h}{i}".to_string(),
    ];

    let output = render_coordinate_grid(&grid, false);

    // All rows should have consistent column positions
    let lines: Vec<&str> = output.lines().collect();

    // Find token positions in content lines (skip header lines)
    let content_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.contains("│"))
        .cloned()
        .collect();

    assert_eq!(content_lines.len(), 3, "Should have 3 content rows");

    // Each content line should have same structure after the │
    for line in content_lines {
        let after_bar = line.split('│').nth(1).expect("Should have content after │");
        // Should have 3 tokens with consistent spacing
        let tokens: Vec<&str> = after_bar.split_whitespace().collect();
        assert_eq!(tokens.len(), 3, "Each row should have 3 tokens");
    }
}
