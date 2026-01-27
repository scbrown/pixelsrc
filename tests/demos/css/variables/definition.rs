//! CSS Variable Definition Demo Tests
//!
//! Tests for defining CSS custom properties (variables) in palettes.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/variables#definition_palette
/// @title Variable Definition in Palette
/// @description CSS custom properties define named color variables in palettes.
#[test]
fn test_variable_definition_palette() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/definition.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("theme_colors")
        .expect("theme_colors palette should exist");
    assert!(
        palette.colors.len() >= 3,
        "Should have at least 3 colors (transparent + primary + secondary)"
    );
}

/// @demo format/css/variables#definition_sprite
/// @title Sprite Using Variable Palette
/// @description Sprites reference palettes with CSS variable-defined colors.
#[test]
fn test_variable_definition_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/definition.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("theme_example")
        .expect("theme_example sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#definition_regions
/// @title Variable-Named Regions
/// @description Sprite regions map to CSS variable names for semantic coloring.
#[test]
fn test_variable_definition_regions() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/definition.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("theme_example")
        .expect("theme_example sprite should exist");
    let regions = sprite.regions.as_ref().expect("sprite should have regions");
    assert!(regions.contains_key("primary"), "Should have primary region");
    assert!(
        regions.contains_key("secondary"),
        "Should have secondary region"
    );
}
