//! Particle system types.

use serde::{Deserialize, Serialize};

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

fn default_rate() -> f64 {
    1.0
}

fn default_lifetime() -> [u32; 2] {
    [10, 20]
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
