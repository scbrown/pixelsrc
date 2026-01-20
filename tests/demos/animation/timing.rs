//! Timing Demo Tests

use crate::demos::{assert_frame_count, assert_validates};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::io::Cursor;

/// @demo format/animation#fps
/// @title FPS Timing
/// @description Animation timing using frames-per-second.
#[test]
fn test_timing_fps() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");
    assert_validates(jsonl, true);

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let _animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "blink" => Some(a),
            _ => None,
        })
        .expect("Animation 'blink' not found");

    // blink animation uses fps: 4, which translates to 250ms duration per frame
    // FPS is converted to duration at parse time
    assert_frame_count(jsonl, "blink", 3);
}

/// @demo format/animation#duration_ms
/// @title Duration in Milliseconds
/// @description Animation timing using duration in milliseconds.
#[test]
fn test_timing_duration_ms() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "fast_blink" => Some(a),
            _ => None,
        })
        .expect("Animation 'fast_blink' not found");

    // fast_blink has duration: 50 (raw milliseconds)
    assert_eq!(animation.duration_ms(), 50, "fast_blink should have 50ms duration");
    assert_frame_count(jsonl, "fast_blink", 2);
}

/// @demo format/animation#duration_string
/// @title Duration Strings
/// @description Animation timing using CSS-style duration strings (e.g., "500ms", "1s").
#[test]
fn test_timing_duration_string() {
    let jsonl = include_str!("../../../examples/demos/animation/timing.jsonl");
    assert_validates(jsonl, true);

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    // Test "500ms" format
    let slow_pulse = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "slow_pulse" => Some(a),
            _ => None,
        })
        .expect("Animation 'slow_pulse' not found");

    assert_eq!(slow_pulse.duration_ms(), 500, "slow_pulse should have 500ms duration");
    assert_frame_count(jsonl, "slow_pulse", 3);

    // Test "1s" format
    let one_second = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "one_second" => Some(a),
            _ => None,
        })
        .expect("Animation 'one_second' not found");

    assert_eq!(one_second.duration_ms(), 1000, "one_second should have 1000ms duration");
    assert_frame_count(jsonl, "one_second", 2);
}
