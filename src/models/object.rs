//! Top-level Pixelsrc object types.

use serde::{Deserialize, Serialize};

use super::animation::Animation;
use super::composition::Composition;
use super::palette::Palette;
use super::particle::Particle;
use super::sprite::Sprite;
use super::transform::TransformDef;
use super::variant::Variant;

/// A Pixelsrc object - Palette, Sprite, Variant, Composition, Animation, Particle, Transform, or StateRules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TtpObject {
    Palette(Palette),
    Sprite(Sprite),
    Variant(Variant),
    Composition(Composition),
    Animation(Animation),
    Particle(Particle),
    Transform(TransformDef),
    StateRules(crate::state::StateRules),
}

/// A warning message from parsing/rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Warning {
    pub message: String,
    pub line: usize,
}
