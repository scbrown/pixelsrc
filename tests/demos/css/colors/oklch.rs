//! OKLCH Color Format Demo Tests
//!
//! Tests for oklch() CSS color function (perceptually uniform color space).

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#oklch_basic
/// @title OKLCH Colors
/// @description oklch(lightness, chroma, hue) perceptually uniform color space.
#[test]
fn test_oklch_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/oklch.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("oklch_basic").expect("oklch_basic palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#oklch_lightness
/// @title OKLCH Lightness
/// @description Perceptually uniform lightness from 0.25 to 0.75.
#[test]
fn test_oklch_lightness() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/oklch.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("oklch_lightness").expect("oklch_lightness palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 lightness levels)");
}

/// @demo format/css/colors#oklch_chroma
/// @title OKLCH Chroma
/// @description Varying chroma (saturation) from 0 to 0.2.
#[test]
fn test_oklch_chroma() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/oklch.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("oklch_chroma").expect("oklch_chroma palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 chroma levels)");
}

/// @demo format/css/colors#oklch_sprite
/// @title OKLCH Color Sprite Rendering
/// @description Sprite using OKLCH colors renders correctly.
#[test]
fn test_oklch_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/oklch.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "oklch_demo");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
