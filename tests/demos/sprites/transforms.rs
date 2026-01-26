//! Sprite transform demos
//!
//! Flipping, rotating, scaling, and recoloring sprites using transform chains.

use crate::demos::parse_content;
/// @title Rotation Transform
/// @description Sprite rotated 90 degrees using transform: ["rotate:90"]./// @demo format/sprite#scale
/// @title Scale Transform
/// @description Sprite scaled 2x using transform: ["scale:2.0,2.0"]./// @demo format/sprite#source_ref
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
