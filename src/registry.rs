//! Palette registry for resolving palette references
//!
//! The registry stores named palettes and resolves palette references for sprites.
//! Supports both lenient mode (warnings + fallback) and strict mode (errors).

use std::collections::HashMap;
use std::fmt;

use crate::models::{Palette, PaletteRef, Sprite};

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
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaletteError::NotFound(name) => write!(f, "Palette '{}' not found", name),
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
    pub fn resolve_strict(&self, sprite: &Sprite) -> Result<ResolvedPalette, PaletteError> {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                if let Some(palette) = self.palettes.get(name) {
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
    pub fn resolve_lenient(&self, sprite: &Sprite) -> LenientResult {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                if let Some(palette) = self.palettes.get(name) {
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
    pub fn resolve(
        &self,
        sprite: &Sprite,
        strict: bool,
    ) -> Result<LenientResult, PaletteError> {
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
        assert_eq!(result.unwrap_err(), PaletteError::NotFound("nonexistent".to_string()));
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
}
