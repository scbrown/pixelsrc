//! Semantic-aware antialiasing system for pixel art
//!
//! Provides optional antialiasing using semantic roles for intelligent smoothing.
//! By default, antialiasing is disabled to preserve crisp pixel art.
//!
//! # Module Structure
//!
//! - [`context`] - Semantic context extraction from palette roles/relationships
//! - [`algorithms`] - Implementation of antialiasing algorithms (aa-blur, scale2x, hq2x, etc.)
//! - `gradient` (planned) - Gradient smoothing for shadow/highlight transitions

pub mod algorithms;
pub mod context;

pub use algorithms::{apply_semantic_blur, scale2x, Scale2xOptions};
pub use context::{AdjacencyInfo, GradientPair, RenderedRegion, SemanticContext};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Antialiasing algorithm selection.
///
/// Each algorithm has different trade-offs between quality and performance:
///
/// | Algorithm | Scale | Quality | Speed |
/// |-----------|-------|---------|-------|
/// | `nearest` | Any | - | Fastest |
/// | `scale2x` | 2x | Good | Fast |
/// | `hq2x` | 2x | Better | Medium |
/// | `hq4x` | 4x | Better | Medium |
/// | `xbr2x` | 2x | Best | Slower |
/// | `xbr4x` | 4x | Best | Slower |
/// | `aa-blur` | Any | Subtle | Fast |
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AAAlgorithm {
    /// No antialiasing (passthrough), default behavior
    #[default]
    Nearest,
    /// Scale2x (EPX) algorithm - 2x upscaling with edge-aware interpolation
    Scale2x,
    /// HQ2x algorithm - 2x upscaling with pattern-based interpolation
    Hq2x,
    /// HQ4x algorithm - 4x upscaling with pattern-based interpolation
    Hq4x,
    /// xBR 2x algorithm - 2x upscaling with edge direction analysis
    Xbr2x,
    /// xBR 4x algorithm - 4x upscaling with edge direction analysis
    Xbr4x,
    /// Gaussian blur with semantic masking
    #[serde(rename = "aa-blur")]
    #[value(name = "aa-blur")]
    AaBlur,
}

impl AAAlgorithm {
    /// Returns the scale factor produced by this algorithm.
    ///
    /// Most algorithms produce a fixed output scale (e.g., hq4x always produces 4x).
    /// `Nearest` and `AaBlur` return 1 as they don't inherently scale.
    ///
    /// # Examples
    ///
    /// ```
    /// use pixelsrc::antialias::AAAlgorithm;
    ///
    /// assert_eq!(AAAlgorithm::Nearest.scale_factor(), 1);
    /// assert_eq!(AAAlgorithm::Scale2x.scale_factor(), 2);
    /// assert_eq!(AAAlgorithm::Hq4x.scale_factor(), 4);
    /// ```
    pub fn scale_factor(&self) -> u8 {
        match self {
            AAAlgorithm::Nearest => 1,
            AAAlgorithm::Scale2x => 2,
            AAAlgorithm::Hq2x => 2,
            AAAlgorithm::Hq4x => 4,
            AAAlgorithm::Xbr2x => 2,
            AAAlgorithm::Xbr4x => 4,
            AAAlgorithm::AaBlur => 1,
        }
    }

    /// Returns true if this algorithm performs any antialiasing.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, AAAlgorithm::Nearest)
    }
}

impl std::fmt::Display for AAAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AAAlgorithm::Nearest => write!(f, "nearest"),
            AAAlgorithm::Scale2x => write!(f, "scale2x"),
            AAAlgorithm::Hq2x => write!(f, "hq2x"),
            AAAlgorithm::Hq4x => write!(f, "hq4x"),
            AAAlgorithm::Xbr2x => write!(f, "xbr2x"),
            AAAlgorithm::Xbr4x => write!(f, "xbr4x"),
            AAAlgorithm::AaBlur => write!(f, "aa-blur"),
        }
    }
}

/// Controls how anchor regions (important details like eyes) are handled during antialiasing.
///
/// Anchors are typically small, important details that should remain crisp.
/// This enum controls the antialiasing strength applied to them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AnchorMode {
    /// No antialiasing on anchor regions (default) - keeps details crisp
    #[default]
    Preserve,
    /// Apply 25% antialiasing strength to anchors
    Reduce,
    /// Apply full antialiasing to anchors (treat like any other region)
    Normal,
}

impl std::fmt::Display for AnchorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnchorMode::Preserve => write!(f, "preserve"),
            AnchorMode::Reduce => write!(f, "reduce"),
            AnchorMode::Normal => write!(f, "normal"),
        }
    }
}

/// Configuration for the antialiasing system.
///
/// This can be specified at multiple levels with increasing precedence:
/// 1. `pxl.toml` defaults
/// 2. Atlas-level configuration
/// 3. Per-sprite configuration
/// 4. CLI flags (highest priority)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AntialiasConfig {
    /// Whether antialiasing is enabled (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// The antialiasing algorithm to use (default: Nearest/disabled)
    #[serde(default)]
    pub algorithm: AAAlgorithm,

    /// Antialiasing strength from 0.0 to 1.0 (default: 0.5)
    #[serde(default = "default_strength")]
    pub strength: f32,

    /// How to handle anchor regions (default: Preserve)
    #[serde(default)]
    pub anchor_mode: AnchorMode,

    /// Enable smooth gradient transitions for shadow/highlight (default: true)
    #[serde(default = "default_true")]
    pub gradient_shadows: bool,

    /// Respect containment boundaries as hard edges (default: true)
    #[serde(default = "default_true")]
    pub respect_containment: bool,

    /// Use semantic role information for intelligent AA decisions (default: false)
    /// Can be disabled with --no-semantic-aa CLI flag
    #[serde(default)]
    pub semantic_aware: bool,

    /// Per-region antialiasing overrides
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub regions: Option<HashMap<String, RegionAAOverride>>,
}

impl Default for AntialiasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: AAAlgorithm::default(),
            strength: 0.5,
            anchor_mode: AnchorMode::default(),
            gradient_shadows: true,
            respect_containment: true,
            semantic_aware: false,
            regions: None,
        }
    }
}

/// Default strength value for serde
fn default_strength() -> f32 {
    0.5
}

/// Default true value for serde
fn default_true() -> bool {
    true
}

impl AntialiasConfig {
    /// Default antialiasing strength
    pub const DEFAULT_STRENGTH: f32 = 0.5;

    /// Create a new config with antialiasing enabled using the specified algorithm
    pub fn with_algorithm(algorithm: AAAlgorithm) -> Self {
        Self { enabled: true, algorithm, ..Default::default() }
    }

    /// Merge another config into this one, overriding any non-default values
    pub fn merge(&mut self, other: &AntialiasConfig) {
        if other.enabled {
            self.enabled = true;
        }
        if other.algorithm != AAAlgorithm::Nearest {
            self.algorithm = other.algorithm;
        }
        if (other.strength - Self::DEFAULT_STRENGTH).abs() > 0.001 {
            self.strength = other.strength;
        }
        if other.anchor_mode != AnchorMode::Preserve {
            self.anchor_mode = other.anchor_mode;
        }
        if !other.gradient_shadows {
            self.gradient_shadows = false;
        }
        if !other.respect_containment {
            self.respect_containment = false;
        }
        if other.semantic_aware {
            self.semantic_aware = true;
        }
        if let Some(ref regions) = other.regions {
            let existing = self.regions.get_or_insert_with(HashMap::new);
            for (k, v) in regions {
                existing.insert(k.clone(), v.clone());
            }
        }
    }

    /// Returns the effective scale factor for the current configuration
    pub fn scale_factor(&self) -> u8 {
        if self.enabled {
            self.algorithm.scale_factor()
        } else {
            1
        }
    }
}

/// Per-region antialiasing configuration override.
///
/// Allows fine-grained control over antialiasing for specific regions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RegionAAOverride {
    /// If true, skip antialiasing for this region entirely
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preserve: Option<bool>,

    /// Override the anchor mode for this region
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mode: Option<AnchorMode>,

    /// Override gradient smoothing for this region
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gradient: Option<bool>,
}

impl RegionAAOverride {
    /// Create an override that preserves the region (no AA)
    pub fn preserved() -> Self {
        Self { preserve: Some(true), ..Default::default() }
    }

    /// Returns true if this region should skip antialiasing
    pub fn should_preserve(&self) -> bool {
        self.preserve.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aa_algorithm_scale_factor() {
        assert_eq!(AAAlgorithm::Nearest.scale_factor(), 1);
        assert_eq!(AAAlgorithm::Scale2x.scale_factor(), 2);
        assert_eq!(AAAlgorithm::Hq2x.scale_factor(), 2);
        assert_eq!(AAAlgorithm::Hq4x.scale_factor(), 4);
        assert_eq!(AAAlgorithm::Xbr2x.scale_factor(), 2);
        assert_eq!(AAAlgorithm::Xbr4x.scale_factor(), 4);
        assert_eq!(AAAlgorithm::AaBlur.scale_factor(), 1);
    }

    #[test]
    fn test_aa_algorithm_is_enabled() {
        assert!(!AAAlgorithm::Nearest.is_enabled());
        assert!(AAAlgorithm::Scale2x.is_enabled());
        assert!(AAAlgorithm::Hq2x.is_enabled());
        assert!(AAAlgorithm::Hq4x.is_enabled());
        assert!(AAAlgorithm::Xbr2x.is_enabled());
        assert!(AAAlgorithm::Xbr4x.is_enabled());
        assert!(AAAlgorithm::AaBlur.is_enabled());
    }

    #[test]
    fn test_aa_algorithm_display() {
        assert_eq!(format!("{}", AAAlgorithm::Nearest), "nearest");
        assert_eq!(format!("{}", AAAlgorithm::Scale2x), "scale2x");
        assert_eq!(format!("{}", AAAlgorithm::Hq2x), "hq2x");
        assert_eq!(format!("{}", AAAlgorithm::Hq4x), "hq4x");
        assert_eq!(format!("{}", AAAlgorithm::Xbr2x), "xbr2x");
        assert_eq!(format!("{}", AAAlgorithm::Xbr4x), "xbr4x");
        assert_eq!(format!("{}", AAAlgorithm::AaBlur), "aa-blur");
    }

    #[test]
    fn test_aa_algorithm_serialization() {
        let algo = AAAlgorithm::Hq4x;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, "\"hq4x\"");

        let algo = AAAlgorithm::AaBlur;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, "\"aa-blur\"");
    }

    #[test]
    fn test_aa_algorithm_deserialization() {
        let algo: AAAlgorithm = serde_json::from_str("\"scale2x\"").unwrap();
        assert_eq!(algo, AAAlgorithm::Scale2x);

        let algo: AAAlgorithm = serde_json::from_str("\"aa-blur\"").unwrap();
        assert_eq!(algo, AAAlgorithm::AaBlur);
    }

    #[test]
    fn test_anchor_mode_default() {
        assert_eq!(AnchorMode::default(), AnchorMode::Preserve);
    }

    #[test]
    fn test_anchor_mode_display() {
        assert_eq!(format!("{}", AnchorMode::Preserve), "preserve");
        assert_eq!(format!("{}", AnchorMode::Reduce), "reduce");
        assert_eq!(format!("{}", AnchorMode::Normal), "normal");
    }

    #[test]
    fn test_anchor_mode_serialization() {
        let mode = AnchorMode::Reduce;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"reduce\"");

        let parsed: AnchorMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AnchorMode::Reduce);
    }

    #[test]
    fn test_antialias_config_default() {
        let config = AntialiasConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.algorithm, AAAlgorithm::Nearest);
        assert!((config.strength - 0.5).abs() < 0.001);
        assert_eq!(config.anchor_mode, AnchorMode::Preserve);
        assert!(config.gradient_shadows);
        assert!(config.respect_containment);
        assert!(!config.semantic_aware);
        assert!(config.regions.is_none());
    }

    #[test]
    fn test_antialias_config_with_algorithm() {
        let config = AntialiasConfig::with_algorithm(AAAlgorithm::Xbr4x);
        assert!(config.enabled);
        assert_eq!(config.algorithm, AAAlgorithm::Xbr4x);
    }

    #[test]
    fn test_antialias_config_scale_factor() {
        let mut config = AntialiasConfig::default();
        assert_eq!(config.scale_factor(), 1);

        config.enabled = true;
        config.algorithm = AAAlgorithm::Hq4x;
        assert_eq!(config.scale_factor(), 4);
    }

    #[test]
    fn test_antialias_config_merge() {
        let mut base = AntialiasConfig::default();
        let override_config = AntialiasConfig {
            enabled: true,
            algorithm: AAAlgorithm::Hq2x,
            strength: 0.8,
            anchor_mode: AnchorMode::Reduce,
            gradient_shadows: false,
            respect_containment: true,
            semantic_aware: true,
            regions: Some(HashMap::from([(
                "eye".to_string(),
                RegionAAOverride::preserved(),
            )])),
        };

        base.merge(&override_config);

        assert!(base.enabled);
        assert_eq!(base.algorithm, AAAlgorithm::Hq2x);
        assert!((base.strength - 0.8).abs() < 0.001);
        assert_eq!(base.anchor_mode, AnchorMode::Reduce);
        assert!(!base.gradient_shadows);
        assert!(base.semantic_aware);
        assert!(base.regions.is_some());
        assert!(base.regions.as_ref().unwrap().contains_key("eye"));
    }

    #[test]
    fn test_antialias_config_serialization_roundtrip() {
        let config = AntialiasConfig {
            enabled: true,
            algorithm: AAAlgorithm::Xbr4x,
            strength: 0.7,
            anchor_mode: AnchorMode::Preserve,
            gradient_shadows: true,
            respect_containment: true,
            semantic_aware: true,
            regions: Some(HashMap::from([(
                "eye".to_string(),
                RegionAAOverride { preserve: Some(true), mode: None, gradient: None },
            )])),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: AntialiasConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, parsed.enabled);
        assert_eq!(config.algorithm, parsed.algorithm);
        assert!((config.strength - parsed.strength).abs() < 0.001);
        assert_eq!(config.anchor_mode, parsed.anchor_mode);
        assert_eq!(config.gradient_shadows, parsed.gradient_shadows);
        assert_eq!(config.respect_containment, parsed.respect_containment);
        assert_eq!(config.semantic_aware, parsed.semantic_aware);
    }

    #[test]
    fn test_region_aa_override_preserved() {
        let override_config = RegionAAOverride::preserved();
        assert!(override_config.should_preserve());
    }

    #[test]
    fn test_region_aa_override_default() {
        let override_config = RegionAAOverride::default();
        assert!(!override_config.should_preserve());
    }

    #[test]
    fn test_region_aa_override_serialization() {
        let override_config =
            RegionAAOverride { preserve: Some(true), mode: Some(AnchorMode::Reduce), gradient: Some(false) };

        let json = serde_json::to_string(&override_config).unwrap();
        let parsed: RegionAAOverride = serde_json::from_str(&json).unwrap();

        assert_eq!(override_config.preserve, parsed.preserve);
        assert_eq!(override_config.mode, parsed.mode);
        assert_eq!(override_config.gradient, parsed.gradient);
    }

    #[test]
    fn test_antialias_config_json_parsing() {
        let json = r#"{
            "enabled": true,
            "algorithm": "hq4x",
            "strength": 0.7,
            "anchor_mode": "preserve",
            "gradient_shadows": true,
            "respect_containment": true,
            "semantic_aware": true,
            "regions": {
                "eye": { "preserve": true }
            }
        }"#;

        let config: AntialiasConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.algorithm, AAAlgorithm::Hq4x);
        assert!((config.strength - 0.7).abs() < 0.001);
        assert_eq!(config.anchor_mode, AnchorMode::Preserve);
        assert!(config.gradient_shadows);
        assert!(config.respect_containment);
        assert!(config.semantic_aware);
        assert!(config.regions.is_some());
        assert!(config.regions.as_ref().unwrap().get("eye").unwrap().should_preserve());
    }

    #[test]
    fn test_antialias_config_minimal_json() {
        // Should work with minimal JSON, using defaults
        let json = r#"{"enabled": true, "algorithm": "scale2x"}"#;
        let config: AntialiasConfig = serde_json::from_str(json).unwrap();

        assert!(config.enabled);
        assert_eq!(config.algorithm, AAAlgorithm::Scale2x);
        assert!((config.strength - 0.5).abs() < 0.001); // default
        assert_eq!(config.anchor_mode, AnchorMode::Preserve); // default
        assert!(config.gradient_shadows); // default
        assert!(config.respect_containment); // default
    }
}
