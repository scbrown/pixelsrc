//! Basic animation frame demos
//!
//! Simple frame sequences demonstrating core animation structure.

use crate::demos::{
    assert_frame_count, assert_validates, capture_gif_info, capture_spritesheet_info, parse_content,
};

/// @demo format/animation#basic
/// @title Basic Frame Sequence
/// @description Simple animation with frames defined as sprite references.
#[test]
fn test_basic_animation() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");
    assert_validates(jsonl, true);

    // Verify animation has 3 frames
    assert_frame_count(jsonl, "blink", 3);

    // Verify frame names are correct
    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("blink").expect("Animation 'blink' not found");

    assert_eq!(anim.frames.len(), 3, "Should have 3 frames");
    assert_eq!(anim.frames[0], "frame1", "First frame should be 'frame1'");
    assert_eq!(anim.frames[1], "frame2", "Second frame should be 'frame2'");
    assert_eq!(anim.frames[2], "frame3", "Third frame should be 'frame3'");
}

/// @demo format/animation#duration
/// @title Animation Default Duration
/// @description Animation uses default duration when not specified.
#[test]
fn test_animation_duration() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("blink").expect("Animation 'blink' not found");

    // Animation uses default duration of 100ms per frame
    // Note: fps field in fixture is ignored; use duration field for timing control
    assert_eq!(
        anim.duration_ms(),
        100,
        "Default duration should be 100ms per frame"
    );
}

/// @demo format/animation#sprites
/// @title Animation Sprite Resolution
/// @description All frame sprites can be resolved from the same file.
#[test]
fn test_animation_sprites_resolve() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);
    let anim = animations.get("blink").expect("Animation 'blink' not found");

    // Verify all frame sprites can be resolved
    for frame_name in &anim.frames {
        sprite_registry
            .resolve(frame_name, &palette_registry, false)
            .unwrap_or_else(|e| panic!("Frame sprite '{frame_name}' should resolve: {e}"));
    }
}

/// @demo format/animation#spritesheet
/// @title Animation as Spritesheet
/// @description Animation frames rendered as horizontal spritesheet.
#[test]
fn test_animation_spritesheet() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");

    // Each frame is 3x3, horizontal strip = 9x3
    let info = capture_spritesheet_info(jsonl, "blink", None);
    assert_eq!(info.width, 9, "3 frames x 3px = 9px wide");
    assert_eq!(info.height, 3, "Single row should be 3px tall");
    assert_eq!(info.frame_count, 3);
    assert_eq!(info.frame_width, 3);
    assert_eq!(info.frame_height, 3);
}

/// @demo format/animation#gif
/// @title Animation as GIF
/// @description Animation rendered as animated GIF with correct frame count.
#[test]
fn test_animation_gif() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");

    let info = capture_gif_info(jsonl, "blink");
    assert_eq!(info.frame_count, 3, "GIF should have 3 frames");
    assert_eq!(info.frame_width, 3, "Frame width should be 3px");
    assert_eq!(info.frame_height, 3, "Frame height should be 3px");
    assert!(info.loops, "Animation should loop by default");
    assert_eq!(info.duration_ms, 100, "Frame duration should be 100ms (default)");
}
