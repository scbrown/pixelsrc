//! Scale Transform Demo Tests
//!
//! Tests for scaling using scale(s), scale(x, y), scaleX(x), scaleY(y).

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/transforms#scale
/// @title Scale Transform
/// @description Scaling using scale(s), scale(x, y), scaleX(x), scaleY(y).
#[test]
fn test_css_transforms_scale() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    sprite_registry
        .resolve("dot", &palette_registry, false)
        .expect("Sprite 'dot' should resolve");
    sprite_registry
        .resolve("square", &palette_registry, false)
        .expect("Sprite 'square' should resolve");

    // Test scale_up animation (uniform scale)
    let scale_up = animations.get("scale_up").expect("Animation 'scale_up' not found");
    assert!(scale_up.is_css_keyframes(), "scale_up should use CSS keyframes");
    let kf = scale_up.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(4)"));

    // Test scale_xy animation (non-uniform scale)
    let scale_xy = animations.get("scale_xy").expect("Animation 'scale_xy' not found");
    let kf = scale_xy.keyframes.as_ref().unwrap();
    assert_eq!(kf["50%"].transform.as_deref(), Some("scale(2, 1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(2, 2)"));
}

/// @demo format/css/transforms#scale_uniform
/// @title Uniform Scale
/// @description Equal scaling in both dimensions using scale(s).
#[test]
fn test_scale_uniform() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let scale_up = animations.get("scale_up").expect("Animation 'scale_up' not found");
    let kf = scale_up.keyframes.as_ref().unwrap();

    // Verify uniform scale from 1x to 4x
    assert_eq!(kf["0%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(4)"));
}

/// @demo format/css/transforms#scale_xy
/// @title Non-Uniform Scale
/// @description Different scaling in X and Y using scale(x, y).
#[test]
fn test_scale_non_uniform() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let scale_xy = animations.get("scale_xy").expect("Animation 'scale_xy' not found");
    let kf = scale_xy.keyframes.as_ref().unwrap();

    // Verify non-uniform scaling: width first, then height
    assert_eq!(kf["0%"].transform.as_deref(), Some("scale(1, 1)"));
    assert_eq!(kf["50%"].transform.as_deref(), Some("scale(2, 1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(2, 2)"));
}

/// @demo format/css/transforms#scale_x
/// @title ScaleX Only
/// @description Horizontal scaling using scaleX(x) shorthand.
#[test]
fn test_scale_x_only() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let scale_x = animations.get("scale_x_only").expect("Animation 'scale_x_only' not found");
    let kf = scale_x.keyframes.as_ref().unwrap();

    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleX(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleX(3)"));
}

/// @demo format/css/transforms#scale_y
/// @title ScaleY Only
/// @description Vertical scaling using scaleY(y) shorthand.
#[test]
fn test_scale_y_only() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let scale_y = animations.get("scale_y_only").expect("Animation 'scale_y_only' not found");
    let kf = scale_y.keyframes.as_ref().unwrap();

    assert_eq!(kf["0%"].transform.as_deref(), Some("scaleY(1)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("scaleY(3)"));
}

/// @demo format/css/transforms#pulse_scale
/// @title Pulse Scale with Opacity
/// @description Scale combined with opacity for pulsing effect.
#[test]
fn test_pulse_scale_with_opacity() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/scale.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let pulse = animations.get("pulse_scale").expect("Animation 'pulse_scale' not found");
    let kf = pulse.keyframes.as_ref().unwrap();

    // Verify scale and opacity work together
    assert_eq!(kf["0%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(kf["0%"].opacity, Some(1.0));

    assert_eq!(kf["50%"].transform.as_deref(), Some("scale(2)"));
    assert_eq!(kf["50%"].opacity, Some(0.6));

    assert_eq!(kf["100%"].transform.as_deref(), Some("scale(1)"));
    assert_eq!(kf["100%"].opacity, Some(1.0));
}
