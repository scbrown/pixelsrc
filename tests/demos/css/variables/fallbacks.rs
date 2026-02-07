//! CSS Variable Fallback Demo Tests
//!
//! Tests for CSS variable fallback values when variables are undefined.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/variables#fallback_simple
/// @title Simple Variable Fallback
/// @description Fallback value used when a CSS variable is undefined.
#[test]
fn test_simple_fallback() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("simple_fallback").expect("simple_fallback palette should exist");
    assert!(palette.colors.len() >= 2, "Should have at least 2 colors (transparent + fb)");
}

/// @demo format/css/variables#fallback_nested
/// @title Nested Variable Fallback
/// @description Nested fallbacks resolve through multiple levels of var() references.
#[test]
fn test_nested_fallback() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette =
        palette_registry.get("nested_fallback").expect("nested_fallback palette should exist");
    assert!(palette.colors.len() >= 2, "Should have at least 2 colors (transparent + nf)");
}

/// @demo format/css/variables#fallback_color_mix
/// @title Color-Mix Fallback
/// @description Fallback using color-mix() function for computed fallback values.
#[test]
fn test_color_mix_fallback() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("color_mix_fallback")
        .expect("color_mix_fallback palette should exist");
    assert!(palette.colors.len() >= 2, "Should have at least 2 colors (transparent + mf)");
}

/// @demo format/css/variables#fallback_sprite
/// @title Fallback Sprite Rendering
/// @description Sprites render correctly using fallback-resolved colors.
#[test]
fn test_fallback_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite =
        sprite_registry.get_sprite("fallback_demo").expect("fallback_demo sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#fallback_nested_sprite
/// @title Nested Fallback Sprite
/// @description Sprite using nested fallback palette renders correctly.
#[test]
fn test_nested_fallback_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("nested_fallback_result")
        .expect("nested_fallback_result sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#fallback_mix_sprite
/// @title Color-Mix Fallback Sprite
/// @description Sprite using color-mix fallback palette renders correctly.
#[test]
fn test_mix_fallback_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/fallbacks.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("mix_fallback_result")
        .expect("mix_fallback_result sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}
