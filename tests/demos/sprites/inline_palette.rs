//! Inline palette demos
//!
//! Sprites with inline color definitions embedded directly in the sprite.

use crate::demos::{assert_color_count, assert_validates, capture_render_info};

/// @demo format/sprite#inline_palette
/// @title Inline Palette Definition
/// @description Sprites can define colors inline without a separate palette.
#[test]
fn test_inline_palette() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");

    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "heart");
    assert_eq!(info.width, 4, "Heart sprite width should be 4");
    assert_eq!(info.height, 4, "Heart sprite height should be 4");
    assert_eq!(info.color_count, 3, "Should have 3 colors (transparent, red, light red)");
    assert!(info.palette_name.is_none(), "Inline palette should have no name");
}

/// @title Inline Color Count
/// @description Heart sprite with 3 colors: transparent, red, and orange.
#[test]
fn test_inline_palette_colors() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");

    // 3 colors: {_} transparent, {r} red, {p} orange/pink
    assert_color_count(jsonl, "heart", 3);
}
/// @title No Named Palette
/// @description Verifies inline palette sprites have no palette_name set.
#[test]
fn test_inline_palette_no_named_ref() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");

    let info = capture_render_info(jsonl, "heart");
    assert!(info.palette_name.is_none(), "Inline palette sprite should not have a palette_name");
}
