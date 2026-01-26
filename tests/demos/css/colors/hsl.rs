//! HSL Color Format Demo Tests
//!
//! Tests for hsl() CSS color function.

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#hsl_basic
/// @title HSL Colors
/// @description hsl(hue, saturation%, lightness%) with hue in degrees.
#[test]
fn test_hsl_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hsl.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hsl_basic").expect("hsl_basic palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#hsl_saturation
/// @title HSL Saturation Levels
/// @description Varying saturation from 0% (grayscale) to 100% (fully saturated).
#[test]
fn test_hsl_saturation() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hsl.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hsl_saturation").expect("hsl_saturation palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 saturation levels)");
}

/// @demo format/css/colors#hsl_lightness
/// @title HSL Lightness Levels
/// @description Varying lightness from dark (25%) to light (75%).
#[test]
fn test_hsl_lightness() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hsl.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hsl_lightness").expect("hsl_lightness palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 lightness levels)");
}

/// @demo format/css/colors#hsl_sprite
/// @title HSL Color Sprite Rendering
/// @description Sprite using HSL colors renders correctly.
#[test]
fn test_hsl_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hsl.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "hsl_demo");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
