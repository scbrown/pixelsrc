//! WASM tests using wasm_bindgen_test
//!
//! Run with: wasm-pack test --headless --chrome --features wasm
//! Or for node: see tests in src/wasm.rs (run with cargo test --features wasm)

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

// Configure tests to run in browser environment
wasm_bindgen_test_configure!(run_in_browser);

use pixelsrc::wasm::{list_sprites, render_to_png, render_to_rgba, validate};

// Test fixtures - using r##""## to avoid # prefix issues in Rust 2021
const MINIMAL_DOT: &str =
    r##"{"type": "sprite", "name": "dot", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}"##;

const HEART_SPRITE: &str = r##"{"type": "palette", "name": "reds", "colors": {"{_}": "#00000000", "{r}": "#FF0000", "{p}": "#FF6B6B"}}
{"type": "sprite", "name": "heart", "palette": "reds", "grid": ["{_}{r}{r}{_}", "{r}{r}{r}{r}", "{_}{r}{r}{_}", "{_}{_}{r}{_}"]}"##;

const TRANSPARENT_SPRITE: &str =
    r##"{"type": "sprite", "name": "transparent", "palette": {"{_}": "#00000000", "{x}": "#FF000080"}, "grid": ["{_}{x}", "{x}{_}"]}"##;

const MULTI_SPRITE: &str = r##"{"type": "sprite", "name": "first", "palette": {"{a}": "#FF0000"}, "grid": ["{a}"]}
{"type": "sprite", "name": "second", "palette": {"{b}": "#00FF00"}, "grid": ["{b}"]}
{"type": "sprite", "name": "third", "palette": {"{c}": "#0000FF"}, "grid": ["{c}"]}"##;

// ============================================================================
// render_to_png tests
// ============================================================================

#[wasm_bindgen_test]
fn test_render_minimal_sprite_to_png() {
    let result = render_to_png(MINIMAL_DOT);

    // Should produce non-empty output
    assert!(!result.is_empty(), "PNG output should not be empty");

    // Should have PNG magic bytes (0x89 0x50 0x4E 0x47 = \x89PNG)
    assert!(result.len() >= 8, "PNG should have at least header bytes");
    assert_eq!(result[0], 0x89, "First PNG magic byte");
    assert_eq!(result[1], 0x50, "Second PNG magic byte (P)");
    assert_eq!(result[2], 0x4E, "Third PNG magic byte (N)");
    assert_eq!(result[3], 0x47, "Fourth PNG magic byte (G)");
}

#[wasm_bindgen_test]
fn test_render_png_empty_input() {
    let result = render_to_png("");
    assert!(result.is_empty(), "Empty input should produce empty PNG");
}

#[wasm_bindgen_test]
fn test_render_png_palette_only() {
    let jsonl = r##"{"type": "palette", "name": "test", "colors": {"{x}": "#FF0000"}}"##;
    let result = render_to_png(jsonl);
    assert!(result.is_empty(), "Palette-only input should produce empty PNG");
}

#[wasm_bindgen_test]
fn test_render_png_complex_sprite() {
    let result = render_to_png(HEART_SPRITE);

    assert!(!result.is_empty(), "Complex sprite should produce PNG");
    assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47], "Should have PNG magic bytes");
}

// ============================================================================
// render_to_rgba tests
// ============================================================================

#[wasm_bindgen_test]
fn test_render_minimal_sprite_to_rgba() {
    let result = render_to_rgba(MINIMAL_DOT);

    // 1x1 pixel
    assert_eq!(result.width(), 1, "Width should be 1");
    assert_eq!(result.height(), 1, "Height should be 1");

    // RGBA = 4 bytes per pixel
    let pixels = result.pixels();
    assert_eq!(pixels.len(), 4, "Should have 4 bytes (1 RGBA pixel)");

    // Red pixel (#FF0000)
    assert_eq!(pixels[0], 255, "Red channel");
    assert_eq!(pixels[1], 0, "Green channel");
    assert_eq!(pixels[2], 0, "Blue channel");
    assert_eq!(pixels[3], 255, "Alpha channel (opaque)");
}

#[wasm_bindgen_test]
fn test_render_rgba_no_sprites() {
    let result = render_to_rgba(r##"{"type": "palette", "name": "empty", "colors": {}}"##);

    assert_eq!(result.width(), 0, "Width should be 0 when no sprites");
    assert_eq!(result.height(), 0, "Height should be 0 when no sprites");
    assert!(result.pixels().is_empty(), "Pixels should be empty");

    let warnings = result.warnings();
    assert!(
        warnings.iter().any(|w| w.contains("No sprites")),
        "Should warn about missing sprites"
    );
}

#[wasm_bindgen_test]
fn test_render_rgba_dimensions() {
    // Heart sprite is 4x4
    let result = render_to_rgba(HEART_SPRITE);

    assert_eq!(result.width(), 4, "Heart width should be 4");
    assert_eq!(result.height(), 4, "Heart height should be 4");

    let pixels = result.pixels();
    assert_eq!(pixels.len(), 4 * 4 * 4, "Should have 16 pixels * 4 bytes");
}

#[wasm_bindgen_test]
fn test_render_rgba_empty_input() {
    let result = render_to_rgba("");

    assert_eq!(result.width(), 0);
    assert_eq!(result.height(), 0);
    assert!(result.pixels().is_empty());
}

// ============================================================================
// render_named_sprite tests (first sprite in file is rendered)
// ============================================================================

#[wasm_bindgen_test]
fn test_render_named_sprite_first() {
    // When multiple sprites exist, first one is rendered
    let result = render_to_rgba(MULTI_SPRITE);

    assert_eq!(result.width(), 1);
    assert_eq!(result.height(), 1);

    let pixels = result.pixels();
    // First sprite is red (#FF0000)
    assert_eq!(pixels[0], 255, "Should render first sprite (red)");
    assert_eq!(pixels[1], 0);
    assert_eq!(pixels[2], 0);
}

#[wasm_bindgen_test]
fn test_render_named_sprite_with_palette_ref() {
    // Sprite referencing a named palette
    let result = render_to_rgba(HEART_SPRITE);

    // Should render successfully
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);

    // No warnings about missing palette
    let warnings = result.warnings();
    assert!(
        !warnings.iter().any(|w| w.contains("not found")),
        "Should not warn about palette: {:?}",
        warnings
    );
}

#[wasm_bindgen_test]
fn test_render_named_sprite_inline_palette() {
    // Sprite with inline palette (no separate palette definition)
    let jsonl = r##"{"type": "sprite", "name": "inline_test", "palette": {"{g}": "#00FF00"}, "grid": ["{g}"]}"##;
    let result = render_to_rgba(jsonl);

    assert_eq!(result.width(), 1);
    assert_eq!(result.height(), 1);

    let pixels = result.pixels();
    // Green pixel
    assert_eq!(pixels[0], 0);
    assert_eq!(pixels[1], 255);
    assert_eq!(pixels[2], 0);
}

// ============================================================================
// list_sprites tests
// ============================================================================

#[wasm_bindgen_test]
fn test_list_sprites_single() {
    let result = list_sprites(MINIMAL_DOT);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "dot");
}

#[wasm_bindgen_test]
fn test_list_sprites_multiple() {
    let result = list_sprites(MULTI_SPRITE);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "first");
    assert_eq!(result[1], "second");
    assert_eq!(result[2], "third");
}

#[wasm_bindgen_test]
fn test_list_sprites_empty() {
    let result = list_sprites("");
    assert!(result.is_empty());
}

#[wasm_bindgen_test]
fn test_list_sprites_palette_only() {
    let jsonl = r##"{"type": "palette", "name": "test", "colors": {}}"##;
    let result = list_sprites(jsonl);
    assert!(result.is_empty());
}

#[wasm_bindgen_test]
fn test_list_sprites_mixed_content() {
    // Palettes and sprites mixed
    let jsonl = r##"{"type": "palette", "name": "p1", "colors": {}}
{"type": "sprite", "name": "s1", "palette": {}, "grid": []}
{"type": "palette", "name": "p2", "colors": {}}
{"type": "sprite", "name": "s2", "palette": {}, "grid": []}"##;

    let result = list_sprites(jsonl);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "s1");
    assert_eq!(result[1], "s2");
}

// ============================================================================
// validate tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_valid_input() {
    let result = validate(HEART_SPRITE);
    assert!(result.is_empty(), "Valid input should have no warnings: {:?}", result);
}

#[wasm_bindgen_test]
fn test_validate_valid_minimal() {
    let result = validate(MINIMAL_DOT);
    assert!(result.is_empty(), "Minimal valid sprite should have no warnings");
}

#[wasm_bindgen_test]
fn test_validate_invalid_json() {
    let result = validate(r##"{"type": "sprite", "name": "bad""##);
    assert!(!result.is_empty(), "Invalid JSON should produce warnings");
}

#[wasm_bindgen_test]
fn test_validate_missing_palette() {
    let jsonl = r##"{"type": "sprite", "name": "orphan", "palette": "nonexistent", "grid": ["{x}"]}"##;
    let result = validate(jsonl);

    assert!(!result.is_empty(), "Missing palette reference should warn");
    assert!(
        result.iter().any(|w| w.contains("not found")),
        "Should mention palette not found: {:?}",
        result
    );
}

#[wasm_bindgen_test]
fn test_validate_empty_input() {
    let result = validate("");
    assert!(result.is_empty(), "Empty input is valid (no errors)");
}

#[wasm_bindgen_test]
fn test_validate_unknown_type() {
    let result = validate(r##"{"type": "unknown", "name": "test"}"##);
    // Unknown types may generate parse warnings but shouldn't crash
    // The exact behavior depends on the parser implementation
    let _ = result;
}

#[wasm_bindgen_test]
fn test_validate_size_mismatch() {
    // Grid row shorter than declared size
    let jsonl = r##"{"type": "sprite", "name": "mismatch", "palette": {"{x}": "#FF0000"}, "size": [3, 1], "grid": ["{x}"]}"##;
    let result = validate(jsonl);
    // Size mismatch may or may not be a validation error depending on implementation
    // Just verify it doesn't crash
    let _ = result;
}

// ============================================================================
// render_with_transparency tests
// ============================================================================

#[wasm_bindgen_test]
fn test_render_with_transparency_full() {
    // Fully transparent pixel (#00000000)
    let jsonl = r##"{"type": "sprite", "name": "clear", "palette": {"{_}": "#00000000"}, "grid": ["{_}"]}"##;
    let result = render_to_rgba(jsonl);

    assert_eq!(result.width(), 1);
    assert_eq!(result.height(), 1);

    let pixels = result.pixels();
    assert_eq!(pixels[3], 0, "Alpha should be 0 (fully transparent)");
}

#[wasm_bindgen_test]
fn test_render_with_transparency_partial() {
    // 50% transparent red (#FF000080)
    let jsonl = r##"{"type": "sprite", "name": "semi", "palette": {"{s}": "#FF000080"}, "grid": ["{s}"]}"##;
    let result = render_to_rgba(jsonl);

    let pixels = result.pixels();
    assert_eq!(pixels[0], 255, "Red channel");
    assert_eq!(pixels[1], 0, "Green channel");
    assert_eq!(pixels[2], 0, "Blue channel");
    assert_eq!(pixels[3], 128, "Alpha should be 128 (50% transparent)");
}

#[wasm_bindgen_test]
fn test_render_with_transparency_mixed() {
    let result = render_to_rgba(TRANSPARENT_SPRITE);

    // 2x2 sprite with mixed transparency
    assert_eq!(result.width(), 2);
    assert_eq!(result.height(), 2);

    let pixels = result.pixels();

    // First row: transparent, semi-transparent red
    // Pixel (0,0) - transparent
    assert_eq!(pixels[3], 0, "First pixel should be transparent");
    // Pixel (1,0) - semi-transparent red
    assert_eq!(pixels[4], 255, "Second pixel red channel");
    assert_eq!(pixels[7], 128, "Second pixel alpha (semi)");

    // Second row: semi-transparent red, transparent
    // Pixel (0,1) - semi-transparent red
    assert_eq!(pixels[8], 255, "Third pixel red channel");
    assert_eq!(pixels[11], 128, "Third pixel alpha (semi)");
    // Pixel (1,1) - transparent
    assert_eq!(pixels[15], 0, "Fourth pixel should be transparent");
}

#[wasm_bindgen_test]
fn test_render_transparency_in_png() {
    // PNG should preserve transparency
    let result = render_to_png(TRANSPARENT_SPRITE);

    assert!(!result.is_empty(), "Should produce PNG");
    assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47], "Valid PNG");
    // PNG will be larger than RGBA since it includes headers and compression
    assert!(result.len() > 16, "PNG should have substantial content");
}

// ============================================================================
// Edge cases and error handling
// ============================================================================

#[wasm_bindgen_test]
fn test_malformed_json_lines() {
    let jsonl = r##"{"type": "sprite", "name": "good", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}
not valid json
{"type": "sprite", "name": "also_good", "palette": {"{y}": "#00FF00"}, "grid": ["{y}"]}"##;

    let sprites = list_sprites(jsonl);
    // Should parse what it can
    assert!(sprites.len() >= 1, "Should parse at least one sprite");
}

#[wasm_bindgen_test]
fn test_unicode_sprite_name() {
    // Use actual unicode character instead of escape sequence
    let jsonl = r##"{"type": "sprite", "name": "emoji_🚀", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}"##;
    let result = list_sprites(jsonl);

    assert_eq!(result.len(), 1);
    // Name should contain the rocket emoji
    assert!(result[0].contains("emoji") || result[0].contains("🚀"));
}

#[wasm_bindgen_test]
fn test_large_sprite() {
    // 10x10 sprite
    let mut grid = Vec::new();
    for _ in 0..10 {
        grid.push(r#""{x}{x}{x}{x}{x}{x}{x}{x}{x}{x}""#);
    }
    let jsonl = format!(
        r##"{{"type": "sprite", "name": "large", "palette": {{"{{x}}": "#FF0000"}}, "grid": [{}]}}"##,
        grid.join(",")
    );

    let result = render_to_rgba(&jsonl);
    assert_eq!(result.width(), 10);
    assert_eq!(result.height(), 10);
    assert_eq!(result.pixels().len(), 10 * 10 * 4);
}
