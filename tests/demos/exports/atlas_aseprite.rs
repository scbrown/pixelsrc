//! Aseprite Atlas Export Demo
//!
//! Demonstrates exporting atlas metadata to Aseprite-compatible JSON format.
//! The Aseprite JSON format is widely supported by game engines and sprite tools.

use pixelsrc::atlas::{AtlasAnimation, AtlasFrame, AtlasMetadata};
use std::collections::HashMap;

/// Build sample AtlasMetadata for Aseprite export testing.
fn build_aseprite_metadata() -> AtlasMetadata {
    let mut frames = HashMap::new();
    frames.insert(
        "player_idle".to_string(),
        AtlasFrame { x: 0, y: 0, w: 16, h: 16, origin: None, boxes: None },
    );
    frames.insert(
        "player_walk_1".to_string(),
        AtlasFrame { x: 16, y: 0, w: 16, h: 16, origin: None, boxes: None },
    );
    frames.insert(
        "player_walk_2".to_string(),
        AtlasFrame { x: 32, y: 0, w: 16, h: 16, origin: None, boxes: None },
    );

    let mut animations = HashMap::new();
    animations.insert(
        "walk".to_string(),
        AtlasAnimation {
            frames: vec!["player_walk_1".to_string(), "player_walk_2".to_string()],
            fps: 8,
            tags: None,
        },
    );

    AtlasMetadata { image: "sprites.png".to_string(), size: [64, 16], frames, animations }
}

/// Format AtlasMetadata as Aseprite-compatible JSON.
///
/// Produces the standard Aseprite JSON hash format with `frames` and `meta` sections.
fn format_aseprite_json(metadata: &AtlasMetadata) -> serde_json::Value {
    let frames: serde_json::Map<String, serde_json::Value> = metadata
        .frames
        .iter()
        .map(|(name, frame)| {
            (
                format!("{}.png", name),
                serde_json::json!({
                    "frame": {"x": frame.x, "y": frame.y, "w": frame.w, "h": frame.h},
                    "rotated": false,
                    "trimmed": false,
                    "spriteSourceSize": {"x": 0, "y": 0, "w": frame.w, "h": frame.h},
                    "sourceSize": {"w": frame.w, "h": frame.h}
                }),
            )
        })
        .collect();

    let meta = serde_json::json!({
        "app": "pixelsrc",
        "version": "1.0",
        "image": metadata.image,
        "format": "RGBA8888",
        "size": {"w": metadata.size[0], "h": metadata.size[1]},
        "scale": "1"
    });

    serde_json::json!({
        "frames": frames,
        "meta": meta
    })
}

/// @demo export/atlas#aseprite
/// @title Aseprite JSON Atlas
/// @description Export atlas metadata to Aseprite-compatible JSON format with frames and meta sections.
#[test]
fn test_atlas_aseprite_export() {
    let metadata = build_aseprite_metadata();
    let json = format_aseprite_json(&metadata);

    // Verify top-level structure matches Aseprite format
    assert!(json.get("frames").is_some(), "Should have 'frames' section");
    assert!(json.get("meta").is_some(), "Should have 'meta' section");

    // Verify meta section
    let meta = &json["meta"];
    assert_eq!(meta["app"], "pixelsrc");
    assert_eq!(meta["image"], "sprites.png");
    assert_eq!(meta["format"], "RGBA8888");
    assert_eq!(meta["size"]["w"], 64);
    assert_eq!(meta["size"]["h"], 16);

    // Verify frames use Aseprite naming convention (name.png)
    let frames = json["frames"].as_object().unwrap();
    assert!(frames.contains_key("player_idle.png"), "Frame key should use .png suffix");
    assert!(frames.contains_key("player_walk_1.png"), "Frame key should use .png suffix");
    assert!(frames.contains_key("player_walk_2.png"), "Frame key should use .png suffix");

    // Verify frame structure matches Aseprite format
    let idle = &frames["player_idle.png"];
    assert_eq!(idle["frame"]["x"], 0);
    assert_eq!(idle["frame"]["y"], 0);
    assert_eq!(idle["frame"]["w"], 16);
    assert_eq!(idle["frame"]["h"], 16);
    assert_eq!(idle["rotated"], false);
    assert_eq!(idle["trimmed"], false);
    assert_eq!(idle["spriteSourceSize"]["w"], 16);
    assert_eq!(idle["sourceSize"]["w"], 16);
}

/// @demo export/atlas#aseprite_frame_positions
/// @title Aseprite Frame Positions
/// @description Verifies atlas frame coordinates are correctly mapped to Aseprite JSON format.
#[test]
fn test_atlas_aseprite_frame_positions() {
    let metadata = build_aseprite_metadata();
    let json = format_aseprite_json(&metadata);
    let frames = json["frames"].as_object().unwrap();

    // Walk frames should have sequential x positions
    let walk1 = &frames["player_walk_1.png"];
    assert_eq!(walk1["frame"]["x"], 16, "walk_1 should start at x=16");

    let walk2 = &frames["player_walk_2.png"];
    assert_eq!(walk2["frame"]["x"], 32, "walk_2 should start at x=32");
}
