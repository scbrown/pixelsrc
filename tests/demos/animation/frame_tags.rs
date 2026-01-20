//! Frame Tags Demo Tests

use crate::demos::{assert_frame_count, assert_validates, capture_render_info};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::io::Cursor;

/// @demo format/animation#tags
/// @title Named Animation Ranges
/// @description Animation with frame tags for game engine integration.
#[test]
fn test_frame_tags() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");
    assert_validates(jsonl, true);

    // Verify animation has 6 frames total
    assert_frame_count(jsonl, "attack_combo", 6);

    // Parse and verify tags exist
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "attack_combo" => Some(a),
            _ => None,
        })
        .expect("Animation 'attack_combo' not found");

    let tags = animation.tags.as_ref().expect("Animation should have tags");

    // Verify all expected tags exist
    assert!(tags.contains_key("idle"), "Should have 'idle' tag");
    assert!(tags.contains_key("windup"), "Should have 'windup' tag");
    assert!(tags.contains_key("attack"), "Should have 'attack' tag");
    assert!(tags.contains_key("recovery"), "Should have 'recovery' tag");
}

/// @demo format/animation#tags_ranges
/// @title Tag Frame Ranges
/// @description Each tag defines a start and end frame index.
#[test]
fn test_frame_tags_ranges() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "attack_combo" => Some(a),
            _ => None,
        })
        .expect("Animation 'attack_combo' not found");

    let tags = animation.tags.as_ref().unwrap();

    // Verify idle tag range
    let idle = &tags["idle"];
    assert_eq!(idle.start, 0, "idle should start at frame 0");
    assert_eq!(idle.end, 1, "idle should end at frame 1");

    // Verify attack tag range
    let attack = &tags["attack"];
    assert_eq!(attack.start, 3, "attack should start at frame 3");
    assert_eq!(attack.end, 4, "attack should end at frame 4");
}

/// @demo format/animation#tags_properties
/// @title Tag Properties
/// @description Tags can specify loop behavior and custom FPS.
#[test]
fn test_frame_tags_properties() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "attack_combo" => Some(a),
            _ => None,
        })
        .expect("Animation 'attack_combo' not found");

    let tags = animation.tags.as_ref().unwrap();

    // Verify idle tag has loop=true
    let idle = &tags["idle"];
    assert_eq!(idle.r#loop, Some(true), "idle tag should loop");

    // Verify attack tag has custom FPS
    let attack = &tags["attack"];
    assert_eq!(attack.fps, Some(12), "attack tag should have fps=12");

    // Verify frame sprites can be rendered
    for sprite_name in ["idle1", "idle2", "windup", "attack1", "attack2", "recover"] {
        let info = capture_render_info(jsonl, sprite_name);
        assert!(info.width > 0, "Sprite '{}' should render", sprite_name);
    }
}
