//! HWB Color Format Demo Tests
//!
//! Tests for hwb() CSS color function (Hue-Whiteness-Blackness).

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#hwb_basic
/// @title HWB Colors
/// @description hwb(hue, whiteness%, blackness%) color specification.
#[test]
fn test_hwb_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hwb.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hwb_basic").expect("hwb_basic palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#hwb_whiteness
/// @title HWB Whiteness
/// @description Varying whiteness from 0% (pure hue) to 50% (tinted).
#[test]
fn test_hwb_whiteness() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hwb.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hwb_whiteness").expect("hwb_whiteness palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 whiteness levels)");
}

/// @demo format/css/colors#hwb_blackness
/// @title HWB Blackness
/// @description Varying blackness from 0% (bright) to 50% (shaded).
#[test]
fn test_hwb_blackness() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hwb.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry.get("hwb_blackness").expect("hwb_blackness palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 blackness levels)");
}

/// @demo format/css/colors#hwb_sprite
/// @title HWB Color Sprite Rendering
/// @description Sprite using HWB colors renders correctly.
#[test]
fn test_hwb_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hwb.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "hwb_demo");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
