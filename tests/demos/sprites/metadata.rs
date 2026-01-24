//! Sprite metadata demos
//!
//! Origin points, collision boxes, and attachment points.

use crate::demos::{assert_dimensions, assert_validates, parse_content};

/// @demo format/sprite#origin
/// @title Origin Point
/// @description Sprite with an origin point `[x, y]` for positioning and rotation.
#[test]
fn test_metadata_origin() {
    let jsonl = include_str!("../../../examples/demos/sprites/metadata.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry.get_sprite("player").expect("Sprite 'player' should exist");

    let metadata = sprite.metadata.as_ref().expect("Player sprite should have metadata");

    // Origin at [2, 3] - bottom center of the character
    let origin = metadata.origin.expect("Player sprite should have origin");
    assert_eq!(origin, [2, 3], "Origin should be at [2, 3]");

    // Verify sprite renders
    sprite_registry
        .resolve("player", &palette_registry, false)
        .expect("Player sprite should resolve");
}

/// @demo format/sprite#collision
/// @title Collision Boxes
/// @description Sprite with named collision boxes for hitbox detection.
#[test]
fn test_metadata_collision_boxes() {
    let jsonl = include_str!("../../../examples/demos/sprites/metadata.jsonl");

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry.get_sprite("player").expect("Sprite 'player' should exist");

    let metadata = sprite.metadata.as_ref().expect("Player sprite should have metadata");

    let boxes = metadata.boxes.as_ref().expect("Player sprite should have collision boxes");

    // Verify "hurt" box (full sprite bounds)
    let hurt = boxes.get("hurt").expect("Should have 'hurt' box");
    assert_eq!(hurt.x, 0, "hurt.x should be 0");
    assert_eq!(hurt.y, 0, "hurt.y should be 0");
    assert_eq!(hurt.w, 5, "hurt.w should be 5");
    assert_eq!(hurt.h, 4, "hurt.h should be 4");

    // Verify "hit" box (smaller, offensive hitbox)
    let hit = boxes.get("hit").expect("Should have 'hit' box");
    assert_eq!(hit.x, 1, "hit.x should be 1");
    assert_eq!(hit.y, 1, "hit.y should be 1");
    assert_eq!(hit.w, 3, "hit.w should be 3");
    assert_eq!(hit.h, 2, "hit.h should be 2");
}

/// @demo format/sprite#attachments
/// @title Attachment Points
/// @description Sprite with attach_in and attach_out points for chaining sprites.
#[test]
fn test_metadata_attachments() {
    let jsonl = include_str!("../../../examples/demos/sprites/metadata.jsonl");

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry.get_sprite("player").expect("Sprite 'player' should exist");

    let metadata = sprite.metadata.as_ref().expect("Player sprite should have metadata");

    // Verify attach_in point (where this sprite connects to parent)
    let attach_in = metadata.attach_in.expect("Should have attach_in point");
    assert_eq!(attach_in, [2, 0], "attach_in should be at [2, 0] (top center)");

    // Verify attach_out point (where next segment attaches)
    let attach_out = metadata.attach_out.expect("Should have attach_out point");
    assert_eq!(attach_out, [2, 4], "attach_out should be at [2, 4] (bottom center)");
}

/// @demo format/sprite#metadata_dimensions
/// @title Metadata Sprite Dimensions
/// @description Verifies sprite with metadata renders at expected dimensions.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_metadata_sprite_dimensions() {
    let jsonl = include_str!("../../../examples/demos/sprites/metadata.jsonl");

    // Player sprite is 5x4 pixels
    assert_dimensions(jsonl, "player", 5, 4);
}
