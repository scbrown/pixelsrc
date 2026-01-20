//! Looping mode demos
//!
//! Animation loop control and playback modes.

use crate::demos::{assert_validates, capture_gif_info, parse_content};

/// @demo format/animation#looping
/// @title Default Looping Behavior
/// @description Animations loop by default when loop field is not specified.
#[test]
fn test_default_looping() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("blink").expect("Animation 'blink' not found");

    // Default is loop: true
    assert!(anim.loops(), "Animation should loop by default");

    let info = capture_gif_info(jsonl, "blink");
    assert!(info.loops, "GIF should loop");
}

/// @demo format/animation#loop_true
/// @title Explicit Loop True
/// @description Animation with explicit loop: true for continuous playback.
#[test]
fn test_explicit_loop_true() {
    // Create inline test with explicit loop: true
    let jsonl = r##"{"type": "sprite", "name": "f1", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}
{"type": "sprite", "name": "f2", "palette": {"{_}": "#00000000", "{x}": "#00FF00"}, "grid": ["{x}"]}
{"type": "animation", "name": "forever", "frames": ["f1", "f2"], "loop": true, "duration": 100}"##;

    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("forever").expect("Animation 'forever' not found");

    assert!(anim.loops(), "Animation with loop: true should loop");

    let info = capture_gif_info(jsonl, "forever");
    assert!(info.loops, "GIF should loop");
}

/// @demo format/animation#loop_false
/// @title Loop False (Play Once)
/// @description Animation with loop: false for single playback.
#[test]
fn test_loop_false() {
    // Create inline test with loop: false
    let jsonl = r##"{"type": "sprite", "name": "f1", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}
{"type": "sprite", "name": "f2", "palette": {"{_}": "#00000000", "{x}": "#00FF00"}, "grid": ["{x}"]}
{"type": "animation", "name": "once", "frames": ["f1", "f2"], "loop": false, "duration": 100}"##;

    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("once").expect("Animation 'once' not found");

    assert!(!anim.loops(), "Animation with loop: false should not loop");

    let info = capture_gif_info(jsonl, "once");
    assert!(!info.loops, "GIF should not loop");
}

/// @demo format/animation#tag_loop
/// @title Tag-Level Looping
/// @description Individual tags can specify their own loop behavior.
#[test]
fn test_tag_looping() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");

    let tags = anim.tags.as_ref().expect("Animation should have tags");

    // idle tag has loop: true (looping idle state)
    let idle = &tags["idle"];
    assert_eq!(idle.r#loop, Some(true), "idle tag should have loop: true");

    // attack tag has no loop specified (plays once by default)
    let attack = &tags["attack"];
    assert_eq!(attack.r#loop, None, "attack tag should not specify loop");
}

/// @demo format/animation#loop_timing
/// @title Looping with Timing
/// @description Loop behavior combined with timing configuration.
#[test]
fn test_loop_with_timing() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    // All timing demos should loop by default
    let fast_info = capture_gif_info(jsonl, "fast_blink");
    assert!(fast_info.loops, "fast_blink should loop");

    let slow_info = capture_gif_info(jsonl, "slow_pulse");
    assert!(slow_info.loops, "slow_pulse should loop");

    let second_info = capture_gif_info(jsonl, "one_second");
    assert!(second_info.loops, "one_second should loop");
}
