//! Ping-Pong Cycling Demo Tests
//!
//! Tests for reverse direction cycling using duplicated tokens pattern.

use crate::demos::{assert_validates, capture_palette_cycle_info, parse_content};

/// @demo format/animation/palette_cycle#ping_pong
/// @title Ping-Pong Cycling
/// @description Reverse direction cycling using duplicated tokens pattern (forward then backward).
#[test]
fn test_palette_cycle_ping_pong() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/ping_pong.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprite can be resolved
    sprite_registry
        .resolve("glow", &palette_registry, false)
        .expect("Sprite 'glow' should resolve");

    // Verify animation has ping-pong pattern
    let anim = animations.get("ping_pong_glow").expect("Animation 'ping_pong_glow' not found");
    let cycles = anim.palette_cycles();
    assert_eq!(cycles.len(), 1, "Should have 1 palette cycle");

    // Ping-pong is achieved by duplicating tokens: [p1, p2, p3, p4, p5, p4, p3, p2]
    // This creates a forward-then-backward pattern
    let cycle = &cycles[0];
    assert_eq!(cycle.tokens.len(), 8, "Ping-pong cycle should have 8 tokens (5 + 3 reverse)");

    // Verify the ping-pong pattern
    assert_eq!(cycle.tokens[0], "{p1}", "Start at p1");
    assert_eq!(cycle.tokens[4], "{p5}", "Peak at p5 (middle)");
    assert_eq!(cycle.tokens[5], "{p4}", "Reverse: p4");
    assert_eq!(cycle.tokens[6], "{p3}", "Reverse: p3");
    assert_eq!(cycle.tokens[7], "{p2}", "Reverse: p2 (ends before p1 to avoid double)");

    // Verify frame count = token count
    let info = capture_palette_cycle_info(jsonl, "ping_pong_glow");
    assert_eq!(info.total_frames, 8, "8 tokens = 8 frames");
    assert_eq!(info.cycle_durations, vec![Some(100)]);
}

/// @demo format/animation/palette_cycle#ping_pong_pattern
/// @title Ping-Pong Token Pattern
/// @description Verify the forward-backward token sequence for smooth oscillation.
#[test]
fn test_ping_pong_token_pattern() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/ping_pong.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "ping_pong_glow");

    // Full token sequence: forward to peak, then reverse (skipping endpoints to avoid double)
    let expected_tokens = vec![
        "{p1}", "{p2}", "{p3}", "{p4}", "{p5}", // Forward
        "{p4}", "{p3}", "{p2}",                 // Reverse (skip p1 to avoid double)
    ];
    assert_eq!(info.cycle_tokens[0], expected_tokens);
}

/// @demo format/animation/palette_cycle#ping_pong_duration
/// @title Ping-Pong Cycle Duration
/// @description Each step in the ping-pong cycle uses the specified duration.
#[test]
fn test_ping_pong_duration() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/ping_pong.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "ping_pong_glow");

    // Duration is 100ms per step
    assert_eq!(info.cycle_durations[0], Some(100), "Each step should be 100ms");

    // Total animation time = 8 frames * 100ms = 800ms for one complete oscillation
    assert_eq!(info.total_frames, 8);
}

/// @demo format/animation/palette_cycle#ping_pong_sprite
/// @title Ping-Pong Glow Sprite
/// @description Verify the glow sprite used for ping-pong cycling.
#[test]
fn test_ping_pong_sprite() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/ping_pong.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Glow sprite is 1x1 with 6 colors (5 cycle colors + transparent)
    let resolved = sprite_registry
        .resolve("glow", &palette_registry, false)
        .expect("Sprite 'glow' should resolve");

    assert_eq!(resolved.palette.len(), 6, "Palette should have 6 colors (5 + transparent)");
}
