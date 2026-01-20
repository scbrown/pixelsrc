//! Inline palette demos
//!
//! Sprites with inline color definitions embedded directly in the sprite.

use crate::demos::{assert_color_count, assert_dimensions, assert_validates, capture_render_info};

/// @demo format/sprite#inline_palette
/// @title Inline Palette Definition
/// @description Sprite with colors defined inline rather than referencing a named palette.
#[test]
fn test_inline_palette_sprite() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");
    assert_validates(jsonl, true);

    // Verify sprite exists and renders
    let info = capture_render_info(jsonl, "heart");
    assert_eq!(info.width, 5, "Heart sprite should be 5 pixels wide");
    assert_eq!(info.height, 5, "Heart sprite should be 5 pixels tall");
}

/// @demo format/sprite#inline_colors
/// @title Inline Color Count
/// @description Heart sprite with 3 colors: transparent, red, and orange.
#[test]
fn test_inline_palette_colors() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");

    // 3 colors: {_} transparent, {r} red, {p} orange/pink
    assert_color_count(jsonl, "heart", 3);
}

/// @demo format/sprite#inline_dimensions
/// @title Inline Palette Dimensions
/// @description Verifies heart sprite renders at expected 5x5 dimensions.
#[test]
fn test_inline_palette_dimensions() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");
    assert_dimensions(jsonl, "heart", 5, 5);
}

/// @demo format/sprite#inline_no_palette_ref
/// @title No Named Palette
/// @description Verifies inline palette sprites have no palette_name set.
#[test]
fn test_inline_palette_no_named_ref() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");

    let info = capture_render_info(jsonl, "heart");
    assert!(
        info.palette_name.is_none(),
        "Inline palette sprite should not have a palette_name"
    );
}
