//! Inline palette demos
//!
//! Sprites with inline color definitions embedded directly in the sprite.

use crate::demos::{assert_color_count, capture_render_info};
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
