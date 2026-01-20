//! Positioning Demo
//!
//! Demonstrates sprite positioning via cell_size and map layouts.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition#positioning
/// @title Grid-Based Positioning
/// @description Composition using cell_size to position sprites on a grid.
#[test]
fn test_grid_positions() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "grid_positions");
    assert_eq!(info.layer_count, 1, "grid_positions should have 1 layer");
    assert_eq!(info.width, Some(8), "grid_positions width should be 8");
    assert_eq!(info.height, Some(8), "grid_positions height should be 8");

    assert_composition_sprites_resolve(jsonl, "grid_positions");
}

/// @demo format/composition#positioning_corners
/// @title Corner Placement
/// @description Composition placing sprites at corner positions using cell_size.
#[test]
fn test_corner_placement() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");

    let info = capture_composition_info(jsonl, "corner_placement");
    assert_eq!(info.layer_count, 1, "corner_placement should have 1 layer");
    assert_eq!(info.width, Some(6), "corner_placement width should be 6");
    assert_eq!(info.height, Some(6), "corner_placement height should be 6");

    assert_composition_sprites_resolve(jsonl, "corner_placement");
}

/// @demo format/composition#positioning_centered
/// @title Centered Positioning
/// @description Composition with sprites positioned in the center area.
#[test]
fn test_centered_positioning() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");

    let info = capture_composition_info(jsonl, "centered");
    assert_eq!(info.layer_count, 1, "centered should have 1 layer");
    assert_eq!(info.width, Some(8), "centered width should be 8");
    assert_eq!(info.height, Some(8), "centered height should be 8");

    assert_composition_sprites_resolve(jsonl, "centered");
}

/// @demo format/composition#positioning_sprites
/// @title Position Sprite Keys
/// @description Verifies sprite key definitions for positioning demos.
#[test]
fn test_positioning_sprite_keys() {
    let jsonl = include_str!("../../../examples/demos/composition/positioning.jsonl");

    // grid_positions uses D for dot, . for null
    let info = capture_composition_info(jsonl, "grid_positions");
    assert!(
        info.sprite_keys.contains(&"D".to_string()),
        "grid_positions should have 'D' sprite key"
    );
    assert!(
        info.sprite_keys.contains(&".".to_string()),
        "grid_positions should have '.' null key"
    );

    // corner_placement uses C for cross
    let info = capture_composition_info(jsonl, "corner_placement");
    assert!(
        info.sprite_keys.contains(&"C".to_string()),
        "corner_placement should have 'C' sprite key"
    );
}
