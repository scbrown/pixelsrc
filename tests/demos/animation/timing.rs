//! Animation timing demos
//!
//! FPS, duration, and timing control for animations.

use crate::demos::{assert_validates, capture_gif_info, parse_content};

/// @demo format/animation#fps
/// @title FPS Timing
/// @description Animation using fps field for frame rate control.
#[test]
fn test_animation_fps_timing() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // fast_blink uses duration: 50 (ms per frame)
    let fast = animations.get("fast_blink").expect("Animation 'fast_blink' not found");
    assert_eq!(fast.duration_ms(), 50, "fast_blink should have 50ms per frame");
}

/// @demo format/animation#duration_ms
/// @title Duration in Milliseconds
/// @description Animation timing using numeric duration (milliseconds).
#[test]
fn test_duration_numeric() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    let (_, _, animations) = parse_content(jsonl);

    // fast_blink: duration: 50 (numeric ms)
    let fast = animations.get("fast_blink").expect("Animation 'fast_blink' not found");
    assert_eq!(fast.duration_ms(), 50, "Numeric duration 50 should be 50ms");

    let info = capture_gif_info(jsonl, "fast_blink");
    assert_eq!(info.duration_ms, 50, "GIF frame duration should be 50ms");
}

/// @demo format/animation#duration_string_ms
/// @title Duration String (ms)
/// @description Animation timing using string duration with 'ms' suffix.
#[test]
fn test_duration_string_ms() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    let (_, _, animations) = parse_content(jsonl);

    // slow_pulse: duration: "500ms"
    let slow = animations.get("slow_pulse").expect("Animation 'slow_pulse' not found");
    assert_eq!(slow.duration_ms(), 500, "Duration '500ms' should be 500ms");

    let info = capture_gif_info(jsonl, "slow_pulse");
    assert_eq!(info.duration_ms, 500, "GIF frame duration should be 500ms");
}

/// @demo format/animation#duration_string_s
/// @title Duration String (seconds)
/// @description Animation timing using string duration with 's' suffix.
#[test]
fn test_duration_string_seconds() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    let (_, _, animations) = parse_content(jsonl);

    // one_second: duration: "1s"
    let one_sec = animations.get("one_second").expect("Animation 'one_second' not found");
    assert_eq!(one_sec.duration_ms(), 1000, "Duration '1s' should be 1000ms");

    let info = capture_gif_info(jsonl, "one_second");
    assert_eq!(info.duration_ms, 1000, "GIF frame duration should be 1000ms");
}

/// @demo format/animation#fast_vs_slow
/// @title Fast vs Slow Animations
/// @description Comparing frame rates across different timing configurations.
#[test]
fn test_timing_comparison() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    let fast_info = capture_gif_info(jsonl, "fast_blink");
    let slow_info = capture_gif_info(jsonl, "slow_pulse");
    let second_info = capture_gif_info(jsonl, "one_second");

    // Verify relative speeds
    assert!(
        fast_info.duration_ms < slow_info.duration_ms,
        "fast_blink ({}ms) should be faster than slow_pulse ({}ms)",
        fast_info.duration_ms,
        slow_info.duration_ms
    );

    assert!(
        slow_info.duration_ms < second_info.duration_ms,
        "slow_pulse ({}ms) should be faster than one_second ({}ms)",
        slow_info.duration_ms,
        second_info.duration_ms
    );
}

/// @demo format/animation#frame_counts
/// @title Animation Frame Counts
/// @description Verifying frame counts across timing demos.
#[test]
fn test_timing_frame_counts() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");

    let (_, _, animations) = parse_content(jsonl);

    // fast_blink: 2 frames
    let fast = animations.get("fast_blink").unwrap();
    assert_eq!(fast.frames.len(), 2, "fast_blink should have 2 frames");

    // slow_pulse: 3 frames
    let slow = animations.get("slow_pulse").unwrap();
    assert_eq!(slow.frames.len(), 3, "slow_pulse should have 3 frames");

    // one_second: 2 frames
    let one_sec = animations.get("one_second").unwrap();
    assert_eq!(one_sec.frames.len(), 2, "one_second should have 2 frames");
}
