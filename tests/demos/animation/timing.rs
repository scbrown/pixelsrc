//! Animation Timing Demo Tests
//!
//! Tests for frame timing: duration in milliseconds and string formats.

use crate::demos::{assert_validates, parse_content};

/// @demo format/animation/timing#fast
/// @title Fast Animation (50ms frames)
/// @description Animation with short frame duration for rapid cycling.
#[test]
fn test_fast_timing() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations.get("fast_blink").expect("Animation 'fast_blink' not found");
    assert_eq!(anim.frames.len(), 2, "fast_blink should have 2 frames");
    assert_eq!(anim.duration_ms(), 50, "Frame duration should be 50ms");
    assert!(anim.loops(), "fast_blink should loop");
}

/// @demo format/animation/timing#slow
/// @title Slow Animation (500ms frames)
/// @description Animation with longer frame duration for slower, more deliberate movement.
#[test]
fn test_slow_timing() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations.get("slow_blink").expect("Animation 'slow_blink' not found");
    assert_eq!(anim.frames.len(), 2, "slow_blink should have 2 frames");
    assert_eq!(anim.duration_ms(), 500, "Frame duration should be 500ms");
}

/// @demo format/animation/timing#duration_string_ms
/// @title Duration as Milliseconds String (250ms)
/// @description Animation duration specified as "250ms" string format.
#[test]
fn test_duration_string_ms() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations.get("duration_ms").expect("Animation 'duration_ms' not found");
    assert_eq!(anim.duration_ms(), 250, "Duration '250ms' should parse to 250");
}

/// @demo format/animation/timing#duration_string_s
/// @title Duration as Seconds String (1s)
/// @description Animation duration specified as "1s" string format (converted to 1000ms).
#[test]
fn test_duration_string_seconds() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations.get("duration_1s").expect("Animation 'duration_1s' not found");
    assert_eq!(anim.frames.len(), 4, "duration_1s should have 4 frames");
    assert_eq!(anim.duration_ms(), 1000, "Duration '1s' should parse to 1000ms");
}

/// @demo format/animation/timing#default
/// @title Default Duration (100ms)
/// @description Animation without explicit duration defaults to 100ms per frame.
#[test]
fn test_default_duration() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let anim = animations.get("default_duration").expect("Animation 'default_duration' not found");
    assert_eq!(
        anim.duration_ms(),
        100,
        "Default duration should be 100ms (Animation::DEFAULT_DURATION_MS)"
    );
}

/// @demo format/animation/timing#compare
/// @title Timing Comparison
/// @description Verify different timing values produce expected ordering.
#[test]
fn test_timing_comparison() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    let (_, _, animations) = parse_content(jsonl);

    // Fast (50ms) should be faster than slow (500ms)
    let fast = animations.get("fast_blink").unwrap();
    let slow = animations.get("slow_blink").unwrap();
    assert!(
        fast.duration_ms() < slow.duration_ms(),
        "fast_blink ({}) should be faster than slow_blink ({})",
        fast.duration_ms(),
        slow.duration_ms()
    );

    // Default (100ms) should be between fast (50ms) and slow (500ms)
    let default = animations.get("default_duration").unwrap();
    assert!(
        default.duration_ms() > fast.duration_ms(),
        "default ({}) should be slower than fast ({})",
        default.duration_ms(),
        fast.duration_ms()
    );
    assert!(
        default.duration_ms() < slow.duration_ms(),
        "default ({}) should be faster than slow ({})",
        default.duration_ms(),
        slow.duration_ms()
    );
}
