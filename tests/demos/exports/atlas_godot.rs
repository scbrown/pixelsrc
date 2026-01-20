//! Godot Atlas Export Demo (DT-8)
//!
//! Demonstrates exporting atlas metadata to Godot engine format (.tres files).
//! Includes AtlasTexture for sprites, SpriteFrames for AnimatedSprite2D,
//! and AnimationLibrary for AnimationPlayer.

use pixelsrc::atlas::{AtlasAnimation, AtlasFrame, AtlasMetadata};
use pixelsrc::export::godot::{export_godot, GodotExporter};
use pixelsrc::export::Exporter;
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::collections::HashMap;
use std::io::Cursor;
use tempfile::TempDir;

/// Build AtlasMetadata from parsed JSONL content.
fn build_atlas_metadata(jsonl: &str) -> AtlasMetadata {
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let mut frames = HashMap::new();
    let mut animations = HashMap::new();
    let mut frame_index = 0u32;

    for obj in parse_result.objects {
        match obj {
            TtpObject::Sprite(s) => {
                // Simulate atlas placement (sequential horizontal layout)
                let w = s.grid[0].len() as u32;
                let h = s.grid.len() as u32;
                let origin = s.metadata.as_ref().and_then(|m| m.origin);

                frames.insert(
                    s.name.clone(),
                    AtlasFrame {
                        x: frame_index * 32, // Simulated position
                        y: 0,
                        w,
                        h,
                        origin,
                        boxes: None,
                    },
                );
                frame_index += 1;
            }
            TtpObject::Animation(a) => {
                animations.insert(
                    a.name.clone(),
                    AtlasAnimation {
                        frames: a.frames.clone(),
                        fps: 1000 / a.duration_ms().max(1), // Convert ms/frame to fps
                        tags: None,
                    },
                );
            }
            _ => {}
        }
    }

    AtlasMetadata { image: "sprites.png".to_string(), size: [256, 64], frames, animations }
}

/// @demo export/atlas#godot
/// @title Godot Atlas Export
/// @description Export atlas metadata to Godot .tres resource files for game character sprites.
#[test]
fn test_atlas_godot_export() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_godot.jsonl");

    // Build atlas metadata from parsed content
    let metadata = build_atlas_metadata(jsonl);

    // Verify we have expected frames
    assert!(metadata.frames.contains_key("player_idle"), "Should have player_idle frame");
    assert!(metadata.frames.contains_key("player_walk_1"), "Should have player_walk_1 frame");
    assert!(metadata.frames.contains_key("player_walk_2"), "Should have player_walk_2 frame");
    assert!(metadata.frames.contains_key("coin"), "Should have coin frame");

    // Verify animation is present
    assert!(metadata.animations.contains_key("walk"), "Should have walk animation");
    let walk_anim = &metadata.animations["walk"];
    assert_eq!(walk_anim.frames.len(), 2, "Walk animation should have 2 frames");
    assert_eq!(walk_anim.fps, 5, "Walk animation should be 5 fps (1000ms / 200ms duration)");
}

/// @demo export/atlas#godot_files
/// @title Godot Export File Generation
/// @description Generates AtlasTexture, SpriteFrames, and AnimationLibrary .tres files.
#[test]
fn test_atlas_godot_file_generation() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_godot.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    let outputs = export_godot(&metadata, temp.path(), "res://assets/sprites").unwrap();

    // Should create AtlasTexture for each frame
    assert!(temp.path().join("player_idle.tres").exists(), "Should create player_idle.tres");
    assert!(temp.path().join("player_walk_1.tres").exists(), "Should create player_walk_1.tres");
    assert!(temp.path().join("player_walk_2.tres").exists(), "Should create player_walk_2.tres");
    assert!(temp.path().join("coin.tres").exists(), "Should create coin.tres");

    // Should create SpriteFrames for animations
    assert!(temp.path().join("sprites_frames.tres").exists(), "Should create SpriteFrames file");

    // Should create AnimationLibrary for AnimationPlayer
    assert!(temp.path().join("sprites_anims.tres").exists(), "Should create AnimationLibrary file");

    // Verify we have all expected output files
    assert!(outputs.len() >= 6, "Should generate at least 6 files (4 frames + frames + anims)");
}

/// @demo export/atlas#godot_content
/// @title Godot Export Content Verification
/// @description Verify AtlasTexture contains correct Rect2 and resource references.
#[test]
fn test_atlas_godot_content() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_godot.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    export_godot(&metadata, temp.path(), "res://game/sprites").unwrap();

    // Check AtlasTexture content
    let player_idle_content =
        std::fs::read_to_string(temp.path().join("player_idle.tres")).unwrap();
    assert!(
        player_idle_content.contains("[gd_resource type=\"AtlasTexture\""),
        "Should be AtlasTexture resource"
    );
    assert!(
        player_idle_content.contains("res://game/sprites/sprites.png"),
        "Should reference correct texture path"
    );
    assert!(player_idle_content.contains("Rect2"), "Should contain Rect2 region");

    // Check SpriteFrames content
    let frames_content = std::fs::read_to_string(temp.path().join("sprites_frames.tres")).unwrap();
    assert!(
        frames_content.contains("[gd_resource type=\"SpriteFrames\""),
        "Should be SpriteFrames resource"
    );
    assert!(frames_content.contains("\"name\": &\"walk\""), "Should contain walk animation");
    assert!(frames_content.contains("\"loop\": true"), "Animation should loop");

    // Check AnimationLibrary content
    let anims_content = std::fs::read_to_string(temp.path().join("sprites_anims.tres")).unwrap();
    assert!(
        anims_content.contains("[gd_resource type=\"AnimationLibrary\""),
        "Should be AnimationLibrary resource"
    );
    assert!(anims_content.contains("Animation_walk"), "Should contain walk animation");
    assert!(anims_content.contains("tracks/0/path = NodePath(\".:texture\")"), "Should animate texture property");
}

/// @demo export/atlas#godot_origin
/// @title Godot Export with Sprite Origins
/// @description Sprites with origin metadata preserve positioning information.
#[test]
fn test_atlas_godot_origin_preserved() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_godot.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Check that sprites with origins have them preserved
    if let Some(player_idle) = metadata.frames.get("player_idle") {
        // player_idle has origin [2, 3] in the fixture
        assert!(player_idle.origin.is_some(), "player_idle should have origin metadata");
        let origin = player_idle.origin.unwrap();
        assert_eq!(origin, [2, 3], "Origin should be [2, 3]");
    }
}

/// @demo export/atlas#godot_exporter
/// @title Godot Exporter Configuration
/// @description Configure Godot exporter with custom resource paths and options.
#[test]
fn test_atlas_godot_exporter_config() {
    let exporter = GodotExporter::new()
        .with_resource_path("res://custom/path")
        .with_sprite_frames(true)
        .with_animation_player(true);

    assert_eq!(exporter.format_name(), "godot");
    assert_eq!(exporter.extension(), "tres");
}
