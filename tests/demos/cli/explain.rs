//! Explain Command Demo Tests
//!
//! Demonstrates the `pxl explain` command functionality for providing
//! human-readable explanations of sprites, palettes, and other objects.

use pixelsrc::explain::{
    explain_animation, explain_composition, explain_palette, explain_sprite, format_explanation,
    resolve_palette_colors, Explanation,
};
use pixelsrc::models::{Animation, Composition, PaletteRef, Sprite};
use std::collections::HashMap;

/// Helper to create a simple sprite for testing
fn make_sprite(name: &str, grid: Vec<&str>, palette: PaletteRef) -> Sprite {
    Sprite {
        name: name.to_string(),
        size: None,
        palette,
        grid: grid.into_iter().map(String::from).collect(),
        source: None,
        transform: None,
        metadata: None,
        nine_slice: None,
    }
}

// ============================================================================
// Sprite Explanation Tests
// ============================================================================

/// @demo cli/explain#sprite_basic
/// @title Explain Basic Sprite
/// @description `pxl explain` provides a human-readable breakdown of sprite structure.
#[test]
fn test_explain_basic_sprite() {
    let sprite = make_sprite(
        "star",
        vec!["{_}{y}{_}", "{y}{y}{y}", "{_}{y}{_}"],
        PaletteRef::Inline(HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{y}".to_string(), "#FFD700".to_string()),
        ])),
    );

    let palette_colors = resolve_palette_colors(&sprite.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite, palette_colors.as_ref());

    assert_eq!(explanation.name, "star");
    assert_eq!(explanation.width, 3);
    assert_eq!(explanation.height, 3);
    assert_eq!(explanation.total_cells, 9);
    assert!(explanation.tokens.len() >= 2, "Should list all tokens used");
}

/// @demo cli/explain#sprite_tokens
/// @title Explain Token Usage
/// @description Shows token frequency and percentage of total grid cells.
#[test]
fn test_explain_token_usage() {
    let sprite = make_sprite(
        "checker",
        vec!["{a}{b}{a}{b}", "{b}{a}{b}{a}", "{a}{b}{a}{b}", "{b}{a}{b}{a}"],
        PaletteRef::Inline(HashMap::from([
            ("{a}".to_string(), "#FF0000".to_string()),
            ("{b}".to_string(), "#00FF00".to_string()),
        ])),
    );

    let palette_colors = resolve_palette_colors(&sprite.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite, palette_colors.as_ref());

    // Both tokens should be 50% each
    for token in &explanation.tokens {
        assert!(
            (token.percentage - 50.0).abs() < 1.0,
            "Each token should be ~50% in a checkerboard"
        );
        assert_eq!(token.count, 8, "Each token should appear 8 times in 4x4");
    }
}

/// @demo cli/explain#sprite_transparency
/// @title Explain Transparency Ratio
/// @description Shows percentage of transparent vs opaque cells.
#[test]
fn test_explain_transparency_ratio() {
    let sprite = make_sprite(
        "diamond",
        vec!["{_}{x}{_}", "{x}{x}{x}", "{_}{x}{_}"],
        PaletteRef::Inline(HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{x}".to_string(), "#FF0000".to_string()),
        ])),
    );

    let palette_colors = resolve_palette_colors(&sprite.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite, palette_colors.as_ref());

    // 4 transparent out of 9
    assert_eq!(explanation.transparent_count, 4);
    let expected_ratio = 4.0 / 9.0 * 100.0;
    assert!(
        (explanation.transparency_ratio - expected_ratio).abs() < 1.0,
        "Transparency ratio should be ~44.4%"
    );
}

// ============================================================================
// Palette Explanation Tests
// ============================================================================

/// @demo cli/explain#palette
/// @title Explain Palette
/// @description Shows palette name, color count, and color mappings.
#[test]
fn test_explain_palette() {
    let colors = HashMap::from([
        ("{_}".to_string(), "#00000000".to_string()),
        ("{skin}".to_string(), "#FFD5B4".to_string()),
        ("{hair}".to_string(), "#8B4513".to_string()),
        ("{shirt}".to_string(), "#4169E1".to_string()),
    ]);

    let explanation = explain_palette("hero_colors", &colors);

    assert_eq!(explanation.name, "hero_colors");
    assert_eq!(explanation.color_count, 4);
    assert!(!explanation.is_builtin);

    // Colors should be listed
    let token_names: Vec<_> = explanation.colors.iter().map(|(t, _, _)| t.clone()).collect();
    assert!(token_names.contains(&"{skin}".to_string()));
    assert!(token_names.contains(&"{hair}".to_string()));
}

// ============================================================================
// Animation Explanation Tests
// ============================================================================

/// @demo cli/explain#animation
/// @title Explain Animation
/// @description Shows frame sequence, timing, and loop settings.
#[test]
fn test_explain_animation() {
    let animation = Animation {
        name: "walk_cycle".to_string(),
        frames: vec![
            "walk_1".to_string(),
            "walk_2".to_string(),
            "walk_3".to_string(),
            "walk_4".to_string(),
        ],
        keyframes: None,
        source: None,
        transform: None,
        duration: Some(pixelsrc::models::Duration::Milliseconds(150)),
        timing_function: None,
        r#loop: None,
        palette_cycle: None,
        tags: None,
        frame_metadata: None,
        attachments: None,
    };

    let explanation = explain_animation(&animation);

    assert_eq!(explanation.name, "walk_cycle");
    assert_eq!(explanation.frame_count, 4);
    assert_eq!(explanation.duration_ms, 150);
    assert_eq!(explanation.frames, vec!["walk_1", "walk_2", "walk_3", "walk_4"]);
}

// ============================================================================
// Composition Explanation Tests
// ============================================================================

/// @demo cli/explain#composition
/// @title Explain Composition
/// @description Shows composition structure including layers and base sprite.
#[test]
fn test_explain_composition() {
    let composition = Composition {
        name: "scene".to_string(),
        base: Some("background".to_string()),
        size: Some([64, 64]),
        cell_size: Some([8, 8]),
        sprites: HashMap::from([
            ("a".to_string(), Some("player".to_string())),
            ("b".to_string(), Some("enemy".to_string())),
        ]),
        layers: vec![],
    };

    let explanation = explain_composition(&composition);

    assert_eq!(explanation.name, "scene");
    assert_eq!(explanation.base, Some("background".to_string()));
    assert_eq!(explanation.size, Some([64, 64]));
    assert_eq!(explanation.sprite_count, 2);
}

// ============================================================================
// Format Explanation Tests
// ============================================================================

/// @demo cli/explain#format_output
/// @title Formatted Explanation Output
/// @description Explanations can be formatted as human-readable text.
#[test]
fn test_format_explanation() {
    let sprite = make_sprite(
        "coin",
        vec!["{_}{g}{g}{_}", "{g}{g}{g}{g}", "{g}{g}{g}{g}", "{_}{g}{g}{_}"],
        PaletteRef::Inline(HashMap::from([
            ("{_}".to_string(), "#0000".to_string()),
            ("{g}".to_string(), "#FFD700".to_string()),
        ])),
    );

    let palette_colors = resolve_palette_colors(&sprite.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite, palette_colors.as_ref());
    let formatted = format_explanation(&Explanation::Sprite(explanation));

    // Formatted output should contain key information
    assert!(formatted.contains("coin"), "Should mention sprite name");
    assert!(formatted.contains("4") || formatted.contains("4x4"), "Should show dimensions");
}

// ============================================================================
// Edge Cases
// ============================================================================

/// @demo cli/explain#inline_vs_named
/// @title Explain Inline vs Named Palette
/// @description Explanation distinguishes between inline and named palette references.
#[test]
fn test_explain_inline_vs_named_palette() {
    // Inline palette
    let sprite_inline = make_sprite(
        "inline_test",
        vec!["{x}"],
        PaletteRef::Inline(HashMap::from([("{x}".to_string(), "#FF0000".to_string())])),
    );

    let palette_colors = resolve_palette_colors(&sprite_inline.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite_inline, palette_colors.as_ref());
    assert_eq!(explanation.palette_ref, "inline");

    // Named palette
    let sprite_named =
        make_sprite("named_test", vec!["{x}"], PaletteRef::Named("my_palette".to_string()));

    let empty_palettes = HashMap::new();
    let palette_colors = resolve_palette_colors(&sprite_named.palette, &empty_palettes);
    let explanation = explain_sprite(&sprite_named, palette_colors.as_ref());
    assert_eq!(explanation.palette_ref, "my_palette");
}

/// @demo cli/explain#inconsistent_rows
/// @title Explain Inconsistent Row Widths
/// @description Detects and reports when grid rows have different widths.
#[test]
fn test_explain_inconsistent_rows() {
    let sprite = make_sprite(
        "uneven",
        vec![
            "{x}{x}{x}", // 3 tokens
            "{x}{x}",    // 2 tokens - inconsistent!
            "{x}{x}{x}", // 3 tokens
        ],
        PaletteRef::Inline(HashMap::from([("{x}".to_string(), "#FF0000".to_string())])),
    );

    let palette_colors = resolve_palette_colors(&sprite.palette, &HashMap::new());
    let explanation = explain_sprite(&sprite, palette_colors.as_ref());

    assert!(!explanation.consistent_rows, "Should detect inconsistent row widths");
}
