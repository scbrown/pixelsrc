//! Basic Animation Demo Tests
//!
//! Tests for basic frame sequence animations.

use crate::demos::{assert_frame_count, assert_validates, capture_spritesheet_info, parse_content};

/// @demo format/animation#basic
/// @title Basic Frame Sequence
/// @description Animation defined as a sequence of sprite frames that play in order.
#[test]
fn test_basic_frame_sequence() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_sequence.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify all frame sprites can be resolved
    for sprite_name in ["frame1", "frame2", "frame3", "frame4"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Frame sprite '{sprite_name}' should resolve"));
    }

    // Verify animation exists and has correct frame count
    let anim = animations.get("color_cycle").expect("Animation 'color_cycle' not found");
    assert_eq!(anim.frames.len(), 4, "color_cycle should have 4 frames");

    // Verify frame order is preserved
    assert_eq!(anim.frames[0], "frame1");
    assert_eq!(anim.frames[1], "frame2");
    assert_eq!(anim.frames[2], "frame3");
    assert_eq!(anim.frames[3], "frame4");
}

/// @demo format/animation/basic#two_frame
/// @title Minimal Two-Frame Animation
/// @description The simplest animation: just two frames alternating.
#[test]
fn test_two_frame_animation() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_sequence.jsonl");
    assert_validates(jsonl, true);

    assert_frame_count(jsonl, "two_frame", 2);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("two_frame").expect("Animation 'two_frame' not found");

    assert_eq!(anim.frames[0], "frame1");
    assert_eq!(anim.frames[1], "frame2");
    assert_eq!(anim.duration_ms(), 200, "Duration should be 200ms");
}

/// @demo format/animation/basic#spritesheet
/// @title Animation as Spritesheet
/// @description Animation frames rendered as a horizontal spritesheet strip.
#[test]
fn test_animation_spritesheet() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_sequence.jsonl");
    assert_validates(jsonl, true);

    let info = capture_spritesheet_info(jsonl, "color_cycle", None);

    // 4 frames at 4x4 pixels each = 16x4 horizontal strip
    assert_eq!(info.frame_count, 4, "Should have 4 frames");
    assert_eq!(info.frame_width, 4, "Frame width should be 4");
    assert_eq!(info.frame_height, 4, "Frame height should be 4");
    assert_eq!(info.width, 16, "Spritesheet width should be 16 (4 frames * 4px)");
    assert_eq!(info.height, 4, "Spritesheet height should be 4");
}
