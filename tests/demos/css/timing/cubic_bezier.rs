//! Cubic Bezier Timing Function Demo Tests
//!
//! Tests for custom CSS cubic-bezier(x1, y1, x2, y2) easing curves.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/timing/cubic_bezier#bounce
/// @title Bounce-Like Cubic Bezier
/// @description Custom easing curve that creates a bounce-like motion.
#[test]
fn test_bounce_fall() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/cubic_bezier.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    for sprite_name in ["ball_top", "ball_middle", "ball_bottom"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Sprite '{sprite_name}' should resolve"));
    }

    let bounce = animations.get("bounce_fall").expect("Animation 'bounce_fall' not found");
    assert!(bounce.is_css_keyframes(), "bounce_fall should use CSS keyframes");
    assert_eq!(
        bounce.timing_function.as_deref(),
        Some("cubic-bezier(0.5, 0, 0.5, 1)")
    );
    assert_eq!(bounce.duration_ms(), 800);

    // Verify keyframes
    let kf = bounce.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 3, "bounce_fall should have 3 keyframes");
    assert!(kf.contains_key("0%"));
    assert!(kf.contains_key("50%"));
    assert!(kf.contains_key("100%"));
}

/// @demo format/css/timing/cubic_bezier#snap
/// @title Snap Easing with Overshoot
/// @description Cubic-bezier with negative/positive y values for overshoot effect.
#[test]
fn test_snap_ease() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/cubic_bezier.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let snap = animations.get("snap_ease").expect("Animation 'snap_ease' not found");
    assert!(snap.is_css_keyframes(), "snap_ease should use CSS keyframes");
    assert_eq!(
        snap.timing_function.as_deref(),
        Some("cubic-bezier(0.68, -0.55, 0.27, 1.55)")
    );
    assert_eq!(snap.duration_ms(), 500);

    // Verify keyframes
    let kf = snap.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 2, "snap_ease should have 2 keyframes (from/to)");
}

/// @demo format/css/timing/cubic_bezier#smooth
/// @title Smooth Deceleration
/// @description Cubic-bezier curve optimized for smooth deceleration.
#[test]
fn test_smooth_decel() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/cubic_bezier.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let smooth = animations.get("smooth_decel").expect("Animation 'smooth_decel' not found");
    assert!(smooth.is_css_keyframes(), "smooth_decel should use CSS keyframes");
    assert_eq!(
        smooth.timing_function.as_deref(),
        Some("cubic-bezier(0.25, 0.1, 0.25, 1.0)")
    );
    assert_eq!(smooth.duration_ms(), 500);
}

/// @demo format/css/timing/cubic_bezier#all
/// @title All Cubic Bezier Examples
/// @description Comparison of different cubic-bezier curves for various animation styles.
#[test]
fn test_all_cubic_bezier() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/cubic_bezier.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // All animations should use CSS keyframes and have cubic-bezier timing
    let expected = [
        ("bounce_fall", "cubic-bezier(0.5, 0, 0.5, 1)"),
        ("snap_ease", "cubic-bezier(0.68, -0.55, 0.27, 1.55)"),
        ("smooth_decel", "cubic-bezier(0.25, 0.1, 0.25, 1.0)"),
    ];

    for (anim_name, timing_func) in expected {
        let anim = animations
            .get(anim_name)
            .unwrap_or_else(|| panic!("Animation '{anim_name}' not found"));
        assert!(anim.is_css_keyframes(), "{anim_name} should use CSS keyframes");
        assert_eq!(
            anim.timing_function.as_deref(),
            Some(timing_func),
            "{anim_name} should have timing function {timing_func}"
        );
    }
}
