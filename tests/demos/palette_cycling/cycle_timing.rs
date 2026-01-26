//! Cycle Timing Demo Tests
//!
//! Tests for controlling cycle speed with duration field.

use crate::demos::{assert_validates, capture_palette_cycle_info, parse_content};

/// @demo format/animation/palette_cycle#timing
/// @title Cycle Timing Control
/// @description Controlling cycle speed with duration field (fast vs slow cycling).
#[test]
fn test_palette_cycle_timing() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/cycle_timing.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // Verify fast cycle (50ms duration)
    let fast_anim = animations.get("fast_cycle").expect("Animation 'fast_cycle' not found");
    let fast_cycles = fast_anim.palette_cycles();
    assert_eq!(fast_cycles.len(), 1);
    assert_eq!(fast_cycles[0].duration, Some(50), "Fast cycle should be 50ms");
    assert_eq!(fast_cycles[0].tokens.len(), 3);

    // Verify slow cycle (500ms duration)
    let slow_anim = animations.get("slow_cycle").expect("Animation 'slow_cycle' not found");
    let slow_cycles = slow_anim.palette_cycles();
    assert_eq!(slow_cycles.len(), 1);
    assert_eq!(slow_cycles[0].duration, Some(500), "Slow cycle should be 500ms");
    assert_eq!(slow_cycles[0].tokens.len(), 3);

    // Both have same number of frames (3 tokens each)
    let fast_info = capture_palette_cycle_info(jsonl, "fast_cycle");
    let slow_info = capture_palette_cycle_info(jsonl, "slow_cycle");
    assert_eq!(
        fast_info.total_frames, slow_info.total_frames,
        "Same token count = same frames"
    );
    assert_eq!(fast_info.total_frames, 3);

    // But different durations
    assert_eq!(fast_info.cycle_durations, vec![Some(50)]);
    assert_eq!(slow_info.cycle_durations, vec![Some(500)]);
}

/// @demo format/animation/palette_cycle#timing_fast
/// @title Fast Cycle Animation
/// @description Rapid color cycling at 50ms per step for energetic effects.
#[test]
fn test_fast_cycle() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/cycle_timing.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "fast_cycle");

    assert_eq!(info.cycle_count, 1);
    assert_eq!(info.cycle_durations[0], Some(50), "Fast cycle: 50ms per step");
    assert_eq!(info.total_frames, 3, "3 tokens = 3 frames");

    // Verify tokens
    assert_eq!(info.cycle_tokens[0], vec!["{t1}", "{t2}", "{t3}"]);
}

/// @demo format/animation/palette_cycle#timing_slow
/// @title Slow Cycle Animation
/// @description Gentle color cycling at 500ms per step for ambient effects.
#[test]
fn test_slow_cycle() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/cycle_timing.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "slow_cycle");

    assert_eq!(info.cycle_count, 1);
    assert_eq!(info.cycle_durations[0], Some(500), "Slow cycle: 500ms per step");
    assert_eq!(info.total_frames, 3, "3 tokens = 3 frames");

    // Verify tokens match fast cycle
    assert_eq!(info.cycle_tokens[0], vec!["{t1}", "{t2}", "{t3}"]);
}

/// @demo format/animation/palette_cycle#timing_sprite
/// @title Timing Demo Sprite
/// @description Verify timing demo sprite resolves correctly.
#[test]
fn test_timing_sprite_resolution() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/cycle_timing.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Verify timing_demo sprite resolves
    let resolved = sprite_registry
        .resolve("timing_demo", &palette_registry, false)
        .expect("Sprite 'timing_demo' should resolve");

    // Timing demo sprite is 3x1 with 4 colors (3 cycle colors + transparent)
    assert_eq!(resolved.palette.len(), 4, "Palette should have 4 colors");
}
