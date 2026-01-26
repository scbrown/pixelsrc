//! Layer Positioning Demo Tests
//!
//! Tests for sprite positioning within composition maps.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition/positioning#diagonal
/// @title Diagonal Placement
/// @description Sprites placed in a diagonal pattern across the grid.
#[test]
fn test_diagonal_placement() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "diagonal_placement");
    assert_eq!(info.layer_count, 1, "Should have 1 layer");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "diagonal_placement");
}

/// @demo format/composition/positioning#corners
/// @title Corner Placement
/// @description Sprites placed at corners of the composition.
#[test]
fn test_corner_placement() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "corner_placement");
    assert_eq!(info.layer_count, 1, "Should have 1 layer");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "corner_placement");
}

/// @demo format/composition/positioning#grid
/// @title Grid Fill Pattern
/// @description Checkerboard pattern filling the entire grid.
#[test]
fn test_grid_fill() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "grid_fill");
    assert_eq!(info.layer_count, 1, "Should have 1 layer");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify all cells are filled (no null sprites in this composition)
    assert_eq!(info.sprite_keys.len(), 2, "Should have exactly 2 sprite keys (R, G)");

    // Verify sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "grid_fill");
}

/// @demo format/composition/positioning#null_cells
/// @title Null Cell Handling
/// @description Verifies that null cells (.) are transparent.
#[test]
fn test_null_cells() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "diagonal_placement");

    // Should have "." as a null key
    assert!(
        info.sprite_keys.contains(&".".to_string()),
        "Should have null key '.'"
    );
}
