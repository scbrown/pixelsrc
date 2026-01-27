//! Basic sprite demos
//!
//! Minimal valid sprite definitions demonstrating core sprite features.

use crate::demos::{assert_dimensions, assert_validates, capture_render_info};

/// @demo format/sprite#basic
/// @title Basic Sprite
/// @description The simplest valid pixelsrc sprite - a minimal 1x1 pixel.
#[test]
fn test_basic_sprite() {
    let jsonl = r##"{"type": "sprite", "name": "dot", "palette": {"{x}": "#FF0000"}, "size": [1, 1], "regions": {"x": {"rect": [0, 0, 1, 1]}}}"##;

    assert_validates(jsonl, true);
    assert_dimensions(jsonl, "dot", 1, 1);

    let info = capture_render_info(jsonl, "dot");
    assert_eq!(info.color_count, 1, "Should have 1 color");
}

/// @demo format/sprite#transparency
/// @title Sprite with Transparency
/// @description Sprites using transparent pixels with the {_} token.
#[test]
fn test_transparency() {
    // Sprite with transparent background - a cross pattern (3x3)
    let jsonl = r##"{"type": "sprite", "name": "cross", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "size": [3, 3], "regions": {"x": {"points": [[1,0], [0,1], [1,1], [2,1], [1,2]]}}}"##;

    assert_validates(jsonl, true);
    assert_dimensions(jsonl, "cross", 3, 3);

    let info = capture_render_info(jsonl, "cross");
    assert_eq!(info.color_count, 2, "Should have 2 colors (transparent + red)");
}

/// @demo format/sprite#multichar_keys
/// @title Multi-Character Color Keys
/// @description Sprites using multi-character tokens like {r1}, {bg}, etc.
#[test]
fn test_multichar_keys() {
    let jsonl = r##"{"type": "sprite", "name": "gradient", "palette": {"{bg}": "#000000", "{r1}": "#330000", "{r2}": "#660000", "{r3}": "#990000", "{r4}": "#CC0000", "{r5}": "#FF0000"}, "size": [6, 1], "regions": {"bg": {"points": [[0,0]]}, "r1": {"points": [[1,0]]}, "r2": {"points": [[2,0]]}, "r3": {"points": [[3,0]]}, "r4": {"points": [[4,0]]}, "r5": {"points": [[5,0]]}}}"##;

    assert_validates(jsonl, true);
    assert_dimensions(jsonl, "gradient", 6, 1);

    let info = capture_render_info(jsonl, "gradient");
    assert_eq!(info.color_count, 6, "Should have 6 colors for gradient");
}
