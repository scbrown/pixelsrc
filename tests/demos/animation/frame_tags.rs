//! Frame tag demos
//!
//! Named animation ranges for complex animations with multiple states.

use crate::demos::{assert_frame_count, assert_validates, parse_content};

/// @demo format/animation#tags
/// @title Named Frame Tags
/// @description Animation with named sub-ranges (tags) for states like idle, attack, recovery.
#[test]
fn test_frame_tags() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");
    assert_validates(jsonl, true);

    // Animation has 6 frames total
    assert_frame_count(jsonl, "attack_combo", 6);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");

    // Verify tags exist
    let tags = anim.tags.as_ref().expect("Animation should have tags");
    assert_eq!(tags.len(), 4, "Should have 4 tags: idle, windup, attack, recovery");

    assert!(tags.contains_key("idle"), "Should have 'idle' tag");
    assert!(tags.contains_key("windup"), "Should have 'windup' tag");
    assert!(tags.contains_key("attack"), "Should have 'attack' tag");
    assert!(tags.contains_key("recovery"), "Should have 'recovery' tag");
}

/// @demo format/animation#tag_range
/// @title Tag Frame Ranges
/// @description Tags specify start/end frame indices for sub-animations.
#[test]
fn test_tag_ranges() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");
    let tags = anim.tags.as_ref().unwrap();

    // idle: frames 0-1
    let idle = &tags["idle"];
    assert_eq!(idle.start, 0, "idle should start at frame 0");
    assert_eq!(idle.end, 1, "idle should end at frame 1");

    // windup: frame 2 only
    let windup = &tags["windup"];
    assert_eq!(windup.start, 2, "windup should start at frame 2");
    assert_eq!(windup.end, 2, "windup should end at frame 2");

    // attack: frames 3-4
    let attack = &tags["attack"];
    assert_eq!(attack.start, 3, "attack should start at frame 3");
    assert_eq!(attack.end, 4, "attack should end at frame 4");

    // recovery: frame 5 only
    let recovery = &tags["recovery"];
    assert_eq!(recovery.start, 5, "recovery should start at frame 5");
    assert_eq!(recovery.end, 5, "recovery should end at frame 5");
}

/// @demo format/animation#tag_loop
/// @title Tag Loop Property
/// @description Individual tags can specify whether they loop.
#[test]
fn test_tag_loop() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");
    let tags = anim.tags.as_ref().unwrap();

    // idle tag has loop: true
    let idle = &tags["idle"];
    assert_eq!(idle.r#loop, Some(true), "idle tag should loop");

    // Other tags don't specify loop (default behavior)
    let windup = &tags["windup"];
    assert_eq!(windup.r#loop, None, "windup tag should not specify loop");
}

/// @demo format/animation#tag_fps
/// @title Tag FPS Override
/// @description Individual tags can override animation FPS.
#[test]
fn test_tag_fps() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");
    let tags = anim.tags.as_ref().unwrap();

    // attack tag has fps: 12 (faster for action)
    let attack = &tags["attack"];
    assert_eq!(attack.fps, Some(12), "attack tag should have fps: 12");

    // Other tags inherit from animation
    let idle = &tags["idle"];
    assert_eq!(idle.fps, None, "idle tag should not override fps");
}

/// @demo format/animation#tag_sprites
/// @title Tag Frame Sprites
/// @description All sprites referenced by tag ranges can be resolved.
#[test]
fn test_tag_sprites_resolve() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_tags.jsonl");

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);
    let anim = animations.get("attack_combo").expect("Animation 'attack_combo' not found");

    // All 6 frames should resolve
    for frame_name in &anim.frames {
        sprite_registry
            .resolve(frame_name, &palette_registry, false)
            .unwrap_or_else(|e| panic!("Frame sprite '{frame_name}' should resolve: {e}"));
    }
}
