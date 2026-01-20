//! Import Demo Tests
//!
//! Demonstrates PNG import functionality for converting raster images to Pixelsrc format.
//! Each test creates a small test PNG and verifies the import produces correct JSONL output.

use image::{Rgba, RgbaImage};
use pixelsrc::import::import_png;
use std::path::PathBuf;

/// Get the fixtures directory path for import demos.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/demos/imports")
}

/// Ensure a test PNG exists, creating it if necessary.
fn ensure_png(name: &str, create_fn: fn() -> RgbaImage) -> PathBuf {
    let path = fixtures_dir().join(format!("{}.png", name));
    if !path.exists() {
        let img = create_fn();
        img.save(&path).expect("Failed to save test PNG");
    }
    path
}

/// Create a simple 4x4 pixel art image with 4 distinct colors.
fn create_basic_4x4() -> RgbaImage {
    let mut img = RgbaImage::new(4, 4);
    let red = Rgba([255, 0, 0, 255]);
    let green = Rgba([0, 255, 0, 255]);
    let blue = Rgba([0, 0, 255, 255]);
    let yellow = Rgba([255, 255, 0, 255]);

    // Create a simple pattern
    for y in 0..4 {
        for x in 0..4 {
            let color = match (x / 2, y / 2) {
                (0, 0) => red,
                (1, 0) => green,
                (0, 1) => blue,
                (1, 1) => yellow,
                _ => red,
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}

/// Create an 8x8 image with a gradient for palette detection testing.
fn create_gradient_8x8() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);
    for y in 0..8 {
        for x in 0..8 {
            // Create 16 distinct colors (4x4 blocks of 2x2 pixels each)
            let r = ((x / 2) * 85) as u8;
            let g = ((y / 2) * 85) as u8;
            let b = 128u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

/// Create a 16x4 spritesheet with 4 frames for multi-frame import testing.
fn create_spritesheet_16x4() -> RgbaImage {
    let mut img = RgbaImage::new(16, 4);
    let colors = [
        Rgba([255, 0, 0, 255]),   // Frame 1: Red
        Rgba([0, 255, 0, 255]),   // Frame 2: Green
        Rgba([0, 0, 255, 255]),   // Frame 3: Blue
        Rgba([255, 255, 0, 255]), // Frame 4: Yellow
    ];

    for frame in 0..4 {
        let color = colors[frame];
        for y in 0..4 {
            for x in 0..4 {
                img.put_pixel(frame as u32 * 4 + x, y, color);
            }
        }
    }
    img
}

/// Create a 4x4 image with transparency for transparency handling testing.
fn create_with_transparency_4x4() -> RgbaImage {
    let mut img = RgbaImage::new(4, 4);
    let red = Rgba([255, 0, 0, 255]);
    let transparent = Rgba([0, 0, 0, 0]);

    // Create checkerboard with transparency
    for y in 0..4 {
        for x in 0..4 {
            let color = if (x + y) % 2 == 0 { red } else { transparent };
            img.put_pixel(x, y, color);
        }
    }
    img
}

// ============================================================================
// Demo Tests
// ============================================================================

/// @demo import/png#basic
/// @title Basic PNG to JSONL
/// @description Import a simple PNG image and convert to Pixelsrc JSONL format.
#[test]
fn test_png_to_jsonl() {
    let path = ensure_png("basic_4x4", create_basic_4x4);

    let result = import_png(&path, "basic_sprite", 16).expect("Import should succeed");

    // Verify dimensions
    assert_eq!(result.width, 4, "Width should be 4 pixels");
    assert_eq!(result.height, 4, "Height should be 4 pixels");
    assert_eq!(result.name, "basic_sprite");

    // Verify we have the expected number of colors (4 distinct colors)
    assert_eq!(result.palette.len(), 4, "Should detect 4 colors");

    // Verify grid has correct number of rows
    assert_eq!(result.grid.len(), 4, "Grid should have 4 rows");

    // Verify JSONL output is valid JSON
    let jsonl = result.to_jsonl();
    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2, "JSONL should have 2 lines (palette + sprite)");

    // Verify first line is palette
    let palette_json: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Palette line should be valid JSON");
    assert_eq!(palette_json["type"], "palette");

    // Verify second line is sprite
    let sprite_json: serde_json::Value =
        serde_json::from_str(lines[1]).expect("Sprite line should be valid JSON");
    assert_eq!(sprite_json["type"], "sprite");
    assert_eq!(sprite_json["name"], "basic_sprite");
}

/// @demo import/png#palette_detection
/// @title Auto-detect Palette from Image
/// @description Import extracts unique colors and generates token mappings automatically.
#[test]
fn test_palette_detection() {
    let path = ensure_png("gradient_8x8", create_gradient_8x8);

    // Import with max 16 colors (image has 16 unique colors in 4x4 block pattern)
    let result = import_png(&path, "gradient", 16).expect("Import should succeed");

    assert_eq!(result.width, 8);
    assert_eq!(result.height, 8);

    // Should detect up to 16 colors
    assert!(result.palette.len() <= 16, "Should have at most 16 colors");
    assert!(result.palette.len() >= 4, "Should have at least 4 colors");

    // Verify palette entries are valid hex colors
    for (token, hex) in &result.palette {
        assert!(token.starts_with('{') && token.ends_with('}'), "Token should be wrapped: {token}");
        assert!(hex.starts_with('#'), "Hex should start with #: {hex}");
    }

    // Test quantization by limiting colors
    let quantized = import_png(&path, "gradient_quantized", 4).expect("Import should succeed");
    assert!(quantized.palette.len() <= 4, "Quantization should limit to 4 colors");
}

/// @demo import/png#multi_frame
/// @title Import Spritesheet as Animation
/// @description Import a horizontal spritesheet where each frame becomes a sprite.
#[test]
fn test_multi_frame_import() {
    let path = ensure_png("spritesheet_16x4", create_spritesheet_16x4);

    // Import the full spritesheet as a single sprite
    let result = import_png(&path, "spritesheet", 8).expect("Import should succeed");

    // Full spritesheet dimensions
    assert_eq!(result.width, 16, "Full width should be 16 pixels (4 frames × 4px)");
    assert_eq!(result.height, 4, "Height should be 4 pixels");

    // Should detect 4 distinct colors (one per frame)
    assert_eq!(result.palette.len(), 4, "Should have 4 colors (one per frame)");

    // Verify grid captures all 4 frames
    assert_eq!(result.grid.len(), 4, "Grid should have 4 rows");

    // Each row should have 16 tokens (4 frames × 4 pixels)
    for row in &result.grid {
        // Count tokens by counting '{' occurrences
        let token_count = row.matches('{').count();
        assert_eq!(token_count, 16, "Each row should have 16 tokens");
    }
}

/// @demo import/png#transparency
/// @title Preserve/Detect Transparency
/// @description Import handles transparent pixels with special {_} token.
#[test]
fn test_transparency_handling() {
    let path = ensure_png("transparent_4x4", create_with_transparency_4x4);

    let result = import_png(&path, "transparent_sprite", 8).expect("Import should succeed");

    assert_eq!(result.width, 4);
    assert_eq!(result.height, 4);

    // Should have 2 colors: red and transparent
    assert_eq!(result.palette.len(), 2, "Should have 2 colors (red + transparent)");

    // Verify transparent token exists
    assert!(result.palette.contains_key("{_}"), "Should have transparent token {{_}}");

    // Verify transparent color value
    let transparent_hex = &result.palette["{_}"];
    assert!(
        transparent_hex.contains("00000000") || transparent_hex == "#00000000",
        "Transparent should be #00000000, got {transparent_hex}"
    );

    // Verify grid contains both tokens in checkerboard pattern
    let jsonl = result.to_jsonl();
    assert!(jsonl.contains("{_}"), "JSONL should contain transparent token");

    // Verify the JSONL can be parsed back
    let sprite_json: serde_json::Value =
        serde_json::from_str(jsonl.lines().nth(1).unwrap()).expect("Should parse");
    let grid = sprite_json["grid"].as_array().expect("Should have grid");
    assert_eq!(grid.len(), 4, "Grid should have 4 rows");
}

/// Test round-trip: import PNG → JSONL → verify structure
#[test]
fn test_import_jsonl_structure() {
    let path = ensure_png("basic_4x4", create_basic_4x4);
    let result = import_png(&path, "roundtrip", 16).expect("Import should succeed");
    let jsonl = result.to_jsonl();

    // Parse and verify structure
    let lines: Vec<&str> = jsonl.lines().collect();

    // Palette line
    let palette: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(palette["type"], "palette");
    assert_eq!(palette["name"], "roundtrip_palette");
    assert!(palette["colors"].is_object());

    // Sprite line
    let sprite: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(sprite["type"], "sprite");
    assert_eq!(sprite["name"], "roundtrip");
    assert_eq!(sprite["palette"], "roundtrip_palette");
    assert_eq!(sprite["size"][0], 4);
    assert_eq!(sprite["size"][1], 4);
    assert!(sprite["grid"].is_array());
}

/// Test import with color quantization (reducing many colors to few)
#[test]
fn test_color_quantization() {
    let path = ensure_png("gradient_8x8", create_gradient_8x8);

    // Import with very limited palette
    let result = import_png(&path, "quantized", 2).expect("Import should succeed");

    // Should have at most 2 colors after quantization
    assert!(
        result.palette.len() <= 2,
        "Quantization should limit to 2 colors, got {}",
        result.palette.len()
    );

    // Grid should still have correct dimensions
    assert_eq!(result.grid.len(), 8, "Grid rows should be preserved");
}
