//! Show Command Demo Tests
//!
//! Demonstrates the `pxl show` command functionality for displaying
//! sprite information and colored terminal output.

use pixelsrc::terminal::{render_ansi_grid, render_coordinate_grid, ANSI_RESET};
use std::collections::HashMap;

// ============================================================================
// Basic Show Tests
// ============================================================================
/// @title Show with Palette Colors
/// @description Terminal output shows actual palette colors as backgrounds./// @demo cli/show#transparency
/// @title Show Transparent Pixels
/// @description Transparent pixels are displayed with a distinct visual representation.// ============================================================================
// Coordinate Grid Tests
// ============================================================================
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
// Edge Cases
// ============================================================================
#[test]
#[ignore = "Grid format deprecated"]
fn test_show_empty_grid() {
    let grid: Vec<String> = vec![];
    let palette = HashMap::new();
    let aliases = HashMap::new();

    let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);

    // Should return empty strings without crashing
    assert!(colored.is_empty() || colored.trim().is_empty(), "Empty grid produces empty output");
    assert!(legend.is_empty() || !legend.contains("{"), "Empty grid has no legend entries");
}
