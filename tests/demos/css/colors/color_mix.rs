//! color-mix() Demo Tests
//!
//! Tests for CSS color-mix() function for blending colors.

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#color_mix_basic
/// @title Basic Color Mix
/// @description color-mix(in srgb, color1 50%, color2 50%) for equal blending.
#[test]
fn test_color_mix_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/color_mix.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("color_mix_basic").expect("color_mix_basic palette should exist");
    assert_eq!(palette.colors.len(), 2, "Should have 2 colors (transparent + mixed)");
}

/// @demo format/css/colors#color_mix_oklch
/// @title Color Mix in OKLCH
/// @description color-mix(in oklch, ...) for perceptually uniform mixing.
#[test]
fn test_color_mix_oklch() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/color_mix.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("shadows_oklch").expect("shadows_oklch palette should exist");
    assert_eq!(palette.colors.len(), 2, "Should have 2 colors (transparent + mixed shadow)");
}

/// @demo format/css/colors#color_mix_highlight
/// @title Color Mix for Highlights
/// @description Using color-mix to create highlight variations.
#[test]
fn test_color_mix_highlight() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/color_mix.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("highlights_srgb").expect("highlights_srgb palette should exist");
    assert_eq!(palette.colors.len(), 2, "Should have 2 colors");
}

/// @demo format/css/colors#color_mix_sprite
/// @title Color Mix Sprite Rendering
/// @description Sprite using color-mix renders correctly.
#[test]
fn test_color_mix_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/color_mix.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "shaded_square");
    assert_eq!(info.width, 2, "Sprite width should be 2");
    assert_eq!(info.height, 2, "Sprite height should be 2");
}
