//! Additional blend mode demo tests.
//!
//! Tests for darken, lighten, add, subtract, and difference blend modes.

use crate::demos::{
    assert_composition_sprites_resolve, assert_layer_blend_mode, assert_validates,
    capture_composition_info,
};

/// @demo format/css/blend#darken
/// @title Darken Blend Mode
/// @description Keeps darker color per channel: result = min(base, blend).
#[test]
fn test_css_blend_darken() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/darken.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_darken_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes
    assert_layer_blend_mode(jsonl, "blend_darken_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_darken_demo", 1, Some("darken"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_darken_demo");
}

/// @demo format/css/blend#lighten
/// @title Lighten Blend Mode
/// @description Keeps lighter color per channel: result = max(base, blend).
#[test]
fn test_css_blend_lighten() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/lighten.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_lighten_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes
    assert_layer_blend_mode(jsonl, "blend_lighten_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_lighten_demo", 1, Some("lighten"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_lighten_demo");
}

/// @demo format/css/blend#add
/// @title Add (Additive) Blend Mode
/// @description Additive blending: result = min(1, base + blend).
/// Useful for lights, glows, and brightening effects.
#[test]
fn test_css_blend_add() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/add.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_add_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes
    assert_layer_blend_mode(jsonl, "blend_add_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_add_demo", 1, Some("add"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_add_demo");
}

/// @demo format/css/blend#subtract
/// @title Subtract (Subtractive) Blend Mode
/// @description Subtractive blending: result = max(0, base - blend).
/// Useful for darkening by removal.
#[test]
fn test_css_blend_subtract() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/subtract.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_subtract_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes
    assert_layer_blend_mode(jsonl, "blend_subtract_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_subtract_demo", 1, Some("subtract"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_subtract_demo");
}

/// @demo format/css/blend#difference
/// @title Difference Blend Mode
/// @description Color difference: result = abs(base - blend).
/// Creates inverted/negative effects where colors differ.
#[test]
fn test_css_blend_difference() {
    let jsonl = include_str!("../../../../examples/demos/css/blend/difference.jsonl");
    assert_validates(jsonl, true);

    // Verify composition structure
    let info = capture_composition_info(jsonl, "blend_difference_demo");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (base + overlay)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify blend modes
    assert_layer_blend_mode(jsonl, "blend_difference_demo", 0, None);
    assert_layer_blend_mode(jsonl, "blend_difference_demo", 1, Some("difference"));

    // Verify all sprites can be resolved
    assert_composition_sprites_resolve(jsonl, "blend_difference_demo");
}
