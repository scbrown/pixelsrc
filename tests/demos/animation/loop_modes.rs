//! Animation Loop Modes Demo Tests
//!
//! Tests for animation looping behavior: loop forever, play once, default behavior.

use crate::demos::{assert_validates, capture_gif_info, parse_content};

/// @demo format/animation/loop#forever
/// @title Loop Forever
/// @description Animation with loop: true plays continuously.
#[test]
fn test_loop_forever() {
    let jsonl = include_str!("../../../examples/demos/animation/loop_modes.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations
        .get("loop_forever")
        .expect("Animation 'loop_forever' not found");
    assert!(anim.loops(), "loop_forever should loop");
    assert_eq!(anim.frames.len(), 4);
}

/// @demo format/animation/loop#once
/// @title Play Once
/// @description Animation with loop: false plays once and stops on last frame.
#[test]
fn test_play_once() {
    let jsonl = include_str!("../../../examples/demos/animation/loop_modes.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations
        .get("play_once")
        .expect("Animation 'play_once' not found");
    assert!(!anim.loops(), "play_once should not loop");
    assert_eq!(anim.frames.len(), 4);
}

/// @demo format/animation/loop#default
/// @title Default Loop Behavior
/// @description Animation without explicit loop field defaults to looping.
#[test]
fn test_default_loop() {
    let jsonl = include_str!("../../../examples/demos/animation/loop_modes.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations
        .get("default_loop")
        .expect("Animation 'default_loop' not found");
    // Default behavior is to loop
    assert!(anim.loops(), "Animation without loop field should default to looping");
    assert_eq!(anim.frames.len(), 3);
}

/// @demo format/animation/loop#gif_loop
/// @title GIF Loop Behavior
/// @description GIF export respects loop setting - looping GIF vs single-play GIF.
#[test]
fn test_gif_loop_behavior() {
    let jsonl = include_str!("../../../examples/demos/animation/loop_modes.jsonl");

    // Looping animation
    let loop_info = capture_gif_info(jsonl, "loop_forever");
    assert!(loop_info.loops, "loop_forever GIF should loop");
    assert_eq!(loop_info.frame_count, 4);

    // Non-looping animation
    let once_info = capture_gif_info(jsonl, "play_once");
    assert!(!once_info.loops, "play_once GIF should not loop");
    assert_eq!(once_info.frame_count, 4);
}

/// @demo format/animation/loop#frame_progression
/// @title Frame Progression
/// @description Verify frames play in correct order for both loop modes.
#[test]
fn test_frame_progression() {
    let jsonl = include_str!("../../../examples/demos/animation/loop_modes.jsonl");
    let (_, _, animations) = parse_content(jsonl);

    // Both animations should have same frame order
    let looping = animations.get("loop_forever").unwrap();
    let once = animations.get("play_once").unwrap();

    assert_eq!(looping.frames, once.frames, "Frame order should be identical");
    assert_eq!(looping.frames[0], "l1");
    assert_eq!(looping.frames[1], "l2");
    assert_eq!(looping.frames[2], "l3");
    assert_eq!(looping.frames[3], "l4");
}
