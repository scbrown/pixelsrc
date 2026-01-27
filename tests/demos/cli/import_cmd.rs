//! Import Command Demo Tests
//!
//! Demonstrates the `pxl import` command functionality for converting
//! PNG images to Pixelsrc JSONL format.

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
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let img = create_fn();
        img.save(&path).expect("Failed to save test PNG");
    }
    path
}

/// Create a simple 4x4 red square for basic import testing.
fn create_red_square() -> RgbaImage {
    let mut img = RgbaImage::new(4, 4);
    let red = Rgba([255, 0, 0, 255]);
    for y in 0..4 {
        for x in 0..4 {
            img.put_pixel(x, y, red);
        }
    }
    img
}

/// Create a 4x4 image with 4 distinct colors.
fn create_multicolor_4x4() -> RgbaImage {
    let mut img = RgbaImage::new(4, 4);
    let colors = [
        Rgba([255, 0, 0, 255]),   // Red
        Rgba([0, 255, 0, 255]),   // Green
        Rgba([0, 0, 255, 255]),   // Blue
        Rgba([255, 255, 0, 255]), // Yellow
    ];

    for y in 0..4 {
        for x in 0..4 {
            let color = colors[((x / 2) + (y / 2) * 2) as usize % 4];
            img.put_pixel(x, y, color);
        }
    }
    img
}

/// Create a 4x4 checkerboard with transparency.
fn create_checkerboard_transparent() -> RgbaImage {
    let mut img = RgbaImage::new(4, 4);
    let red = Rgba([255, 0, 0, 255]);
    let transparent = Rgba([0, 0, 0, 0]);

    for y in 0..4 {
        for x in 0..4 {
            let color = if (x + y) % 2 == 0 { red } else { transparent };
            img.put_pixel(x, y, color);
        }
    }
    img
}

// ============================================================================
// Basic Import Tests
// ============================================================================
/// @demo cli/core#import
/// @title Import Command
/// @description The pxl import command for converting PNG to JSONL.
#[test]
fn test_import_basic() {
    let path = ensure_png("cli_red_square", create_red_square);

    let result = import_png(&path, "red_square", 16).expect("Import should succeed");

    // Verify basic properties
    assert_eq!(result.width, 4, "Width should be 4 pixels");
    assert_eq!(result.height, 4, "Height should be 4 pixels");
    assert_eq!(result.name, "red_square", "Name should match parameter");

    // Should have just 1 color (solid red)
    assert_eq!(result.palette.len(), 1, "Solid color image should have 1 color");

    // Verify JSONL output is valid
    let jsonl = result.to_jsonl();
    assert!(!jsonl.is_empty(), "JSONL output should not be empty");

    // Should have 2 lines: palette + sprite
    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2, "Output should be palette + sprite lines");
}

/// @demo cli/import#output_spec
/// @title Import Output Specification
/// @description Custom sprite name is used in output.
#[test]
fn test_import_output_specification() {
    let path = ensure_png("cli_red_square", create_red_square);

    let result = import_png(&path, "custom_name", 16).expect("Import should succeed");

    // Verify the sprite name from --name parameter
    assert_eq!(result.name, "custom_name");

    // JSONL would be written to --output path by CLI
    let jsonl = result.to_jsonl();
    let sprite_json: serde_json::Value =
        serde_json::from_str(jsonl.lines().nth(1).unwrap()).expect("Should parse sprite");

    assert_eq!(sprite_json["name"], "custom_name");
}

/// @demo cli/import#max_colors
/// @title Import Max Colors
/// @description Color quantization limits palette size.
#[test]
fn test_import_max_colors() {
    let path = ensure_png("cli_multicolor", create_multicolor_4x4);

    // Import with exact color count
    let result_16 = import_png(&path, "colors_16", 16).expect("Import should succeed");
    assert_eq!(result_16.palette.len(), 4, "Should detect all 4 colors");

    // Import with reduced color count (quantization)
    let result_2 = import_png(&path, "colors_2", 2).expect("Import should succeed");
    assert!(result_2.palette.len() <= 2, "Quantization should limit to 2 colors");
}

/// @demo cli/import#transparency
/// @title Import Transparency
/// @description Transparent pixels are detected and assigned {_} token.
#[test]
fn test_import_transparency() {
    let path = ensure_png("cli_checkerboard", create_checkerboard_transparent);

    let result = import_png(&path, "checkerboard", 16).expect("Import should succeed");

    // Should have 2 colors: red and transparent
    assert_eq!(result.palette.len(), 2, "Should have red + transparent");

    // Verify transparent token exists
    assert!(result.palette.contains_key("{_}"), "Should have {{_}} transparent token");

    // Verify transparent color is fully transparent
    let transparent_color = &result.palette["{_}"];
    assert!(
        transparent_color.ends_with("00") || transparent_color == "#00000000",
        "Transparent should have 00 alpha"
    );

    // Verify grid contains transparent tokens
    let jsonl = result.to_jsonl();
    assert!(jsonl.contains("{_}"), "Grid should contain transparent tokens");
}

/// @demo cli/import#name_from_filename
/// @title Import Name from Filename
/// @description Sprite name can be derived from input filename.
#[test]
fn test_import_name_from_filename() {
    let path = ensure_png("cli_my_sprite_name", create_red_square);

    // When --name is not provided, CLI derives from filename
    // The import function receives the derived name
    let result = import_png(&path, "my_sprite_name", 16).expect("Import should succeed");

    assert_eq!(result.name, "my_sprite_name", "Name should match derived name");

    // Palette name should be based on sprite name
    let jsonl = result.to_jsonl();
    let palette_json: serde_json::Value =
        serde_json::from_str(jsonl.lines().next().unwrap()).expect("Should parse palette");

    assert_eq!(
        palette_json["name"], "my_sprite_name_palette",
        "Palette name should be {{sprite}}_palette"
    );
}

// ============================================================================
// Output Format Tests
// ============================================================================
/// @demo cli/import#jsonl_structure
/// @title Import JSONL Structure
/// @description Output has correct palette and sprite structure.
#[test]
fn test_import_jsonl_structure() {
    let path = ensure_png("cli_multicolor", create_multicolor_4x4);

    let result = import_png(&path, "structured", 16).expect("Import should succeed");
    let jsonl = result.to_jsonl();

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2, "Should have 2 lines");

    // Parse palette line
    let palette: serde_json::Value = serde_json::from_str(lines[0]).expect("Should parse palette");
    assert_eq!(palette["type"], "palette", "First line should be palette");
    assert!(palette["name"].is_string(), "Palette should have name");
    assert!(palette["colors"].is_object(), "Palette should have colors");

    // Parse sprite line
    let sprite: serde_json::Value = serde_json::from_str(lines[1]).expect("Should parse sprite");
    assert_eq!(sprite["type"], "sprite", "Second line should be sprite");
    assert!(sprite["name"].is_string(), "Sprite should have name");
    assert!(sprite["palette"].is_string(), "Sprite should reference palette");
    assert!(sprite["size"].is_array(), "Sprite should have size");
    assert!(sprite["regions"].is_object(), "Sprite should have regions");
}

/// @demo cli/import#token_generation
/// @title Import Token Generation
/// @description Unique tokens are generated for each color.
#[test]
fn test_import_token_generation() {
    let path = ensure_png("cli_multicolor", create_multicolor_4x4);

    let result = import_png(&path, "tokens", 16).expect("Import should succeed");

    // Verify all tokens are unique and properly formatted
    let tokens: Vec<&String> = result.palette.keys().collect();
    let unique_count = tokens.iter().collect::<std::collections::HashSet<_>>().len();

    assert_eq!(unique_count, tokens.len(), "All tokens should be unique");

    for token in &tokens {
        assert!(token.starts_with('{'), "Token should start with {{");
        assert!(token.ends_with('}'), "Token should end with }}");
        assert!(token.len() >= 3, "Token should have content between braces");
    }
}

/// @demo cli/import#grid_tokens
/// @title Import Grid Tokens
/// @description Grid tokens reference palette entries.
#[test]
fn test_import_grid_tokens() {
    let path = ensure_png("cli_red_square", create_red_square);

    let result = import_png(&path, "grid_test", 16).expect("Import should succeed");

    // Grid should have 4 rows (4 pixel height)
    assert_eq!(result.grid.len(), 4, "Grid should have 4 rows");

    // Each row should have 4 tokens (4 pixel width)
    for row in &result.grid {
        let token_count = row.matches('{').count();
        assert_eq!(token_count, 4, "Each row should have 4 tokens");
    }

    // All tokens should reference palette entries
    for row in &result.grid {
        for token in row.split('}').filter(|s| !s.is_empty()) {
            let full_token = format!("{}}}", token);
            assert!(
                result.palette.contains_key(&full_token),
                "Grid token {full_token} should exist in palette"
            );
        }
    }
}
