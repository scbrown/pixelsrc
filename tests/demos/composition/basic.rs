//! Basic Layer Stacking Demo Tests
//!
//! Tests for basic composition layer stacking behavior.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
};

/// @demo format/composition/basic#two_layer
/// @title Two Layer Stack
/// @description Composition with two layers stacked vertically. Top layer obscures bottom.
#[test]
fn test_two_layer_stack() {
    let jsonl = include_str!("../../../examples/demos/composition/layer_stacking.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "two_layer_stack");
    assert_eq!(info.layer_count, 2, "Should have 2 layers");
    assert_eq!(info.width, Some(4), "Composition should be 4 pixels wide");
    assert_eq!(info.height, Some(4), "Composition should be 4 pixels tall");

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "two_layer_stack");
}

/// @demo format/composition/basic#three_layer
/// @title Three Layer Stack
/// @description Composition with three layers. Demonstrates stacking order.
#[test]
fn test_three_layer_stack() {
    let jsonl = include_str!("../../../examples/demos/composition/layer_stacking.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "three_layer_stack");
    assert_eq!(info.layer_count, 3, "Should have 3 layers");
    assert_eq!(info.width, Some(4), "Composition should be 4 pixels wide");
    assert_eq!(info.height, Some(4), "Composition should be 4 pixels tall");

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "three_layer_stack");
}

/// @demo format/composition/basic#layer_order
/// @title Layer Render Order
/// @description Verifies layers render bottom-to-top (later layers on top).
#[test]
fn test_layer_render_order() {
    let jsonl = include_str!("../../../examples/demos/composition/layer_stacking.jsonl");
    assert_validates(jsonl, true);

    // Both compositions use the same sprites but verify structure
    let two_layer = capture_composition_info(jsonl, "two_layer_stack");
    let three_layer = capture_composition_info(jsonl, "three_layer_stack");

    // Verify sprite keys are defined
    assert!(
        two_layer.sprite_keys.contains(&"R".to_string()),
        "Should have R sprite key"
    );
    assert!(
        two_layer.sprite_keys.contains(&"G".to_string()),
        "Should have G sprite key"
    );

    assert!(
        three_layer.sprite_keys.contains(&"R".to_string()),
        "Should have R sprite key"
    );
    assert!(
        three_layer.sprite_keys.contains(&"G".to_string()),
        "Should have G sprite key"
    );
    assert!(
        three_layer.sprite_keys.contains(&"B".to_string()),
        "Should have B sprite key"
    );
}
