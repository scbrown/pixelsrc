//! Background Fill Demo Tests
//!
//! Tests for composition layer background fills.

use crate::demos::{
    assert_composition_sprites_resolve, assert_validates, capture_composition_info,
    parse_compositions,
};

/// @demo format/composition/background#solid
/// @title Solid Background Fill
/// @description Layer with solid color background fill (black).
#[test]
fn test_solid_background() {
    let jsonl = include_str!("../../../examples/demos/composition/background_fill.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "solid_background");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (background + content)");
    assert_eq!(info.width, Some(6), "Composition should be 6 pixels wide");
    assert_eq!(info.height, Some(6), "Composition should be 6 pixels tall");

    // Verify the background layer has a fill
    let (_, _, compositions) = parse_compositions(jsonl);
    let comp = compositions.get("solid_background").unwrap();
    assert!(
        comp.layers[0].fill.is_some(),
        "Background layer should have fill"
    );
    assert_eq!(
        comp.layers[0].fill.as_deref(),
        Some("#000000FF"),
        "Background should be black"
    );

    assert_composition_sprites_resolve(jsonl, "solid_background");
}

/// @demo format/composition/background#transparent
/// @title Transparent Background
/// @description Composition with no background fill (transparent).
#[test]
fn test_transparent_background() {
    let jsonl = include_str!("../../../examples/demos/composition/background_fill.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "transparent_background");
    assert_eq!(info.layer_count, 1, "Should have 1 layer (content only)");

    // Verify the layer has no fill
    let (_, _, compositions) = parse_compositions(jsonl);
    let comp = compositions.get("transparent_background").unwrap();
    assert!(
        comp.layers[0].fill.is_none(),
        "Layer should not have fill"
    );

    assert_composition_sprites_resolve(jsonl, "transparent_background");
}

/// @demo format/composition/background#colored
/// @title Colored Background Fill
/// @description Layer with custom color background fill (green).
#[test]
fn test_colored_background() {
    let jsonl = include_str!("../../../examples/demos/composition/background_fill.jsonl");
    assert_validates(jsonl, true);

    let info = capture_composition_info(jsonl, "colored_background");
    assert_eq!(info.layer_count, 2, "Should have 2 layers (background + content)");

    // Verify the background layer has a green fill
    let (_, _, compositions) = parse_compositions(jsonl);
    let comp = compositions.get("colored_background").unwrap();
    assert_eq!(
        comp.layers[0].fill.as_deref(),
        Some("#00FF00FF"),
        "Background should be green"
    );

    assert_composition_sprites_resolve(jsonl, "colored_background");
}
