//! Region definitions for structured sprites.

use serde::{Deserialize, Serialize};

use super::palette::Role;
use crate::antialias::RegionAAOverride;

/// Jitter specification for controlled randomness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JitterSpec {
    /// Horizontal jitter range: [min, max]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub x: Option<[i32; 2]>,

    /// Vertical jitter range: [min, max]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub y: Option<[i32; 2]>,
}

/// Region definition for structured sprites (Format v2).
///
/// Defines a single region (token) using shape primitives, compound operations,
/// constraints, and modifiers.
///
/// Example:
/// ```json5
/// {
///   "eye": {
///     "rect": [5, 6, 2, 2],
///     "symmetric": "x",
///     "within": "face"
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RegionDef {
    // Shape primitives (exactly one, or compound)
    /// Individual pixels at specific coordinates: [[x, y], ...]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub points: Option<Vec<[u32; 2]>>,

    /// Bresenham line between points: [[x1, y1], [x2, y2], ...]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub line: Option<Vec<[u32; 2]>>,

    /// Filled rectangle: [x, y, width, height]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rect: Option<[u32; 4]>,

    /// Rectangle outline (unfilled): [x, y, width, height]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stroke: Option<[u32; 4]>,

    /// Filled ellipse: [cx, cy, rx, ry]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ellipse: Option<[u32; 4]>,

    /// Shorthand for equal-radius ellipse: [cx, cy, r]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub circle: Option<[u32; 3]>,

    /// Filled polygon from vertices: [[x, y], ...]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub polygon: Option<Vec<[u32; 2]>>,

    /// SVG-lite path syntax (M, L, H, V, Z commands only)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub path: Option<String>,

    /// Flood fill inside a boundary: "inside(token_name)"
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fill: Option<String>,

    // Compound operations
    /// Combine multiple shapes
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub union: Option<Vec<RegionDef>>,

    /// Base shape for subtraction operations
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub base: Option<Box<RegionDef>>,

    /// Remove shapes from base
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subtract: Option<Vec<RegionDef>>,

    /// Keep only overlapping area
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub intersect: Option<Vec<RegionDef>>,

    // Pixel-affecting modifiers (require forward definition)
    /// Subtract these tokens' pixels
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub except: Option<Vec<String>>,

    /// Generate outline around token
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "auto-outline")]
    pub auto_outline: Option<String>,

    /// Generate shadow from token
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "auto-shadow")]
    pub auto_shadow: Option<String>,

    /// Offset for auto-shadow: [x, y]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<[i32; 2]>,

    // Validation constraints (checked after all regions resolved)
    /// Must be inside token's bounds
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub within: Option<String>,

    /// Must touch token
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "adjacent-to")]
    pub adjacent_to: Option<String>,

    // Range constraints
    /// Limit region to specific columns: [min, max]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub x: Option<[u32; 2]>,

    /// Limit region to specific rows: [min, max]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub y: Option<[u32; 2]>,

    // Modifiers
    /// Auto-mirror across axis: "x", "y", "xy", or specific coordinate
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub symmetric: Option<String>,

    /// Explicit render order (default: definition order)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub z: Option<i32>,

    /// Corner radius for rect/stroke
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub round: Option<u32>,

    /// Line thickness for stroke/line
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thickness: Option<u32>,

    // Transform modifiers
    /// Tile a shape: [count_x, count_y]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub repeat: Option<[u32; 2]>,

    /// Spacing between repeated tiles: [x, y]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub spacing: Option<[u32; 2]>,

    /// Offset alternating rows in repeat
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "offset-alternate")]
    pub offset_alternate: Option<bool>,

    /// Geometric transform: rotate/translate/scale
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<String>,

    /// Controlled randomness for jitter
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub jitter: Option<JitterSpec>,

    /// Random seed for jitter
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<u32>,

    // Semantic metadata
    /// Semantic role of this region (boundary, fill, shadow, etc.)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub role: Option<Role>,

    // Antialiasing override
    /// Per-region antialiasing configuration (overrides sprite/atlas/defaults)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub antialias: Option<RegionAAOverride>,
}
