//! Percentage Keyframes Demo Tests
//!
//! Tests for CSS @keyframes using percentage notation (0%, 50%, 100%).

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/keyframes#percentage
/// @title Percentage Keyframes
/// @description Animation using 0%, 50%, 100% keyframes with opacity and sprite changes.
#[test]
fn test_percentage_keyframes() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/percentage.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    for sprite_name in ["walk_1", "walk_2"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Sprite '{sprite_name}' should resolve"));
    }

    let anim = animations.get("fade_walk").expect("Animation 'fade_walk' not found");

    // Verify this is a keyframes-based animation
    assert!(anim.is_css_keyframes(), "Should use CSS keyframes format");

    // Check keyframe count
    let keyframes = anim.keyframes.as_ref().unwrap();
    assert_eq!(keyframes.len(), 3, "Should have 3 keyframes (0%, 50%, 100%)");

    // Verify keyframe keys
    assert!(keyframes.contains_key("0%"), "Should have 0% keyframe");
    assert!(keyframes.contains_key("50%"), "Should have 50% keyframe");
    assert!(keyframes.contains_key("100%"), "Should have 100% keyframe");
}

/// @demo format/css/keyframes#percentage_sprite
/// @title Sprite Changes at Percentage Keyframes
/// @description Verify sprite property changes at each percentage keyframe.
#[test]
fn test_percentage_sprite_changes() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/percentage.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_walk").expect("Animation 'fade_walk' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Verify sprite changes at each keyframe
    assert_eq!(keyframes["0%"].sprite.as_deref(), Some("walk_1"));
    assert_eq!(keyframes["50%"].sprite.as_deref(), Some("walk_2"));
    assert_eq!(keyframes["100%"].sprite.as_deref(), Some("walk_1"));
}

/// @demo format/css/keyframes#percentage_opacity
/// @title Opacity Changes at Percentage Keyframes
/// @description Verify opacity property changes at each percentage keyframe.
#[test]
fn test_percentage_opacity_changes() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/percentage.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_walk").expect("Animation 'fade_walk' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Verify opacity values at each keyframe
    assert_eq!(keyframes["0%"].opacity, Some(0.0));
    assert_eq!(keyframes["50%"].opacity, Some(1.0));
    assert_eq!(keyframes["100%"].opacity, Some(0.0));
}

/// @demo format/css/keyframes#percentage_timing
/// @title Timing Function with Percentage Keyframes
/// @description Verify timing function is applied correctly to percentage keyframes.
#[test]
fn test_percentage_timing_function() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/percentage.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_walk").expect("Animation 'fade_walk' not found");

    // Verify timing function
    assert_eq!(anim.timing_function.as_deref(), Some("ease-in-out"));

    // Verify duration
    assert_eq!(anim.duration_ms(), 1000);
}
