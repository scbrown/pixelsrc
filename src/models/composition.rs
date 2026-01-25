//! Composition types for layering sprites.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::VarOr;
use super::transform::TransformSpec;

/// A layer within a composition.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CompositionLayer {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub map: Option<Vec<String>>,
    /// Transforms to apply to this layer
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<Vec<TransformSpec>>,
    /// Blend mode for this layer (ATF-10). Default: "normal"
    /// Supports var() syntax for CSS variable references (CSS-9).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub blend: Option<String>,
    /// Layer opacity from 0.0 (transparent) to 1.0 (opaque). Default: 1.0
    /// Supports var() syntax for CSS variable references (CSS-9).
    /// Can be a number (0.5) or a var() string ("var(--layer-opacity)").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub opacity: Option<VarOr<f64>>,
}

/// A composition that layers sprites onto a canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Composition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cell_size: Option<[u32; 2]>,
    pub sprites: HashMap<String, Option<String>>,
    pub layers: Vec<CompositionLayer>,
}

impl Composition {
    /// Default cell size when not specified: 1x1 pixels.
    pub const DEFAULT_CELL_SIZE: [u32; 2] = [1, 1];

    /// Returns the cell size for tiling (default: [1, 1]).
    pub fn cell_size(&self) -> [u32; 2] {
        self.cell_size.unwrap_or(Self::DEFAULT_CELL_SIZE)
    }
}
