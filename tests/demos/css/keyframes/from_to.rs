//! From/To Keyframes Demo Tests
//!
//! Tests for CSS @keyframes using from/to aliases (equivalent to 0%/100%).

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/keyframes#from_to
/// @title From/To Keyframes
/// @description Animation using from/to aliases (equivalent to 0%/100%).
#[test]
fn test_from_to_keyframes() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/from_to.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprite can be resolved
    sprite_registry
        .resolve("dot", &palette_registry, false)
        .expect("Sprite 'dot' should resolve");

    let anim = animations.get("fade_in").expect("Animation 'fade_in' not found");

    // Verify this is a keyframes-based animation
    assert!(anim.is_css_keyframes(), "Should use CSS keyframes format");

    // Check keyframe keys use from/to aliases
    let keyframes = anim.keyframes.as_ref().unwrap();
    assert_eq!(keyframes.len(), 2, "Should have 2 keyframes (from, to)");
    assert!(keyframes.contains_key("from"), "Should have 'from' keyframe");
    assert!(keyframes.contains_key("to"), "Should have 'to' keyframe");
}

/// @demo format/css/keyframes#from_alias
/// @title From Keyframe (0% Alias)
/// @description Verify 'from' keyframe represents initial state (0%).
#[test]
fn test_from_keyframe() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/from_to.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_in").expect("Animation 'fade_in' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Verify from keyframe (0% alias) - starts transparent
    let kf_from = &keyframes["from"];
    assert_eq!(kf_from.sprite.as_deref(), Some("dot"));
    assert_eq!(kf_from.opacity, Some(0.0));
}

/// @demo format/css/keyframes#to_alias
/// @title To Keyframe (100% Alias)
/// @description Verify 'to' keyframe represents final state (100%).
#[test]
fn test_to_keyframe() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/from_to.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_in").expect("Animation 'fade_in' not found");
    let keyframes = anim.keyframes.as_ref().unwrap();

    // Verify to keyframe (100% alias) - ends opaque
    let kf_to = &keyframes["to"];
    assert_eq!(kf_to.sprite.as_deref(), Some("dot"));
    assert_eq!(kf_to.opacity, Some(1.0));
}

/// @demo format/css/keyframes#from_to_duration
/// @title From/To Animation Duration
/// @description Verify duration parsing for from/to animations.
#[test]
fn test_from_to_duration() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/from_to.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("fade_in").expect("Animation 'fade_in' not found");

    // Verify duration is 1 second (1000ms)
    assert_eq!(anim.duration_ms(), 1000);
}
