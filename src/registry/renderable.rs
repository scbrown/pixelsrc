//! Unified renderable lookup across sprite and composition registries.

use crate::models::{Composition, Sprite};

use super::composition::CompositionRegistry;
use super::sprite::SpriteRegistry;

/// A renderable entity that can be either a sprite or a composition.
///
/// This enum provides unified lookup across sprite and composition registries,
/// allowing rendering code to handle both types through a single interface.
#[derive(Debug, Clone)]
pub enum Renderable<'a> {
    /// A sprite (direct or resolved from variant)
    Sprite(&'a Sprite),
    /// A composition of layered sprites
    Composition(&'a Composition),
}

impl<'a> Renderable<'a> {
    /// Get the name of the renderable entity.
    pub fn name(&self) -> &str {
        match self {
            Renderable::Sprite(sprite) => &sprite.name,
            Renderable::Composition(composition) => &composition.name,
        }
    }

    /// Check if this is a sprite.
    pub fn is_sprite(&self) -> bool {
        matches!(self, Renderable::Sprite(_))
    }

    /// Check if this is a composition.
    pub fn is_composition(&self) -> bool {
        matches!(self, Renderable::Composition(_))
    }

    /// Get the sprite if this is a Sprite variant.
    pub fn as_sprite(&self) -> Option<&'a Sprite> {
        match self {
            Renderable::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    /// Get the composition if this is a Composition variant.
    pub fn as_composition(&self) -> Option<&'a Composition> {
        match self {
            Renderable::Composition(composition) => Some(composition),
            _ => None,
        }
    }
}

/// Look up a renderable by name across sprite and composition registries.
///
/// Searches sprites first, then compositions. Returns the first match found.
/// This enables unified rendering where a name can refer to either a sprite
/// or a composition without the caller needing to know which.
pub fn lookup_renderable<'a>(
    name: &str,
    sprite_registry: &'a SpriteRegistry,
    composition_registry: &'a CompositionRegistry,
) -> Option<Renderable<'a>> {
    // Check sprites first (including variants via the direct sprite lookup)
    if let Some(sprite) = sprite_registry.get_sprite(name) {
        return Some(Renderable::Sprite(sprite));
    }

    // Then check compositions
    if let Some(composition) = composition_registry.get(name) {
        return Some(Renderable::Composition(composition));
    }

    None
}
