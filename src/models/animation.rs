//! Animation-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::Duration;
use super::palette::PaletteCycle;
use super::sprite::{FrameMetadata, FrameTag};
use super::transform::TransformSpec;

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
        self.keyframes.as_ref().is_some_and(|kf| !kf.is_empty())
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
