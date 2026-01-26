//! Animation Frame Metadata Demo Tests
//!
//! Tests for animation and frame-level metadata: origins, hitboxes, frame tags.

use crate::demos::{assert_validates, parse_content};

/// @demo format/animation/metadata#sprite_origin
/// @title Sprite Origin Points
/// @description Per-sprite origin metadata for positioning and rotation anchors.
#[test]
fn test_sprite_origins() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_metadata.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Get idle frame and check origin
    let idle_1 = sprite_registry.get_sprite("idle_1").expect("Sprite 'idle_1' not found");
    assert!(idle_1.metadata.is_some(), "idle_1 should have metadata");

    let meta = idle_1.metadata.as_ref().unwrap();
    assert!(meta.origin.is_some(), "idle_1 should have origin");

    let origin = meta.origin.unwrap();
    assert_eq!(origin[0], 4, "origin x should be 4");
    assert_eq!(origin[1], 8, "origin y should be 8 (foot position)");

    // Verify sprite can still be resolved with metadata
    sprite_registry.resolve("idle_1", &palette_registry, false).expect("idle_1 should resolve");
}

/// @demo format/animation/metadata#hitbox
/// @title Frame Hitbox/Collision Boxes
/// @description Per-sprite collision box metadata for gameplay integration.
#[test]
fn test_sprite_hitbox() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_metadata.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    // Get walk frame with hitbox
    let walk_1 = sprite_registry.get_sprite("walk_1").expect("Sprite 'walk_1' not found");
    assert!(walk_1.metadata.is_some(), "walk_1 should have metadata");

    let meta = walk_1.metadata.as_ref().unwrap();

    // Check origin
    let origin = meta.origin.expect("walk_1 should have origin");
    assert_eq!(origin[0], 4, "origin x should be 4");
    assert_eq!(origin[1], 8, "origin y should be 8");

    // Check hitbox
    let boxes = meta.boxes.as_ref().expect("walk_1 should have boxes");
    let hitbox = boxes.get("hitbox").expect("walk_1 should have hitbox");

    // Hitbox has x, y, w, h fields
    assert_eq!(hitbox.x, 1, "hitbox x should be 1");
    assert_eq!(hitbox.y, 0, "hitbox y should be 0");
    assert_eq!(hitbox.w, 6, "hitbox width should be 6");
    assert_eq!(hitbox.h, 8, "hitbox height should be 8");
}

/// @demo format/animation/metadata#frame_tags
/// @title Animation Frame Tags
/// @description Named frame ranges for game engine integration (idle, walk, attack phases).
#[test]
fn test_animation_frame_tags() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_metadata.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // Test idle animation tags
    let idle = animations.get("char_idle").expect("Animation 'char_idle' not found");
    assert!(idle.tags.is_some(), "char_idle should have tags");

    let idle_tags = idle.tags.as_ref().unwrap();
    let idle_tag = idle_tags.get("idle").expect("Should have 'idle' tag");
    assert_eq!(idle_tag.start, 0, "idle tag should start at frame 0");
    assert_eq!(idle_tag.end, 1, "idle tag should end at frame 1");

    // Test walk animation tags
    let walk = animations.get("char_walk").expect("Animation 'char_walk' not found");
    assert!(walk.tags.is_some(), "char_walk should have tags");

    let walk_tags = walk.tags.as_ref().unwrap();
    let walk_tag = walk_tags.get("walk").expect("Should have 'walk' tag");
    assert_eq!(walk_tag.start, 0, "walk tag should start at frame 0");
    assert_eq!(walk_tag.end, 1, "walk tag should end at frame 1");
}

/// @demo format/animation/metadata#state_timing
/// @title State-Based Timing Differences
/// @description Different animation states have different timing (idle=slow, walk=fast).
#[test]
fn test_state_timing_differences() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_metadata.jsonl");
    let (_, _, animations) = parse_content(jsonl);

    let idle = animations.get("char_idle").unwrap();
    let walk = animations.get("char_walk").unwrap();

    // Idle is slower (500ms per frame)
    // Walk is faster (150ms per frame)
    assert!(
        idle.duration_ms() > walk.duration_ms(),
        "Idle ({}) should be slower than walk ({})",
        idle.duration_ms(),
        walk.duration_ms()
    );

    assert_eq!(idle.duration_ms(), 500, "Idle should be 500ms per frame");
    assert_eq!(walk.duration_ms(), 150, "Walk should be 150ms per frame");
}

/// @demo format/animation/metadata#consistent_origins
/// @title Consistent Origins Across Frames
/// @description All frames in an animation should have matching origin points.
#[test]
fn test_consistent_origins_across_frames() {
    let jsonl = include_str!("../../../examples/demos/animation/frame_metadata.jsonl");
    let (_, sprite_registry, animations) = parse_content(jsonl);

    // Verify idle animation frames have consistent origins
    let idle = animations.get("char_idle").unwrap();
    let mut expected_origin: Option<[i32; 2]> = None;

    for frame_name in &idle.frames {
        let sprite = sprite_registry
            .get_sprite(frame_name)
            .unwrap_or_else(|| panic!("Frame sprite '{frame_name}' not found"));

        let origin = sprite
            .metadata
            .as_ref()
            .and_then(|m| m.origin)
            .unwrap_or_else(|| panic!("Frame '{frame_name}' should have origin"));

        match expected_origin {
            None => expected_origin = Some(origin),
            Some(expected) => {
                assert_eq!(
                    origin, expected,
                    "Frame '{}' origin {:?} doesn't match expected {:?}",
                    frame_name, origin, expected
                );
            }
        }
    }
}
