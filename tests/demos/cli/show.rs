//! Show Command Demo Tests
//!
//! Demonstrates the `pxl show` command functionality for displaying
//! sprite information and colored terminal output.

use pixelsrc::terminal::{render_ansi_grid, render_coordinate_grid, ANSI_RESET};
use std::collections::HashMap;

// ============================================================================
// Basic Show Tests
// ============================================================================

/// @demo cli/show#basic
/// @title Basic Sprite Display
/// @description `pxl show` displays a sprite with colored ANSI output in the terminal.
#[test]
fn test_show_basic_sprite() {
    let grid = vec!["{_}{r}{_}".to_string(), "{r}{r}{r}".to_string(), "{_}{r}{_}".to_string()];
    let palette = HashMap::from([
        ("{_}".to_string(), "#00000000".to_string()),
        ("{r}".to_string(), "#FF0000".to_string()),
    ]);
    let aliases = HashMap::new();

    let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);

    // Output should contain ANSI escape sequences
    assert!(colored.contains("\x1b["), "Output should contain ANSI escape sequences");
    // Output should have reset at the end
    assert!(colored.contains(ANSI_RESET), "Output should contain ANSI reset sequence");
    // Legend should mention the tokens
    assert!(legend.contains("{r}") || legend.contains("r"), "Legend should list token names");
}

/// @demo cli/show#palette_colors
/// @title Show with Palette Colors
/// @description Terminal output shows actual palette colors as backgrounds.
#[test]
fn test_show_palette_colors() {
    let grid = vec!["{red}{green}{blue}".to_string()];
    let palette = HashMap::from([
        ("{red}".to_string(), "#FF0000".to_string()),
        ("{green}".to_string(), "#00FF00".to_string()),
        ("{blue}".to_string(), "#0000FF".to_string()),
    ]);
    let aliases = HashMap::new();

    let (colored, _legend) = render_ansi_grid(&grid, &palette, &aliases);

    // Should contain RGB ANSI sequences for the colors
    assert!(
        colored.contains("255;0;0") || colored.contains("48;2;255;0;0"),
        "Should have red background color"
    );
    assert!(
        colored.contains("0;255;0") || colored.contains("48;2;0;255;0"),
        "Should have green background color"
    );
    assert!(
        colored.contains("0;0;255") || colored.contains("48;2;0;0;255"),
        "Should have blue background color"
    );
}

/// @demo cli/show#transparency
/// @title Show Transparent Pixels
/// @description Transparent pixels are displayed with a distinct visual representation.
#[test]
fn test_show_transparency() {
    let grid = vec!["{_}{x}{_}".to_string()];
    let palette = HashMap::from([
        ("{_}".to_string(), "#00000000".to_string()),
        ("{x}".to_string(), "#FF0000".to_string()),
    ]);
    let aliases = HashMap::new();

    let (colored, _legend) = render_ansi_grid(&grid, &palette, &aliases);

    // Transparent uses 256-color mode gray (48;5;236)
    assert!(colored.contains("48;5;236"), "Transparent should use distinct background (gray)");
}

// ============================================================================
// Coordinate Grid Tests
// ============================================================================

/// @demo cli/show#coordinates
/// @title Show with Coordinates (--coords)
/// @description `pxl show --coords` displays pixel coordinates alongside the sprite.
#[test]
fn test_show_with_coordinates() {
    let grid = vec!["{a}{b}".to_string(), "{c}{d}".to_string()];

    // render_coordinate_grid takes grid and full_names flag
    let coord_output = render_coordinate_grid(&grid, false);

    // Coordinate grid should show row/column numbers or be non-empty
    assert!(!coord_output.is_empty(), "Coordinate display should produce output");
}

// ============================================================================
// Alias Tests
// ============================================================================

/// @demo cli/show#aliases
/// @title Show with Token Aliases
/// @description Custom single-character aliases can be used for compact display.
#[test]
fn test_show_with_aliases() {
    let grid = vec!["{skin}{hair}".to_string(), "{skin}{hair}".to_string()];
    let palette = HashMap::from([
        ("{skin}".to_string(), "#FFD5B4".to_string()),
        ("{hair}".to_string(), "#8B4513".to_string()),
    ]);
    let aliases = HashMap::from([('s', "{skin}".to_string()), ('h', "{hair}".to_string())]);

    let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);

    // The colored output should use the alias characters
    assert!(colored.contains('s') || colored.contains('h'), "Should display alias characters");
    // Legend should contain "Legend:" header and color info
    assert!(legend.contains("Legend:"), "Legend should have header");
    // Should have ANSI color codes in the output
    assert!(colored.contains("\x1b["), "Should contain ANSI escape sequences");
}

// ============================================================================
// Edge Cases
// ============================================================================

/// @demo cli/show#empty_grid
/// @title Show Empty Sprite
/// @description Empty sprites are handled gracefully.
#[test]
fn test_show_empty_grid() {
    let grid: Vec<String> = vec![];
    let palette = HashMap::new();
    let aliases = HashMap::new();

    let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);

    // Should return empty strings without crashing
    assert!(colored.is_empty() || colored.trim().is_empty(), "Empty grid produces empty output");
    assert!(legend.is_empty() || !legend.contains("{"), "Empty grid has no legend entries");
}

/// @demo cli/show#single_pixel
/// @title Show Single Pixel
/// @description Single pixel sprites display correctly.
#[test]
fn test_show_single_pixel() {
    let grid = vec!["{x}".to_string()];
    let palette = HashMap::from([("{x}".to_string(), "#FF00FF".to_string())]);
    let aliases = HashMap::new();

    let (colored, _legend) = render_ansi_grid(&grid, &palette, &aliases);

    assert!(!colored.is_empty(), "Single pixel should produce output");
    assert!(colored.contains("\x1b["), "Should have ANSI formatting");
}

/// @demo cli/show#large_sprite
/// @title Show Large Sprite
/// @description Large sprites are rendered efficiently.
#[test]
fn test_show_large_sprite() {
    // Create a 32x32 checkerboard pattern
    let mut grid = Vec::new();
    for row in 0..32 {
        let mut row_str = String::new();
        for col in 0..32 {
            if (row + col) % 2 == 0 {
                row_str.push_str("{a}");
            } else {
                row_str.push_str("{b}");
            }
        }
        grid.push(row_str);
    }
    let palette = HashMap::from([
        ("{a}".to_string(), "#FFFFFF".to_string()),
        ("{b}".to_string(), "#000000".to_string()),
    ]);
    let aliases = HashMap::new();

    let (colored, _) = render_ansi_grid(&grid, &palette, &aliases);

    // Should produce output without panicking
    assert!(!colored.is_empty(), "Large sprite should render");
    // Output should have 32 lines (one per row)
    let line_count = colored.lines().count();
    assert!(line_count >= 32, "Should have at least 32 lines for 32 rows");
}
