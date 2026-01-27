//! Sprite and variant registry.

use std::collections::HashMap;
use thiserror::Error;

use crate::models::{RegionDef, Sprite, Variant};

use super::palette::PaletteRegistry;
use super::traits::Registry;

/// Error when resolving a sprite or variant.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum SpriteError {
    /// Referenced sprite/variant was not found
    #[error("Sprite or variant '{0}' not found")]
    NotFound(String),
    /// Variant references a base sprite that doesn't exist
    #[error("Variant '{variant}' references unknown base sprite '{base}'")]
    BaseNotFound { variant: String, base: String },
    /// Sprite references a source sprite that doesn't exist
    #[error("Sprite '{sprite}' references unknown source sprite '{source_name}'")]
    SourceNotFound { sprite: String, source_name: String },
    /// Circular reference detected during source resolution
    #[error("Circular reference detected for sprite '{sprite}': {}", chain.join(" -> "))]
    CircularReference { sprite: String, chain: Vec<String> },
    /// Error applying transform
    #[error("Transform error for sprite '{sprite}': {message}")]
    TransformError { sprite: String, message: String },
}

/// Warning when resolving a sprite or variant in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteWarning {
    pub message: String,
}

impl SpriteWarning {
    pub fn not_found(name: &str) -> Self {
        Self { message: format!("Sprite or variant '{}' not found", name) }
    }

    pub fn base_not_found(variant: &str, base: &str) -> Self {
        Self { message: format!("Variant '{}' references unknown base sprite '{}'", variant, base) }
    }

    pub fn source_not_found(sprite: &str, source: &str) -> Self {
        Self {
            message: format!("Sprite '{}' references unknown source sprite '{}'", sprite, source),
        }
    }

    pub fn transform_error(sprite: &str, message: &str) -> Self {
        Self { message: format!("Transform error for sprite '{}': {}", sprite, message) }
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
    /// The merged palette for rendering (base palette + variant overrides)
    pub palette: HashMap<String, String>,
    /// Any warnings generated during resolution
    pub warnings: Vec<SpriteWarning>,
    /// Nine-slice region definition (from base sprite)
    pub nine_slice: Option<crate::models::NineSlice>,
    /// Structured regions for rendering
    pub regions: Option<HashMap<String, RegionDef>>,
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
        Self { sprites: HashMap::new(), variants: HashMap::new() }
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
                palette: HashMap::new(),
                warnings: vec![SpriteWarning::not_found(name)],
                nine_slice: None,
                regions: None,
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
        // Use the internal resolver with cycle detection
        self.resolve_sprite_internal(sprite, palette_registry, strict, &mut Vec::new())
    }

    /// Internal sprite resolution with cycle detection.
    ///
    /// The `visited` parameter tracks sprites in the current resolution chain
    /// to detect circular references.
    fn resolve_sprite_internal(
        &self,
        sprite: &Sprite,
        palette_registry: &PaletteRegistry,
        strict: bool,
        visited: &mut Vec<String>,
    ) -> Result<ResolvedSprite, SpriteError> {
        let mut warnings = Vec::new();

        // Check for circular reference
        if visited.contains(&sprite.name) {
            visited.push(sprite.name.clone());
            if strict {
                return Err(SpriteError::CircularReference {
                    sprite: sprite.name.clone(),
                    chain: visited.clone(),
                });
            } else {
                return Ok(ResolvedSprite {
                    name: sprite.name.clone(),
                    size: None,
                    palette: HashMap::new(),
                    warnings: vec![SpriteWarning {
                        message: format!("Circular reference detected: {}", visited.join(" -> ")),
                    }],
                    nine_slice: None,
                    regions: None,
                });
            }
        }

        // Mark as visited
        visited.push(sprite.name.clone());

        // Resolve source sprite's regions and size if this sprite references another
        let (base_regions, base_size) = if let Some(source_name) = &sprite.source {
            match self.sprites.get(source_name) {
                Some(source_sprite) => {
                    let source_resolved = self.resolve_sprite_internal(
                        source_sprite,
                        palette_registry,
                        strict,
                        visited,
                    )?;
                    warnings.extend(source_resolved.warnings);
                    (source_resolved.regions, source_resolved.size)
                }
                None => {
                    if strict {
                        return Err(SpriteError::SourceNotFound {
                            sprite: sprite.name.clone(),
                            source_name: source_name.clone(),
                        });
                    } else {
                        warnings.push(SpriteWarning::source_not_found(&sprite.name, source_name));
                        (None, None)
                    }
                }
            }
        } else {
            (sprite.regions.clone(), None)
        };

        // Resolve the sprite's palette
        let palette = match palette_registry.resolve(sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning { message: warning.message });
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
            // Use sprite's explicit size, or fall back to source sprite's size
            size: sprite.size.or(base_size),
            palette,
            warnings,
            nine_slice: sprite.nine_slice.clone(),
            regions: base_regions,
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
                        palette: HashMap::new(),
                        warnings: vec![SpriteWarning::base_not_found(&variant.name, &variant.base)],
                        nine_slice: None,
                        regions: None,
                    });
                }
            }
        };

        let mut warnings = Vec::new();

        // Resolve the base sprite's palette
        let base_palette = match palette_registry.resolve(base_sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning { message: warning.message });
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
            palette: merged_palette,
            warnings,
            nine_slice: base_sprite.nine_slice.clone(),
            regions: base_sprite.regions.clone(),
        })
    }

    /// Get all sprite and variant names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.sprites.keys().chain(self.variants.keys())
    }

    /// Get the total number of sprites and variants in the registry.
    pub fn len(&self) -> usize {
        self.sprites.len() + self.variants.len()
    }

    /// Check if the registry is empty (no sprites or variants).
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty() && self.variants.is_empty()
    }

    /// Clear all sprites and variants from the registry.
    pub fn clear(&mut self) {
        self.sprites.clear();
        self.variants.clear();
    }

    /// Get the number of sprites (excluding variants).
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }

    /// Get the number of variants (excluding sprites).
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    /// Iterate over all sprites in the registry.
    pub fn sprites(&self) -> impl Iterator<Item = (&String, &Sprite)> {
        self.sprites.iter()
    }

    /// Iterate over all variants in the registry.
    pub fn variants(&self) -> impl Iterator<Item = (&String, &Variant)> {
        self.variants.iter()
    }
}

impl Registry<Sprite> for SpriteRegistry {
    fn contains(&self, name: &str) -> bool {
        self.sprites.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    fn len(&self) -> usize {
        self.sprites.len()
    }

    fn clear(&mut self) {
        self.sprites.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.sprites.keys())
    }
}

impl Registry<Variant> for SpriteRegistry {
    fn contains(&self, name: &str) -> bool {
        self.variants.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Variant> {
        self.variants.get(name)
    }

    fn len(&self) -> usize {
        self.variants.len()
    }

    fn clear(&mut self) {
        self.variants.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.variants.keys())
    }
}
