//! Basic sprite demos
//!
//! Minimal valid sprite definitions.

use crate::demos::{assert_color_count, assert_dimensions, assert_validates, capture_render_info};

/// @demo format/sprite#basic
/// @title Minimal Valid Sprite
/// @description A 3x3 sprite with an inline palette demonstrating the minimum required fields.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_basic_sprite() {
    let jsonl = include_str!("../../../examples/demos/sprites/basic.jsonl");
    assert_validates(jsonl, true);

    // Verify sprite dimensions (3x3)
    let info = capture_render_info(jsonl, "square");
    assert_eq!(info.width, 3, "Basic sprite should be 3 pixels wide");
    assert_eq!(info.height, 3, "Basic sprite should be 3 pixels tall");
    assert_eq!(info.frame_count, 1, "Static sprite should have 1 frame");
}

/// @demo format/sprite#colors
/// @title Sprite Color Count
/// @description Verifies palette color count is correctly reported.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_basic_sprite_colors() {
    let jsonl = include_str!("../../../examples/demos/sprites/basic.jsonl");

    // 2 colors: transparent {_} and red {r}
    assert_color_count(jsonl, "square", 2);
}

/// @demo format/sprite#dimensions
/// @title Sprite Dimension Verification
/// @description Uses assert_dimensions helper for explicit dimension checks.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_basic_sprite_dimensions() {
    let jsonl = include_str!("../../../examples/demos/sprites/basic.jsonl");
    assert_dimensions(jsonl, "square", 3, 3);
}
