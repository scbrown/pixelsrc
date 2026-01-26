//! Named Color Demo Tests
//!
//! Tests for CSS named colors (e.g., "red", "blue", "coral").

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#named_basic
/// @title Named CSS Colors
/// @description Standard CSS named colors like "red", "green", "blue".
#[test]
fn test_named_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/named.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("named_basic").expect("named_basic palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#named_warm
/// @title Warm Named Colors
/// @description Warm-toned named colors: coral, orange, gold.
#[test]
fn test_named_warm() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/named.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("named_warm").expect("named_warm palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors");
}

/// @demo format/css/colors#named_cool
/// @title Cool Named Colors
/// @description Cool-toned named colors: teal, cyan, aqua.
#[test]
fn test_named_cool() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/named.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("named_cool").expect("named_cool palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors");
}

/// @demo format/css/colors#named_neutral
/// @title Neutral Named Colors
/// @description Grayscale named colors: white, silver, gray.
#[test]
fn test_named_neutral() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/named.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("named_neutral").expect("named_neutral palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors");
}

/// @demo format/css/colors#named_sprite
/// @title Named Color Sprite Rendering
/// @description Sprite using named colors renders correctly.
#[test]
fn test_named_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/named.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "named_demo");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
