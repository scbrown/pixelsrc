//! Transform-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
