//! Render Command Demo Tests
//!
//! Demonstrates the `pxl render` command functionality for converting
//! Pixelsrc definitions to PNG output.

use crate::demos::{assert_validates, capture_render_info, capture_spritesheet_info};
use pixelsrc::parser::parse_stream;
use pixelsrc::registry::{PaletteRegistry, SpriteRegistry};
use pixelsrc::renderer::render_resolved;
use pixelsrc::output::scale_image;
use std::io::Cursor;

/// Parse JSONL and build registries for rendering.
fn setup_render(jsonl: &str) -> (PaletteRegistry, SpriteRegistry) {
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let mut palette_registry = PaletteRegistry::new();
    let mut sprite_registry = SpriteRegistry::new();

    for obj in parse_result.objects {
        match obj {
            pixelsrc::models::TtpObject::Palette(p) => palette_registry.register(p),
            pixelsrc::models::TtpObject::Sprite(s) => sprite_registry.register_sprite(s),
            pixelsrc::models::TtpObject::Variant(v) => sprite_registry.register_variant(v),
            _ => {}
        }
    }

    (palette_registry, sprite_registry)
}

// ============================================================================
// Basic Render Tests
// ============================================================================

/// @demo cli/render#basic
/// @title Basic Render Command
/// @description Render a single sprite to PNG using `pxl render input.jsonl`.
#[test]
fn test_render_basic() {
    let jsonl = include_str!("../../../examples/demos/sprites/basic.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry) = setup_render(jsonl);

    // Resolve and render the sprite (what `pxl render` does internally)
    let resolved = sprite_registry
        .resolve("square", &palette_registry, false)
        .expect("Should resolve sprite");

    let (image, warnings) = render_resolved(&resolved);

    // Verify render output
    assert_eq!(image.width(), 3, "Rendered width should be 3 pixels");
    assert_eq!(image.height(), 3, "Rendered height should be 3 pixels");
    assert!(warnings.is_empty(), "Should have no render warnings");
}

/// @demo cli/render#sprite_filter
/// @title Render Specific Sprite
/// @description Render only a named sprite using `pxl render input.jsonl --sprite name`.
#[test]
fn test_render_sprite_filter() {
    // Content with multiple sprites
    let jsonl = r##"{"type": "palette", "name": "colors", "colors": {"{r}": "#FF0000", "{g}": "#00FF00", "{b}": "#0000FF"}}
{"type": "sprite", "name": "red_dot", "palette": "colors", "grid": ["{r}"]}
{"type": "sprite", "name": "green_dot", "palette": "colors", "grid": ["{g}"]}
{"type": "sprite", "name": "blue_dot", "palette": "colors", "grid": ["{b}"]}"##;

    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry) = setup_render(jsonl);

    // Simulate --sprite red_dot flag
    let sprite_name = "red_dot";
    let resolved = sprite_registry
        .resolve(sprite_name, &palette_registry, false)
        .expect("Should resolve filtered sprite");

    let (image, _) = render_resolved(&resolved);

    assert_eq!(image.width(), 1, "red_dot should be 1 pixel wide");
    assert_eq!(image.height(), 1, "red_dot should be 1 pixel tall");

    // Verify specific sprite was rendered (check pixel color)
    let pixel = image.get_pixel(0, 0);
    assert_eq!(pixel[0], 255, "Red channel should be 255");
    assert_eq!(pixel[1], 0, "Green channel should be 0");
    assert_eq!(pixel[2], 0, "Blue channel should be 0");
}

/// @demo cli/render#scale
/// @title Scaled Render Output
/// @description Render with integer scaling using `pxl render input.jsonl --scale 4`.
#[test]
fn test_render_scaled() {
    let jsonl = include_str!("../../../examples/demos/exports/png_scaled.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry) = setup_render(jsonl);

    let resolved = sprite_registry
        .resolve("scalable", &palette_registry, false)
        .expect("Should resolve sprite");

    let (image, _) = render_resolved(&resolved);

    // Base size is 3x3
    assert_eq!(image.width(), 3);
    assert_eq!(image.height(), 3);

    // Apply 4x scale (what --scale 4 does)
    let scaled = scale_image(image, 4);

    assert_eq!(scaled.width(), 12, "4x scaled width should be 12 (3 × 4)");
    assert_eq!(scaled.height(), 12, "4x scaled height should be 12 (3 × 4)");
}

/// @demo cli/render#spritesheet
/// @title Spritesheet Render
/// @description Render animation as spritesheet using `pxl render input.jsonl --spritesheet`.
#[test]
fn test_render_spritesheet() {
    let jsonl = include_str!("../../../examples/demos/exports/spritesheet_horizontal.jsonl");
    assert_validates(jsonl, true);

    // Use the spritesheet capture helper to verify output
    let info = capture_spritesheet_info(jsonl, "walk_cycle", None);

    // 4 frames of 8x8 in horizontal strip
    assert_eq!(info.frame_count, 4, "Should have 4 frames");
    assert_eq!(info.width, 32, "Horizontal strip should be 4 × 8 = 32 pixels wide");
    assert_eq!(info.height, 8, "Horizontal strip should be 8 pixels tall");
}

/// @demo cli/render#named_palette
/// @title Render with Named Palette
/// @description Render sprite that references a named palette definition.
#[test]
fn test_render_named_palette() {
    let jsonl = include_str!("../../../examples/demos/sprites/named_palette.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "icon");

    // Verify sprite renders correctly with named palette
    assert_eq!(info.width, 3, "Icon should be 3 pixels wide");
    assert_eq!(info.height, 3, "Icon should be 3 pixels tall");
    assert_eq!(info.palette_name.as_deref(), Some("retro"), "Should use 'retro' palette");
}

/// @demo cli/render#inline_palette
/// @title Render with Inline Palette
/// @description Render sprite with palette defined inline in the sprite object.
#[test]
fn test_render_inline_palette() {
    let jsonl = include_str!("../../../examples/demos/sprites/inline_palette.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "heart");

    assert_eq!(info.width, 5, "Heart sprite should be 5 pixels wide");
    assert_eq!(info.height, 5, "Heart sprite should be 5 pixels tall");
    assert!(info.palette_name.is_none(), "Inline palette should have no name");
}

/// @demo cli/render#output_path
/// @title Output Path Generation
/// @description Demonstrates output path patterns: default, explicit file, directory.
#[test]
fn test_render_output_paths() {
    use pixelsrc::output::generate_output_path;
    use std::path::Path;

    let input = Path::new("sprites/hero.jsonl");

    // Default: {input}_{sprite}.png (no output specified)
    let default_path = generate_output_path(input, "idle", None, true);
    assert!(
        default_path.to_string_lossy().contains("hero_idle.png"),
        "Default should be hero_idle.png"
    );

    // Explicit file output (single sprite)
    let explicit = generate_output_path(input, "idle", Some(Path::new("output/character.png")), true);
    assert_eq!(
        explicit.to_string_lossy(),
        "output/character.png",
        "Explicit path should be used directly"
    );

    // Directory output (ends with /)
    let dir_output = generate_output_path(input, "walk", Some(Path::new("renders/")), true);
    assert!(
        dir_output.to_string_lossy().contains("walk.png"),
        "Directory output should use sprite name"
    );
}
