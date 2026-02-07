//! Sprite-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::palette::PaletteRef;
use super::region::RegionDef;
use super::transform::TransformSpec;
use crate::antialias::AntialiasConfig;

/// Nine-slice region definition for scalable sprites.
///
/// Nine-slice (or 9-patch) sprites have fixed corners and stretchable edges/center,
/// allowing them to be scaled without distorting the corners. Common for UI elements
/// like buttons, panels, and dialog boxes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NineSlice {
    /// Left border width in pixels
    pub left: u32,
    /// Right border width in pixels
    pub right: u32,
    /// Top border height in pixels
    pub top: u32,
    /// Bottom border height in pixels
    pub bottom: u32,
}

/// A collision box (hit/hurt/collide/trigger region).
///
/// Used for game engine integration to define collision regions on sprites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollisionBox {
    /// Box X position relative to sprite origin
    pub x: i32,
    /// Box Y position relative to sprite origin
    pub y: i32,
    /// Box width in pixels
    pub w: u32,
    /// Box height in pixels
    pub h: u32,
}

/// Sprite metadata for game engine integration.
///
/// Contains origin point, collision boxes, and attachment points for sprites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SpriteMetadata {
    /// Sprite origin point `[x, y]` - used for positioning and rotation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub origin: Option<[i32; 2]>,
    /// Collision boxes (hit, hurt, collide, trigger, etc.)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub boxes: Option<HashMap<String, CollisionBox>>,
    /// Where this sprite connects to parent/previous segment in a chain `[x, y]`
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub attach_in: Option<[i32; 2]>,
    /// Where the next segment attaches to this sprite `[x, y]`
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub attach_out: Option<[i32; 2]>,
}

/// Per-frame metadata for animations.
///
/// Allows defining frame-specific collision boxes that change during animation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FrameMetadata {
    /// Per-frame collision boxes (can override or nullify sprite-level boxes)
    /// Use `null` value to disable a box for this frame
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub boxes: Option<HashMap<String, Option<CollisionBox>>>,
}

/// A frame tag for game engine integration - identifies named ranges of frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameTag {
    /// Start frame index (0-based, inclusive)
    pub start: u32,
    /// End frame index (0-based, inclusive)
    pub end: u32,
    /// Whether this tag's frames should loop (overrides animation default)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub r#loop: Option<bool>,
    /// Tag-specific FPS override
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fps: Option<u32>,
}

/// A sprite definition.
///
/// A sprite uses `regions` for structured rendering, or can reference another sprite via `source`
/// with optional transforms applied. The `regions` and `source` fields are mutually exclusive.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Sprite {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<[u32; 2]>,
    pub palette: PaletteRef,
    /// Reference to another sprite by name (mutually exclusive with `regions`)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source: Option<String>,
    /// Structured regions for rendering
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub regions: Option<HashMap<String, RegionDef>>,
    /// Transforms to apply when resolving this sprite
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<Vec<TransformSpec>>,
    /// Sprite metadata for game engine integration (origin, collision boxes)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<SpriteMetadata>,
    /// Nine-slice region definition for scalable UI sprites
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nine_slice: Option<NineSlice>,
    /// Per-sprite antialiasing configuration (overrides atlas/defaults)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub antialias: Option<AntialiasConfig>,
    /// Grid-based pixel data (alternative to `regions`).
    ///
    /// Each string is one row of `{token}` patterns, e.g. `"{skin}{eye}{skin}"`.
    /// Mutually exclusive with `regions`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub grid: Option<Vec<String>>,
}
