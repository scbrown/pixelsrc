//! Multi-Sprite Scene Demo Tests
//!
//! Tests for complex scenes with multiple sprites and layers.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition#multi_sprite
/// @title Multi-Sprite Scenes
/// @description Multi-layer scene with sky, ground, and multiple sprite types.
#[test]
fn test_simple_scene() {
    let jsonl = include_str!("../../../examples/demos/composition/multi_sprite.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "simple_scene");
    assert_eq!(info.layer_count, 3, "Should have 3 layers (sky, ground, objects)");
    assert_eq!(info.width, Some(12), "Composition should be 12 pixels wide");
    assert_eq!(info.height, Some(8), "Composition should be 8 pixels tall");

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "simple_scene");
}

/// @demo format/composition/scene#layered
/// @title Layered Scene
/// @description Scene with overlapping foreground and background layers.
#[test]
fn test_layered_scene() {
    let jsonl = include_str!("../../../examples/demos/composition/multi_sprite.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "layered_scene");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (background, foreground)");
    assert_eq!(info.width, Some(8), "Composition should be 8 pixels wide");
    assert_eq!(info.height, Some(8), "Composition should be 8 pixels tall");

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "layered_scene");
}

/// @demo format/composition/scene#tile_grid
/// @title Tile Grid Scene
/// @description Scene using tiled sprites in a grid pattern.
#[test]
fn test_tile_grid() {
    let jsonl = include_str!("../../../examples/demos/composition/multi_sprite.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "tile_grid");
    assert_eq!(info.layer_count, 1, "Should have 1 layer");
    assert_eq!(info.width, Some(12), "Composition should be 12 pixels wide");
    assert_eq!(info.height, Some(12), "Composition should be 12 pixels tall");

    // Verify no null sprites (all cells filled)
    assert!(!info.sprite_keys.contains(&".".to_string()), "Should not have null sprite key");

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "tile_grid");
}

/// @demo format/composition/scene#sprite_variety
/// @title Multiple Sprite Types
/// @description Scene demonstrating multiple distinct sprite types.
#[test]
fn test_sprite_variety() {
    let jsonl = include_str!("../../../examples/demos/composition/multi_sprite.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "simple_scene");

    // Count unique sprite mappings (excluding null)
    let sprite_count = info.sprite_keys.iter().filter(|k| *k != ".").count();

    assert!(
        sprite_count >= 5,
        "Scene should have at least 5 different sprite types, got {sprite_count}"
    );
}
