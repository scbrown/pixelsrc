//! Hex Color Format Demo Tests
//!
//! Tests for hex color notation: short (#RGB), full (#RRGGBB), and with alpha (#RRGGBBAA).

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo format/css/colors#hex_short
/// @title Short Hex Colors
/// @description 3-character hex notation (#RGB) expands to full #RRGGBB.
#[test]
fn test_hex_short() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hex.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    // Verify short hex palette exists
    let palette = palette_registry.get("hex_short").expect("hex_short palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#hex_full
/// @title Full Hex Colors
/// @description 6-character hex notation (#RRGGBB) for precise color specification.
#[test]
fn test_hex_full() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hex.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    // Verify full hex palette exists
    let palette = palette_registry.get("hex_full").expect("hex_full palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + r, g, b)");
}

/// @demo format/css/colors#hex_alpha
/// @title Hex with Alpha
/// @description 8-character hex notation (#RRGGBBAA) for colors with transparency.
#[test]
fn test_hex_alpha() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hex.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    // Verify alpha hex palette exists
    let palette = palette_registry.get("hex_alpha").expect("hex_alpha palette should exist");
    assert_eq!(palette.colors.len(), 4, "Should have 4 colors (transparent + 3 alpha levels)");
}

/// @demo format/css/colors#hex_sprite
/// @title Hex Color Sprite Rendering
/// @description Sprite using hex colors renders with expected dimensions.
#[test]
fn test_hex_sprite_render() {
    let jsonl = include_str!("../../../../examples/demos/css/colors/hex.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "rgb_short");
    assert_eq!(info.width, 3, "Sprite width should be 3");
    assert_eq!(info.height, 1, "Sprite height should be 1");
}
