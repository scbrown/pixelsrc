//! Integration tests for CSS variable support
//!
//! Tests the full palette parsing flow with CSS custom properties:
//! - Variable definition (`--name: value`)
//! - Variable resolution (`var(--name)` and `var(--name, fallback)`)
//! - Lenient vs strict mode handling
//! - Integration with color parsing

use pixelsrc::palette_parser::{PaletteParser, ParseMode, MAGENTA};
use pixelsrc::variables::VariableRegistry;
use image::Rgba;
use std::collections::HashMap;

fn make_palette(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ========== End-to-end palette parsing tests ==========

#[test]
fn test_realistic_palette_with_variables() {
    // A realistic palette using CSS variables for theming
    let raw = make_palette(&[
        // Theme variables
        ("--primary", "#4169E1"),
        ("--secondary", "#8B4513"),
        ("--accent", "#FFD700"),
        ("--bg", "#2D2D2D"),
        ("--fg", "#FFFFFF"),
        // Derived colors using variables
        ("{_}", "transparent"),
        ("{outline}", "var(--bg)"),
        ("{skin}", "#FFCC99"),
        ("{hair}", "var(--secondary)"),
        ("{shirt}", "var(--primary)"),
        ("{highlight}", "var(--accent)"),
        ("{text}", "var(--fg)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    // Check resolved colors
    assert_eq!(result.colors.get("{outline}"), Some(&Rgba([45, 45, 45, 255])));
    assert_eq!(result.colors.get("{hair}"), Some(&Rgba([139, 69, 19, 255])));
    assert_eq!(result.colors.get("{shirt}"), Some(&Rgba([65, 105, 225, 255])));
    assert_eq!(result.colors.get("{highlight}"), Some(&Rgba([255, 215, 0, 255])));
    assert_eq!(result.colors.get("{text}"), Some(&Rgba([255, 255, 255, 255])));

    // No warnings for valid palette
    assert!(result.warnings.is_empty());

    // Variables should be preserved in registry
    assert!(result.variables.contains("--primary"));
    assert!(result.variables.contains("--secondary"));
}

#[test]
fn test_variable_with_css_color_functions() {
    // Variables can contain partial values for use in CSS functions
    let raw = make_palette(&[
        ("--hue", "240"),
        ("--sat", "100%"),
        ("--light", "50%"),
        ("{blue}", "hsl(var(--hue), var(--sat), var(--light))"),
        // RGB components
        ("--r", "255"),
        ("--g", "128"),
        ("--b", "0"),
        ("{orange}", "rgb(var(--r), var(--g), var(--b))"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{blue}"), Some(&Rgba([0, 0, 255, 255])));
    assert_eq!(result.colors.get("{orange}"), Some(&Rgba([255, 128, 0, 255])));
}

#[test]
fn test_fallback_for_optional_theming() {
    // Fallbacks enable optional theme overrides
    let raw = make_palette(&[
        // Only some theme variables defined
        ("--primary", "#FF0000"),
        // Use fallback for undefined optional override
        ("{main}", "var(--primary)"),
        ("{alt}", "var(--secondary, #00FF00)"), // --secondary not defined
        ("{accent}", "var(--accent-override, var(--primary))"), // Nested fallback
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{main}"), Some(&Rgba([255, 0, 0, 255])));
    assert_eq!(result.colors.get("{alt}"), Some(&Rgba([0, 255, 0, 255]))); // Uses fallback
    assert_eq!(result.colors.get("{accent}"), Some(&Rgba([255, 0, 0, 255]))); // Nested to --primary
    assert!(result.warnings.is_empty());
}

// ========== External variable inheritance tests ==========

#[test]
fn test_external_theme_variables() {
    // Simulate a theme file providing base variables
    let mut theme_vars = VariableRegistry::new();
    theme_vars.define("--bg", "#1A1A2E");
    theme_vars.define("--fg", "#EAEAEA");
    theme_vars.define("--accent", "#E94560");

    // Local palette uses theme variables
    let raw = make_palette(&[
        ("{background}", "var(--bg)"),
        ("{text}", "var(--fg)"),
        ("{highlight}", "var(--accent)"),
        ("{_}", "transparent"),
    ]);

    let parser = PaletteParser::with_external_vars(theme_vars);
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{background}"), Some(&Rgba([26, 26, 46, 255])));
    assert_eq!(result.colors.get("{text}"), Some(&Rgba([234, 234, 234, 255])));
    assert_eq!(result.colors.get("{highlight}"), Some(&Rgba([233, 69, 96, 255])));
}

#[test]
fn test_local_overrides_external() {
    // External theme
    let mut theme_vars = VariableRegistry::new();
    theme_vars.define("--primary", "#FF0000"); // Red in theme

    // Local palette overrides theme
    let raw = make_palette(&[
        ("--primary", "#0000FF"), // Blue override
        ("{color}", "var(--primary)"),
    ]);

    let parser = PaletteParser::with_external_vars(theme_vars);
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    // Local should win
    assert_eq!(result.colors.get("{color}"), Some(&Rgba([0, 0, 255, 255])));
}

// ========== Lenient mode error recovery tests ==========

#[test]
fn test_lenient_undefined_variable_uses_magenta() {
    let raw = make_palette(&[
        ("{valid}", "#FF0000"),
        ("{undefined}", "var(--nonexistent)"), // No fallback
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{valid}"), Some(&Rgba([255, 0, 0, 255])));
    assert_eq!(result.colors.get("{undefined}"), Some(&MAGENTA));
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("undefined"));
}

#[test]
fn test_lenient_circular_reference_uses_magenta() {
    let raw = make_palette(&[
        ("--a", "var(--b)"),
        ("--b", "var(--a)"),
        ("{color}", "var(--a)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{color}"), Some(&MAGENTA));
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_lenient_mixed_valid_and_invalid() {
    let raw = make_palette(&[
        ("--valid", "#00FF00"),
        ("{good1}", "var(--valid)"),
        ("{bad1}", "var(--missing)"),
        ("{good2}", "#0000FF"),
        ("{bad2}", "not-a-color"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    // Valid colors parsed correctly
    assert_eq!(result.colors.get("{good1}"), Some(&Rgba([0, 255, 0, 255])));
    assert_eq!(result.colors.get("{good2}"), Some(&Rgba([0, 0, 255, 255])));

    // Invalid colors become magenta
    assert_eq!(result.colors.get("{bad1}"), Some(&MAGENTA));
    assert_eq!(result.colors.get("{bad2}"), Some(&MAGENTA));

    // Two warnings
    assert_eq!(result.warnings.len(), 2);
}

// ========== Strict mode tests ==========

#[test]
fn test_strict_fails_on_undefined_variable() {
    let raw = make_palette(&[
        ("{color}", "var(--undefined)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Strict);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("undefined"));
}

#[test]
fn test_strict_fails_on_circular_reference() {
    let raw = make_palette(&[
        ("--self", "var(--self)"),
        ("{color}", "var(--self)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Strict);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("circular"));
}

#[test]
fn test_strict_succeeds_with_valid_palette() {
    let raw = make_palette(&[
        ("--primary", "#FF0000"),
        ("{main}", "var(--primary)"),
        ("{alt}", "var(--missing, #00FF00)"), // Fallback makes this valid
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Strict);

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.warnings.is_empty());
}

// ========== Variable registry preservation tests ==========

#[test]
fn test_variable_registry_available_for_reuse() {
    let raw = make_palette(&[
        ("--brand-primary", "#4169E1"),
        ("--brand-secondary", "#8B4513"),
        ("{color}", "var(--brand-primary)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    // Can use the registry for further resolution
    let registry = &result.variables;
    assert!(registry.contains("--brand-primary"));
    assert!(registry.contains("--brand-secondary"));

    // Can resolve additional values
    assert_eq!(
        registry.resolve("var(--brand-primary)").unwrap(),
        "#4169E1"
    );
}

// ========== resolve_to_strings tests ==========

#[test]
fn test_resolve_to_strings_for_serialization() {
    let raw = make_palette(&[
        ("--primary", "#FF0000"),
        ("{main}", "var(--primary)"),
        ("{static}", "#00FF00"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.resolve_to_strings(&raw, ParseMode::Lenient).unwrap();

    // Get resolved color strings (not RGBA)
    assert_eq!(result.colors.get("{main}"), Some(&"#FF0000".to_string()));
    assert_eq!(result.colors.get("{static}"), Some(&"#00FF00".to_string()));
}

// ========== Edge case tests ==========

#[test]
fn test_deep_nesting_chain() {
    let raw = make_palette(&[
        ("--l1", "#FF0000"),
        ("--l2", "var(--l1)"),
        ("--l3", "var(--l2)"),
        ("--l4", "var(--l3)"),
        ("--l5", "var(--l4)"),
        ("{color}", "var(--l5)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{color}"), Some(&Rgba([255, 0, 0, 255])));
}

#[test]
fn test_complex_fallback_chain() {
    let raw = make_palette(&[
        ("--final", "#00FF00"),
        ("{color}", "var(--a, var(--b, var(--c, var(--final))))"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{color}"), Some(&Rgba([0, 255, 0, 255])));
}

#[test]
fn test_variables_with_all_color_formats() {
    let raw = make_palette(&[
        ("--hex", "#FF0000"),
        ("--rgb", "rgb(0, 255, 0)"),
        ("--hsl", "hsl(240, 100%, 50%)"),
        ("--named", "coral"),
        ("{hex_ref}", "var(--hex)"),
        ("{rgb_ref}", "var(--rgb)"),
        ("{hsl_ref}", "var(--hsl)"),
        ("{named_ref}", "var(--named)"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{hex_ref}"), Some(&Rgba([255, 0, 0, 255])));
    assert_eq!(result.colors.get("{rgb_ref}"), Some(&Rgba([0, 255, 0, 255])));
    assert_eq!(result.colors.get("{hsl_ref}"), Some(&Rgba([0, 0, 255, 255])));
    assert_eq!(result.colors.get("{named_ref}"), Some(&Rgba([255, 127, 80, 255])));
}

#[test]
fn test_whitespace_in_var_reference() {
    let raw = make_palette(&[
        ("--color", "#FF0000"),
        ("{a}", "var(  --color  )"),
        ("{b}", "var(--color,   #00FF00   )"),
    ]);

    let parser = PaletteParser::new();
    let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

    assert_eq!(result.colors.get("{a}"), Some(&Rgba([255, 0, 0, 255])));
    // {b} uses defined --color, not fallback
    assert_eq!(result.colors.get("{b}"), Some(&Rgba([255, 0, 0, 255])));
}
