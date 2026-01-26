//! Flip Transform Demo Tests
//!
//! Tests for flipping sprites using scaleX(-1) and scaleY(-1).

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/transforms#flip
/// @title Flip Transform
/// @description Flipping sprites using scaleX(-1) and scaleY(-1).
#[test]
fn test_css_transforms_flip() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/flip.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    sprite_registry
        .resolve("face_right", &palette_registry, false)
        .expect("Sprite 'face_right' should resolve");
    sprite_registry
        .resolve("arrow_left", &palette_registry, false)
        .expect("Sprite 'arrow_left' should resolve");

    // Test flip_horizontal animation
    let flip_h = animations.get("flip_horizontal").expect("Animation 'flip_horizontal' not found");
    assert!(flip_h.is_css_keyframes(), "flip_horizontal should use CSS keyframes");
    let kf = flip_h.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleX(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleX(-1)"));

    // Test flip_vertical animation
    let flip_v = animations.get("flip_vertical").expect("Animation 'flip_vertical' not found");
    let kf = flip_v.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleY(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleY(-1)"));

    // Test flip_both animation
    let flip_both = animations.get("flip_both").expect("Animation 'flip_both' not found");
    let kf = flip_both.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 3, "flip_both should have 3 keyframes");
    assert_eq!(kf["50%"].transform.as_deref(), Some("scale(-1, 1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(-1, -1)"));
}

/// @demo format/css/transforms#flip_horizontal
/// @title Horizontal Flip
/// @description Mirror sprite horizontally using scaleX(-1).
#[test]
fn test_flip_horizontal() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/flip.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let flip_h = animations.get("flip_horizontal").expect("Animation 'flip_horizontal' not found");
    let kf = flip_h.keyframes.as_ref().unwrap();

    // Verify horizontal flip (mirror along Y axis)
    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleX(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleX(-1)"));
}

/// @demo format/css/transforms#flip_vertical
/// @title Vertical Flip
/// @description Mirror sprite vertically using scaleY(-1).
#[test]
fn test_flip_vertical() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/flip.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let flip_v = animations.get("flip_vertical").expect("Animation 'flip_vertical' not found");
    let kf = flip_v.keyframes.as_ref().unwrap();

    // Verify vertical flip (mirror along X axis)
    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleY(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleY(-1)"));
}

/// @demo format/css/transforms#flip_both
/// @title Flip Both Axes
/// @description Flip horizontally then vertically using scale(-1, -1).
#[test]
fn test_flip_both_axes() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/flip.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let flip_both = animations.get("flip_both").expect("Animation 'flip_both' not found");
    let kf = flip_both.keyframes.as_ref().unwrap();

    // Verify flip sequence: normal -> flip X -> flip both
    assert_eq!(kf["0%"].transform.as_deref(), Some("scale(1, 1)"));
    assert_eq!(kf["50%"].transform.as_deref(), Some("scale(-1, 1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(-1, -1)"));
}

/// @demo format/css/transforms#mirror_walk
/// @title Mirror Walk
/// @description Combine translate and flip for walking animation that turns around.
#[test]
fn test_mirror_walk_animation() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/flip.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let mirror = animations.get("mirror_walk").expect("Animation 'mirror_walk' not found");
    let kf = mirror.keyframes.as_ref().unwrap();

    assert_eq!(kf.len(), 4, "mirror_walk should have 4 keyframes");

    // Walk right, turn around, walk back
    assert_eq!(kf["0%"].transform.as_deref(), Some("translate(0, 0) scaleX(1)"));
    assert_eq!(kf["50%"].transform.as_deref(), Some("translate(8px, 0) scaleX(1)"));
    assert_eq!(kf["51%"].transform.as_deref(), Some("translate(8px, 0) scaleX(-1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translate(0, 0) scaleX(-1)"));
}
