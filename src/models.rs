//! Data models for Pixelsrc objects (palettes, sprites, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A value that can be either a literal value or a CSS variable reference.
///
/// Used for composition layer properties like `opacity` and `blend` that can
/// use `var()` syntax to reference CSS custom properties.
///
/// # Examples
///
/// ```
/// use pixelsrc::models::VarOr;
///
/// // Can be deserialized from either a literal or a var() string
/// let literal: VarOr<f64> = serde_json::from_str("0.5").unwrap();
/// let var_ref: VarOr<f64> = serde_json::from_str("\"var(--opacity)\"").unwrap();
///
/// assert!(matches!(literal, VarOr::Value(0.5)));
/// assert!(matches!(var_ref, VarOr::Var(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VarOr<T> {
    /// A literal value
    Value(T),
    /// A CSS variable reference (e.g., "var(--name)" or "var(--name, fallback)")
    Var(String),
}

impl<T: Default> Default for VarOr<T> {
    fn default() -> Self {
        VarOr::Value(T::default())
    }
}

impl<T> VarOr<T> {
    /// Returns true if this is a var() reference
    pub fn is_var(&self) -> bool {
        matches!(self, VarOr::Var(_))
    }

    /// Returns true if this is a literal value
    pub fn is_value(&self) -> bool {
        matches!(self, VarOr::Value(_))
    }

    /// Returns the literal value if present
    pub fn as_value(&self) -> Option<&T> {
        match self {
            VarOr::Value(v) => Some(v),
            VarOr::Var(_) => None,
        }
    }

    /// Returns the var() string if present
    pub fn as_var(&self) -> Option<&str> {
        match self {
            VarOr::Value(_) => None,
            VarOr::Var(s) => Some(s),
        }
    }
}

impl<T: Copy> VarOr<T> {
    /// Get the value, returning None if it's a var() reference
    pub fn value(&self) -> Option<T> {
        match self {
            VarOr::Value(v) => Some(*v),
            VarOr::Var(_) => None,
        }
    }
}

impl From<f64> for VarOr<f64> {
    fn from(v: f64) -> Self {
        VarOr::Value(v)
    }
}

impl From<String> for VarOr<f64> {
    fn from(s: String) -> Self {
        VarOr::Var(s)
    }
}

/// A duration value that can be either a raw millisecond number or a CSS time string.
///
/// # Examples
///
/// ```
/// use pixelsrc::models::Duration;
///
/// // Can be deserialized from either format
/// let ms: Duration = serde_json::from_str("100").unwrap();
/// let css: Duration = serde_json::from_str("\"500ms\"").unwrap();
///
/// assert_eq!(ms.as_milliseconds(), Some(100));
/// assert_eq!(css.as_milliseconds(), Some(500));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Duration {
    /// Raw milliseconds (backwards compatible)
    Milliseconds(u32),
    /// CSS time string (e.g., "500ms", "1s", "0.5s")
    CssString(String),
}

impl Duration {
    /// Parse the duration and return milliseconds.
    ///
    /// Returns `None` if the CSS string cannot be parsed.
    pub fn as_milliseconds(&self) -> Option<u32> {
        match self {
            Duration::Milliseconds(ms) => Some(*ms),
            Duration::CssString(s) => parse_css_duration(s),
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::Milliseconds(100)
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Duration::Milliseconds(ms) => write!(f, "{}", ms),
            Duration::CssString(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl From<u32> for Duration {
    fn from(ms: u32) -> Self {
        Duration::Milliseconds(ms)
    }
}

impl From<&str> for Duration {
    fn from(s: &str) -> Self {
        Duration::CssString(s.to_string())
    }
}

/// Parse a CSS duration string into milliseconds.
///
/// Supports:
/// - `<number>ms` - milliseconds (e.g., "500ms")
/// - `<number>s` - seconds (e.g., "1.5s")
fn parse_css_duration(s: &str) -> Option<u32> {
    let s = s.trim().to_lowercase();

    if let Some(ms_str) = s.strip_suffix("ms") {
        ms_str.trim().parse::<f64>().ok().map(|v| v as u32)
    } else if let Some(s_str) = s.strip_suffix('s') {
        s_str.trim().parse::<f64>().ok().map(|v| (v * 1000.0) as u32)
    } else {
        // Try parsing as raw number (assume milliseconds)
        s.parse::<f64>().ok().map(|v| v as u32)
    }
}

/// A CSS-style keyframe defining properties at a specific point in an animation.
///
/// Used with percentage keys (e.g., "0%", "50%", "100%") or "from"/"to" aliases.
///
/// # Examples
///
/// ```
/// use pixelsrc::models::CssKeyframe;
///
/// // Keyframe with sprite and opacity
/// let kf: CssKeyframe = serde_json::from_str(r#"{
///     "sprite": "walk_1",
///     "opacity": 1.0
/// }"#).unwrap();
/// assert_eq!(kf.sprite, Some("walk_1".to_string()));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CssKeyframe {
    /// Sprite to display at this keyframe
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sprite: Option<String>,
    /// CSS transform string (e.g., "rotate(45deg) scale(2)")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<String>,
    /// Opacity at this keyframe (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub opacity: Option<f64>,
    /// Position offset at this keyframe `[x, y]`
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<[i32; 2]>,
}

/// Per-step color shift for ramp generation.
///
/// All values are deltas applied per step. For example, `lightness: -15` means
/// each shadow step is 15% darker than the previous.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ColorShift {
    /// Lightness delta per step (-100 to 100)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lightness: Option<f64>,
    /// Hue rotation in degrees per step (-180 to 180)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hue: Option<f64>,
    /// Saturation delta per step (-100 to 100)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub saturation: Option<f64>,
}

impl ColorShift {
    /// Default shadow shift: darker, warmer (hue shifts toward red/orange)
    pub fn default_shadow() -> Self {
        Self { lightness: Some(-15.0), hue: Some(10.0), saturation: Some(5.0) }
    }

    /// Default highlight shift: lighter, cooler (hue shifts toward blue)
    pub fn default_highlight() -> Self {
        Self { lightness: Some(12.0), hue: Some(-5.0), saturation: Some(-10.0) }
    }
}

/// A color ramp definition for automatic color generation.
///
/// Generates a series of colors from shadow to highlight based on a base color
/// with configurable hue/saturation/lightness shifts per step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorRamp {
    /// Base color in CSS format (e.g., "#E8B89D", "rgb(232, 184, 157)")
    pub base: String,
    /// Total number of steps (odd numbers center on base). Default: 3
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub steps: Option<u32>,
    /// Per-step shift toward shadows (applied to steps below base)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub shadow_shift: Option<ColorShift>,
    /// Per-step shift toward highlights (applied to steps above base)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub highlight_shift: Option<ColorShift>,
}

impl ColorRamp {
    /// Default number of steps in a ramp
    pub const DEFAULT_STEPS: u32 = 3;

    /// Returns the number of steps in this ramp
    pub fn steps(&self) -> u32 {
        self.steps.unwrap_or(Self::DEFAULT_STEPS)
    }

    /// Returns the shadow shift, using defaults if not specified
    pub fn shadow_shift(&self) -> ColorShift {
        self.shadow_shift.clone().unwrap_or_else(ColorShift::default_shadow)
    }

    /// Returns the highlight shift, using defaults if not specified
    pub fn highlight_shift(&self) -> ColorShift {
        self.highlight_shift.clone().unwrap_or_else(ColorShift::default_highlight)
    }
}

/// Semantic role for a color token in a palette.
///
/// Roles provide semantic meaning to tokens, enabling tools to understand
/// the purpose of each color in the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Boundary/outline color (edges, borders)
    Boundary,
    /// Anchor/key color (main identifying color)
    Anchor,
    /// Fill color (interior regions)
    Fill,
    /// Shadow color (darker variants for depth)
    Shadow,
    /// Highlight color (lighter variants for emphasis)
    Highlight,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Boundary => write!(f, "boundary"),
            Role::Anchor => write!(f, "anchor"),
            Role::Fill => write!(f, "fill"),
            Role::Shadow => write!(f, "shadow"),
            Role::Highlight => write!(f, "highlight"),
        }
    }
}

/// Type of relationship between palette tokens.
///
/// Defines semantic relationships between color tokens for tooling and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipType {
    /// Token color is derived from another token (e.g., shadow from base)
    DerivesFrom,
    /// Token is visually contained within another region
    ContainedWithin,
    /// Token is adjacent to another (e.g., outline next to fill)
    AdjacentTo,
    /// Token is semantically paired with another (e.g., left/right eyes)
    PairedWith,
}

/// A relationship definition for a palette token.
///
/// Defines how one token relates to another for semantic analysis,
/// tooling hints, and validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    /// The type of relationship
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    /// The target token this relationship points to
    pub target: String,
}

/// A named palette defining color tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Palette {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    /// Color ramps for automatic generation of shadow/highlight variants
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ramps: Option<HashMap<String, ColorRamp>>,
    /// Semantic roles for tokens (maps token to its role)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub roles: Option<HashMap<String, Role>>,
    /// Semantic relationships between tokens
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub relationships: Option<HashMap<String, Relationship>>,
}

/// Reference to a palette - either a named reference or inline definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PaletteRef {
    Named(String),
    Inline(HashMap<String, String>),
}

impl Default for PaletteRef {
    fn default() -> Self {
        PaletteRef::Named(String::new())
    }
}

/// Transform specification - can be string or object in JSON.
///
/// Supports both simple string syntax (`"mirror-h"`, `"rotate:90"`) and
/// object syntax for complex parameters (`{"op": "tile", "w": 3, "h": 2}`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TransformSpec {
    /// String syntax: "mirror-h", "rotate:90", "tile:3x2"
    String(String),
    /// Object syntax: {"op": "tile", "w": 3, "h": 2}
    Object {
        op: String,
        #[serde(flatten)]
        params: HashMap<String, serde_json::Value>,
    },
}

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
}

/// A palette cycle definition for animating colors without changing frames.
///
/// Palette cycling rotates colors through a set of tokens, creating animated
/// effects like shimmering water, flickering fire, or pulsing energy without
/// needing multiple sprite frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaletteCycle {
    /// Tokens whose colors will be cycled (e.g., ["{water1}", "{water2}", "{water3}"])
    pub tokens: Vec<String>,
    /// Duration per cycle step in milliseconds (default: animation duration)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<u32>,
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

/// Motion follow mode for secondary motion attachments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FollowMode {
    /// Chain follows parent position changes
    #[default]
    Position,
    /// Chain reacts to parent velocity (more dynamic)
    Velocity,
    /// Chain follows parent rotation
    Rotation,
}

/// Keyframe data for an attachment offset at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentKeyframe {
    /// Offset from the base anchor position `[x, y]`
    pub offset: [i32; 2],
}

/// An animation attachment for secondary motion (hair, capes, tails).
///
/// Attachments follow the parent animation with configurable delay,
/// creating natural-looking motion for appendages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    /// Identifier for this attachment
    pub name: String,
    /// Attachment point `[x, y]` on parent sprite
    pub anchor: [i32; 2],
    /// Array of sprite names forming the chain
    pub chain: Vec<String>,
    /// Frame delay between chain segments (default: 1)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub delay: Option<u32>,
    /// Motion follow mode
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub follow: Option<FollowMode>,
    /// Oscillation damping (0.0-1.0, default: 0.8)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub damping: Option<f32>,
    /// Spring stiffness (0.0-1.0, default: 0.5)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stiffness: Option<f32>,
    /// Render order relative to parent (negative = behind)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub z_index: Option<i32>,
    /// Keyframe data for explicit positioning per frame (keyed by frame number as string)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub keyframes: Option<HashMap<String, AttachmentKeyframe>>,
}

impl Attachment {
    /// Default frame delay between chain segments.
    pub const DEFAULT_DELAY: u32 = 1;
    /// Default oscillation damping.
    pub const DEFAULT_DAMPING: f32 = 0.8;
    /// Default spring stiffness.
    pub const DEFAULT_STIFFNESS: f32 = 0.5;

    /// Returns the frame delay between chain segments.
    pub fn delay(&self) -> u32 {
        self.delay.unwrap_or(Self::DEFAULT_DELAY)
    }

    /// Returns the follow mode for this attachment.
    pub fn follow_mode(&self) -> FollowMode {
        self.follow.clone().unwrap_or_default()
    }

    /// Returns the damping factor.
    pub fn damping(&self) -> f32 {
        self.damping.unwrap_or(Self::DEFAULT_DAMPING)
    }

    /// Returns the stiffness factor.
    pub fn stiffness(&self) -> f32 {
        self.stiffness.unwrap_or(Self::DEFAULT_STIFFNESS)
    }

    /// Returns the z-index for render ordering (default: 0).
    pub fn z_index(&self) -> i32 {
        self.z_index.unwrap_or(0)
    }

    /// Returns whether this attachment uses keyframed motion.
    pub fn is_keyframed(&self) -> bool {
        self.keyframes.is_some()
    }
}

/// An animation definition (Phase 3).
///
/// Supports two formats:
/// - **Frame array format** (legacy): `frames: ["sprite1", "sprite2", ...]`
/// - **CSS keyframes format** (CSS-13): `keyframes: {"0%": {...}, "100%": {...}}`
///
/// The `frames` and `keyframes` fields are mutually exclusive.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Animation {
    pub name: String,
    /// Frame sprite names (mutually exclusive with `source` and `keyframes`)
    #[serde(default)]
    pub frames: Vec<String>,
    /// CSS-style percentage-based keyframes (mutually exclusive with `frames`)
    ///
    /// Keys are percentages ("0%", "50%", "100%") or aliases ("from" = "0%", "to" = "100%").
    /// Each keyframe can specify sprite, transform, opacity, and offset.
    ///
    /// # Example
    /// ```json
    /// {
    ///   "type": "animation",
    ///   "name": "fade_walk",
    ///   "keyframes": {
    ///     "0%": { "sprite": "walk_1", "opacity": 0.0 },
    ///     "50%": { "sprite": "walk_2", "opacity": 1.0 },
    ///     "100%": { "sprite": "walk_1", "opacity": 0.0 }
    ///   },
    ///   "duration": "500ms",
    ///   "timing_function": "ease-in-out"
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub keyframes: Option<HashMap<String, CssKeyframe>>,
    /// Reference to another animation by name (mutually exclusive with `frames`)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source: Option<String>,
    /// Transforms to apply when resolving this animation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<Vec<TransformSpec>>,
    /// Duration per frame (for frames format) or total animation duration (for keyframes format).
    /// Accepts both raw milliseconds (100) and CSS time strings ("500ms", "1s").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<Duration>,
    /// CSS timing function for keyframes interpolation (e.g., "linear", "ease", "ease-in-out",
    /// "cubic-bezier(0.25, 0.1, 0.25, 1.0)", "steps(4, jump-end)")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub timing_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub r#loop: Option<bool>,
    /// Palette cycles for color animation effects (water, fire, energy, etc.)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub palette_cycle: Option<Vec<PaletteCycle>>,
    /// Frame tags for game engine integration - maps tag name to frame range
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tags: Option<HashMap<String, FrameTag>>,
    /// Per-frame metadata (collision boxes that vary per frame)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frame_metadata: Option<Vec<FrameMetadata>>,
    /// Attachments for secondary motion (hair, capes, tails)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub attachments: Option<Vec<Attachment>>,
}

/// A variant is a palette-only modification of a base sprite.
///
/// Variants allow creating color variations of sprites without duplicating
/// the region data. The variant copies the base sprite's regions and applies
/// palette overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variant {
    pub name: String,
    pub base: String,
    pub palette: HashMap<String, String>,
    /// Transforms to apply when resolving this variant
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<Vec<TransformSpec>>,
}

impl Animation {
    /// Default duration per frame in milliseconds.
    pub const DEFAULT_DURATION_MS: u32 = 100;

    /// Returns the duration in milliseconds (default: 100ms).
    ///
    /// For frame-based animations, this is the duration per frame.
    /// For CSS keyframe animations, this is the total animation duration.
    pub fn duration_ms(&self) -> u32 {
        self.duration
            .as_ref()
            .and_then(|d| d.as_milliseconds())
            .unwrap_or(Self::DEFAULT_DURATION_MS)
    }

    /// Returns whether the animation should loop (default: true).
    pub fn loops(&self) -> bool {
        self.r#loop.unwrap_or(true)
    }

    /// Returns whether this animation uses CSS-style keyframes.
    pub fn is_css_keyframes(&self) -> bool {
        self.keyframes.is_some() && !self.keyframes.as_ref().unwrap().is_empty()
    }

    /// Returns whether this animation uses frame array format.
    pub fn is_frame_based(&self) -> bool {
        !self.frames.is_empty()
    }

    /// Returns the CSS keyframes, or None if using frame-based format.
    pub fn css_keyframes(&self) -> Option<&HashMap<String, CssKeyframe>> {
        self.keyframes.as_ref()
    }

    /// Returns whether this animation uses palette cycling.
    pub fn has_palette_cycle(&self) -> bool {
        self.palette_cycle.as_ref().map(|cycles| !cycles.is_empty()).unwrap_or(false)
    }

    /// Returns the palette cycles, or an empty slice if none.
    pub fn palette_cycles(&self) -> &[PaletteCycle] {
        self.palette_cycle.as_deref().unwrap_or(&[])
    }

    /// Returns whether this animation has secondary motion attachments.
    pub fn has_attachments(&self) -> bool {
        self.attachments.as_ref().map(|a| !a.is_empty()).unwrap_or(false)
    }

    /// Returns the attachments, or an empty slice if none.
    pub fn attachments(&self) -> &[Attachment] {
        self.attachments.as_deref().unwrap_or(&[])
    }

    /// Parse the keyframe percentage key to a normalized value (0.0 to 1.0).
    ///
    /// Supports:
    /// - Percentage strings: "0%", "50%", "100%"
    /// - Aliases: "from" (= 0%), "to" (= 100%)
    ///
    /// Returns `None` if the key cannot be parsed.
    pub fn parse_keyframe_percent(key: &str) -> Option<f64> {
        let key = key.trim().to_lowercase();

        match key.as_str() {
            "from" => Some(0.0),
            "to" => Some(1.0),
            _ => {
                if let Some(pct_str) = key.strip_suffix('%') {
                    pct_str.trim().parse::<f64>().ok().map(|v| (v / 100.0).clamp(0.0, 1.0))
                } else {
                    None
                }
            }
        }
    }

    /// Returns the sorted keyframe entries as (normalized_percent, keyframe) pairs.
    ///
    /// The keyframes are sorted by their percentage value from 0.0 to 1.0.
    pub fn sorted_keyframes(&self) -> Vec<(f64, &CssKeyframe)> {
        let Some(keyframes) = &self.keyframes else {
            return vec![];
        };

        let mut entries: Vec<_> = keyframes
            .iter()
            .filter_map(|(key, kf)| Self::parse_keyframe_percent(key).map(|pct| (pct, kf)))
            .collect();

        entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        entries
    }
}

impl PaletteCycle {
    /// Returns the duration per cycle step in milliseconds.
    /// Falls back to the provided default (typically animation duration).
    pub fn duration_ms(&self, default: u32) -> u32 {
        self.duration.unwrap_or(default)
    }

    /// Returns the number of cycle steps (= number of tokens).
    pub fn cycle_length(&self) -> usize {
        self.tokens.len()
    }
}

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

/// Velocity range for particle emitter (ATF-16)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VelocityRange {
    /// X velocity range [min, max]
    pub x: [f64; 2],
    /// Y velocity range [min, max]
    pub y: [f64; 2],
}

impl Default for VelocityRange {
    fn default() -> Self {
        Self { x: [0.0, 0.0], y: [0.0, 0.0] }
    }
}

/// Particle emitter configuration (ATF-16)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleEmitter {
    /// Particles to emit per frame
    #[serde(default = "default_rate")]
    pub rate: f64,
    /// Particle lifetime in frames [min, max]
    #[serde(default = "default_lifetime")]
    pub lifetime: [u32; 2],
    /// Initial velocity range
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub velocity: Option<VelocityRange>,
    /// Gravity acceleration (pixels per frame^2)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gravity: Option<f64>,
    /// Whether particles fade out over lifetime
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fade: Option<bool>,
    /// Rotation range in degrees [min, max]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotation: Option<[f64; 2]>,
    /// Random seed for reproducible effects
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<u64>,
}

fn default_rate() -> f64 {
    1.0
}

fn default_lifetime() -> [u32; 2] {
    [10, 20]
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            rate: default_rate(),
            lifetime: default_lifetime(),
            velocity: None,
            gravity: None,
            fade: None,
            rotation: None,
            seed: None,
        }
    }
}

/// A particle system definition (ATF-16)
///
/// Particle systems emit sprites with randomized motion for effects
/// like sparks, dust, rain, snow, fire, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Particle {
    /// Name of this particle system
    pub name: String,
    /// Reference to the sprite to emit as particles
    pub sprite: String,
    /// Emitter configuration
    pub emitter: ParticleEmitter,
}

/// Easing function for keyframe interpolation.
///
/// Controls how values transition between keyframes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Easing {
    /// Constant speed
    #[default]
    Linear,
    /// Slow start, fast end (acceleration)
    EaseIn,
    /// Fast start, slow end (deceleration)
    EaseOut,
    /// Slow start and end (smooth S-curve)
    EaseInOut,
    /// Overshoots and settles
    Bounce,
    /// Spring-like oscillation
    Elastic,
}

impl Easing {
    /// Apply the easing function to a normalized time value (0.0 to 1.0).
    ///
    /// Returns the eased value (also 0.0 to 1.0, but may exceed bounds for bounce/elastic).
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let t = 1.0 - t;
                let bounce = if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                };
                1.0 - bounce
            }
            Easing::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f64::consts::PI) / 3.0;
                    2.0_f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
        }
    }

    /// Parse an easing function from a string.
    pub fn from_str(s: &str) -> Option<Easing> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "linear" => Some(Easing::Linear),
            "ease-in" | "easein" => Some(Easing::EaseIn),
            "ease-out" | "easeout" => Some(Easing::EaseOut),
            "ease-in-out" | "easeinout" => Some(Easing::EaseInOut),
            "bounce" => Some(Easing::Bounce),
            "elastic" => Some(Easing::Elastic),
            _ => None,
        }
    }
}

/// A single keyframe defining values at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    /// Frame index (0-based)
    pub frame: u32,
    /// Property values at this keyframe (property name -> value)
    #[serde(flatten)]
    pub values: HashMap<String, f64>,
}

/// Keyframes for a single property, either via expression or explicit values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertyKeyframes {
    /// Mathematical expression for the property value.
    /// Available variables: `frame`, `t` (normalized 0.0-1.0), `total_frames`, and user params.
    /// Available functions: sin, cos, tan, pow, sqrt, min, max, abs, floor, ceil, round.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expr: Option<String>,
    /// Explicit keyframe pairs: [[frame, value], ...]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub keyframes: Option<Vec<[f64; 2]>>,
    /// Per-property easing function (overrides transform-level easing)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub easing: Option<Easing>,
}

/// Keyframe specification - array of keyframes or per-property expressions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyframeSpec {
    /// Array of keyframes with frame numbers and values
    Array(Vec<Keyframe>),
    /// Per-property keyframe definitions with expressions or explicit values
    Properties(HashMap<String, PropertyKeyframes>),
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
}

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

/// A user-defined transform.
///
/// Allows creating reusable, parameterized transforms with keyframe animation support.
///
/// # Examples
///
/// ## Named transform sequence
/// ```json
/// {
///   "type": "transform",
///   "name": "flip-glow",
///   "ops": ["mirror-h", "outline"]
/// }
/// ```
///
/// ## Parameterized transform
/// ```json
/// {
///   "type": "transform",
///   "name": "padded-outline",
///   "params": ["padding", "outline_width"],
///   "ops": [
///     {"op": "pad", "size": "${padding}"},
///     {"op": "outline", "width": "${outline_width}"}
///   ]
/// }
/// ```
///
/// ## Keyframe animation transform
/// ```json
/// {
///   "type": "transform",
///   "name": "hop",
///   "frames": 8,
///   "keyframes": [
///     {"frame": 0, "shift-y": 0},
///     {"frame": 4, "shift-y": -4},
///     {"frame": 8, "shift-y": 0}
///   ],
///   "easing": "ease-out"
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TransformDef {
    /// Name of this user-defined transform
    pub name: String,
    /// Parameter names for parameterized transforms
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<Vec<String>>,
    /// Simple sequence of transform operations
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ops: Option<Vec<TransformSpec>>,
    /// Parallel composition of transforms (computed together per-frame)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compose: Option<Vec<TransformSpec>>,
    /// Per-frame transform cycling
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cycle: Option<Vec<Vec<TransformSpec>>>,
    /// Number of frames for keyframe animation generation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frames: Option<u32>,
    /// Keyframe data for animation generation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub keyframes: Option<KeyframeSpec>,
    /// Default easing function for keyframe interpolation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub easing: Option<Easing>,
}

impl TransformDef {
    /// Returns whether this is a simple ops-only transform (no keyframes/expressions).
    pub fn is_simple(&self) -> bool {
        self.ops.is_some()
            && self.compose.is_none()
            && self.cycle.is_none()
            && self.keyframes.is_none()
    }

    /// Returns whether this transform generates animation frames.
    pub fn generates_animation(&self) -> bool {
        self.frames.is_some() && self.keyframes.is_some()
    }

    /// Returns whether this is a parameterized transform.
    pub fn is_parameterized(&self) -> bool {
        self.params.as_ref().map(|p| !p.is_empty()).unwrap_or(false)
    }

    /// Returns whether this is a cycling transform.
    pub fn is_cycling(&self) -> bool {
        self.cycle.as_ref().map(|c| !c.is_empty()).unwrap_or(false)
    }
}

/// A Pixelsrc object - Palette, Sprite, Variant, Composition, Animation, Particle, or Transform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TtpObject {
    Palette(Palette),
    Sprite(Sprite),
    Variant(Variant),
    Composition(Composition),
    Animation(Animation),
    Particle(Particle),
    Transform(TransformDef),
}

/// A warning message from parsing/rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Warning {
    pub message: String,
    pub line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_roundtrip() {
        let palette = Palette {
            name: "mono".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{on}".to_string(), "#FFFFFF".to_string()),
            ]),
            ..Default::default()
        };
        let json = serde_json::to_string(&palette).unwrap();
        let parsed: Palette = serde_json::from_str(&json).unwrap();
        assert_eq!(palette, parsed);
    }

    #[test]
    fn test_role_enum_values() {
        // Test all Role enum values
        assert_eq!(Role::Boundary.to_string(), "boundary");
        assert_eq!(Role::Anchor.to_string(), "anchor");
        assert_eq!(Role::Fill.to_string(), "fill");
        assert_eq!(Role::Shadow.to_string(), "shadow");
        assert_eq!(Role::Highlight.to_string(), "highlight");
    }

    #[test]
    fn test_role_serialization() {
        // Test that Role serializes to lowercase
        let role = Role::Boundary;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"boundary\"");

        let role = Role::Highlight;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"highlight\"");
    }

    #[test]
    fn test_role_deserialization() {
        // Test that Role deserializes from lowercase
        let role: Role = serde_json::from_str("\"boundary\"").unwrap();
        assert_eq!(role, Role::Boundary);

        let role: Role = serde_json::from_str("\"anchor\"").unwrap();
        assert_eq!(role, Role::Anchor);

        let role: Role = serde_json::from_str("\"fill\"").unwrap();
        assert_eq!(role, Role::Fill);

        let role: Role = serde_json::from_str("\"shadow\"").unwrap();
        assert_eq!(role, Role::Shadow);

        let role: Role = serde_json::from_str("\"highlight\"").unwrap();
        assert_eq!(role, Role::Highlight);
    }

    #[test]
    fn test_invalid_role_deserialization() {
        // Test that invalid role values fail to deserialize
        let result: Result<Role, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());

        let result: Result<Role, _> = serde_json::from_str("\"BOUNDARY\"");
        assert!(result.is_err(), "Role should be case-sensitive (lowercase only)");
    }

    #[test]
    fn test_palette_with_roles_roundtrip() {
        let palette = Palette {
            name: "character".to_string(),
            colors: HashMap::from([
                ("{outline}".to_string(), "#000000".to_string()),
                ("{skin}".to_string(), "#E8B89D".to_string()),
                ("{skin_shadow}".to_string(), "#C49A82".to_string()),
                ("{skin_highlight}".to_string(), "#FFD4BB".to_string()),
            ]),
            roles: Some(HashMap::from([
                ("{outline}".to_string(), Role::Boundary),
                ("{skin}".to_string(), Role::Anchor),
                ("{skin_shadow}".to_string(), Role::Shadow),
                ("{skin_highlight}".to_string(), Role::Highlight),
            ])),
            ..Default::default()
        };
        let json = serde_json::to_string(&palette).unwrap();
        let parsed: Palette = serde_json::from_str(&json).unwrap();
        assert_eq!(palette, parsed);
    }

    #[test]
    fn test_palette_roles_json_parsing() {
        // Test parsing palette with roles from JSON
        let json = r##"{
            "name": "skin",
            "colors": {
                "{base}": "#E8B89D",
                "{shadow}": "#C49A82",
                "{highlight}": "#FFD4BB"
            },
            "roles": {
                "{base}": "anchor",
                "{shadow}": "shadow",
                "{highlight}": "highlight"
            }
        }"##;
        let palette: Palette = serde_json::from_str(json).unwrap();
        assert_eq!(palette.name, "skin");
        assert_eq!(palette.colors.len(), 3);

        let roles = palette.roles.unwrap();
        assert_eq!(roles.len(), 3);
        assert_eq!(roles.get("{base}"), Some(&Role::Anchor));
        assert_eq!(roles.get("{shadow}"), Some(&Role::Shadow));
        assert_eq!(roles.get("{highlight}"), Some(&Role::Highlight));
    }

    #[test]
    fn test_ttp_object_palette_with_roles() {
        let json = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}, "roles": {"{a}": "boundary", "{b}": "fill"}}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Palette(palette) => {
                assert_eq!(palette.name, "test");
                let roles = palette.roles.unwrap();
                assert_eq!(roles.get("{a}"), Some(&Role::Boundary));
                assert_eq!(roles.get("{b}"), Some(&Role::Fill));
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    fn test_sprite_with_inline_palette_roundtrip() {
        let sprite = Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&sprite).unwrap();
        let parsed: Sprite = serde_json::from_str(&json).unwrap();
        assert_eq!(sprite, parsed);
    }

    #[test]
    fn test_sprite_with_named_palette_roundtrip() {
        let sprite = Sprite {
            name: "checker".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Named("mono".to_string()),
            grid: vec!["{on}{off}{on}{off}".to_string(), "{off}{on}{off}{on}".to_string()],
            metadata: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&sprite).unwrap();
        let parsed: Sprite = serde_json::from_str(&json).unwrap();
        assert_eq!(sprite, parsed);
    }

    #[test]
    fn test_ttp_object_palette_roundtrip() {
        let obj = TtpObject::Palette(Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{a}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        });
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"palette""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        assert_eq!(obj, parsed);
    }

    #[test]
    fn test_ttp_object_sprite_roundtrip() {
        let obj = TtpObject::Sprite(Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("colors".to_string()),
            grid: vec!["{a}{b}".to_string()],
            metadata: None,
            ..Default::default()
        });
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"sprite""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        assert_eq!(obj, parsed);
    }

    #[test]
    fn test_warning_roundtrip() {
        let warning = Warning { message: "Row 1 has 3 tokens, expected 4".to_string(), line: 5 };
        let json = serde_json::to_string(&warning).unwrap();
        let parsed: Warning = serde_json::from_str(&json).unwrap();
        assert_eq!(warning, parsed);
    }

    #[test]
    fn test_minimal_dot_fixture() {
        // {"type": "sprite", "name": "dot", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}
        let json = r##"{"type": "sprite", "name": "dot", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                assert_eq!(sprite.name, "dot");
                assert!(sprite.size.is_none());
                assert_eq!(sprite.grid, vec!["{x}"]);
                match sprite.palette {
                    PaletteRef::Inline(colors) => {
                        assert_eq!(colors.get("{x}"), Some(&"#FF0000".to_string()));
                    }
                    _ => panic!("Expected inline palette"),
                }
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_named_palette_fixture() {
        // {"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}
        let json = r##"{"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Palette(palette) => {
                assert_eq!(palette.name, "mono");
                assert_eq!(palette.colors.len(), 3);
                assert_eq!(palette.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
            }
            _ => panic!("Expected palette"),
        }

        // {"type": "sprite", "name": "checker", "palette": "mono", "grid": [...]}
        let json = r#"{"type": "sprite", "name": "checker", "palette": "mono", "grid": ["{on}{off}{on}{off}", "{off}{on}{off}{on}"]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                assert_eq!(sprite.name, "checker");
                match sprite.palette {
                    PaletteRef::Named(name) => assert_eq!(name, "mono"),
                    _ => panic!("Expected named palette reference"),
                }
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_composition_basic_parse() {
        let json = r#"{"type": "composition", "name": "test_comp", "sprites": {".": null, "X": "sprite_x"}, "layers": [{"name": "layer1", "map": ["X."]}]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Composition(comp) => {
                assert_eq!(comp.name, "test_comp");
                assert!(comp.base.is_none());
                assert!(comp.size.is_none());
                assert!(comp.cell_size.is_none());
                assert_eq!(comp.sprites.len(), 2);
                assert_eq!(comp.sprites.get("."), Some(&None));
                assert_eq!(comp.sprites.get("X"), Some(&Some("sprite_x".to_string())));
                assert_eq!(comp.layers.len(), 1);
                assert_eq!(comp.layers[0].name, Some("layer1".to_string()));
                assert_eq!(comp.layers[0].map, Some(vec!["X.".to_string()]));
            }
            _ => panic!("Expected composition"),
        }
    }

    #[test]
    fn test_composition_with_all_fields() {
        let json = r#"{"type": "composition", "name": "full_comp", "base": "hero_base", "size": [64, 64], "cell_size": [8, 8], "sprites": {".": null, "H": "hat"}, "layers": [{"name": "gear", "fill": "H", "map": ["H.", ".H"]}]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Composition(comp) => {
                assert_eq!(comp.name, "full_comp");
                assert_eq!(comp.base, Some("hero_base".to_string()));
                assert_eq!(comp.size, Some([64, 64]));
                assert_eq!(comp.cell_size, Some([8, 8]));
                assert_eq!(comp.layers[0].fill, Some("H".to_string()));
            }
            _ => panic!("Expected composition"),
        }
    }

    #[test]
    fn test_composition_roundtrip() {
        let comp = Composition {
            name: "roundtrip_test".to_string(),
            base: Some("base_sprite".to_string()),
            size: Some([32, 32]),
            cell_size: Some([4, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("A".to_string(), Some("sprite_a".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: Some("layer1".to_string()),
                fill: None,
                map: Some(vec!["A.".to_string(), ".A".to_string()]),
                ..Default::default()
            }],
        };
        let obj = TtpObject::Composition(comp.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"composition""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Composition(parsed_comp) => {
                assert_eq!(comp, parsed_comp);
            }
            _ => panic!("Expected composition"),
        }
    }

    #[test]
    fn test_composition_default_cell_size() {
        // cell_size should default to None when not specified
        let json =
            r#"{"type": "composition", "name": "no_cell_size", "sprites": {}, "layers": []}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Composition(comp) => {
                assert!(comp.cell_size.is_none());
                // Helper method should return default [1, 1]
                assert_eq!(comp.cell_size(), [1, 1]);
            }
            _ => panic!("Expected composition"),
        }
    }

    #[test]
    fn test_composition_cell_size_helper() {
        // When cell_size is specified, helper returns it
        let comp = Composition {
            name: "test".to_string(),
            base: None,
            size: None,
            cell_size: Some([8, 8]),
            sprites: HashMap::new(),
            layers: vec![],
        };
        assert_eq!(comp.cell_size(), [8, 8]);

        // When cell_size is None, helper returns default [1, 1]
        let comp_default = Composition {
            name: "test_default".to_string(),
            base: None,
            size: None,
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        assert_eq!(comp_default.cell_size(), Composition::DEFAULT_CELL_SIZE);
        assert_eq!(comp_default.cell_size(), [1, 1]);
    }

    #[test]
    fn test_animation_parse_full() {
        // Animation with all fields specified
        let json = r#"{"type": "animation", "name": "blink_anim", "frames": ["on", "off"], "duration": 500, "loop": true}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "blink_anim");
                assert_eq!(anim.frames, vec!["on", "off"]);
                assert_eq!(anim.duration, Some(Duration::Milliseconds(500)));
                assert_eq!(anim.r#loop, Some(true));
                // Helper methods should return specified values
                assert_eq!(anim.duration_ms(), 500);
                assert!(anim.loops());
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_default_duration() {
        // Animation without duration - should default to 100ms
        let json = r#"{"type": "animation", "name": "walk", "frames": ["frame1", "frame2"]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "walk");
                assert!(anim.duration.is_none());
                assert_eq!(anim.duration_ms(), 100); // Default
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_default_loop() {
        // Animation without loop - should default to true
        let json = r#"{"type": "animation", "name": "idle", "frames": ["f1"]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert!(anim.r#loop.is_none());
                assert!(anim.loops()); // Default is true
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_loop_false() {
        // Animation with loop=false
        let json =
            r#"{"type": "animation", "name": "death", "frames": ["f1", "f2"], "loop": false}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.r#loop, Some(false));
                assert!(!anim.loops());
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_roundtrip() {
        let anim = Animation {
            name: "test_anim".to_string(),
            frames: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            duration: Some(Duration::Milliseconds(200)),
            r#loop: Some(false),
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        let obj = TtpObject::Animation(anim.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"animation""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Animation(parsed_anim) => {
                assert_eq!(anim, parsed_anim);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_with_palette_cycle() {
        // Animation with palette cycling for water effect
        let json = r#"{"type": "animation", "name": "water", "frames": ["water_tile"], "duration": 100, "palette_cycle": [{"tokens": ["{w1}", "{w2}", "{w3}"], "duration": 150}]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "water");
                assert!(anim.has_palette_cycle());
                let cycles = anim.palette_cycles();
                assert_eq!(cycles.len(), 1);
                assert_eq!(cycles[0].tokens, vec!["{w1}", "{w2}", "{w3}"]);
                assert_eq!(cycles[0].duration, Some(150));
                assert_eq!(cycles[0].duration_ms(100), 150);
                assert_eq!(cycles[0].cycle_length(), 3);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_palette_cycle_default_duration() {
        // Palette cycle without explicit duration uses animation duration
        let json = r#"{"type": "animation", "name": "fire", "frames": ["flame"], "duration": 80, "palette_cycle": [{"tokens": ["{f1}", "{f2}"]}]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                let cycles = anim.palette_cycles();
                assert_eq!(cycles.len(), 1);
                assert!(cycles[0].duration.is_none());
                // Should use animation duration as fallback
                assert_eq!(cycles[0].duration_ms(anim.duration_ms()), 80);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_multiple_palette_cycles() {
        // Animation with multiple independent cycles
        let json = r#"{"type": "animation", "name": "scene", "frames": ["scene_frame"], "palette_cycle": [{"tokens": ["{water1}", "{water2}"], "duration": 200}, {"tokens": ["{fire1}", "{fire2}", "{fire3}"], "duration": 100}]}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                let cycles = anim.palette_cycles();
                assert_eq!(cycles.len(), 2);
                // Water cycle
                assert_eq!(cycles[0].tokens.len(), 2);
                assert_eq!(cycles[0].duration, Some(200));
                // Fire cycle
                assert_eq!(cycles[1].tokens.len(), 3);
                assert_eq!(cycles[1].duration, Some(100));
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_palette_cycle_roundtrip() {
        let anim = Animation {
            name: "cycle_test".to_string(),
            frames: vec!["sprite".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: Some(true),
            palette_cycle: Some(vec![PaletteCycle {
                tokens: vec!["{a}".to_string(), "{b}".to_string()],
                duration: Some(150),
            }]),
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        let obj = TtpObject::Animation(anim.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("palette_cycle"));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Animation(parsed_anim) => {
                assert_eq!(anim, parsed_anim);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_no_palette_cycle() {
        // Animation without palette_cycle should have has_palette_cycle() return false
        let anim = Animation {
            name: "normal".to_string(),
            frames: vec!["f1".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        assert!(!anim.has_palette_cycle());
        assert!(anim.palette_cycles().is_empty());
    }

    #[test]
    fn test_variant_parse_basic() {
        // Variant with single color override
        let json = r##"{"type": "variant", "name": "hero_red", "base": "hero", "palette": {"{skin}": "#FF0000"}}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Variant(variant) => {
                assert_eq!(variant.name, "hero_red");
                assert_eq!(variant.base, "hero");
                assert_eq!(variant.palette.len(), 1);
                assert_eq!(variant.palette.get("{skin}"), Some(&"#FF0000".to_string()));
            }
            _ => panic!("Expected variant"),
        }
    }

    #[test]
    fn test_variant_parse_multiple_overrides() {
        // Variant with multiple color overrides
        let json = r##"{"type": "variant", "name": "hero_alt", "base": "hero", "palette": {"{skin}": "#00FF00", "{hair}": "#0000FF", "{eyes}": "#FFFF00"}}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Variant(variant) => {
                assert_eq!(variant.name, "hero_alt");
                assert_eq!(variant.base, "hero");
                assert_eq!(variant.palette.len(), 3);
                assert_eq!(variant.palette.get("{skin}"), Some(&"#00FF00".to_string()));
                assert_eq!(variant.palette.get("{hair}"), Some(&"#0000FF".to_string()));
                assert_eq!(variant.palette.get("{eyes}"), Some(&"#FFFF00".to_string()));
            }
            _ => panic!("Expected variant"),
        }
    }

    #[test]
    fn test_variant_roundtrip() {
        let variant = Variant {
            name: "test_variant".to_string(),
            base: "base_sprite".to_string(),
            palette: HashMap::from([
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ]),
            ..Default::default()
        };
        let obj = TtpObject::Variant(variant.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r##""type":"variant""##));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Variant(parsed_variant) => {
                assert_eq!(variant, parsed_variant);
            }
            _ => panic!("Expected variant"),
        }
    }

    #[test]
    fn test_variant_empty_palette() {
        // Variant with empty palette (inherits all colors from base)
        let json = r#"{"type": "variant", "name": "hero_copy", "base": "hero", "palette": {}}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Variant(variant) => {
                assert_eq!(variant.name, "hero_copy");
                assert_eq!(variant.base, "hero");
                assert!(variant.palette.is_empty());
            }
            _ => panic!("Expected variant"),
        }
    }

    // ========================================================================
    // Particle System Tests (ATF-16)
    // ========================================================================

    #[test]
    fn test_particle_parse_basic() {
        let json = r#"{
            "type": "particle",
            "name": "sparkle",
            "sprite": "spark",
            "emitter": {
                "rate": 5,
                "lifetime": [10, 20]
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Particle(p) => {
                assert_eq!(p.name, "sparkle");
                assert_eq!(p.sprite, "spark");
                assert_eq!(p.emitter.rate, 5.0);
                assert_eq!(p.emitter.lifetime, [10, 20]);
            }
            _ => panic!("Expected particle"),
        }
    }

    #[test]
    fn test_particle_parse_full() {
        let json = r#"{
            "type": "particle",
            "name": "rain",
            "sprite": "raindrop",
            "emitter": {
                "rate": 10,
                "lifetime": [30, 60],
                "velocity": {"x": [-1, 1], "y": [5, 8]},
                "gravity": 0.5,
                "fade": true,
                "rotation": [0, 360],
                "seed": 12345
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Particle(p) => {
                assert_eq!(p.name, "rain");
                assert_eq!(p.sprite, "raindrop");
                assert_eq!(p.emitter.rate, 10.0);
                assert_eq!(p.emitter.lifetime, [30, 60]);
                let vel = p.emitter.velocity.unwrap();
                assert_eq!(vel.x, [-1.0, 1.0]);
                assert_eq!(vel.y, [5.0, 8.0]);
                assert_eq!(p.emitter.gravity, Some(0.5));
                assert_eq!(p.emitter.fade, Some(true));
                assert_eq!(p.emitter.rotation, Some([0.0, 360.0]));
                assert_eq!(p.emitter.seed, Some(12345));
            }
            _ => panic!("Expected particle"),
        }
    }

    #[test]
    fn test_particle_roundtrip() {
        let particle = Particle {
            name: "dust".to_string(),
            sprite: "dust_mote".to_string(),
            emitter: ParticleEmitter {
                rate: 2.0,
                lifetime: [5, 15],
                velocity: Some(VelocityRange { x: [-2.0, 2.0], y: [-1.0, 0.0] }),
                gravity: Some(0.1),
                fade: Some(true),
                rotation: None,
                seed: Some(42),
            },
        };
        let obj = TtpObject::Particle(particle.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"particle""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Particle(parsed_particle) => {
                assert_eq!(particle, parsed_particle);
            }
            _ => panic!("Expected particle"),
        }
    }

    #[test]
    fn test_particle_emitter_defaults() {
        // Emitter with minimal fields should use defaults
        let json = r#"{
            "type": "particle",
            "name": "minimal",
            "sprite": "dot",
            "emitter": {}
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Particle(p) => {
                assert_eq!(p.emitter.rate, 1.0); // default
                assert_eq!(p.emitter.lifetime, [10, 20]); // default
                assert!(p.emitter.velocity.is_none());
                assert!(p.emitter.gravity.is_none());
                assert!(p.emitter.fade.is_none());
                assert!(p.emitter.rotation.is_none());
                assert!(p.emitter.seed.is_none());
            }
            _ => panic!("Expected particle"),
        }
    }

    // ========== Hit/Hurt Boxes Tests (ATF-7) ==========

    #[test]
    fn test_collision_box_roundtrip() {
        let box_data = CollisionBox { x: 4, y: 0, w: 24, h: 32 };
        let json = serde_json::to_string(&box_data).unwrap();
        let parsed: CollisionBox = serde_json::from_str(&json).unwrap();
        assert_eq!(box_data, parsed);
    }

    #[test]
    fn test_sprite_metadata_roundtrip() {
        let metadata = SpriteMetadata {
            origin: Some([16, 32]),
            boxes: Some(HashMap::from([
                ("hurt".to_string(), CollisionBox { x: 4, y: 0, w: 24, h: 32 }),
                ("hit".to_string(), CollisionBox { x: 20, y: 8, w: 20, h: 16 }),
            ])),
            attach_in: None,
            attach_out: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: SpriteMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata, parsed);
    }

    #[test]
    fn test_sprite_with_metadata_parse() {
        // Sprite with metadata as specified in ATF-7
        let json = r#"{
            "type": "sprite",
            "name": "player_attack",
            "palette": "characters",
            "grid": ["{x}"],
            "metadata": {
                "origin": [16, 32],
                "boxes": {
                    "hurt": {"x": 4, "y": 0, "w": 24, "h": 32},
                    "hit": {"x": 20, "y": 8, "w": 20, "h": 16}
                }
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                assert_eq!(sprite.name, "player_attack");
                assert!(sprite.metadata.is_some());
                let meta = sprite.metadata.unwrap();
                assert_eq!(meta.origin, Some([16, 32]));
                assert!(meta.boxes.is_some());
                let boxes = meta.boxes.unwrap();
                assert_eq!(boxes.len(), 2);
                assert!(boxes.contains_key("hurt"));
                assert!(boxes.contains_key("hit"));
                let hurt_box = &boxes["hurt"];
                assert_eq!(hurt_box.x, 4);
                assert_eq!(hurt_box.y, 0);
                assert_eq!(hurt_box.w, 24);
                assert_eq!(hurt_box.h, 32);
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_velocity_range_default() {
        let vel = VelocityRange::default();
        assert_eq!(vel.x, [0.0, 0.0]);
        assert_eq!(vel.y, [0.0, 0.0]);
    }

    #[test]
    fn test_particle_emitter_default() {
        let emitter = ParticleEmitter::default();
        assert_eq!(emitter.rate, 1.0);
        assert_eq!(emitter.lifetime, [10, 20]);
        assert!(emitter.velocity.is_none());
        assert!(emitter.gravity.is_none());
        assert!(emitter.fade.is_none());
        assert!(emitter.rotation.is_none());
        assert!(emitter.seed.is_none());
    }

    #[test]
    fn test_sprite_metadata_origin_only() {
        // Sprite with only origin, no boxes
        let json = r#"{
            "type": "sprite",
            "name": "centered_sprite",
            "palette": "default",
            "grid": ["{x}"],
            "metadata": {
                "origin": [8, 16]
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                let meta = sprite.metadata.unwrap();
                assert_eq!(meta.origin, Some([8, 16]));
                assert!(meta.boxes.is_none());
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_sprite_metadata_boxes_only() {
        // Sprite with only boxes, no origin
        let json = r#"{
            "type": "sprite",
            "name": "collider",
            "palette": "default",
            "grid": ["{x}"],
            "metadata": {
                "boxes": {
                    "collide": {"x": 0, "y": 0, "w": 16, "h": 16}
                }
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                let meta = sprite.metadata.unwrap();
                assert!(meta.origin.is_none());
                assert!(meta.boxes.is_some());
                assert!(meta.boxes.unwrap().contains_key("collide"));
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_animation_frame_metadata_parse() {
        // Animation with per-frame metadata as specified in ATF-7
        let json = r#"{
            "type": "animation",
            "name": "attack",
            "frames": ["f1", "f2", "f3"],
            "frame_metadata": [
                {"boxes": {"hit": null}},
                {"boxes": {"hit": {"x": 20, "y": 8, "w": 20, "h": 16}}},
                {"boxes": {"hit": {"x": 24, "y": 4, "w": 24, "h": 20}}}
            ]
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "attack");
                assert_eq!(anim.frames.len(), 3);
                assert!(anim.frame_metadata.is_some());
                let frame_meta = anim.frame_metadata.unwrap();
                assert_eq!(frame_meta.len(), 3);

                // Frame 0: hit box is null (disabled)
                let f0_boxes = frame_meta[0].boxes.as_ref().unwrap();
                assert!(f0_boxes.get("hit").unwrap().is_none());

                // Frame 1: hit box is active
                let f1_boxes = frame_meta[1].boxes.as_ref().unwrap();
                let f1_hit = f1_boxes.get("hit").unwrap().as_ref().unwrap();
                assert_eq!(f1_hit.x, 20);
                assert_eq!(f1_hit.y, 8);

                // Frame 2: hit box is active with different values
                let f2_boxes = frame_meta[2].boxes.as_ref().unwrap();
                let f2_hit = f2_boxes.get("hit").unwrap().as_ref().unwrap();
                assert_eq!(f2_hit.x, 24);
                assert_eq!(f2_hit.w, 24);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_sprite_without_metadata_roundtrip() {
        // Sprite without metadata should serialize without metadata field
        let sprite = Sprite {
            name: "simple".to_string(),
            size: None,
            palette: PaletteRef::Named("default".to_string()),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&sprite).unwrap();
        // Should not contain "metadata" key when None
        assert!(!json.contains("metadata"));
        let parsed: Sprite = serde_json::from_str(&json).unwrap();
        assert_eq!(sprite, parsed);
    }

    #[test]
    fn test_animation_without_frame_metadata_roundtrip() {
        // Animation without frame_metadata should serialize without the field
        let anim = Animation {
            name: "simple".to_string(),
            frames: vec!["f1".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&anim).unwrap();
        // Should not contain "frame_metadata" key when None
        assert!(!json.contains("frame_metadata"));
        let parsed: Animation = serde_json::from_str(&json).unwrap();
        assert_eq!(anim, parsed);
    }

    #[test]
    fn test_collision_box_negative_coordinates() {
        // Collision boxes can have negative x,y for positions relative to origin
        let box_data = CollisionBox { x: -8, y: -16, w: 16, h: 32 };
        let json = serde_json::to_string(&box_data).unwrap();
        let parsed: CollisionBox = serde_json::from_str(&json).unwrap();
        assert_eq!(box_data, parsed);
        assert_eq!(parsed.x, -8);
        assert_eq!(parsed.y, -16);
    }

    // ========== Secondary Motion Tests (ATF-14) ==========

    #[test]
    fn test_follow_mode_parse() {
        // Test parsing of follow modes
        assert_eq!(
            serde_json::from_str::<FollowMode>(r#""position""#).unwrap(),
            FollowMode::Position
        );
        assert_eq!(
            serde_json::from_str::<FollowMode>(r#""velocity""#).unwrap(),
            FollowMode::Velocity
        );
        assert_eq!(
            serde_json::from_str::<FollowMode>(r#""rotation""#).unwrap(),
            FollowMode::Rotation
        );
    }

    #[test]
    fn test_follow_mode_default() {
        // Default should be Position
        assert_eq!(FollowMode::default(), FollowMode::Position);
    }

    #[test]
    fn test_attachment_keyframe_roundtrip() {
        let keyframe = AttachmentKeyframe { offset: [5, -3] };
        let json = serde_json::to_string(&keyframe).unwrap();
        let parsed: AttachmentKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(keyframe, parsed);
    }

    #[test]
    fn test_attachment_basic_roundtrip() {
        let attachment = Attachment {
            name: "hair".to_string(),
            anchor: [12, 4],
            chain: vec!["hair_1".to_string(), "hair_2".to_string()],
            delay: None,
            follow: None,
            damping: None,
            stiffness: None,
            z_index: None,
            keyframes: None,
        };
        let json = serde_json::to_string(&attachment).unwrap();
        let parsed: Attachment = serde_json::from_str(&json).unwrap();
        assert_eq!(attachment, parsed);
    }

    #[test]
    fn test_attachment_with_all_fields() {
        let attachment = Attachment {
            name: "cape".to_string(),
            anchor: [8, 8],
            chain: vec!["cape_top".to_string(), "cape_mid".to_string(), "cape_bottom".to_string()],
            delay: Some(2),
            follow: Some(FollowMode::Velocity),
            damping: Some(0.7),
            stiffness: Some(0.4),
            z_index: Some(-1),
            keyframes: None,
        };
        let json = serde_json::to_string(&attachment).unwrap();
        let parsed: Attachment = serde_json::from_str(&json).unwrap();
        assert_eq!(attachment, parsed);
    }

    #[test]
    fn test_attachment_with_keyframes() {
        let mut keyframes = HashMap::new();
        keyframes.insert("0".to_string(), AttachmentKeyframe { offset: [0, 0] });
        keyframes.insert("1".to_string(), AttachmentKeyframe { offset: [2, 1] });
        keyframes.insert("2".to_string(), AttachmentKeyframe { offset: [3, 2] });

        let attachment = Attachment {
            name: "hair".to_string(),
            anchor: [12, 4],
            chain: vec!["hair_1".to_string()],
            delay: None,
            follow: None,
            damping: None,
            stiffness: None,
            z_index: None,
            keyframes: Some(keyframes),
        };

        let json = serde_json::to_string(&attachment).unwrap();
        let parsed: Attachment = serde_json::from_str(&json).unwrap();
        assert_eq!(attachment, parsed);
        assert!(parsed.is_keyframed());
    }

    #[test]
    fn test_attachment_helper_methods() {
        // Test default values
        let attachment = Attachment {
            name: "test".to_string(),
            anchor: [0, 0],
            chain: vec!["sprite".to_string()],
            delay: None,
            follow: None,
            damping: None,
            stiffness: None,
            z_index: None,
            keyframes: None,
        };

        assert_eq!(attachment.delay(), 1); // DEFAULT_DELAY
        assert_eq!(attachment.follow_mode(), FollowMode::Position);
        assert!((attachment.damping() - 0.8).abs() < 0.001); // DEFAULT_DAMPING
        assert!((attachment.stiffness() - 0.5).abs() < 0.001); // DEFAULT_STIFFNESS
        assert_eq!(attachment.z_index(), 0);
        assert!(!attachment.is_keyframed());

        // Test with custom values
        let attachment_custom = Attachment {
            name: "custom".to_string(),
            anchor: [0, 0],
            chain: vec!["sprite".to_string()],
            delay: Some(3),
            follow: Some(FollowMode::Velocity),
            damping: Some(0.5),
            stiffness: Some(0.9),
            z_index: Some(-2),
            keyframes: None,
        };

        assert_eq!(attachment_custom.delay(), 3);
        assert_eq!(attachment_custom.follow_mode(), FollowMode::Velocity);
        assert!((attachment_custom.damping() - 0.5).abs() < 0.001);
        assert!((attachment_custom.stiffness() - 0.9).abs() < 0.001);
        assert_eq!(attachment_custom.z_index(), -2);
    }

    #[test]
    fn test_animation_with_attachments_parse() {
        // Animation with attachments as specified in ATF-14
        let json = r#"{
            "type": "animation",
            "name": "hero_walk",
            "frames": ["walk_1", "walk_2", "walk_3", "walk_4"],
            "duration": 100,
            "attachments": [
                {
                    "name": "hair",
                    "anchor": [12, 4],
                    "chain": ["hair_1", "hair_2", "hair_3"],
                    "delay": 1,
                    "follow": "position"
                },
                {
                    "name": "cape",
                    "anchor": [8, 8],
                    "chain": ["cape_top", "cape_mid", "cape_bottom"],
                    "delay": 2,
                    "follow": "velocity",
                    "z_index": -1
                }
            ]
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "hero_walk");
                assert!(anim.attachments.is_some());
                let attachments = anim.attachments.unwrap();
                assert_eq!(attachments.len(), 2);

                // Hair attachment
                let hair = &attachments[0];
                assert_eq!(hair.name, "hair");
                assert_eq!(hair.anchor, [12, 4]);
                assert_eq!(hair.chain.len(), 3);
                assert_eq!(hair.delay(), 1);
                assert_eq!(hair.follow_mode(), FollowMode::Position);

                // Cape attachment
                let cape = &attachments[1];
                assert_eq!(cape.name, "cape");
                assert_eq!(cape.anchor, [8, 8]);
                assert_eq!(cape.chain.len(), 3);
                assert_eq!(cape.delay(), 2);
                assert_eq!(cape.follow_mode(), FollowMode::Velocity);
                assert_eq!(cape.z_index(), -1);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_attachments_roundtrip() {
        let anim = Animation {
            name: "test_anim".to_string(),
            frames: vec!["f1".to_string(), "f2".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: Some(true),
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: Some(vec![Attachment {
                name: "tail".to_string(),
                anchor: [4, 8],
                chain: vec!["tail_1".to_string(), "tail_2".to_string()],
                delay: Some(1),
                follow: Some(FollowMode::Position),
                damping: Some(0.8),
                stiffness: Some(0.5),
                z_index: Some(1),
                keyframes: None,
            }]),
            ..Default::default()
        };
        let obj = TtpObject::Animation(anim.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("attachments"));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Animation(parsed_anim) => {
                assert_eq!(anim, parsed_anim);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_without_attachments_roundtrip() {
        // Animation without attachments should serialize without the field
        let anim = Animation {
            name: "simple".to_string(),
            frames: vec!["f1".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&anim).unwrap();
        // Should not contain "attachments" key when None
        assert!(!json.contains("attachments"));
        let parsed: Animation = serde_json::from_str(&json).unwrap();
        assert_eq!(anim, parsed);
    }

    #[test]
    fn test_sprite_metadata_with_attach_points() {
        // Chain sprite with attachment points as specified in ATF-14
        let json = r#"{
            "type": "sprite",
            "name": "hair_2",
            "palette": "character",
            "grid": ["{x}"],
            "metadata": {
                "attach_in": [4, 0],
                "attach_out": [4, 8]
            }
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                assert_eq!(sprite.name, "hair_2");
                assert!(sprite.metadata.is_some());
                let meta = sprite.metadata.unwrap();
                assert_eq!(meta.attach_in, Some([4, 0]));
                assert_eq!(meta.attach_out, Some([4, 8]));
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_sprite_metadata_attach_points_roundtrip() {
        let metadata = SpriteMetadata {
            origin: Some([8, 8]),
            boxes: None,
            attach_in: Some([4, 0]),
            attach_out: Some([4, 8]),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: SpriteMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata, parsed);
    }

    #[test]
    fn test_attachment_keyframed_parse() {
        // Attachment with explicit keyframes
        let json = r#"{
            "name": "hair",
            "anchor": [12, 4],
            "chain": ["hair_1", "hair_2"],
            "keyframes": {
                "0": {"offset": [0, 0]},
                "1": {"offset": [2, 1]},
                "2": {"offset": [3, 2]},
                "3": {"offset": [1, 1]}
            }
        }"#;
        let attachment: Attachment = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.name, "hair");
        assert!(attachment.is_keyframed());
        let keyframes = attachment.keyframes.unwrap();
        assert_eq!(keyframes.len(), 4);
        assert_eq!(keyframes.get("0").unwrap().offset, [0, 0]);
        assert_eq!(keyframes.get("2").unwrap().offset, [3, 2]);
    }

    // ========================================================================
    // Duration Tests (CSS-13)
    // ========================================================================

    #[test]
    fn test_duration_milliseconds_parse() {
        let dur: Duration = serde_json::from_str("100").unwrap();
        assert_eq!(dur, Duration::Milliseconds(100));
        assert_eq!(dur.as_milliseconds(), Some(100));
    }

    #[test]
    fn test_duration_css_string_ms() {
        let dur: Duration = serde_json::from_str(r#""500ms""#).unwrap();
        assert!(matches!(dur, Duration::CssString(_)));
        assert_eq!(dur.as_milliseconds(), Some(500));
    }

    #[test]
    fn test_duration_css_string_seconds() {
        let dur: Duration = serde_json::from_str(r#""1.5s""#).unwrap();
        assert!(matches!(dur, Duration::CssString(_)));
        assert_eq!(dur.as_milliseconds(), Some(1500));
    }

    #[test]
    fn test_duration_display() {
        assert_eq!(format!("{}", Duration::Milliseconds(100)), "100");
        assert_eq!(format!("{}", Duration::CssString("500ms".to_string())), "\"500ms\"");
    }

    #[test]
    fn test_duration_default() {
        let dur = Duration::default();
        assert_eq!(dur, Duration::Milliseconds(100));
    }

    #[test]
    fn test_duration_from_u32() {
        let dur: Duration = 250u32.into();
        assert_eq!(dur, Duration::Milliseconds(250));
    }

    #[test]
    fn test_duration_from_str() {
        let dur: Duration = "1s".into();
        assert_eq!(dur, Duration::CssString("1s".to_string()));
        assert_eq!(dur.as_milliseconds(), Some(1000));
    }

    // ========================================================================
    // CSS Keyframe Tests (CSS-13)
    // ========================================================================

    #[test]
    fn test_css_keyframe_parse_basic() {
        let kf: CssKeyframe = serde_json::from_str(r#"{"sprite": "walk_1"}"#).unwrap();
        assert_eq!(kf.sprite, Some("walk_1".to_string()));
        assert!(kf.transform.is_none());
        assert!(kf.opacity.is_none());
        assert!(kf.offset.is_none());
    }

    #[test]
    fn test_css_keyframe_parse_full() {
        let json = r#"{
            "sprite": "walk_1",
            "transform": "rotate(45deg) scale(2)",
            "opacity": 0.5,
            "offset": [10, -5]
        }"#;
        let kf: CssKeyframe = serde_json::from_str(json).unwrap();
        assert_eq!(kf.sprite, Some("walk_1".to_string()));
        assert_eq!(kf.transform, Some("rotate(45deg) scale(2)".to_string()));
        assert_eq!(kf.opacity, Some(0.5));
        assert_eq!(kf.offset, Some([10, -5]));
    }

    #[test]
    fn test_css_keyframe_roundtrip() {
        let kf = CssKeyframe {
            sprite: Some("test".to_string()),
            transform: Some("scale(2)".to_string()),
            opacity: Some(0.8),
            offset: Some([5, 10]),
        };
        let json = serde_json::to_string(&kf).unwrap();
        let parsed: CssKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, parsed);
    }

    // ========================================================================
    // Animation CSS Keyframes Tests (CSS-13)
    // ========================================================================

    #[test]
    fn test_animation_css_keyframes_parse() {
        let json = r#"{
            "type": "animation",
            "name": "fade_walk",
            "keyframes": {
                "0%": {"sprite": "walk_1", "opacity": 0.0},
                "50%": {"sprite": "walk_2", "opacity": 1.0},
                "100%": {"sprite": "walk_1", "opacity": 0.0}
            },
            "duration": "500ms",
            "timing_function": "ease-in-out"
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert_eq!(anim.name, "fade_walk");
                assert!(anim.is_css_keyframes());
                assert!(!anim.is_frame_based());

                let keyframes = anim.css_keyframes().unwrap();
                assert_eq!(keyframes.len(), 3);

                let kf_0 = keyframes.get("0%").unwrap();
                assert_eq!(kf_0.sprite, Some("walk_1".to_string()));
                assert_eq!(kf_0.opacity, Some(0.0));

                let kf_50 = keyframes.get("50%").unwrap();
                assert_eq!(kf_50.sprite, Some("walk_2".to_string()));
                assert_eq!(kf_50.opacity, Some(1.0));

                assert_eq!(anim.duration_ms(), 500);
                assert_eq!(anim.timing_function, Some("ease-in-out".to_string()));
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_css_keyframes_from_to_aliases() {
        let json = r#"{
            "type": "animation",
            "name": "fade",
            "keyframes": {
                "from": {"opacity": 0.0},
                "to": {"opacity": 1.0}
            },
            "duration": "1s"
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                assert!(anim.is_css_keyframes());
                let keyframes = anim.css_keyframes().unwrap();
                assert!(keyframes.contains_key("from"));
                assert!(keyframes.contains_key("to"));
                assert_eq!(anim.duration_ms(), 1000);
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_parse_keyframe_percent() {
        // Percentage strings
        assert_eq!(Animation::parse_keyframe_percent("0%"), Some(0.0));
        assert_eq!(Animation::parse_keyframe_percent("50%"), Some(0.5));
        assert_eq!(Animation::parse_keyframe_percent("100%"), Some(1.0));
        assert_eq!(Animation::parse_keyframe_percent("25%"), Some(0.25));

        // Aliases
        assert_eq!(Animation::parse_keyframe_percent("from"), Some(0.0));
        assert_eq!(Animation::parse_keyframe_percent("to"), Some(1.0));

        // Case insensitive
        assert_eq!(Animation::parse_keyframe_percent("FROM"), Some(0.0));
        assert_eq!(Animation::parse_keyframe_percent("TO"), Some(1.0));

        // Invalid
        assert_eq!(Animation::parse_keyframe_percent("invalid"), None);
        assert_eq!(Animation::parse_keyframe_percent("50"), None);
    }

    #[test]
    fn test_animation_sorted_keyframes() {
        let anim = Animation {
            name: "test".to_string(),
            keyframes: Some(HashMap::from([
                (
                    "100%".to_string(),
                    CssKeyframe { sprite: Some("c".to_string()), ..Default::default() },
                ),
                (
                    "0%".to_string(),
                    CssKeyframe { sprite: Some("a".to_string()), ..Default::default() },
                ),
                (
                    "50%".to_string(),
                    CssKeyframe { sprite: Some("b".to_string()), ..Default::default() },
                ),
            ])),
            ..Default::default()
        };

        let sorted = anim.sorted_keyframes();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].0, 0.0);
        assert_eq!(sorted[0].1.sprite, Some("a".to_string()));
        assert_eq!(sorted[1].0, 0.5);
        assert_eq!(sorted[1].1.sprite, Some("b".to_string()));
        assert_eq!(sorted[2].0, 1.0);
        assert_eq!(sorted[2].1.sprite, Some("c".to_string()));
    }

    #[test]
    fn test_animation_css_keyframes_with_transforms() {
        let json = r#"{
            "type": "animation",
            "name": "spin",
            "keyframes": {
                "0%": {"sprite": "star", "transform": "rotate(0deg)"},
                "100%": {"sprite": "star", "transform": "rotate(360deg)"}
            },
            "duration": 1000,
            "timing_function": "linear"
        }"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Animation(anim) => {
                let keyframes = anim.css_keyframes().unwrap();
                assert_eq!(
                    keyframes.get("0%").unwrap().transform,
                    Some("rotate(0deg)".to_string())
                );
                assert_eq!(
                    keyframes.get("100%").unwrap().transform,
                    Some("rotate(360deg)".to_string())
                );
                assert_eq!(anim.timing_function, Some("linear".to_string()));
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_animation_frame_vs_keyframe() {
        // Frame-based animation
        let frame_anim = Animation {
            name: "frames".to_string(),
            frames: vec!["f1".to_string(), "f2".to_string()],
            ..Default::default()
        };
        assert!(frame_anim.is_frame_based());
        assert!(!frame_anim.is_css_keyframes());

        // CSS keyframe animation
        let keyframe_anim = Animation {
            name: "keyframes".to_string(),
            keyframes: Some(HashMap::from([
                ("0%".to_string(), CssKeyframe::default()),
                ("100%".to_string(), CssKeyframe::default()),
            ])),
            ..Default::default()
        };
        assert!(!keyframe_anim.is_frame_based());
        assert!(keyframe_anim.is_css_keyframes());
    }

    #[test]
    fn test_animation_css_keyframes_roundtrip() {
        let anim = Animation {
            name: "test_kf".to_string(),
            keyframes: Some(HashMap::from([
                (
                    "0%".to_string(),
                    CssKeyframe {
                        sprite: Some("start".to_string()),
                        opacity: Some(0.0),
                        ..Default::default()
                    },
                ),
                (
                    "100%".to_string(),
                    CssKeyframe {
                        sprite: Some("end".to_string()),
                        opacity: Some(1.0),
                        ..Default::default()
                    },
                ),
            ])),
            duration: Some(Duration::CssString("500ms".to_string())),
            timing_function: Some("ease".to_string()),
            ..Default::default()
        };

        let obj = TtpObject::Animation(anim.clone());
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("keyframes"));
        assert!(json.contains("timing_function"));

        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        match parsed {
            TtpObject::Animation(parsed_anim) => {
                assert_eq!(anim.name, parsed_anim.name);
                assert_eq!(anim.timing_function, parsed_anim.timing_function);
                assert!(parsed_anim.is_css_keyframes());
            }
            _ => panic!("Expected animation"),
        }
    }

    #[test]
    fn test_region_def_simple_rect() {
        let region = RegionDef {
            rect: Some([5, 6, 2, 2]),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }

    #[test]
    fn test_region_def_with_modifiers() {
        let region = RegionDef {
            points: Some(vec![[4, 6]]),
            symmetric: Some("x".to_string()),
            z: Some(10),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }

    #[test]
    fn test_region_def_with_constraints() {
        let region = RegionDef {
            circle: Some([8, 8, 3]),
            within: Some("face".to_string()),
            x: Some([0, 10]),
            y: Some([5, 15]),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }

    #[test]
    fn test_region_def_compound_union() {
        let region = RegionDef {
            union: Some(vec![
                RegionDef {
                    rect: Some([2, 0, 12, 2]),
                    ..Default::default()
                },
                RegionDef {
                    rect: Some([0, 2, 16, 2]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }

    #[test]
    fn test_region_def_with_subtraction() {
        let region = RegionDef {
            base: Some(Box::new(RegionDef {
                rect: Some([2, 4, 12, 8]),
                ..Default::default()
            })),
            subtract: Some(vec![RegionDef {
                points: Some(vec![[5, 6], [10, 6]]),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }

    #[test]
    fn test_region_def_renamed_fields() {
        // Test that hyphenated fields serialize/deserialize correctly
        let json = r#"{"auto-outline":"body","adjacent-to":"skin","offset-alternate":true}"#;
        let region: RegionDef = serde_json::from_str(json).unwrap();
        assert_eq!(region.auto_outline, Some("body".to_string()));
        assert_eq!(region.adjacent_to, Some("skin".to_string()));
        assert_eq!(region.offset_alternate, Some(true));

        let serialized = serde_json::to_string(&region).unwrap();
        assert!(serialized.contains("auto-outline"));
        assert!(serialized.contains("adjacent-to"));
        assert!(serialized.contains("offset-alternate"));
    }

    #[test]
    fn test_region_def_all_shapes() {
        // Test deserialization of each shape type
        let test_cases = vec![
            (r#"{"points":[[1,2],[3,4]]}"#, "points"),
            (r#"{"line":[[0,0],[10,10]]}"#, "line"),
            (r#"{"rect":[1,2,3,4]}"#, "rect"),
            (r#"{"stroke":[0,0,16,16]}"#, "stroke"),
            (r#"{"ellipse":[8,8,4,6]}"#, "ellipse"),
            (r#"{"circle":[8,8,4]}"#, "circle"),
            (r#"{"polygon":[[0,0],[4,0],[4,4]]}"#, "polygon"),
            (r#"{"path":"M0,0 L10,10 Z"}"#, "path"),
            (r#"{"fill":"inside(outline)"}"#, "fill"),
        ];

        for (json, shape_type) in test_cases {
            let region: Result<RegionDef, _> = serde_json::from_str(json);
            assert!(region.is_ok(), "Failed to parse {} shape: {}", shape_type, json);
        }
    }

    #[test]
    fn test_jitter_spec() {
        let jitter = JitterSpec {
            x: Some([-2, 2]),
            y: Some([-1, 1]),
        };
        let json = serde_json::to_string(&jitter).unwrap();
        let parsed: JitterSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(jitter, parsed);
    }

    #[test]
    fn test_region_def_with_jitter() {
        let region = RegionDef {
            points: Some(vec![[0, 15], [4, 15], [8, 15]]),
            jitter: Some(JitterSpec {
                x: None,
                y: Some([-2, 0]),
            }),
            seed: Some(42),
            ..Default::default()
        };
        let json = serde_json::to_string(&region).unwrap();
        let parsed: RegionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(region, parsed);
    }
}
