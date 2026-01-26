//! Blend Mode Demo Tests
//!
//! Tests for composition layer blend modes.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/composition/blend#multiply
/// @title Multiply Blend Mode
/// @description Layer blended using multiply mode (darkens).
#[test]
fn test_multiply_blend() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "multiply_blend");
    assert_eq!(info.layer_count, 2, "Should have 2 layers");

    // Base layer has no blend mode (default normal)
    assert_layer_blend_mode(jsonl, "multiply_blend", 0, None);
    // Overlay layer has multiply
    assert_layer_blend_mode(jsonl, "multiply_blend", 1, Some("multiply"));

    assert_composition_sprites_resolve(jsonl, "multiply_blend");
}

/// @demo format/composition/blend#screen
/// @title Screen Blend Mode
/// @description Layer blended using screen mode (lightens).
#[test]
fn test_screen_blend() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "screen_blend");
    assert_eq!(info.layer_count, 2, "Should have 2 layers");

    // Overlay layer has screen
    assert_layer_blend_mode(jsonl, "screen_blend", 1, Some("screen"));

    assert_composition_sprites_resolve(jsonl, "screen_blend");
}

/// @demo format/composition/blend#overlay
/// @title Overlay Blend Mode
/// @description Layer blended using overlay mode (increases contrast).
#[test]
fn test_overlay_blend() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "overlay_blend");
    assert_eq!(info.layer_count, 2, "Should have 2 layers");

    // Overlay layer has overlay blend
    assert_layer_blend_mode(jsonl, "overlay_blend", 1, Some("overlay"));

    assert_composition_sprites_resolve(jsonl, "overlay_blend");
}

/// @demo format/composition/blend#add
/// @title Additive Blend Mode
/// @description Layer blended using additive mode (brightens).
#[test]
fn test_add_blend() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "add_blend");
    assert_eq!(info.layer_count, 2, "Should have 2 layers");

    // Overlay layer has add blend
    assert_layer_blend_mode(jsonl, "add_blend", 1, Some("add"));

    assert_composition_sprites_resolve(jsonl, "add_blend");
}
