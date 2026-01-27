//! Render Command Demo Tests
//!
//! Demonstrates the `pxl render` command functionality for converting
//! Pixelsrc definitions to PNG output.

use pixelsrc::parser::parse_stream;
use pixelsrc::registry::{PaletteRegistry, SpriteRegistry};
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
/// @demo cli/core#render
/// @title Render Command
/// @description Demonstrates the pxl render command for converting JSONL to PNG output.
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
    let explicit =
        generate_output_path(input, "idle", Some(Path::new("output/character.png")), true);
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
