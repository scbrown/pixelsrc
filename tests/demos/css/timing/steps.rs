//! Steps Timing Function Demo Tests
//!
//! Tests for discrete step-based timing: steps(n), steps(n, position), step-start, step-end.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/timing/steps#basic
/// @title Basic Steps Timing
/// @description Animation that moves in discrete steps rather than smooth transitions.
#[test]
fn test_steps_basic() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    for sprite_name in ["step1", "step2", "step3", "step4"] {
        sprite_registry
            .resolve(sprite_name, &palette_registry, false)
            .unwrap_or_else(|_| panic!("Sprite '{sprite_name}' should resolve"));
    }

    let steps4 = animations.get("steps_4").expect("Animation 'steps_4' not found");
    assert!(steps4.is_css_keyframes(), "steps_4 should use CSS keyframes");
    assert_eq!(steps4.timing_function.as_deref(), Some("steps(4)"));
    assert_eq!(steps4.duration_ms(), 1000);

    // Verify 5 keyframes (0%, 25%, 50%, 75%, 100%)
    let kf = steps4.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 5, "steps_4 should have 5 keyframes");
}

/// @demo format/css/timing/steps#jump_start
/// @title Steps with Jump-Start
/// @description Steps timing where the first step happens immediately at start.
#[test]
fn test_steps_jump_start() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let jump_start =
        animations.get("steps_jump_start").expect("Animation 'steps_jump_start' not found");
    assert!(jump_start.is_css_keyframes(), "steps_jump_start should use CSS keyframes");
    assert_eq!(jump_start.timing_function.as_deref(), Some("steps(4, jump-start)"));
    assert_eq!(jump_start.duration_ms(), 1000);

    // Verify from/to keyframes
    let kf = jump_start.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 2, "steps_jump_start should have 2 keyframes");
    assert!(kf.contains_key("from"));
    assert!(kf.contains_key("to"));
}

/// @demo format/css/timing/steps#jump_end
/// @title Steps with Jump-End
/// @description Steps timing where the last step happens at the end.
#[test]
fn test_steps_jump_end() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let jump_end = animations.get("steps_jump_end").expect("Animation 'steps_jump_end' not found");
    assert!(jump_end.is_css_keyframes(), "steps_jump_end should use CSS keyframes");
    assert_eq!(jump_end.timing_function.as_deref(), Some("steps(4, jump-end)"));
    assert_eq!(jump_end.duration_ms(), 1000);
}

/// @demo format/css/timing/steps#step_start
/// @title Step-Start Timing
/// @description Instant jump to final value at animation start (alias for steps(1, jump-start)).
#[test]
fn test_step_start() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let step_start =
        animations.get("step_start_instant").expect("Animation 'step_start_instant' not found");
    assert!(step_start.is_css_keyframes(), "step_start_instant should use CSS keyframes");
    assert_eq!(step_start.timing_function.as_deref(), Some("step-start"));
    assert_eq!(step_start.duration_ms(), 500);
}

/// @demo format/css/timing/steps#step_end
/// @title Step-End Timing
/// @description Delayed jump to final value at animation end (alias for steps(1, jump-end)).
#[test]
fn test_step_end() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let step_end =
        animations.get("step_end_delayed").expect("Animation 'step_end_delayed' not found");
    assert!(step_end.is_css_keyframes(), "step_end_delayed should use CSS keyframes");
    assert_eq!(step_end.timing_function.as_deref(), Some("step-end"));
    assert_eq!(step_end.duration_ms(), 500);
}

/// @demo format/css/timing/steps#all
/// @title All Steps Timing Variants
/// @description Comparison of steps(n), jump-start, jump-end, step-start, and step-end.
#[test]
fn test_all_steps_variants() {
    let jsonl = include_str!("../../../../examples/demos/css/timing/steps.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let expected = [
        ("steps_4", "steps(4)"),
        ("steps_jump_start", "steps(4, jump-start)"),
        ("steps_jump_end", "steps(4, jump-end)"),
        ("step_start_instant", "step-start"),
        ("step_end_delayed", "step-end"),
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
