//! Registry traits and implementations for named items.
//!
//! This module provides:
//! - A unified `Registry` trait for consistent registry interfaces
//! - `PaletteRegistry` for storing named palettes and resolving palette references
//! - `SpriteRegistry` for storing sprites and variants with transform support
//! - `TransformRegistry` for storing user-defined transforms
//! - `CompositionRegistry` for storing layered sprite compositions
//! - `Renderable` enum for unified sprite/composition lookup
//!
//! Most registries support lenient mode (warnings + fallback) and strict mode (errors).

mod composition;
mod palette;
mod renderable;
mod sprite;
mod traits;
mod transform;

// Re-export all public items from submodules
pub use composition::CompositionRegistry;
pub use palette::{
    LenientResult, PaletteError, PaletteRegistry, PaletteSource, PaletteWarning, ResolvedPalette,
    MAGENTA_FALLBACK,
};
pub use renderable::{lookup_renderable, Renderable};
pub use sprite::{ResolvedSprite, SpriteError, SpriteRegistry, SpriteWarning};
pub use traits::Registry;
pub use transform::TransformRegistry;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Composition, Palette, PaletteRef, RegionDef, Sprite, TransformSpec, Variant,
    };
    use std::collections::HashMap;

    fn mono_palette() -> Palette {
        Palette {
            name: "mono".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{on}".to_string(), "#FFFFFF".to_string()),
                ("{off}".to_string(), "#000000".to_string()),
            ]),
            ..Default::default()
        }
    }

    fn checker_sprite_named() -> Sprite {
        Sprite {
            name: "checker".to_string(),
            size: None,
            palette: PaletteRef::Named("mono".to_string()),

            metadata: None,
            ..Default::default()
        }
    }

    fn dot_sprite_inline() -> Sprite {
        Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        }
    }

    fn bad_ref_sprite() -> Sprite {
        Sprite {
            name: "bad_ref".to_string(),
            size: None,
            palette: PaletteRef::Named("nonexistent".to_string()),

            metadata: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_registry_new_is_empty() {
        let registry = PaletteRegistry::new();
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = PaletteRegistry::new();
        let palette = mono_palette();
        registry.register(palette.clone());

        assert!(registry.contains("mono"));
        let retrieved = registry.get("mono").unwrap();
        assert_eq!(retrieved.name, "mono");
        assert_eq!(retrieved.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
    }

    #[test]
    fn test_register_overwrites() {
        let mut registry = PaletteRegistry::new();
        let palette1 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{a}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        };
        let palette2 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{b}".to_string(), "#00FF00".to_string())]),
            ..Default::default()
        };

        registry.register(palette1);
        registry.register(palette2);

        let retrieved = registry.get("test").unwrap();
        assert!(retrieved.colors.contains_key("{b}"));
        assert!(!retrieved.colors.contains_key("{a}"));
    }

    #[test]
    fn test_resolve_strict_named_found() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        let sprite = checker_sprite_named();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Named("mono".to_string()));
        assert_eq!(result.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
    }

    #[test]
    fn test_resolve_strict_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(result, Err(PaletteError::NotFound("nonexistent".to_string())));
    }

    #[test]
    fn test_resolve_strict_inline() {
        let registry = PaletteRegistry::new();
        let sprite = dot_sprite_inline();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Inline);
        assert_eq!(result.colors.get("{x}"), Some(&"#FF0000".to_string()));
    }

    #[test]
    fn test_resolve_lenient_named_found() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        let sprite = checker_sprite_named();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Named("mono".to_string()));
    }

    #[test]
    fn test_resolve_lenient_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert!(result.warning.as_ref().unwrap().message.contains("nonexistent"));
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_lenient_inline() {
        let registry = PaletteRegistry::new();
        let sprite = dot_sprite_inline();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Inline);
    }

    #[test]
    fn test_resolve_combined_strict() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_combined_lenient() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve(&sprite, false).unwrap();
        assert!(result.warning.is_some());
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // Tests matching fixture: tests/fixtures/valid/named_palette.jsonl
    #[test]
    fn test_fixture_named_palette() {
        let mut registry = PaletteRegistry::new();

        // {"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}
        registry.register(mono_palette());

        // {"type": "sprite", "name": "checker", "palette": "mono", "regions": {...}}
        let sprite = checker_sprite_named();
        let result = registry.resolve_strict(&sprite).unwrap();

        assert_eq!(result.source, PaletteSource::Named("mono".to_string()));
        assert_eq!(result.colors.len(), 3);
        assert_eq!(result.colors.get("{_}"), Some(&"#00000000".to_string()));
        assert_eq!(result.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
        assert_eq!(result.colors.get("{off}"), Some(&"#000000".to_string()));
    }

    // Tests matching fixture: tests/fixtures/invalid/unknown_palette_ref.jsonl
    #[test]
    fn test_fixture_unknown_palette_ref_strict() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "regions": {...}}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_strict(&sprite);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PaletteError::NotFound("nonexistent".to_string()));
    }

    #[test]
    fn test_fixture_unknown_palette_ref_lenient() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "regions": {...}}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_lenient(&sprite);

        assert!(result.warning.is_some());
        assert_eq!(result.warning.unwrap().message, "Palette 'nonexistent' not found");
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // ============================================================
    // Built-in palette resolution tests (@name syntax)
    // ============================================================

    fn builtin_gameboy_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@gameboy".to_string()),

            metadata: None,
            ..Default::default()
        }
    }

    fn builtin_nonexistent_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@nonexistent".to_string()),

            metadata: None,
            ..Default::default()
        }
    }

    // ========== SpriteRegistry Tests ==========

    fn hero_sprite() -> Sprite {
        Sprite {
            name: "hero".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{skin}".to_string(), "#FFCC99".to_string()),
                ("{hair}".to_string(), "#333333".to_string()),
            ])),
            metadata: None,
            ..Default::default()
        }
    }

    fn hero_red_variant() -> Variant {
        Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([("{skin}".to_string(), "#FF6666".to_string())]),
            ..Default::default()
        }
    }

    fn hero_alt_variant() -> Variant {
        Variant {
            name: "hero_alt".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([
                ("{skin}".to_string(), "#66FF66".to_string()),
                ("{hair}".to_string(), "#FFFF00".to_string()),
            ]),
            ..Default::default()
        }
    }

    fn bad_base_variant() -> Variant {
        Variant {
            name: "ghost".to_string(),
            base: "nonexistent".to_string(),
            palette: HashMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_strict_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        assert_eq!(result.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
    }

    #[test]
    fn test_resolve_strict_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(result, Err(PaletteError::BuiltinNotFound("nonexistent".to_string())));
    }

    #[test]
    fn test_resolve_lenient_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Builtin("gameboy".to_string()));
        assert_eq!(result.palette.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
    }

    #[test]
    fn test_resolve_lenient_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert_eq!(result.warning.unwrap().message, "Built-in palette 'nonexistent' not found");
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_combined_builtin_strict() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PaletteError::BuiltinNotFound("nonexistent".to_string()));
    }

    #[test]
    fn test_resolve_combined_builtin_lenient() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, false).unwrap();
        assert!(result.warning.is_some());
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // Test fixture matching plan doc:
    // {"type": "sprite", "name": "test", "palette": "@gameboy", "regions": {...}}
    #[test]
    fn test_fixture_builtin_palette() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        // Verify correct gameboy colors
        assert_eq!(result.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
        assert_eq!(result.colors.get("{light}"), Some(&"#8BAC0F".to_string()));
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
        assert_eq!(result.colors.get("{darkest}"), Some(&"#0F380F".to_string()));
    }

    #[test]
    fn test_all_builtins_resolvable() {
        let registry = PaletteRegistry::new();
        let builtin_names = ["gameboy", "nes", "pico8", "grayscale", "1bit"];

        for name in builtin_names {
            let sprite = Sprite {
                name: "test".to_string(),
                size: None,
                palette: PaletteRef::Named(format!("@{}", name)),

                metadata: None,
                ..Default::default()
            };
            let result = registry.resolve_strict(&sprite);
            assert!(result.is_ok(), "Built-in palette @{} should be resolvable", name);
            assert_eq!(result.unwrap().source, PaletteSource::Builtin(name.to_string()));
        }
    }

    #[test]
    fn test_sprite_registry_new() {
        let registry = SpriteRegistry::new();
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_sprite_registry_register_sprite() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());

        assert!(registry.contains("hero"));
        assert!(registry.get_sprite("hero").is_some());
        assert!(registry.get_variant("hero").is_none());
    }

    #[test]
    fn test_sprite_registry_register_variant() {
        let mut registry = SpriteRegistry::new();
        registry.register_variant(hero_red_variant());

        assert!(registry.contains("hero_red"));
        assert!(registry.get_sprite("hero_red").is_none());
        assert!(registry.get_variant("hero_red").is_some());
    }

    #[test]
    fn test_sprite_registry_resolve_sprite() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero");
        assert_eq!(result.size, Some([4, 4]));
        assert_eq!(result.palette.get("{skin}"), Some(&"#FFCC99".to_string()));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_sprite_registry_resolve_variant_single_override() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero_red");
        assert_eq!(result.size, Some([4, 4])); // Inherited from base

        // skin should be overridden
        assert_eq!(result.palette.get("{skin}"), Some(&"#FF6666".to_string()));
        // hair and _ should be inherited from base
        assert_eq!(result.palette.get("{hair}"), Some(&"#333333".to_string()));
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));

        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_sprite_registry_resolve_variant_multiple_overrides() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_alt_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero_alt", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero_alt");

        // Both skin and hair should be overridden
        assert_eq!(result.palette.get("{skin}"), Some(&"#66FF66".to_string()));
        assert_eq!(result.palette.get("{hair}"), Some(&"#FFFF00".to_string()));
        // _ should be inherited from base
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));
    }

    #[test]
    fn test_sprite_registry_variant_unknown_base_strict() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_variant(bad_base_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("ghost", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::BaseNotFound { variant, base } => {
                assert_eq!(variant, "ghost");
                assert_eq!(base, "nonexistent");
            }
            _ => panic!("Expected BaseNotFound error"),
        }
    }

    #[test]
    fn test_sprite_registry_variant_unknown_base_lenient() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_variant(bad_base_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("ghost", &palette_registry, false).unwrap();
        assert_eq!(result.name, "ghost");
        assert!(result.palette.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_sprite_registry_not_found_strict() {
        let sprite_registry = SpriteRegistry::new();
        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("missing", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::NotFound(name) => assert_eq!(name, "missing"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_sprite_registry_not_found_lenient() {
        let sprite_registry = SpriteRegistry::new();
        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("missing", &palette_registry, false).unwrap();
        assert_eq!(result.name, "missing");
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_sprite_registry_variant_preserves_grid() {
        // Ensure variant copies base grid exactly
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let sprite_result = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        let variant_result = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();

        // Size should be identical
        assert_eq!(sprite_result.size, variant_result.size);
    }

    #[test]
    fn test_sprite_registry_variant_with_named_palette() {
        // Test variant of a sprite that uses a named palette
        let mut sprite_registry = SpriteRegistry::new();
        let mut palette_registry = PaletteRegistry::new();

        palette_registry.register(mono_palette());

        let sprite = checker_sprite_named();
        sprite_registry.register_sprite(sprite);

        // Create a variant that overrides {on}
        let variant = Variant {
            name: "checker_red".to_string(),
            base: "checker".to_string(),
            palette: HashMap::from([("{on}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        };
        sprite_registry.register_variant(variant);

        let result = sprite_registry.resolve("checker_red", &palette_registry, false).unwrap();
        assert_eq!(result.name, "checker_red");
        // {on} should be overridden
        assert_eq!(result.palette.get("{on}"), Some(&"#FF0000".to_string()));
        // {off} and {_} should be inherited from the mono palette
        assert_eq!(result.palette.get("{off}"), Some(&"#000000".to_string()));
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));
    }

    #[test]
    fn test_sprite_registry_names() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"hero".to_string()));
        assert!(names.contains(&&"hero_red".to_string()));
    }

    // ========== Transform Resolution Tests ==========

    #[test]
    fn test_resolve_sprite_with_source() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite that sources from base
        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#00FF00".to_string()), // Different color
            ])),
            // Empty grid - should get from source
            source: Some("base".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Resolve derived should get base's grid
        let result = sprite_registry.resolve("derived", &palette_registry, false).unwrap();

        assert_eq!(result.name, "derived");
    }

    #[test]
    fn test_resolve_sprite_with_mirror_h_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with horizontal mirror transform
        let mirrored = Sprite {
            name: "mirrored".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),

            source: Some("base".to_string()),
            transform: Some(vec![TransformSpec::String("mirror-h".to_string())]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(mirrored);

        let _result = sprite_registry.resolve("mirrored", &palette_registry, false).unwrap();
    }

    #[test]
    fn test_resolve_sprite_with_rotate_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a 2x2 base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{1}".to_string(), "#FF0000".to_string()),
                ("{2}".to_string(), "#00FF00".to_string()),
                ("{3}".to_string(), "#0000FF".to_string()),
                ("{4}".to_string(), "#FFFF00".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with 90 degree rotation
        let rotated = Sprite {
            name: "rotated".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("base".to_string()),
            transform: Some(vec![TransformSpec::String("rotate:90".to_string())]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(rotated);

        let _result = sprite_registry.resolve("rotated", &palette_registry, false).unwrap();

        // 90 degree clockwise rotation:
        // Original:    Rotated:
        // {1}{2}       {3}{1}
        // {3}{4}       {4}{2}
    }

    #[test]
    fn test_resolve_sprite_with_chained_transforms() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with chained transforms: mirror-h then tile 2x1
        let transformed = Sprite {
            name: "transformed".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("base".to_string()),
            transform: Some(vec![
                TransformSpec::String("mirror-h".to_string()),
                TransformSpec::String("tile:2x1".to_string()),
            ]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(transformed);

        let _result = sprite_registry.resolve("transformed", &palette_registry, false).unwrap();

        // First mirror-h: "{a}{b}" -> "{b}{a}"
        // Then tile 2x1: "{b}{a}" -> "{b}{a}{b}{a}"
    }

    /// Test that derived sprites with source + transform properly inherit regions and size
    /// from the source sprite. This tests the fix for TTP-c948t where transforms on
    /// regions-based source sprites produced 0x0 output.
    #[test]
    fn test_resolve_derived_sprite_inherits_source_regions_and_size() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a regions-based source sprite with explicit size
        let base = Sprite {
            name: "base".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Inline(HashMap::from([("bg".to_string(), "#FF0000".to_string())])),
            regions: Some(HashMap::from([(
                "bg".to_string(),
                RegionDef { rect: Some([0, 0, 4, 4]), ..Default::default() },
            )])),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a derived sprite with source + transform (no explicit regions or size)
        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([("bg".to_string(), "#FF0000".to_string())])),
            source: Some("base".to_string()),
            transform: Some(vec![TransformSpec::String("skew-x:26.57".to_string())]),
            regions: None, // No explicit regions - should inherit from source
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Resolve the derived sprite
        let result = sprite_registry.resolve("derived", &palette_registry, false).unwrap();

        // Verify the derived sprite inherited size from source
        assert_eq!(result.size, Some([4, 4]), "Derived sprite should inherit size from source");

        // Verify the derived sprite inherited regions from source
        assert!(result.regions.is_some(), "Derived sprite should inherit regions from source");
        let regions = result.regions.as_ref().unwrap();
        assert!(regions.contains_key("bg"), "Regions should contain 'bg' from source");
    }

    #[test]
    fn test_resolve_sprite_source_not_found_strict() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("nonexistent".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Strict mode should error
        let result = sprite_registry.resolve("derived", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::SourceNotFound { sprite, source_name } => {
                assert_eq!(sprite, "derived");
                assert_eq!(source_name, "nonexistent");
            }
            e => panic!("Expected SourceNotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_sprite_source_not_found_lenient() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("nonexistent".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Lenient mode should return empty grid with warning
        let result = sprite_registry.resolve("derived", &palette_registry, false).unwrap();
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_resolve_sprite_circular_reference_strict() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Create sprites that reference each other
        let a = Sprite {
            name: "a".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("b".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        let b = Sprite {
            name: "b".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            source: Some("a".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(a);
        sprite_registry.register_sprite(b);

        // Strict mode should detect circular reference
        let result = sprite_registry.resolve("a", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::CircularReference { sprite, chain } => {
                assert_eq!(sprite, "a");
                assert!(chain.len() >= 2);
            }
            e => panic!("Expected CircularReference, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_variant_with_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Variant with transform
        let variant = Variant {
            name: "variant".to_string(),
            base: "base".to_string(),
            palette: HashMap::from([("{a}".to_string(), "#0000FF".to_string())]),
            transform: Some(vec![TransformSpec::String("mirror-h".to_string())]),
        };
        sprite_registry.register_variant(variant);

        let result = sprite_registry.resolve("variant", &palette_registry, false).unwrap();

        // Palette should have overridden color
        assert_eq!(result.palette.get("{a}").unwrap(), "#0000FF");
        // Original color for {b} should be from base
        assert_eq!(result.palette.get("{b}").unwrap(), "#00FF00");
    }

    // ========== Registry Trait Tests ==========

    #[test]
    fn test_palette_registry_trait_len_and_empty() {
        let mut registry = PaletteRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(mono_palette());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register(Palette {
            name: "other".to_string(),
            colors: HashMap::new(),
            ..Default::default()
        });
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_palette_registry_trait_names() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        registry.register(Palette {
            name: "other".to_string(),
            colors: HashMap::new(),
            ..Default::default()
        });

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"mono".to_string()));
        assert!(names.contains(&&"other".to_string()));
    }

    #[test]
    fn test_palette_registry_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        assert_eq!(check_registry::<Palette>(&registry), 1);
    }

    #[test]
    fn test_sprite_registry_len_and_empty() {
        let mut registry = SpriteRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register_sprite(hero_sprite());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.sprite_count(), 1);
        assert_eq!(registry.variant_count(), 0);

        registry.register_variant(hero_red_variant());
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.sprite_count(), 1);
        assert_eq!(registry.variant_count(), 1);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_sprite_registry_sprite_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Via the Sprite trait, only sprites count
        assert_eq!(check_registry::<Sprite>(&registry), 1);
        // Via the Variant trait, only variants count
        assert_eq!(check_registry::<Variant>(&registry), 1);
    }

    #[test]
    fn test_sprite_registry_trait_contains() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Registry<Sprite> contains checks sprites only
        assert!(Registry::<Sprite>::contains(&registry, "hero"));
        assert!(!Registry::<Sprite>::contains(&registry, "hero_red"));

        // Registry<Variant> contains checks variants only
        assert!(!Registry::<Variant>::contains(&registry, "hero"));
        assert!(Registry::<Variant>::contains(&registry, "hero_red"));

        // The regular contains method checks both
        assert!(registry.contains("hero"));
        assert!(registry.contains("hero_red"));
    }

    #[test]
    fn test_sprite_registry_trait_get() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Registry<Sprite>::get returns sprites
        let sprite = Registry::<Sprite>::get(&registry, "hero");
        assert!(sprite.is_some());
        assert_eq!(sprite.unwrap().name, "hero");

        // Registry<Variant>::get returns variants
        let variant = Registry::<Variant>::get(&registry, "hero_red");
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().name, "hero_red");
    }

    // ========== CompositionRegistry Tests (NC-1) ==========

    fn test_composition() -> Composition {
        Composition {
            name: "hero_scene".to_string(),
            base: None,
            size: Some([16, 16]),
            cell_size: Some([8, 8]),
            sprites: HashMap::from([
                ("hero".to_string(), Some("hero".to_string())),
                ("bg".to_string(), Some("background".to_string())),
            ]),
            layers: vec![],
        }
    }

    fn alt_composition() -> Composition {
        Composition {
            name: "alt_scene".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        }
    }

    #[test]
    fn test_composition_registry_new() {
        let registry = CompositionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_composition_registry_register_and_get() {
        let mut registry = CompositionRegistry::new();
        let composition = test_composition();
        registry.register(composition);

        assert!(registry.contains("hero_scene"));
        let retrieved = registry.get("hero_scene").unwrap();
        assert_eq!(retrieved.name, "hero_scene");
        assert_eq!(retrieved.size, Some([16, 16]));
    }

    #[test]
    fn test_composition_registry_register_overwrites() {
        let mut registry = CompositionRegistry::new();
        let comp1 = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        let comp2 = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([16, 16]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };

        registry.register(comp1);
        registry.register(comp2);

        assert_eq!(registry.len(), 1);
        let retrieved = registry.get("scene").unwrap();
        assert_eq!(retrieved.size, Some([16, 16]));
    }

    #[test]
    fn test_composition_registry_len_and_empty() {
        let mut registry = CompositionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(test_composition());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register(alt_composition());
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_composition_registry_names() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        registry.register(alt_composition());

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"hero_scene".to_string()));
        assert!(names.contains(&&"alt_scene".to_string()));
    }

    #[test]
    fn test_composition_registry_iter() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        registry.register(alt_composition());

        let items: Vec<_> = registry.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_composition_registry_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        assert_eq!(check_registry::<Composition>(&registry), 1);
    }

    #[test]
    fn test_composition_registry_trait_contains_and_get() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());

        assert!(Registry::<Composition>::contains(&registry, "hero_scene"));
        assert!(!Registry::<Composition>::contains(&registry, "nonexistent"));

        let composition = Registry::<Composition>::get(&registry, "hero_scene");
        assert!(composition.is_some());
        assert_eq!(composition.unwrap().name, "hero_scene");
    }

    // ========== Renderable and Unified Lookup Tests (NC-1) ==========

    #[test]
    fn test_renderable_sprite() {
        let sprite = hero_sprite();
        let renderable = Renderable::Sprite(&sprite);

        assert_eq!(renderable.name(), "hero");
        assert!(renderable.is_sprite());
        assert!(!renderable.is_composition());
        assert!(renderable.as_sprite().is_some());
        assert!(renderable.as_composition().is_none());
    }

    #[test]
    fn test_renderable_composition() {
        let composition = test_composition();
        let renderable = Renderable::Composition(&composition);

        assert_eq!(renderable.name(), "hero_scene");
        assert!(!renderable.is_sprite());
        assert!(renderable.is_composition());
        assert!(renderable.as_sprite().is_none());
        assert!(renderable.as_composition().is_some());
    }

    #[test]
    fn test_lookup_renderable_finds_sprite() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        let composition_registry = CompositionRegistry::new();

        let result = lookup_renderable("hero", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        let renderable = result.unwrap();
        assert!(renderable.is_sprite());
        assert_eq!(renderable.name(), "hero");
    }

    #[test]
    fn test_lookup_renderable_finds_composition() {
        let sprite_registry = SpriteRegistry::new();
        let mut composition_registry = CompositionRegistry::new();
        composition_registry.register(test_composition());

        let result = lookup_renderable("hero_scene", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        let renderable = result.unwrap();
        assert!(renderable.is_composition());
        assert_eq!(renderable.name(), "hero_scene");
    }

    #[test]
    fn test_lookup_renderable_sprite_takes_precedence() {
        // If both sprite and composition have the same name, sprite wins
        let mut sprite_registry = SpriteRegistry::new();
        let sprite = Sprite {
            name: "shared_name".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),

            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(sprite);

        let mut composition_registry = CompositionRegistry::new();
        let composition = Composition {
            name: "shared_name".to_string(),
            base: None,
            size: None,
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        composition_registry.register(composition);

        let result = lookup_renderable("shared_name", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        // Sprite takes precedence
        assert!(result.unwrap().is_sprite());
    }

    #[test]
    fn test_lookup_renderable_not_found() {
        let sprite_registry = SpriteRegistry::new();
        let composition_registry = CompositionRegistry::new();

        let result = lookup_renderable("nonexistent", &sprite_registry, &composition_registry);
        assert!(result.is_none());
    }

    // ========================================================================
    // Palette Ramp Expansion Tests
    // ========================================================================

    #[test]
    fn test_palette_ramp_expansion() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "skin_tones".to_string(),
            colors: HashMap::from([("{_}".to_string(), "#00000000".to_string())]),
            ramps: Some(HashMap::from([(
                "skin".to_string(),
                ColorRamp {
                    base: "#E8B89D".to_string(),
                    steps: Some(3),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
            roles: None,
            relationships: None,
        };

        registry.register(palette);

        let stored = registry.get("skin_tones").unwrap();

        // The ramp should have been expanded into colors
        assert!(stored.colors.contains_key("{skin}"), "Base color token should exist");
        assert!(stored.colors.contains_key("{skin_1}"), "Shadow token should exist");
        assert!(stored.colors.contains_key("{skin+1}"), "Highlight token should exist");

        // Verify base color is correct
        assert_eq!(stored.colors.get("{skin}").unwrap(), "#E8B89D");

        // Ramps should be cleared after expansion
        assert!(stored.ramps.is_none(), "Ramps should be None after expansion");
    }

    #[test]
    fn test_palette_ramp_expansion_5_steps() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "metals".to_string(),
            colors: HashMap::new(),
            ramps: Some(HashMap::from([(
                "gold".to_string(),
                ColorRamp {
                    base: "#FFD700".to_string(),
                    steps: Some(5),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
            roles: None,
            relationships: None,
        };

        registry.register(palette);
        let stored = registry.get("metals").unwrap();

        // 5 steps: _2, _1, base, +1, +2
        assert!(stored.colors.contains_key("{gold_2}"), "Darkest shadow should exist");
        assert!(stored.colors.contains_key("{gold_1}"), "Shadow should exist");
        assert!(stored.colors.contains_key("{gold}"), "Base should exist");
        assert!(stored.colors.contains_key("{gold+1}"), "Highlight should exist");
        assert!(stored.colors.contains_key("{gold+2}"), "Brightest should exist");
        assert_eq!(stored.colors.len(), 5);
    }

    #[test]
    fn test_palette_ramp_preserves_existing_colors() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "character".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{hair}".to_string(), "#4A3728".to_string()),
            ]),
            ramps: Some(HashMap::from([(
                "skin".to_string(),
                ColorRamp {
                    base: "#E8B89D".to_string(),
                    steps: Some(3),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
            roles: None,
            relationships: None,
        };

        registry.register(palette);
        let stored = registry.get("character").unwrap();

        // Original colors should still be there
        assert_eq!(stored.colors.get("{_}").unwrap(), "#00000000");
        assert_eq!(stored.colors.get("{hair}").unwrap(), "#4A3728");

        // Plus the ramp colors
        assert!(stored.colors.contains_key("{skin}"));
        assert!(stored.colors.contains_key("{skin_1}"));
        assert!(stored.colors.contains_key("{skin+1}"));
    }

    #[test]
    fn test_palette_multiple_ramps() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "sprite".to_string(),
            colors: HashMap::new(),
            ramps: Some(HashMap::from([
                (
                    "skin".to_string(),
                    ColorRamp {
                        base: "#E8B89D".to_string(),
                        steps: Some(3),
                        shadow_shift: None,
                        highlight_shift: None,
                    },
                ),
                (
                    "hair".to_string(),
                    ColorRamp {
                        base: "#4A3728".to_string(),
                        steps: Some(3),
                        shadow_shift: None,
                        highlight_shift: None,
                    },
                ),
            ])),
            roles: None,
            relationships: None,
        };

        registry.register(palette);
        let stored = registry.get("sprite").unwrap();

        // Both ramps should be expanded
        assert!(stored.colors.contains_key("{skin}"));
        assert!(stored.colors.contains_key("{skin_1}"));
        assert!(stored.colors.contains_key("{skin+1}"));
        assert!(stored.colors.contains_key("{hair}"));
        assert!(stored.colors.contains_key("{hair_1}"));
        assert!(stored.colors.contains_key("{hair+1}"));
    }
}
