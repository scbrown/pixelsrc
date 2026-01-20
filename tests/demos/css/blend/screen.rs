//! Screen blend mode demo tests.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/css/blend#screen
/// @title Screen Blend Mode
/// @description Lightens underlying colors: result = 1 - (1 - base) * (1 - blend).
/// Useful for highlights and lightening effects.
#[test]
fn test_css_blend_screen() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/screen.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_screen_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(
        info.width,
        Some(6),
        "Composition should be 6 pixels wide"
    );
    assert_eq!(
        info.height,
        Some(6),
        "Composition should be 6 pixels tall"
    );

    // Verify blend modes: base layer has no blend (default), overlay has "screen"
    assert_layer_blend_mode(jsonl, "blend_screen_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_screen_demo", 1, Some("screen"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_screen_demo");
}
