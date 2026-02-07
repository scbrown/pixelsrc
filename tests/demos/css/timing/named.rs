//! Named Timing Function Demo Tests
//!
//! Tests for CSS named easing functions: linear, ease, ease-in, ease-out, ease-in-out.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/timing#linear
/// @title Linear Timing Function
/// @description Constant-speed animation with no easing.
#[test]
fn test_linear() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    for sprite_name in ["box_left", "box_center", "box_right"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Sprite '{sprite_name}' should resolve"));
    }

    let linear = animations.get("linear_slide").expect("Animation 'linear_slide' not found");
    assert!(linear.is_css_keyframes(), "linear_slide should use CSS keyframes");
    assert_eq!(linear.timing_function.as_deref(), Some("linear"));
    assert_eq!(linear.duration_ms(), 500);
}

/// @demo format/css/timing#ease
/// @title Ease Timing Function
/// @description Default CSS easing with slow start and end.
#[test]
fn test_ease() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let ease = animations.get("ease_slide").expect("Animation 'ease_slide' not found");
    assert!(ease.is_css_keyframes(), "ease_slide should use CSS keyframes");
    assert_eq!(ease.timing_function.as_deref(), Some("ease"));
    assert_eq!(ease.duration_ms(), 500);
}

/// @demo format/css/timing#ease_in
/// @title Ease-In Timing Function
/// @description Animation that starts slowly and accelerates.
#[test]
fn test_ease_in() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let ease_in = animations.get("ease_in_slide").expect("Animation 'ease_in_slide' not found");
    assert!(ease_in.is_css_keyframes(), "ease_in_slide should use CSS keyframes");
    assert_eq!(ease_in.timing_function.as_deref(), Some("ease-in"));
    assert_eq!(ease_in.duration_ms(), 500);
}

/// @demo format/css/timing#ease_out
/// @title Ease-Out Timing Function
/// @description Animation that starts fast and decelerates.
#[test]
fn test_ease_out() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let ease_out = animations.get("ease_out_slide").expect("Animation 'ease_out_slide' not found");
    assert!(ease_out.is_css_keyframes(), "ease_out_slide should use CSS keyframes");
    assert_eq!(ease_out.timing_function.as_deref(), Some("ease-out"));
    assert_eq!(ease_out.duration_ms(), 500);
}

/// @demo format/css/timing#ease_in_out
/// @title Ease-In-Out Timing Function
/// @description Animation with slow start and end, fast middle.
#[test]
fn test_ease_in_out() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let ease_in_out =
        animations.get("ease_in_out_slide").expect("Animation 'ease_in_out_slide' not found");
    assert!(ease_in_out.is_css_keyframes(), "ease_in_out_slide should use CSS keyframes");
    assert_eq!(ease_in_out.timing_function.as_deref(), Some("ease-in-out"));
    assert_eq!(ease_in_out.duration_ms(), 500);
}

/// @demo format/css/timing#named_all
/// @title All Named Timing Functions
/// @description Comparison of all five named timing functions side by side.
#[test]
fn test_all_named_timing_functions() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/named.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // Verify all five named timing functions exist
    let timing_funcs = ["linear", "ease", "ease-in", "ease-out", "ease-in-out"];
    let anim_names =
        ["linear_slide", "ease_slide", "ease_in_slide", "ease_out_slide", "ease_in_out_slide"];

    for (timing_func, anim_name) in timing_funcs.iter().zip(anim_names.iter()) {
        let anim = animations
            .get(*anim_name)
            .unwrap_or_else(|| panic!("Animation '{anim_name}' not found"));
        assert!(anim.is_css_keyframes(), "{anim_name} should use CSS keyframes");
        assert_eq!(
            anim.timing_function.as_deref(),
            Some(*timing_func),
            "{anim_name} should have timing function {timing_func}"
        );
    }
}
