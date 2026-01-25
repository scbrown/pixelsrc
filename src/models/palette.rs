//! Palette and color-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
