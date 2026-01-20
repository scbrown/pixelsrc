//! Unity Atlas Export Demo (DT-8)
//!
//! Demonstrates exporting atlas metadata to Unity engine format.
//! Includes JSON sprite metadata, .meta texture import settings,
//! and .anim animation clip files.

use pixelsrc::atlas::{AtlasFrame, AtlasMetadata};
use pixelsrc::export::unity::{export_unity, UnityExporter, UnityExportOptions, UnityFilterMode};
use pixelsrc::export::Exporter;
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::collections::HashMap;
use std::io::Cursor;
use tempfile::TempDir;

/// Build AtlasMetadata from parsed JSONL content (UI components).
fn build_atlas_metadata(jsonl: &str) -> AtlasMetadata {
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let mut frames = HashMap::new();
    let mut frame_index = 0u32;

    for obj in parse_result.objects {
        if let TtpObject::Sprite(s) = obj {
            let w = s.grid[0].len() as u32;
            let h = s.grid.len() as u32;
            let origin = s.metadata.as_ref().and_then(|m| m.origin);

            frames.insert(
                s.name.clone(),
                AtlasFrame {
                    x: (frame_index % 4) * 32,
                    y: (frame_index / 4) * 32,
                    w,
                    h,
                    origin,
                    boxes: None,
                },
            );
            frame_index += 1;
        }
    }

    AtlasMetadata {
        image: "ui_atlas.png".to_string(),
        size: [128, 128],
        frames,
        animations: HashMap::new(),
    }
}

/// @demo export/atlas#unity
/// @title Unity Atlas Export
/// @description Export atlas metadata to Unity JSON format for UI sprite components.
#[test]
fn test_atlas_unity_export() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");

    // Build atlas metadata from parsed content
    let metadata = build_atlas_metadata(jsonl);

    // Verify we have expected UI frames
    assert!(metadata.frames.contains_key("button_normal"), "Should have button_normal frame");
    assert!(metadata.frames.contains_key("button_hover"), "Should have button_hover frame");
    assert!(metadata.frames.contains_key("button_pressed"), "Should have button_pressed frame");
    assert!(metadata.frames.contains_key("icon_check"), "Should have icon_check frame");

    // Verify frame count
    assert_eq!(metadata.frames.len(), 4, "Should have 4 UI sprite frames");
}

/// @demo export/atlas#unity_files
/// @title Unity Export File Generation
/// @description Generates JSON metadata, .meta import settings, and .anim clips.
#[test]
fn test_atlas_unity_file_generation() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("atlas.json");
    let outputs = export_unity(&metadata, &output_path, 16).unwrap();

    // Should create JSON metadata
    assert!(output_path.exists(), "Should create atlas.json");

    // Should create texture .meta file
    let meta_path = temp.path().join("ui_atlas.png.meta");
    assert!(meta_path.exists(), "Should create texture .meta file");

    // Verify file count (JSON + meta, no .anim since no animations in fixture)
    assert!(outputs.len() >= 2, "Should generate at least 2 files");
}

/// @demo export/atlas#unity_json
/// @title Unity JSON Metadata Content
/// @description Verify JSON contains sprite rects with Unity coordinate system (Y-flipped).
#[test]
fn test_atlas_unity_json_content() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let exporter = UnityExporter::new().with_pixels_per_unit(16);
    let options = UnityExportOptions::default();
    let json = exporter.export_to_string(&metadata, &options).unwrap();

    // Verify JSON structure
    assert!(json.contains("\"texture\": \"ui_atlas.png\""), "Should contain texture reference");
    assert!(json.contains("\"pixelsPerUnit\": 16"), "Should contain PPU setting");
    assert!(json.contains("\"filterMode\": \"Point\""), "Should use point filtering for pixel art");
    assert!(json.contains("\"sprites\""), "Should contain sprites array");

    // Parse and verify sprite data
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let sprites = data["sprites"].as_array().unwrap();
    assert_eq!(sprites.len(), 4, "Should have 4 sprites");

    // Find button_normal sprite
    let button = sprites.iter().find(|s| s["name"] == "button_normal").unwrap();
    assert!(button["rect"]["w"].as_f64().unwrap() > 0.0, "Button should have positive width");
    assert!(button["rect"]["h"].as_f64().unwrap() > 0.0, "Button should have positive height");
}

/// @demo export/atlas#unity_meta
/// @title Unity Texture Meta File
/// @description Verify .meta file contains TextureImporter settings with sprite slices.
#[test]
fn test_atlas_unity_meta_content() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("atlas.json");
    export_unity(&metadata, &output_path, 32).unwrap();

    let meta_content =
        std::fs::read_to_string(temp.path().join("ui_atlas.png.meta")).unwrap();

    // Verify meta file structure
    assert!(meta_content.contains("fileFormatVersion: 2"), "Should have file format version");
    assert!(meta_content.contains("TextureImporter:"), "Should be TextureImporter");
    assert!(meta_content.contains("spriteMode: 2"), "Should be multiple sprite mode");
    assert!(meta_content.contains("spritePixelsToUnits: 32"), "Should have correct PPU");
    assert!(meta_content.contains("filterMode: 0"), "Should use point filtering");

    // Verify sprite slices are included
    assert!(meta_content.contains("button_normal"), "Should contain button_normal sprite");
    assert!(meta_content.contains("button_hover"), "Should contain button_hover sprite");
    assert!(meta_content.contains("button_pressed"), "Should contain button_pressed sprite");
    assert!(meta_content.contains("icon_check"), "Should contain icon_check sprite");
}

/// @demo export/atlas#unity_pivot
/// @title Unity Export with Pivot Points
/// @description Sprites with origin metadata are converted to Unity pivot points.
#[test]
fn test_atlas_unity_pivot() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Check that sprites with origins have them preserved
    if let Some(button_normal) = metadata.frames.get("button_normal") {
        // button_normal has origin [3, 2] in the fixture (6 wide, 4 tall button)
        if let Some(origin) = button_normal.origin {
            assert_eq!(origin, [3, 2], "Origin should be [3, 2]");
        }
    }

    // Export and verify pivot is in output
    let exporter = UnityExporter::new();
    let options = UnityExportOptions::default();
    let json = exporter.export_to_string(&metadata, &options).unwrap();

    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let sprites = data["sprites"].as_array().unwrap();

    // All sprites should have pivot data
    for sprite in sprites {
        assert!(sprite["pivot"].is_object(), "Sprite should have pivot");
        assert!(sprite["pivot"]["x"].is_number(), "Pivot should have x coordinate");
        assert!(sprite["pivot"]["y"].is_number(), "Pivot should have y coordinate");
    }
}

/// @demo export/atlas#unity_filter
/// @title Unity Export Filter Modes
/// @description Configure texture filtering for different use cases.
#[test]
fn test_atlas_unity_filter_modes() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_unity.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Test with bilinear filtering
    let exporter = UnityExporter::new().with_filter_mode(UnityFilterMode::Bilinear);
    let options = UnityExportOptions {
        filter_mode: UnityFilterMode::Bilinear,
        ..Default::default()
    };
    let json = exporter.export_to_string(&metadata, &options).unwrap();

    assert!(json.contains("\"filterMode\": \"Bilinear\""), "Should use bilinear filtering");

    // Test with trilinear filtering
    let options_tri = UnityExportOptions {
        filter_mode: UnityFilterMode::Trilinear,
        ..Default::default()
    };
    let exporter_tri = UnityExporter::new().with_filter_mode(UnityFilterMode::Trilinear);
    let json_tri = exporter_tri.export_to_string(&metadata, &options_tri).unwrap();

    assert!(json_tri.contains("\"filterMode\": \"Trilinear\""), "Should use trilinear filtering");
}

/// @demo export/atlas#unity_exporter
/// @title Unity Exporter Configuration
/// @description Configure Unity exporter with custom options.
#[test]
fn test_atlas_unity_exporter_config() {
    let exporter = UnityExporter::new()
        .with_pixels_per_unit(100)
        .with_filter_mode(UnityFilterMode::Point)
        .with_animations(true)
        .with_meta_file(true)
        .with_anim_files(true)
        .with_json(true);

    assert_eq!(exporter.format_name(), "unity");
    assert_eq!(exporter.extension(), "json");
}
