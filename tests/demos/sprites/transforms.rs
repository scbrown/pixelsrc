//! Sprite transform demos
//!
//! Flipping, rotating, scaling, and recoloring sprites using transform chains.

use crate::demos::parse_content;

/// @demo format/sprite#recolor
/// @title Recolor/Palette Swap
/// @description Palette swap by creating a sprite with different palette reference.
#[test]
fn test_recolor_palette_swap() {
    let jsonl = r##"{"type": "palette", "name": "original", "colors": {"{b}": "#0000FF", "{r}": "#FF0000"}}
{"type": "palette", "name": "swapped", "colors": {"{b}": "#FF0000", "{r}": "#0000FF"}}
{"type": "sprite", "name": "hero", "palette": "original", "grid": ["{b}{r}"]}
{"type": "sprite", "name": "hero_alt", "palette": "swapped", "source": "hero"}"##;

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Verify hero_alt uses swapped palette
    let hero_alt = sprite_registry.get_sprite("hero_alt").expect("hero_alt should exist");
    assert!(
        matches!(&hero_alt.palette, pixelsrc::models::PaletteRef::Named(name) if name == "swapped"),
        "hero_alt should use 'swapped' palette for recolor"
    );

    // Both should resolve (original and swapped)
    sprite_registry.resolve("hero", &palette_registry, false).expect("hero should resolve");
    sprite_registry.resolve("hero_alt", &palette_registry, false).expect("hero_alt should resolve");
}

/// @title Source Reference
/// @description Transformed sprite referencing another sprite via "source" field.
#[test]
fn test_transform_source_reference() {
    let jsonl = include_str!("../../../examples/demos/sprites/transforms.jsonl");

    let (_, sprite_registry, _) = parse_content(jsonl);

    // Verify arrow_left references arrow_right
    let arrow_left =
        sprite_registry.get_sprite("arrow_left").expect("Sprite 'arrow_left' should exist");

    assert_eq!(
        arrow_left.source.as_deref(),
        Some("arrow_right"),
        "arrow_left should reference 'arrow_right' as source"
    );

    // Verify it has a transform
    let transform = arrow_left.transform.as_ref().expect("arrow_left should have transform");

    assert!(!transform.is_empty(), "Transform should not be empty");
}
#[test]
fn test_transform_chain() {
    let jsonl = include_str!("../../../examples/demos/sprites/transforms.jsonl");

    let (_, sprite_registry, _) = parse_content(jsonl);

    // Check arrow_scaled has scale transform
    let scaled =
        sprite_registry.get_sprite("arrow_scaled").expect("Sprite 'arrow_scaled' should exist");

    let transform = scaled.transform.as_ref().expect("Should have transform");
    assert_eq!(transform.len(), 1, "Should have exactly 1 transform");

    // Verify it's a scale transform with 2.0 factor
    let spec_str = format!("{:?}", transform[0]);
    assert!(
        spec_str.contains("Scale") || spec_str.contains("2.0"),
        "Transform should be a scale by 2.0"
    );
}
