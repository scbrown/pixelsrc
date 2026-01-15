//! Palette registry for resolving palette references
//!
//! The registry stores named palettes and resolves palette references for sprites.
//! Supports both lenient mode (warnings + fallback) and strict mode (errors).

use std::collections::HashMap;
use std::fmt;

use crate::models::{Palette, PaletteRef, Sprite, Variant};
use crate::palettes;

/// Magenta fallback color for missing palettes/tokens
pub const MAGENTA_FALLBACK: &str = "#FF00FF";

/// A resolved palette ready for rendering - maps tokens to color strings.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPalette {
    pub colors: HashMap<String, String>,
    pub source: PaletteSource,
}

/// Indicates where the resolved palette came from.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteSource {
    /// Resolved from a named palette in the registry
    Named(String),
    /// Resolved from a built-in palette (@name syntax)
    Builtin(String),
    /// Inline palette defined in the sprite
    Inline,
    /// Fallback used when palette was not found (lenient mode)
    Fallback,
}

/// Error when resolving a palette in strict mode.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteError {
    /// Referenced palette name was not found in registry
    NotFound(String),
    /// Referenced built-in palette (@name) was not found
    BuiltinNotFound(String),
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaletteError::NotFound(name) => write!(f, "Palette '{}' not found", name),
            PaletteError::BuiltinNotFound(name) => {
                write!(f, "Built-in palette '{}' not found", name)
            }
        }
    }
}

impl std::error::Error for PaletteError {}

/// Warning when resolving a palette in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteWarning {
    pub message: String,
}

impl PaletteWarning {
    pub fn not_found(name: &str) -> Self {
        Self {
            message: format!("Palette '{}' not found", name),
        }
    }

    pub fn builtin_not_found(name: &str) -> Self {
        Self {
            message: format!("Built-in palette '{}' not found", name),
        }
    }
}

/// Resolution result for lenient mode - always succeeds but may have warnings.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientResult {
    pub palette: ResolvedPalette,
    pub warning: Option<PaletteWarning>,
}

/// Registry for named palettes.
#[derive(Debug, Clone, Default)]
pub struct PaletteRegistry {
    palettes: HashMap<String, Palette>,
}

impl PaletteRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            palettes: HashMap::new(),
        }
    }

    /// Register a palette in the registry.
    ///
    /// If a palette with the same name already exists, it is replaced.
    pub fn register(&mut self, palette: Palette) {
        self.palettes.insert(palette.name.clone(), palette);
    }

    /// Get a palette by name.
    pub fn get(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(name)
    }

    /// Check if a palette with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.palettes.contains_key(name)
    }

    /// Resolve a sprite's palette reference in strict mode.
    ///
    /// Returns an error if a named palette is not found.
    /// Handles @name syntax for built-in palettes.
    pub fn resolve_strict(&self, sprite: &Sprite) -> Result<ResolvedPalette, PaletteError> {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                // Check for built-in palette reference (@name syntax)
                if let Some(builtin_name) = name.strip_prefix('@') {
                    if let Some(palette) = palettes::get_builtin(builtin_name) {
                        Ok(ResolvedPalette {
                            colors: palette.colors.clone(),
                            source: PaletteSource::Builtin(builtin_name.to_string()),
                        })
                    } else {
                        Err(PaletteError::BuiltinNotFound(builtin_name.to_string()))
                    }
                } else if let Some(palette) = self.palettes.get(name) {
                    Ok(ResolvedPalette {
                        colors: palette.colors.clone(),
                        source: PaletteSource::Named(name.clone()),
                    })
                } else {
                    Err(PaletteError::NotFound(name.clone()))
                }
            }
            PaletteRef::Inline(colors) => Ok(ResolvedPalette {
                colors: colors.clone(),
                source: PaletteSource::Inline,
            }),
        }
    }

    /// Resolve a sprite's palette reference in lenient mode.
    ///
    /// Always returns a palette. If a named palette is not found, returns
    /// an empty fallback palette with a warning.
    /// Handles @name syntax for built-in palettes.
    pub fn resolve_lenient(&self, sprite: &Sprite) -> LenientResult {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                // Check for built-in palette reference (@name syntax)
                if let Some(builtin_name) = name.strip_prefix('@') {
                    if let Some(palette) = palettes::get_builtin(builtin_name) {
                        LenientResult {
                            palette: ResolvedPalette {
                                colors: palette.colors.clone(),
                                source: PaletteSource::Builtin(builtin_name.to_string()),
                            },
                            warning: None,
                        }
                    } else {
                        // Fallback: empty palette (tokens will get magenta during rendering)
                        LenientResult {
                            palette: ResolvedPalette {
                                colors: HashMap::new(),
                                source: PaletteSource::Fallback,
                            },
                            warning: Some(PaletteWarning::builtin_not_found(builtin_name)),
                        }
                    }
                } else if let Some(palette) = self.palettes.get(name) {
                    LenientResult {
                        palette: ResolvedPalette {
                            colors: palette.colors.clone(),
                            source: PaletteSource::Named(name.clone()),
                        },
                        warning: None,
                    }
                } else {
                    // Fallback: empty palette (tokens will get magenta during rendering)
                    LenientResult {
                        palette: ResolvedPalette {
                            colors: HashMap::new(),
                            source: PaletteSource::Fallback,
                        },
                        warning: Some(PaletteWarning::not_found(name)),
                    }
                }
            }
            PaletteRef::Inline(colors) => LenientResult {
                palette: ResolvedPalette {
                    colors: colors.clone(),
                    source: PaletteSource::Inline,
                },
                warning: None,
            },
        }
    }

    /// Resolve a sprite's palette reference.
    ///
    /// In strict mode, returns an error for missing palettes.
    /// In lenient mode, returns a fallback with a warning.
    pub fn resolve(&self, sprite: &Sprite, strict: bool) -> Result<LenientResult, PaletteError> {
        if strict {
            self.resolve_strict(sprite).map(|palette| LenientResult {
                palette,
                warning: None,
            })
        } else {
            Ok(self.resolve_lenient(sprite))
        }
    }
}

// ============================================================================
// Sprite and Variant Registry
// ============================================================================

/// Error when resolving a sprite or variant.
#[derive(Debug, Clone, PartialEq)]
pub enum SpriteError {
    /// Referenced sprite/variant was not found
    NotFound(String),
    /// Variant references a base sprite that doesn't exist
    BaseNotFound { variant: String, base: String },
}

impl fmt::Display for SpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpriteError::NotFound(name) => write!(f, "Sprite or variant '{}' not found", name),
            SpriteError::BaseNotFound { variant, base } => {
                write!(
                    f,
                    "Variant '{}' references unknown base sprite '{}'",
                    variant, base
                )
            }
        }
    }
}

impl std::error::Error for SpriteError {}

/// Warning when resolving a sprite or variant in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteWarning {
    pub message: String,
}

impl SpriteWarning {
    pub fn not_found(name: &str) -> Self {
        Self {
            message: format!("Sprite or variant '{}' not found", name),
        }
    }

    pub fn base_not_found(variant: &str, base: &str) -> Self {
        Self {
            message: format!(
                "Variant '{}' references unknown base sprite '{}'",
                variant, base
            ),
        }
    }
}

/// A resolved sprite ready for rendering.
///
/// This can be either a direct sprite or a variant expanded to sprite form.
#[derive(Debug, Clone)]
pub struct ResolvedSprite {
    /// The effective name (sprite name or variant name)
    pub name: String,
    /// The size from the base sprite (if any)
    pub size: Option<[u32; 2]>,
    /// The grid data (from base sprite for variants)
    pub grid: Vec<String>,
    /// The merged palette for rendering (base palette + variant overrides)
    pub palette: HashMap<String, String>,
    /// Any warnings generated during resolution
    pub warnings: Vec<SpriteWarning>,
}

/// Registry for sprites and variants.
///
/// Handles resolution of sprite names to renderable sprites, including
/// expanding variants to their effective sprite representation.
#[derive(Debug, Clone, Default)]
pub struct SpriteRegistry {
    sprites: HashMap<String, Sprite>,
    variants: HashMap<String, Variant>,
}

impl SpriteRegistry {
    /// Create a new empty sprite registry.
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            variants: HashMap::new(),
        }
    }

    /// Register a sprite.
    pub fn register_sprite(&mut self, sprite: Sprite) {
        self.sprites.insert(sprite.name.clone(), sprite);
    }

    /// Register a variant.
    pub fn register_variant(&mut self, variant: Variant) {
        self.variants.insert(variant.name.clone(), variant);
    }

    /// Get a sprite by name (does not resolve variants).
    pub fn get_sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    /// Get a variant by name.
    pub fn get_variant(&self, name: &str) -> Option<&Variant> {
        self.variants.get(name)
    }

    /// Check if a name refers to a sprite or variant.
    pub fn contains(&self, name: &str) -> bool {
        self.sprites.contains_key(name) || self.variants.contains_key(name)
    }

    /// Resolve a name to a sprite-like structure, expanding variants.
    ///
    /// In strict mode, returns an error if the name or base is not found.
    /// In lenient mode, returns a fallback result with warnings.
    ///
    /// The returned ResolvedSprite contains the effective grid and merged palette.
    pub fn resolve(
        &self,
        name: &str,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        // First, check if it's a direct sprite
        if let Some(sprite) = self.sprites.get(name) {
            return self.resolve_sprite(sprite, palette_registry, strict);
        }

        // Check if it's a variant
        if let Some(variant) = self.variants.get(name) {
            return self.resolve_variant(variant, palette_registry, strict);
        }

        // Not found
        if strict {
            Err(SpriteError::NotFound(name.to_string()))
        } else {
            Ok(ResolvedSprite {
                name: name.to_string(),
                size: None,
                grid: vec![],
                palette: HashMap::new(),
                warnings: vec![SpriteWarning::not_found(name)],
            })
        }
    }

    /// Resolve a direct sprite to a ResolvedSprite.
    fn resolve_sprite(
        &self,
        sprite: &Sprite,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        let mut warnings = Vec::new();

        // Resolve the sprite's palette
        let palette = match palette_registry.resolve(sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning {
                        message: warning.message,
                    });
                }
                result.palette.colors
            }
            Err(e) => {
                // In strict mode, this would have returned an error from resolve()
                // In lenient mode, we got a fallback. Map the error for strict.
                if strict {
                    // The resolve() function already handles strict vs lenient
                    return Err(SpriteError::NotFound(format!("palette error: {}", e)));
                }
                HashMap::new()
            }
        };

        Ok(ResolvedSprite {
            name: sprite.name.clone(),
            size: sprite.size,
            grid: sprite.grid.clone(),
            palette,
            warnings,
        })
    }

    /// Resolve a variant to a ResolvedSprite by expanding from its base.
    fn resolve_variant(
        &self,
        variant: &Variant,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        // Look up the base sprite
        let base_sprite = match self.sprites.get(&variant.base) {
            Some(sprite) => sprite,
            None => {
                if strict {
                    return Err(SpriteError::BaseNotFound {
                        variant: variant.name.clone(),
                        base: variant.base.clone(),
                    });
                } else {
                    return Ok(ResolvedSprite {
                        name: variant.name.clone(),
                        size: None,
                        grid: vec![],
                        palette: HashMap::new(),
                        warnings: vec![SpriteWarning::base_not_found(&variant.name, &variant.base)],
                    });
                }
            }
        };

        let mut warnings = Vec::new();

        // Resolve the base sprite's palette
        let base_palette = match palette_registry.resolve(base_sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning {
                        message: warning.message,
                    });
                }
                result.palette.colors
            }
            Err(e) => {
                if strict {
                    return Err(SpriteError::NotFound(format!("base palette error: {}", e)));
                }
                HashMap::new()
            }
        };

        // Merge palettes: start with base, override with variant's palette
        let mut merged_palette = base_palette;
        for (token, color) in &variant.palette {
            merged_palette.insert(token.clone(), color.clone());
        }

        Ok(ResolvedSprite {
            name: variant.name.clone(),
            size: base_sprite.size,
            grid: base_sprite.grid.clone(),
            palette: merged_palette,
            warnings,
        })
    }

    /// Get all sprite and variant names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.sprites.keys().chain(self.variants.keys())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mono_palette() -> Palette {
        Palette {
            name: "mono".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{on}".to_string(), "#FFFFFF".to_string()),
                ("{off}".to_string(), "#000000".to_string()),
            ]),
        }
    }

    fn checker_sprite_named() -> Sprite {
        Sprite {
            name: "checker".to_string(),
            size: None,
            palette: PaletteRef::Named("mono".to_string()),
            grid: vec![
                "{on}{off}{on}{off}".to_string(),
                "{off}{on}{off}{on}".to_string(),
            ],
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
            grid: vec!["{x}".to_string()],
        }
    }

    fn bad_ref_sprite() -> Sprite {
        Sprite {
            name: "bad_ref".to_string(),
            size: None,
            palette: PaletteRef::Named("nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
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
        };
        let palette2 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{b}".to_string(), "#00FF00".to_string())]),
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
        assert_eq!(
            result,
            Err(PaletteError::NotFound("nonexistent".to_string()))
        );
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
        assert_eq!(
            result.palette.source,
            PaletteSource::Named("mono".to_string())
        );
    }

    #[test]
    fn test_resolve_lenient_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert!(result
            .warning
            .as_ref()
            .unwrap()
            .message
            .contains("nonexistent"));
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

        // {"type": "sprite", "name": "checker", "palette": "mono", "grid": [...]}
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

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "grid": ["{x}{x}"]}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_strict(&sprite);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            PaletteError::NotFound("nonexistent".to_string())
        );
    }

    #[test]
    fn test_fixture_unknown_palette_ref_lenient() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "grid": ["{x}{x}"]}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_lenient(&sprite);

        assert!(result.warning.is_some());
        assert_eq!(
            result.warning.unwrap().message,
            "Palette 'nonexistent' not found"
        );
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
            grid: vec!["{lightest}{dark}".to_string()],
        }
    }

    fn builtin_nonexistent_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
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
            grid: vec![
                "{_}{hair}{hair}{_}".to_string(),
                "{hair}{skin}{skin}{hair}".to_string(),
                "{_}{skin}{skin}{_}".to_string(),
                "{_}{skin}{skin}{_}".to_string(),
            ],
        }
    }

    fn hero_red_variant() -> Variant {
        Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([("{skin}".to_string(), "#FF6666".to_string())]),
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
        }
    }

    fn bad_base_variant() -> Variant {
        Variant {
            name: "ghost".to_string(),
            base: "nonexistent".to_string(),
            palette: HashMap::new(),
        }
    }

    #[test]
    fn test_resolve_strict_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        assert_eq!(
            result.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
    }

    #[test]
    fn test_resolve_strict_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(
            result,
            Err(PaletteError::BuiltinNotFound("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_resolve_lenient_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(
            result.palette.source,
            PaletteSource::Builtin("gameboy".to_string())
        );
        assert_eq!(
            result.palette.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
    }

    #[test]
    fn test_resolve_lenient_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert_eq!(
            result.warning.unwrap().message,
            "Built-in palette 'nonexistent' not found"
        );
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_combined_builtin_strict() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            PaletteError::BuiltinNotFound("nonexistent".to_string())
        );
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
    // {"type": "sprite", "name": "test", "palette": "@gameboy", "grid": ["{lightest}{dark}"]}
    #[test]
    fn test_fixture_builtin_palette() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        // Verify correct gameboy colors
        assert_eq!(
            result.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
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
                grid: vec!["{_}".to_string()],
            };
            let result = registry.resolve_strict(&sprite);
            assert!(
                result.is_ok(),
                "Built-in palette @{} should be resolvable",
                name
            );
            assert_eq!(
                result.unwrap().source,
                PaletteSource::Builtin(name.to_string())
            );
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

        let result = sprite_registry
            .resolve("hero", &palette_registry, false)
            .unwrap();
        assert_eq!(result.name, "hero");
        assert_eq!(result.size, Some([4, 4]));
        assert_eq!(result.grid.len(), 4);
        assert_eq!(result.palette.get("{skin}"), Some(&"#FFCC99".to_string()));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_sprite_registry_resolve_variant_single_override() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry
            .resolve("hero_red", &palette_registry, false)
            .unwrap();
        assert_eq!(result.name, "hero_red");
        assert_eq!(result.size, Some([4, 4])); // Inherited from base
        assert_eq!(result.grid.len(), 4); // Copied from base

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

        let result = sprite_registry
            .resolve("hero_alt", &palette_registry, false)
            .unwrap();
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

        let result = sprite_registry
            .resolve("ghost", &palette_registry, false)
            .unwrap();
        assert_eq!(result.name, "ghost");
        assert!(result.grid.is_empty());
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

        let result = sprite_registry
            .resolve("missing", &palette_registry, false)
            .unwrap();
        assert_eq!(result.name, "missing");
        assert!(result.grid.is_empty());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_sprite_registry_variant_preserves_grid() {
        // Ensure variant copies base grid exactly
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let sprite_result = sprite_registry
            .resolve("hero", &palette_registry, false)
            .unwrap();
        let variant_result = sprite_registry
            .resolve("hero_red", &palette_registry, false)
            .unwrap();

        // Grid should be identical
        assert_eq!(sprite_result.grid, variant_result.grid);
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
        };
        sprite_registry.register_variant(variant);

        let result = sprite_registry
            .resolve("checker_red", &palette_registry, false)
            .unwrap();
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
}
