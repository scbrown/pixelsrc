//! Sprite Changes Keyframes Demo Tests
//!
//! Tests for CSS @keyframes that change sprites at different keyframe percentages.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/keyframes#sprite_changes
/// @title Sprite Changes at Keyframes
/// @description Animation that changes sprites at different keyframes (idle -> jump -> land -> idle).
#[test]
fn test_sprite_changes() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/sprite_changes.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);
    let anim = animations.get("jump_cycle").expect("Animation 'jump_cycle' not found");

    // Verify this is a keyframes-based animation
    assert!(anim.is_css_keyframes(), "Should use CSS keyframes format");

    // Check all keyframes exist
    let keyframes = anim.keyframes.as_ref().unwrap();
    assert_eq!(keyframes.len(), 4, "Should have 4 keyframes (0%, 25%, 75%, 100%)");

    // Verify all referenced sprites can be resolved
    for sprite_name in ["char_idle", "char_jump", "char_land"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Sprite '{sprite_name}' should resolve"));
    }
}

/// @demo format/css/keyframes#sprite_sequence
/// @title Sprite Sequence at Keyframes
/// @description Verify the correct sprite is displayed at each keyframe position.
#[test]
fn test_sprite_sequence() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/sprite_changes.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("jump_cycle").expect("Animation 'jump_cycle' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Verify sprite changes at each keyframe
    assert_eq!(keyframes["0%"].sprite.as_deref(), Some("char_idle"), "0%: should be idle");
    assert_eq!(keyframes["25%"].sprite.as_deref(), Some("char_jump"), "25%: should be jump");
    assert_eq!(keyframes["75%"].sprite.as_deref(), Some("char_land"), "75%: should be land");
    assert_eq!(keyframes["100%"].sprite.as_deref(), Some("char_idle"), "100%: should be idle");
}

/// @demo format/css/keyframes#sprite_cycle_duration
/// @title Sprite Cycle Duration
/// @description Verify total animation duration for sprite-change animations.
#[test]
fn test_sprite_cycle_duration() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/sprite_changes.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("jump_cycle").expect("Animation 'jump_cycle' not found");

    // Verify duration is 800ms
    assert_eq!(anim.duration_ms(), 800);
}

/// @demo format/css/keyframes#sprite_resolution
/// @title Sprite Resolution in Keyframes
/// @description All sprites referenced in keyframes must resolve correctly.
#[test]
fn test_all_sprites_resolve() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/sprite_changes.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);
    let anim = animations.get("jump_cycle").expect("Animation 'jump_cycle' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Collect all unique sprites from keyframes
    let sprites: Vec<&str> = keyframes.values().filter_map(|kf| kf.sprite.as_deref()).collect();

    // Verify each sprite resolves
    for sprite_name in sprites {
        let result = sprite_registry.resolve(sprite_name, &palette_registry, false);
        assert!(result.is_ok(), "Sprite '{sprite_name}' should resolve: {:?}", result.err());
    }
}
