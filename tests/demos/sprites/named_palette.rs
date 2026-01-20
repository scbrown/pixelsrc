//! Named palette demos
//!
//! Sprites using palette references instead of inline colors.

use crate::demos::{
    assert_color_count, assert_uses_palette, assert_validates, capture_render_info,
};

/// @demo format/sprite#named_palette
/// @title Named Palette Reference
/// @description Sprite referencing a separately-defined palette by name.
#[test]
fn test_named_palette_sprite() {
    let jsonl = include_str!("../../../examples/demos/sprites/named_palette.jsonl");
    assert_validates(jsonl, true);

    // Verify sprite exists and renders
    let info = capture_render_info(jsonl, "icon");
    assert_eq!(info.width, 3, "Icon sprite should be 3 pixels wide");
    assert_eq!(info.height, 3, "Icon sprite should be 3 pixels tall");
}

/// @demo format/sprite#palette_reference
/// @title Palette Reference Verification
/// @description Verifies sprite correctly references its named palette.
#[test]
fn test_named_palette_reference() {
    let jsonl = include_str!("../../../examples/demos/sprites/named_palette.jsonl");

    // Verify the sprite uses the "retro" palette
    assert_uses_palette(jsonl, "icon", "retro");
}

/// @demo format/sprite#palette_colors
/// @title Palette Color Count
/// @description Named palette with 4 colors: transparent, bg, fg, accent.
#[test]
fn test_named_palette_colors() {
    let jsonl = include_str!("../../../examples/demos/sprites/named_palette.jsonl");

    // 4 colors: {_} transparent, {bg}, {fg}, {accent}
    assert_color_count(jsonl, "icon", 4);
}
