//! Looping Demo Tests

use crate::demos::{assert_frame_count, assert_validates};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::io::Cursor;

/// @demo format/animation#looping
/// @title Default Loop Behavior
/// @description Animations loop by default when loop property is not specified.
#[test]
fn test_looping_default() {
    let jsonl = include_str!("../../../examples/demos/animation/looping.jsonl");
    assert_validates(jsonl, true);

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "looping_default" => Some(a),
            _ => None,
        })
        .expect("Animation 'looping_default' not found");

    // Default should loop (true when not specified)
    assert!(animation.loops(), "Default animation should loop");
    assert!(animation.r#loop.is_none(), "loop property should not be set");
    assert_frame_count(jsonl, "looping_default", 2);
}

/// @demo format/animation#looping_true
/// @title Explicit Loop True
/// @description Animation with explicit loop: true property.
#[test]
fn test_looping_explicit_true() {
    let jsonl = include_str!("../../../examples/demos/animation/looping.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "looping_true" => Some(a),
            _ => None,
        })
        .expect("Animation 'looping_true' not found");

    assert!(animation.loops(), "Animation with loop: true should loop");
    assert_eq!(animation.r#loop, Some(true), "loop property should be Some(true)");
    assert_frame_count(jsonl, "looping_true", 2);
}

/// @demo format/animation#looping_false
/// @title One-Shot Animation
/// @description Animation with loop: false plays once and stops.
#[test]
fn test_looping_false() {
    let jsonl = include_str!("../../../examples/demos/animation/looping.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "looping_false" => Some(a),
            _ => None,
        })
        .expect("Animation 'looping_false' not found");

    assert!(!animation.loops(), "Animation with loop: false should not loop");
    assert_eq!(animation.r#loop, Some(false), "loop property should be Some(false)");
    assert_frame_count(jsonl, "looping_false", 2);
}
