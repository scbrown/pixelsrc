//! Blend Modes Demo
//!
//! Demonstrates layer blend modes: normal, multiply, screen, overlay.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/composition#blend
/// @title Normal Blend Mode
/// @description Composition with explicit normal blend mode on top layer.
#[test]
fn test_blend_normal() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "blend_normal");
    assert_eq!(info.layer_count, 2, "blend_normal should have 2 layers");

    // Layer 0 (base) has no blend mode (default)
    assert_layer_blend_mode(jsonl, "blend_normal", 0, None);
    // Layer 1 uses "normal" blend mode
    assert_layer_blend_mode(jsonl, "blend_normal", 1, Some("normal"));

    assert_composition_sprites_resolve(jsonl, "blend_normal");
}

/// @demo format/composition#blend_multiply
/// @title Multiply Blend Mode
/// @description Composition using multiply blend to darken overlapping areas.
#[test]
fn test_blend_multiply() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");

    let info = capture_composition_info(jsonl, "blend_multiply");
    assert_eq!(info.layer_count, 2, "blend_multiply should have 2 layers");

    // Layer 1 uses "multiply" blend mode
    assert_layer_blend_mode(jsonl, "blend_multiply", 1, Some("multiply"));

    assert_composition_sprites_resolve(jsonl, "blend_multiply");
}

/// @demo format/composition#blend_screen
/// @title Screen Blend Mode
/// @description Composition using screen blend to lighten overlapping areas.
#[test]
fn test_blend_screen() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");

    let info = capture_composition_info(jsonl, "blend_screen");
    assert_eq!(info.layer_count, 2, "blend_screen should have 2 layers");

    // Layer 1 uses "screen" blend mode
    assert_layer_blend_mode(jsonl, "blend_screen", 1, Some("screen"));

    assert_composition_sprites_resolve(jsonl, "blend_screen");
}

/// @demo format/composition#blend_overlay
/// @title Overlay Blend Mode
/// @description Composition using overlay blend for contrast effects.
#[test]
fn test_blend_overlay() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");

    let info = capture_composition_info(jsonl, "blend_overlay");
    assert_eq!(info.layer_count, 2, "blend_overlay should have 2 layers");

    // Layer 1 uses "overlay" blend mode
    assert_layer_blend_mode(jsonl, "blend_overlay", 1, Some("overlay"));

    assert_composition_sprites_resolve(jsonl, "blend_overlay");
}

/// @demo format/composition#blend_all
/// @title All Blend Modes Validate
/// @description Verifies all blend mode compositions in the fixture are valid.
#[test]
fn test_all_blend_modes_valid() {
    let jsonl = include_str!("../../../examples/demos/composition/blend_modes.jsonl");
    assert_validates(jsonl, true);

    // Verify all compositions have expected size
    for name in &["blend_normal", "blend_multiply", "blend_screen", "blend_overlay"] {
        let info = capture_composition_info(jsonl, name);
        assert_eq!(info.width, Some(4), "{} should be 4 pixels wide", name);
        assert_eq!(info.height, Some(4), "{} should be 4 pixels tall", name);
    }
}
