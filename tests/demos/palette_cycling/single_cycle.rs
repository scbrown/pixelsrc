//! Single Palette Cycle Demo Tests
//!
//! Tests for single color cycling through a sequence of values.

use crate::demos::{assert_validates, capture_palette_cycle_info, parse_content};

/// @demo format/animation/palette_cycle#single
/// @title Single Palette Cycle
/// @description Single color cycling through a sequence of values (classic water/fire shimmer).
#[test]
fn test_palette_cycle_single() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/single_cycle.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprite can be resolved
    sprite_registry
        .resolve("wave", &palette_registry, false)
        .expect("Sprite 'wave' should resolve");

    // Verify animation has palette cycle
    let anim = animations.get("wave_cycle").expect("Animation 'wave_cycle' not found");
    let cycles = anim.palette_cycles();
    assert_eq!(cycles.len(), 1, "Should have 1 palette cycle");

    // Verify cycle properties
    let cycle = &cycles[0];
    assert_eq!(cycle.tokens.len(), 4, "Cycle should have 4 tokens");
    assert_eq!(cycle.tokens[0], "{c1}");
    assert_eq!(cycle.tokens[3], "{c4}");
    assert_eq!(cycle.duration, Some(200), "Cycle duration should be 200ms");

    // Verify using helper
    let info = capture_palette_cycle_info(jsonl, "wave_cycle");
    assert_eq!(info.cycle_count, 1);
    assert_eq!(info.total_frames, 4, "4 tokens = 4 frames for single cycle");
    assert_eq!(info.cycle_lengths, vec![4]);
}

/// @demo format/animation/palette_cycle#single_tokens
/// @title Single Cycle Token Sequence
/// @description Verify the token sequence is preserved in order for single cycles.
#[test]
fn test_single_cycle_token_order() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/single_cycle.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);
    let anim = animations.get("wave_cycle").expect("Animation 'wave_cycle' not found");
    let cycles = anim.palette_cycles();
    let cycle = &cycles[0];

    // Verify token order is preserved
    let expected_tokens = vec!["{c1}", "{c2}", "{c3}", "{c4}"];
    assert_eq!(cycle.tokens, expected_tokens, "Token order should be preserved");
}

/// @demo format/animation/palette_cycle#single_sprite
/// @title Single Cycle Sprite Resolution
/// @description Verify the sprite using palette cycling can be resolved with all color tokens.
#[test]
fn test_single_cycle_sprite_resolution() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/single_cycle.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Verify sprite resolves with all palette colors
    let resolved = sprite_registry
        .resolve("wave", &palette_registry, false)
        .expect("Sprite 'wave' should resolve");

    // Wave sprite is 4x2 with 5 colors (4 cycle colors + transparent)
    assert_eq!(resolved.palette.len(), 5, "Palette should have 5 colors (4 + transparent)");
}
