//! Rotate Transform Demo Tests
//!
//! Tests for rotation using rotate(deg) - pixel art supports 90, 180, 270 degrees.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/transforms#rotate
/// @title Rotate Transform
/// @description Rotation using rotate(deg) - pixel art supports 90, 180, 270 degrees.
#[test]
fn test_css_transforms_rotate() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/rotate.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    sprite_registry
        .resolve("L_shape", &palette_registry, false)
        .expect("Sprite 'L_shape' should resolve");
    sprite_registry
        .resolve("arrow_up", &palette_registry, false)
        .expect("Sprite 'arrow_up' should resolve");

    // Test rotate_90 animation
    let rotate_90 = animations.get("rotate_90").expect("Animation 'rotate_90' not found");
    assert!(rotate_90.is_css_keyframes(), "rotate_90 should use CSS keyframes");
    let kf = rotate_90.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(90deg)"));

    // Test rotate_180 animation
    let rotate_180 = animations.get("rotate_180").expect("Animation 'rotate_180' not found");
    let kf = rotate_180.keyframes.as_ref().unwrap();
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(180deg)"));

    // Test rotate_270 animation
    let rotate_270 = animations.get("rotate_270").expect("Animation 'rotate_270' not found");
    let kf = rotate_270.keyframes.as_ref().unwrap();
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(270deg)"));
}

/// @demo format/css/transforms#rotate_90
/// @title 90 Degree Rotation
/// @description Quarter turn clockwise rotation.
#[test]
fn test_rotate_90_degrees() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/rotate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let rotate_90 = animations.get("rotate_90").expect("Animation 'rotate_90' not found");
    let kf = rotate_90.keyframes.as_ref().unwrap();

    assert_eq!(kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(90deg)"));
}

/// @demo format/css/transforms#rotate_180
/// @title 180 Degree Rotation
/// @description Half turn rotation (upside down).
#[test]
fn test_rotate_180_degrees() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/rotate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let rotate_180 = animations.get("rotate_180").expect("Animation 'rotate_180' not found");
    let kf = rotate_180.keyframes.as_ref().unwrap();

    assert_eq!(kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(180deg)"));
}

/// @demo format/css/transforms#rotate_270
/// @title 270 Degree Rotation
/// @description Three-quarter turn clockwise (or 90 counter-clockwise).
#[test]
fn test_rotate_270_degrees() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/rotate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let rotate_270 = animations.get("rotate_270").expect("Animation 'rotate_270' not found");
    let kf = rotate_270.keyframes.as_ref().unwrap();

    assert_eq!(kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(270deg)"));
}

/// @demo format/css/transforms#spin_full
/// @title Full 360 Spin
/// @description Complete rotation through all four cardinal directions.
#[test]
fn test_spin_full_rotation() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/rotate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let spin_full = animations.get("spin_full").expect("Animation 'spin_full' not found");
    let kf = spin_full.keyframes.as_ref().unwrap();

    assert_eq!(kf.len(), 5, "spin_full should have 5 keyframes (0%, 25%, 50%, 75%, 100%)");
    assert_eq!(kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(kf["25%"].transform.as_deref(), Some("rotate(90deg)"));
    assert_eq!(kf["50%"].transform.as_deref(), Some("rotate(180deg)"));
    assert_eq!(kf["75%"].transform.as_deref(), Some("rotate(270deg)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("rotate(360deg)"));
}
