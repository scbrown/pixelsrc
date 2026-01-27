//! CSS Variable Resolution Demo Tests
//!
//! Tests for var() resolution of CSS custom properties.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/variables#resolution_palette
/// @title var() Resolution Palette
/// @description CSS var() function resolves variable references to color values.
#[test]
fn test_var_resolution_palette() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/resolution.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("var_resolution")
        .expect("var_resolution palette should exist");
    assert!(
        palette.colors.len() >= 3,
        "Should have at least 3 colors (transparent + a + b)"
    );
}

/// @demo format/css/variables#resolution_sprite
/// @title Resolved Variable Sprite
/// @description Sprite using resolved CSS variables renders correctly.
#[test]
fn test_var_resolution_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/resolution.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("resolved_colors")
        .expect("resolved_colors sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#resolution_regions
/// @title Variable Resolution Regions
/// @description Regions reference resolved variable names.
#[test]
fn test_var_resolution_regions() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/resolution.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("resolved_colors")
        .expect("resolved_colors sprite should exist");
    let regions = sprite.regions.as_ref().expect("sprite should have regions");
    assert!(regions.contains_key("a"), "Should have region a");
    assert!(regions.contains_key("b"), "Should have region b");
}
