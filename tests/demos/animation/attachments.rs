//! Attachments Demo Tests

use crate::demos::{assert_frame_count, assert_validates, capture_render_info};
use pixelsrc::models::{FollowMode, TtpObject};
use pixelsrc::parser::parse_stream;
use std::io::Cursor;

/// @demo format/animation#attachments
/// @title Attachment Chains
/// @description Secondary motion using attachment chains (tails, capes, hair).
#[test]
fn test_attachments_basic() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");
    assert_validates(jsonl, true);

    // Verify animation has 2 frames
    assert_frame_count(jsonl, "walk_with_tail", 2);

    // Verify body frames can be rendered
    for frame_name in ["body1", "body2"] {
        let info = capture_render_info(jsonl, frame_name);
        assert_eq!(info.width, 3, "Body frame '{}' should be 3 pixels wide", frame_name);
        assert_eq!(info.height, 3, "Body frame '{}' should be 3 pixels tall", frame_name);
    }
}

/// @demo format/animation#attachments_chain
/// @title Attachment Chain Segments
/// @description Attachment chains consist of multiple sprite segments.
#[test]
fn test_attachments_chain_segments() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "walk_with_tail" => Some(a),
            _ => None,
        })
        .expect("Animation 'walk_with_tail' not found");

    let attachments = animation.attachments.as_ref().expect("Should have attachments");
    assert_eq!(attachments.len(), 1, "Should have 1 attachment");

    let tail = &attachments[0];
    assert_eq!(tail.name, "tail", "Attachment should be named 'tail'");
    assert_eq!(tail.chain.len(), 3, "Tail chain should have 3 segments");
    assert_eq!(tail.chain, vec!["tail_seg1", "tail_seg2", "tail_seg3"]);

    // Verify tail segment sprites can be rendered
    for seg_name in &tail.chain {
        let info = capture_render_info(jsonl, seg_name);
        assert!(info.width > 0, "Tail segment '{}' should render", seg_name);
    }
}

/// @demo format/animation#attachments_properties
/// @title Attachment Physics Properties
/// @description Attachments support physics simulation properties.
#[test]
fn test_attachments_properties() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let animation = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Animation(a) if a.name == "walk_with_tail" => Some(a),
            _ => None,
        })
        .expect("Animation 'walk_with_tail' not found");

    let attachments = animation.attachments.as_ref().unwrap();
    let tail = &attachments[0];

    // Verify anchor point
    assert_eq!(tail.anchor, [1, 2], "Anchor should be at [1, 2]");

    // Verify physics properties
    assert_eq!(tail.delay, Some(2), "Delay should be 2 frames");
    assert_eq!(tail.follow, Some(FollowMode::Position), "Follow mode should be 'position'");
    assert!((tail.damping.unwrap() - 0.7).abs() < 0.01, "Damping should be 0.7");
    assert!((tail.stiffness.unwrap() - 0.4).abs() < 0.01, "Stiffness should be 0.4");
    assert_eq!(tail.z_index, Some(-1), "Z-index should be -1");
}

/// @demo format/animation#attachments_metadata
/// @title Attachment Point Metadata
/// @description Sprites can define attach_in and attach_out points.
#[test]
fn test_attachments_metadata() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    // Find tail segment sprites and verify metadata
    let tail_seg1 = parse_result
        .objects
        .iter()
        .find_map(|obj| match obj {
            TtpObject::Sprite(s) if s.name == "tail_seg1" => Some(s),
            _ => None,
        })
        .expect("Sprite 'tail_seg1' not found");

    let metadata = tail_seg1.metadata.as_ref().expect("tail_seg1 should have metadata");
    assert!(metadata.attach_in.is_some(), "Should have attach_in");
    assert!(metadata.attach_out.is_some(), "Should have attach_out");
}
