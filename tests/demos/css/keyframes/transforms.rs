//! Transform Keyframes Demo Tests
//!
//! Tests for CSS @keyframes using CSS transforms (rotate, scale) at keyframes.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/keyframes#transforms
/// @title Transform Animations
/// @description Animations using CSS transforms (rotate, scale) at keyframes.
#[test]
fn test_transforms_overview() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprite can be resolved
    sprite_registry
        .resolve("shape", &palette_registry, false)
        .expect("Sprite 'shape' should resolve");

    // Verify both animations exist and use keyframes
    let spin = animations.get("spin").expect("Animation 'spin' not found");
    assert!(spin.is_css_keyframes(), "spin should use CSS keyframes format");

    let pulse = animations.get("pulse").expect("Animation 'pulse' not found");
    assert!(pulse.is_css_keyframes(), "pulse should use CSS keyframes format");
}

/// @demo format/css/keyframes#rotate
/// @title Rotate Transform
/// @description Animation using rotate() transform for full rotation.
#[test]
fn test_rotate_transform() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let spin = animations.get("spin").expect("Animation 'spin' not found");
    let spin_kf = spin.keyframes.as_ref().unwrap();

    // Verify rotation keyframes
    assert_eq!(spin_kf.len(), 2, "spin should have 2 keyframes");
    assert_eq!(spin_kf["0%"].transform.as_deref(), Some("rotate(0deg)"));
    assert_eq!(spin_kf["100%"].transform.as_deref(), Some("rotate(360deg)"));

    // Verify linear timing for smooth rotation
    assert_eq!(spin.timing_function.as_deref(), Some("linear"));

    // Verify duration
    assert_eq!(spin.duration_ms(), 1000);
}

/// @demo format/css/keyframes#scale
/// @title Scale Transform
/// @description Animation using scale() transform for pulse effect.
#[test]
fn test_scale_transform() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let pulse = animations.get("pulse").expect("Animation 'pulse' not found");
    let pulse_kf = pulse.keyframes.as_ref().unwrap();

    // Verify scale keyframes
    assert_eq!(pulse_kf.len(), 3, "pulse should have 3 keyframes");
    assert_eq!(pulse_kf["0%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(pulse_kf["50%"].transform.as_deref(), Some("scale(1.5)"));
    assert_eq!(pulse_kf["100%"].transform.as_deref(), Some("scale(1)"));
}

/// @demo format/css/keyframes#scale_opacity
/// @title Scale with Opacity
/// @description Animation combining scale transform with opacity changes.
#[test]
fn test_scale_with_opacity() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let pulse = animations.get("pulse").expect("Animation 'pulse' not found");
    let pulse_kf = pulse.keyframes.as_ref().unwrap();

    // Verify combined transform and opacity at each keyframe
    // 0%: normal size, full opacity
    assert_eq!(pulse_kf["0%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(pulse_kf["0%"].opacity, Some(1.0));

    // 50%: larger, half opacity
    assert_eq!(pulse_kf["50%"].transform.as_deref(), Some("scale(1.5)"));
    assert_eq!(pulse_kf["50%"].opacity, Some(0.5));

    // 100%: back to normal
    assert_eq!(pulse_kf["100%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(pulse_kf["100%"].opacity, Some(1.0));
}

/// @demo format/css/keyframes#transform_timing
/// @title Transform Timing Functions
/// @description Verify correct timing functions for transform animations.
#[test]
fn test_transform_timing() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // Spin uses linear for constant rotation speed
    let spin = animations.get("spin").expect("Animation 'spin' not found");
    assert_eq!(spin.timing_function.as_deref(), Some("linear"));

    // Pulse uses ease-in-out for smooth start/stop
    let pulse = animations.get("pulse").expect("Animation 'pulse' not found");
    assert_eq!(pulse.timing_function.as_deref(), Some("ease-in-out"));
}

/// @demo format/css/keyframes#transform_duration
/// @title Transform Animation Durations
/// @description Verify duration parsing for transform animations.
#[test]
fn test_transform_durations() {
    let jsonl = include_str!("../../../../examples/demos/css/keyframes/transforms.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // Spin: 1 second
    let spin = animations.get("spin").expect("Animation 'spin' not found");
    assert_eq!(spin.duration_ms(), 1000);

    // Pulse: 500ms
    let pulse = animations.get("pulse").expect("Animation 'pulse' not found");
    assert_eq!(pulse.duration_ms(), 500);
}
