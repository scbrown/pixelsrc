//! Multiply blend mode demo tests.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/css/blend#multiply
/// @title Multiply Blend Mode
/// @description Darkens underlying colors by multiplying base and blend values: result = base * blend.
/// Useful for shadows and darkening effects.
#[test]
fn test_css_blend_multiply() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/multiply.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_multiply_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes: base layer has no blend (default), overlay has "multiply"
    assert_layer_blend_mode(jsonl, "blend_multiply_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_multiply_demo", 1, Some("multiply"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_multiply_demo");
}
