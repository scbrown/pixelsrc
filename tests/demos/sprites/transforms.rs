//! Sprite transform demos
//!
//! Flipping, rotating, scaling, and recoloring sprites using transform chains.

use crate::demos::{assert_dimensions, assert_validates, capture_render_info, parse_content};

/// @demo format/sprite#flip_horizontal
/// @title Horizontal Flip (Mirror)
/// @description Sprite mirrored horizontally using transform: ["mirror-h"].
#[test]
    #[ignore = "Grid format deprecated"]
fn test_transform_mirror_horizontal() {
    let jsonl = include_str!("../../../examples/demos/sprites/transforms.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    // Verify source sprite exists
    sprite_registry
        .resolve("arrow_right", &palette_registry, false)
        .expect("Source sprite 'arrow_right' should resolve");

    // Verify transformed sprite resolves
    sprite_registry
        .resolve("arrow_left", &palette_registry, false)
        .expect("Transformed sprite 'arrow_left' should resolve");

    // Mirroring preserves dimensions (use capture_render_info for dimensions)
    let info = capture_render_info(jsonl, "arrow_left");
    assert_eq!(info.width, 3, "Mirrored sprite width should be 3");
    assert_eq!(info.height, 3, "Mirrored sprite height should be 3");
}

/// @demo format/sprite#rotate
/// @title Rotation Transform
/// @description Sprite rotated 90 degrees using transform: ["rotate:90"].
#[test]
    #[ignore = "Grid format deprecated"]
fn test_transform_rotate() {
    let jsonl = include_str!("../../../examples/demos/sprites/transforms.jsonl");

    // arrow_right is 3x3 (width × height)
    assert_dimensions(jsonl, "arrow_right", 3, 3);

    // arrow_down is rotated 90 degrees - dimensions swap for non-square
    // For 3x3 source, rotation preserves dimensions
    assert_dimensions(jsonl, "arrow_down", 3, 3);
}

/// @demo format/sprite#scale
/// @title Scale Transform
/// @description Sprite scaled 2x using transform: ["scale:2.0,2.0"].
#[test]
    #[ignore = "Grid format deprecated"]
fn test_transform_scale() {
    let jsonl = include_str!("../../../examples/demos/sprites/transforms.jsonl");

    // arrow_right is 3x3
    assert_dimensions(jsonl, "arrow_right", 3, 3);

    // arrow_scaled uses scale:2.0,2.0 - should be 6x6
    assert_dimensions(jsonl, "arrow_scaled", 6, 6);
}

/// @demo format/sprite#source_ref
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

/// @demo format/sprite#transform_chain
/// @title Transform Chain Verification
/// @description Verifies parsed transform specs match expected operations.
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
