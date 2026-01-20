//! Normal blend mode demo tests.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/css/blend#normal
/// @title Normal Blend Mode
/// @description Standard alpha compositing (source over destination). The default blend mode.
#[test]
fn test_css_blend_normal() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/normal.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_normal_demo");
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

    // Verify blend modes: base layer has no blend (default), overlay has "normal"
    assert_layer_blend_mode(jsonl, "blend_normal_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_normal_demo", 1, Some("normal"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_normal_demo");
}
