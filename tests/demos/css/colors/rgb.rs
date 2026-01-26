//! RGB/RGBA Color Format Demo Tests
//!
//! Tests for rgb() and rgba() CSS color functions.

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#rgb_basic
/// @title RGB Colors
/// @description rgb(r, g, b) function with integer values 0-255.
#[test]
fn test_rgb_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/rgb.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("rgb_basic").expect("rgb_basic palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#rgba_alpha
/// @title RGBA with Alpha
/// @description rgba(r, g, b, a) function with alpha channel 0.0-1.0.
#[test]
fn test_rgba_alpha() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/rgb.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("rgba_alpha").expect("rgba_alpha palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 alpha levels)");
}

/// @demo format/css/colors#rgb_sprite
/// @title RGB Color Sprite Rendering
/// @description Sprite using RGB colors renders correctly.
#[test]
fn test_rgb_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/rgb.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "rgb_demo");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
