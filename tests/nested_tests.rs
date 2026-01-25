//! Integration tests for nested compositions (NC-5)
//!
//! Tests verify that compositions can reference other compositions in their
//! sprite maps, enabling hierarchical scene building.

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use pixelsrc::composition::{render_composition_nested, CompositionError, RenderContext};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use pixelsrc::registry::{CompositionRegistry, PaletteRegistry, SpriteRegistry};
use pixelsrc::renderer::render_resolved;

/// Parse a JSONL file and return registries
fn parse_file(
    path: &Path,
) -> (PaletteRegistry, SpriteRegistry, CompositionRegistry, HashMap<String, image::RgbaImage>) {
    let content = fs::read_to_string(path).expect("Failed to read fixture");
    parse_content(&content)
}

/// Parse JSONL content and return registries
fn parse_content(
    jsonl: &str,
) -> (PaletteRegistry, SpriteRegistry, CompositionRegistry, HashMap<String, image::RgbaImage>) {
    let cursor = Cursor::new(jsonl);
    let parse_result = parse_stream(cursor);

    let mut palette_registry = PaletteRegistry::new();
    let mut sprite_registry = SpriteRegistry::new();
    let mut composition_registry = CompositionRegistry::new();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(p) => palette_registry.register(p),
            TtpObject::Sprite(s) => sprite_registry.register_sprite(s),
            TtpObject::Variant(v) => sprite_registry.register_variant(v),
            TtpObject::Composition(c) => composition_registry.register(c),
            _ => {}
        }
    }

    // Pre-render sprites to images
    let mut sprite_images: HashMap<String, image::RgbaImage> = HashMap::new();
    for name in sprite_registry.names() {
        if let Ok(resolved) = sprite_registry.resolve(name, &palette_registry, false) {
            let (image, _) = render_resolved(&resolved);
            sprite_images.insert(name.clone(), image);
        }
    }

    (palette_registry, sprite_registry, composition_registry, sprite_images)
}

// ============================================================================
// Valid Nested Composition Tests
// ============================================================================

/// Test basic nested composition rendering
#[test]
fn test_nested_composition_basic() {
    let path = Path::new("tests/fixtures/compositions/nested_composition.jsonl");
    let (_, _, composition_registry, sprite_images) = parse_file(path);

    // Render the outer composition which references inner_comp
    let outer = composition_registry.get("outer_comp").expect("outer_comp not found");

    let mut ctx = RenderContext::new();
    let result = render_composition_nested(
        outer,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    );

    assert!(result.is_ok(), "Nested composition should render successfully");

    let (image, warnings) = result.unwrap();

    // outer_comp is 16x8 pixels
    assert_eq!(image.width(), 16, "Width should be 16");
    assert_eq!(image.height(), 8, "Height should be 8");

    // Should have no warnings
    assert!(warnings.is_empty(), "Should have no warnings");
}

/// Test that inner composition is rendered correctly within outer/// Test caching works for repeated composition references
#[test]
fn test_nested_composition_caching() {
    let path = Path::new("tests/fixtures/compositions/nested_composition.jsonl");
    let (_, _, composition_registry, sprite_images) = parse_file(path);

    let outer = composition_registry.get("outer_comp").expect("outer_comp not found");

    let mut ctx = RenderContext::new();

    // Render once
    let _ = render_composition_nested(
        outer,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    )
    .expect("Should render");

    // inner_comp should be cached
    assert!(ctx.is_cached("inner_comp"), "inner_comp should be cached after rendering");
}

// ============================================================================
// Cycle Detection Tests
// ============================================================================

/// Test that direct cycles are detected
#[test]
fn test_composition_cycle_detected() {
    let path = Path::new("tests/fixtures/runtime_errors/composition_cycle.jsonl");
    let (_, _, composition_registry, sprite_images) = parse_file(path);

    // comp_a references comp_b which references comp_a
    let comp_a = composition_registry.get("comp_a").expect("comp_a not found");

    let mut ctx = RenderContext::new();
    let result = render_composition_nested(
        comp_a,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    );

    assert!(result.is_err(), "Cycle should be detected");

    match result {
        Err(CompositionError::CycleDetected { cycle_path }) => {
            assert!(cycle_path.len() >= 2, "Cycle path should contain at least 2 elements");
        }
        Err(e) => panic!("Expected CycleDetected error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }
}

/// Test that self-referencing compositions are detected
#[test]
fn test_composition_self_reference_detected() {
    let jsonl = r##"{"type": "palette", "name": "test", "colors": {"{_}": "#00000000", "{x}": "#FF0000"}}
{"type": "composition", "name": "self_ref", "size": [8, 8], "cell_size": [8, 8], "sprites": {"S": "self_ref", ".": null}, "layers": [{"map": ["S"]}]}"##;

    let (_, _, composition_registry, sprite_images) = parse_content(jsonl);

    let comp = composition_registry.get("self_ref").expect("self_ref not found");

    let mut ctx = RenderContext::new();
    let result = render_composition_nested(
        comp,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    );

    assert!(result.is_err(), "Self-reference should be detected");

    match result {
        Err(CompositionError::CycleDetected { .. }) => {}
        Err(e) => panic!("Expected CycleDetected error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }
}

// ============================================================================
// Example File Tests
// ============================================================================

/// Test that nested_building.jsonl example renders successfully
#[test]
fn test_example_nested_building() {
    let path = Path::new("examples/nested_building.jsonl");
    let (_, _, composition_registry, sprite_images) = parse_file(path);

    // Render the city_block which references building_3w compositions
    let city_block = composition_registry.get("city_block").expect("city_block not found");

    let mut ctx = RenderContext::new();
    let result = render_composition_nested(
        city_block,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    );

    assert!(result.is_ok(), "nested_building example should render: {:?}", result.err());

    let (image, _) = result.unwrap();

    // city_block is 72x24 pixels (3 buildings of 24x24 each)
    assert_eq!(image.width(), 72, "Width should be 72");
    assert_eq!(image.height(), 24, "Height should be 24");

    // building_3w should be cached
    assert!(ctx.is_cached("building_3w"), "building_3w should be cached");
}

/// Test that nested_ui.jsonl example renders successfully
#[test]
fn test_example_nested_ui() {
    let path = Path::new("examples/nested_ui.jsonl");
    let (_, _, composition_registry, sprite_images) = parse_file(path);

    // Render the settings_panel which has multiple levels of nesting
    let settings_panel =
        composition_registry.get("settings_panel").expect("settings_panel not found");

    let mut ctx = RenderContext::new();
    let result = render_composition_nested(
        settings_panel,
        &sprite_images,
        Some(&composition_registry),
        &mut ctx,
        false,
        None,
    );

    assert!(result.is_ok(), "nested_ui example should render: {:?}", result.err());

    let (image, _) = result.unwrap();

    // settings_panel is 48x24 pixels
    assert_eq!(image.width(), 48, "Width should be 48");
    assert_eq!(image.height(), 24, "Height should be 24");
}

// ============================================================================
// Render Context Tests
// ============================================================================

/// Test RenderContext cycle detection
#[test]
fn test_render_context_cycle_detection() {
    let mut ctx = RenderContext::new();

    // Push A, B, C - should work
    assert!(ctx.push("A").is_ok());
    assert!(ctx.push("B").is_ok());
    assert!(ctx.push("C").is_ok());

    // Push A again - should fail (cycle)
    let result = ctx.push("A");
    assert!(result.is_err());

    match result {
        Err(CompositionError::CycleDetected { cycle_path }) => {
            assert_eq!(cycle_path, vec!["A", "B", "C", "A"]);
        }
        _ => panic!("Expected CycleDetected error"),
    }
}

/// Test RenderContext caching
#[test]
fn test_render_context_caching() {
    let mut ctx = RenderContext::new();

    assert!(!ctx.is_cached("test"));
    assert_eq!(ctx.len(), 0);

    let image = image::RgbaImage::new(10, 10);
    ctx.cache("test".to_string(), image);

    assert!(ctx.is_cached("test"));
    assert_eq!(ctx.len(), 1);

    let cached = ctx.get_cached("test");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().width(), 10);
}
