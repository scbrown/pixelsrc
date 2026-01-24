//! libGDX Atlas Export Demo (DT-8)
//!
//! Demonstrates exporting atlas metadata to libGDX TextureAtlas format (.atlas).
//! The .atlas format is a text-based format used by libGDX's TexturePacker
//! and loadable via the TextureAtlas class.

use pixelsrc::atlas::{AtlasAnimation, AtlasFrame, AtlasMetadata};
use pixelsrc::export::libgdx::{
    LibGdxExportOptions, LibGdxExporter, LibGdxFilterMode, LibGdxRepeatMode,
};
use pixelsrc::export::{ExportOptions, Exporter};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::collections::HashMap;
use std::io::Cursor;
use tempfile::TempDir;

/// Build AtlasMetadata from parsed JSONL content (RPG items).
fn build_atlas_metadata(jsonl: &str) -> AtlasMetadata {
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let mut frames = HashMap::new();
    let mut animations = HashMap::new();
    let mut frame_index = 0u32;

    for obj in parse_result.objects {
        match obj {
            TtpObject::Sprite(s) => {
                let w = s.size.map(|[w, _]| w).unwrap_or(0);
                let h = s.size.map(|[_, h]| h).unwrap_or(0);
                let origin = s.metadata.as_ref().and_then(|m| m.origin);

                frames.insert(
                    s.name.clone(),
                    AtlasFrame {
                        x: (frame_index % 4) * 16,
                        y: (frame_index / 4) * 16,
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

    AtlasMetadata { image: "rpg_items.png".to_string(), size: [64, 64], frames, animations }
}

/// @demo export/atlas#libgdx
/// @title libGDX Atlas Export
/// @description Export atlas metadata to libGDX TextureAtlas format for RPG game items.
#[test]
fn test_atlas_libgdx_export() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");

    // Build atlas metadata from parsed content
    let metadata = build_atlas_metadata(jsonl);

    // Verify we have expected RPG item frames
    assert!(metadata.frames.contains_key("sword"), "Should have sword frame");
    assert!(metadata.frames.contains_key("shield"), "Should have shield frame");
    assert!(metadata.frames.contains_key("potion_1"), "Should have potion_1 frame");
    assert!(metadata.frames.contains_key("potion_2"), "Should have potion_2 frame");

    // Verify animation is present
    assert!(
        metadata.animations.contains_key("potion_shimmer"),
        "Should have potion_shimmer animation"
    );
    let shimmer = &metadata.animations["potion_shimmer"];
    assert_eq!(shimmer.frames.len(), 2, "Shimmer animation should have 2 frames");
}

/// @demo export/atlas#libgdx_format
/// @title libGDX Atlas Format
/// @description Verify the text-based .atlas format structure.
#[test]
fn test_atlas_libgdx_format() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let exporter = LibGdxExporter::new();
    let content = exporter.export_to_string(&metadata);

    // Verify header section
    assert!(content.starts_with("rpg_items.png\n"), "Should start with image filename");
    assert!(content.contains("size: 64, 64\n"), "Should contain atlas size");
    assert!(content.contains("format: RGBA8888\n"), "Should specify pixel format");
    assert!(content.contains("filter: Nearest, Nearest\n"), "Should use nearest filtering");
    assert!(content.contains("repeat: none\n"), "Should specify no repeat");

    // Verify sprite entries
    assert!(content.contains("sword\n"), "Should contain sword sprite");
    assert!(content.contains("shield\n"), "Should contain shield sprite");
    assert!(content.contains("potion_1\n"), "Should contain potion_1 sprite");
    assert!(content.contains("potion_2\n"), "Should contain potion_2 sprite");

    // Verify sprite properties are indented
    assert!(content.contains("  rotate: false\n"), "Should have rotate property");
    assert!(content.contains("  xy:"), "Should have xy position");
    assert!(content.contains("  size:"), "Should have size");
    assert!(content.contains("  orig:"), "Should have orig (original size)");
    assert!(content.contains("  offset:"), "Should have offset");
    assert!(content.contains("  index:"), "Should have index");
}

/// @demo export/atlas#libgdx_animation
/// @title libGDX Animation Indices
/// @description Animation frames get sequential index values for sprite sheet animation.
#[test]
fn test_atlas_libgdx_animation_indices() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let exporter = LibGdxExporter::new();
    let content = exporter.export_to_string(&metadata);

    // Parse and find index values
    let lines: Vec<&str> = content.lines().collect();

    // Find potion_1 and check its index (should be 0 as first animation frame)
    let potion1_idx = lines.iter().position(|l| *l == "potion_1").unwrap();
    let potion1_index_line = lines[potion1_idx + 6]; // index is 6 lines after name
    assert_eq!(potion1_index_line, "  index: 0", "potion_1 should have index 0");

    // Find potion_2 and check its index (should be 1 as second animation frame)
    let potion2_idx = lines.iter().position(|l| *l == "potion_2").unwrap();
    let potion2_index_line = lines[potion2_idx + 6];
    assert_eq!(potion2_index_line, "  index: 1", "potion_2 should have index 1");

    // Non-animated sprites should have index -1
    let sword_idx = lines.iter().position(|l| *l == "sword").unwrap();
    let sword_index_line = lines[sword_idx + 6];
    assert_eq!(sword_index_line, "  index: -1", "sword should have index -1 (not animated)");
}

/// @demo export/atlas#libgdx_origin
/// @title libGDX Export with Offsets
/// @description Sprites with origin metadata export as offset values.
#[test]
fn test_atlas_libgdx_origin_offset() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Verify sword has origin metadata preserved
    if let Some(sword) = metadata.frames.get("sword") {
        // sword has origin [0, 3] in the fixture (bottom-left grip position)
        if let Some(origin) = sword.origin {
            assert_eq!(origin, [0, 3], "Sword origin should be [0, 3]");
        }
    }

    let exporter = LibGdxExporter::new();
    let content = exporter.export_to_string(&metadata);

    // Sprites with origins should have non-zero offsets
    assert!(content.contains("  offset: 0, 3\n"), "Sword should have offset from origin");
}

/// @demo export/atlas#libgdx_file
/// @title libGDX Export File Generation
/// @description Generate .atlas file to disk.
#[test]
fn test_atlas_libgdx_file_generation() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("items.atlas");

    let exporter = LibGdxExporter::new();
    let options = LibGdxExportOptions::default();
    exporter.export_libgdx(&metadata, &output_path, &options).unwrap();

    assert!(output_path.exists(), "Should create .atlas file");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.starts_with("rpg_items.png"), "File should contain atlas data");
}

/// @demo export/atlas#libgdx_filter
/// @title libGDX Filter Modes
/// @description Configure texture filtering for different rendering styles.
#[test]
fn test_atlas_libgdx_filter_modes() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Test with linear filtering
    let exporter = LibGdxExporter::new().with_filter(LibGdxFilterMode::Linear);
    let content = exporter.export_to_string(&metadata);
    assert!(content.contains("filter: Linear, Linear\n"), "Should use linear filtering");

    // Test with mixed filtering
    let exporter_mixed = LibGdxExporter::new()
        .with_min_filter(LibGdxFilterMode::MipMapLinearLinear)
        .with_mag_filter(LibGdxFilterMode::Linear);
    let content_mixed = exporter_mixed.export_to_string(&metadata);
    assert!(
        content_mixed.contains("filter: MipMapLinearLinear, Linear\n"),
        "Should use mixed filtering"
    );
}

/// @demo export/atlas#libgdx_repeat
/// @title libGDX Repeat Modes
/// @description Configure texture wrapping for tiling textures.
#[test]
fn test_atlas_libgdx_repeat_modes() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    // Test X repeat
    let exporter_x = LibGdxExporter::new().with_repeat(LibGdxRepeatMode::X);
    let content_x = exporter_x.export_to_string(&metadata);
    assert!(content_x.contains("repeat: x\n"), "Should repeat in X");

    // Test XY repeat
    let exporter_xy = LibGdxExporter::new().with_repeat(LibGdxRepeatMode::XY);
    let content_xy = exporter_xy.export_to_string(&metadata);
    assert!(content_xy.contains("repeat: xy\n"), "Should repeat in both axes");
}

/// @demo export/atlas#libgdx_exporter
/// @title libGDX Exporter Configuration
/// @description Configure libGDX exporter with custom options.
#[test]
fn test_atlas_libgdx_exporter_config() {
    let exporter = LibGdxExporter::new()
        .with_filter(LibGdxFilterMode::Nearest)
        .with_repeat(LibGdxRepeatMode::None)
        .with_format("RGBA4444");

    assert_eq!(exporter.format_name(), "libGDX TextureAtlas");
    assert_eq!(exporter.extension(), "atlas");
}

/// @demo export/atlas#libgdx_trait
/// @title libGDX Export via Trait
/// @description Use the Exporter trait for generic export handling.
#[test]
fn test_atlas_libgdx_exporter_trait() {
    let jsonl = include_str!("../../../examples/demos/exports/atlas_libgdx.jsonl");
    let metadata = build_atlas_metadata(jsonl);

    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("via_trait.atlas");

    let exporter = LibGdxExporter::new();
    let options = ExportOptions::default();

    exporter.export(&metadata, &output_path, &options).unwrap();

    assert!(output_path.exists(), "Should create file via trait");
}
