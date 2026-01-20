//! Fills Demo
//!
//! Demonstrates background fills using solid colors and patterns.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition#fills
/// @title Solid Background Fill
/// @description Composition using a 1x1 sprite tiled to fill the background.
#[test]
fn test_solid_fill() {
    let jsonl = include_str!("../../../examples/demos/composition/fills.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "solid_fill");
    assert_eq!(info.layer_count, 1, "solid_fill should have 1 layer");
    assert_eq!(info.width, Some(4), "solid_fill width should be 4");
    assert_eq!(info.height, Some(4), "solid_fill height should be 4");

    assert_composition_sprites_resolve(jsonl, "solid_fill");
}

/// @demo format/composition#fills_pattern
/// @title Pattern Fill
/// @description Composition using a tiled pattern sprite as background.
#[test]
fn test_pattern_fill() {
    let jsonl = include_str!("../../../examples/demos/composition/fills.jsonl");

    let info = capture_composition_info(jsonl, "pattern_fill");
    assert_eq!(info.layer_count, 1, "pattern_fill should have 1 layer");
    assert_eq!(info.width, Some(4), "pattern_fill width should be 4");
    assert_eq!(info.height, Some(4), "pattern_fill height should be 4");

    assert_composition_sprites_resolve(jsonl, "pattern_fill");
}

/// @demo format/composition#fills_overlay
/// @title Fill with Overlay
/// @description Composition with a solid fill background and sprite overlay.
#[test]
fn test_fill_with_overlay() {
    let jsonl = include_str!("../../../examples/demos/composition/fills.jsonl");

    let info = capture_composition_info(jsonl, "fill_with_overlay");
    assert_eq!(
        info.layer_count, 2,
        "fill_with_overlay should have 2 layers (fill + overlay)"
    );
    assert_eq!(info.width, Some(6), "fill_with_overlay width should be 6");
    assert_eq!(info.height, Some(6), "fill_with_overlay height should be 6");

    assert_composition_sprites_resolve(jsonl, "fill_with_overlay");
}

/// @demo format/composition#fills_sprites
/// @title Fill Sprite Definitions
/// @description Verifies sprite keys for fill compositions.
#[test]
fn test_fill_sprite_keys() {
    let jsonl = include_str!("../../../examples/demos/composition/fills.jsonl");

    // solid_fill uses B for bg_blue
    let info = capture_composition_info(jsonl, "solid_fill");
    assert!(
        info.sprite_keys.contains(&"B".to_string()),
        "solid_fill should have 'B' sprite key"
    );

    // pattern_fill uses C for checker_tile
    let info = capture_composition_info(jsonl, "pattern_fill");
    assert!(
        info.sprite_keys.contains(&"C".to_string()),
        "pattern_fill should have 'C' sprite key"
    );

    // fill_with_overlay uses G for gray, S for star
    let info = capture_composition_info(jsonl, "fill_with_overlay");
    assert!(
        info.sprite_keys.contains(&"G".to_string()),
        "fill_with_overlay should have 'G' sprite key"
    );
    assert!(
        info.sprite_keys.contains(&"S".to_string()),
        "fill_with_overlay should have 'S' sprite key"
    );
}
