//! Attachment chain demos
//!
//! Hierarchical sprite attachments that follow animation movement.

use crate::demos::{assert_validates, parse_content};
use pixelsrc::models::FollowMode;

/// @demo format/animation#attachments
/// @title Basic Attachment Chain
/// @description Animation with attached sprites that follow the main body.
#[test]
fn test_basic_attachment() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").expect("Animation 'walk_with_tail' not found");

    // Verify attachments exist
    let attachments = anim.attachments.as_ref().expect("Animation should have attachments");
    assert_eq!(attachments.len(), 1, "Should have 1 attachment");
}

/// @demo format/animation#attachment_name
/// @title Attachment Name
/// @description Attachments have a name for identification.
#[test]
fn test_attachment_name() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.name, "tail", "Attachment should be named 'tail'");
}

/// @demo format/animation#attachment_anchor
/// @title Attachment Anchor Point
/// @description Anchor specifies where attachment connects to parent.
#[test]
fn test_attachment_anchor() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.anchor, [1, 2], "Anchor should be at [1, 2]");
}

/// @demo format/animation#attachment_chain
/// @title Attachment Sprite Chain
/// @description Multiple sprites form a connected chain (e.g., tail segments).
#[test]
fn test_attachment_chain() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.chain.len(), 3, "Chain should have 3 segments");
    assert_eq!(tail.chain[0], "tail_seg1", "First segment");
    assert_eq!(tail.chain[1], "tail_seg2", "Second segment");
    assert_eq!(tail.chain[2], "tail_seg3", "Third segment (tip)");
}

/// @demo format/animation#attachment_delay
/// @title Attachment Follow Delay
/// @description Delay in frames before attachment follows parent movement.
#[test]
fn test_attachment_delay() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.delay, Some(2), "Delay should be 2 frames");
}

/// @demo format/animation#attachment_follow
/// @title Attachment Follow Mode
/// @description How attachment tracks parent: position, rotation, or both.
#[test]
fn test_attachment_follow() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(
        tail.follow,
        Some(FollowMode::Position),
        "Follow mode should be Position"
    );
}

/// @demo format/animation#attachment_physics
/// @title Attachment Physics Properties
/// @description Damping and stiffness control attachment movement smoothness.
#[test]
fn test_attachment_physics() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.damping, Some(0.7), "Damping should be 0.7");
    assert_eq!(tail.stiffness, Some(0.4), "Stiffness should be 0.4");
}

/// @demo format/animation#attachment_z_index
/// @title Attachment Z-Index
/// @description Z-index controls attachment render order relative to parent.
#[test]
fn test_attachment_z_index() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    let tail = &attachments[0];
    assert_eq!(tail.z_index, Some(-1), "Z-index should be -1 (behind parent)");
}

/// @demo format/animation#attachment_sprites
/// @title Attachment Chain Sprites
/// @description All sprites in attachment chain can be resolved.
#[test]
fn test_attachment_sprites_resolve() {
    let jsonl = include_str!("../../../examples/demos/animation/attachments.jsonl");

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);
    let anim = animations.get("walk_with_tail").unwrap();
    let attachments = anim.attachments.as_ref().unwrap();

    // Body frames
    for frame_name in &anim.frames {
        sprite_registry
            .resolve(frame_name, &palette_registry, false)
            .unwrap_or_else(|e| panic!("Body sprite '{frame_name}' should resolve: {e}"));
    }

    // Attachment chain sprites
    for segment in &attachments[0].chain {
        sprite_registry
            .resolve(segment, &palette_registry, false)
            .unwrap_or_else(|e| panic!("Chain segment '{segment}' should resolve: {e}"));
    }
}
