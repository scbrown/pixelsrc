//! Multiple Palette Cycles Demo Tests
//!
//! Tests for multiple palette cycles running simultaneously at different speeds.

use crate::demos::{assert_validates, capture_palette_cycle_info, parse_content};

/// @demo format/animation/palette_cycle#multiple
/// @title Multiple Independent Cycles
/// @description Multiple palette cycles running simultaneously at different speeds (water + fire).
#[test]
fn test_palette_cycle_multiple() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/multiple_cycles.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprite can be resolved
    sprite_registry
        .resolve("waterfire", &palette_registry, false)
        .expect("Sprite 'waterfire' should resolve");

    // Verify animation has multiple palette cycles
    let anim = animations.get("dual_cycle").expect("Animation 'dual_cycle' not found");
    let cycles = anim.palette_cycles();
    assert_eq!(cycles.len(), 2, "Should have 2 palette cycles");

    // Verify water cycle (3 tokens, 300ms)
    let water_cycle = &cycles[0];
    assert_eq!(water_cycle.tokens.len(), 3, "Water cycle should have 3 tokens");
    assert!(water_cycle.tokens.iter().all(|t| t.starts_with("{w")));
    assert_eq!(water_cycle.duration, Some(300), "Water cycle duration should be 300ms");

    // Verify fire cycle (3 tokens, 200ms)
    let fire_cycle = &cycles[1];
    assert_eq!(fire_cycle.tokens.len(), 3, "Fire cycle should have 3 tokens");
    assert!(fire_cycle.tokens.iter().all(|t| t.starts_with("{f")));
    assert_eq!(fire_cycle.duration, Some(200), "Fire cycle duration should be 200ms");

    // Verify total frames = LCM(3, 3) = 3
    let info = capture_palette_cycle_info(jsonl, "dual_cycle");
    assert_eq!(info.cycle_count, 2);
    assert_eq!(info.total_frames, 3, "LCM(3,3) = 3 total frames");
    assert_eq!(info.cycle_lengths, vec![3, 3]);
}

/// @demo format/animation/palette_cycle#multiple_independent
/// @title Independent Cycle Timing
/// @description Each cycle has its own duration, running independently.
#[test]
fn test_multiple_cycles_independent_durations() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/multiple_cycles.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "dual_cycle");

    // Each cycle has its own duration
    assert_eq!(info.cycle_durations.len(), 2, "Should have 2 cycle durations");
    assert_eq!(info.cycle_durations[0], Some(300), "Water cycle: 300ms");
    assert_eq!(info.cycle_durations[1], Some(200), "Fire cycle: 200ms");
}

/// @demo format/animation/palette_cycle#multiple_tokens
/// @title Multiple Cycle Token Groups
/// @description Different token groups cycle independently without interference.
#[test]
fn test_multiple_cycles_token_groups() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/multiple_cycles.jsonl");
    assert_validates(jsonl, true);

    let info = capture_palette_cycle_info(jsonl, "dual_cycle");

    // Verify token groups are preserved
    assert_eq!(info.cycle_tokens.len(), 2, "Should have 2 token groups");

    // Water tokens
    let water_tokens = &info.cycle_tokens[0];
    assert_eq!(water_tokens, &vec!["{w1}", "{w2}", "{w3}"]);

    // Fire tokens
    let fire_tokens = &info.cycle_tokens[1];
    assert_eq!(fire_tokens, &vec!["{f1}", "{f2}", "{f3}"]);
}

/// @demo format/animation/palette_cycle#multiple_sprite
/// @title Multiple Cycles Sprite Structure
/// @description Sprite with multiple color regions for independent cycling.
#[test]
fn test_multiple_cycles_sprite_structure() {
    let jsonl = include_str!("../../../examples/demos/palette_cycling/multiple_cycles.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    let resolved = sprite_registry
        .resolve("waterfire", &palette_registry, false)
        .expect("Sprite 'waterfire' should resolve");

    // Waterfire sprite is 6x2 with 7 colors (3 water + 3 fire + transparent)
    assert_eq!(resolved.palette.len(), 7, "Palette should have 7 colors");
}
