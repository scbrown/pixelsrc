//! Basic Layers Demo
//!
//! Demonstrates simple sprite stacking with multiple layers in a composition.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition#basic
/// @title Basic Layer Stacking
/// @description A composition with two layers: a background and foreground sprite.
#[test]
fn test_basic_layer_stacking() {
    let jsonl = include_str!("../../../examples/demos/composition/basic_layers.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "scene");
    assert_eq!(info.layer_count, 2, "Scene should have 2 layers (bg and fg)");
    assert_eq!(info.width, Some(5), "Scene width should be 5");
    assert_eq!(info.height, Some(5), "Scene height should be 5");
}

/// @demo format/composition#sprites_resolve
/// @title Layer Sprites Resolution
/// @description Verifies all sprites referenced by the composition can be resolved.
#[test]
fn test_layer_sprites_resolve() {
    let jsonl = include_str!("../../../examples/demos/composition/basic_layers.jsonl");

    // Verify all referenced sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "scene");
}

/// @demo format/composition#sprite_keys
/// @title Composition Sprite Key Mapping
/// @description Verifies sprite key definitions in the composition.
#[test]
fn test_sprite_key_mapping() {
    let jsonl = include_str!("../../../examples/demos/composition/basic_layers.jsonl");

    let info = capture_composition_info(jsonl, "scene");

    // Verify expected sprite keys are defined
    assert!(
        info.sprite_keys.contains(&"B".to_string()),
        "Scene should have 'B' sprite key for background"
    );
    assert!(
        info.sprite_keys.contains(&"F".to_string()),
        "Scene should have 'F' sprite key for foreground"
    );
}
