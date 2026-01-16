//! Data models for Pixelsrc objects (palettes, sprites, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A named palette defining color tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Palette {
    pub name: String,
    pub colors: HashMap<String, String>,
}

/// Reference to a palette - either a named reference or inline definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PaletteRef {
    Named(String),
    Inline(HashMap<String, String>),
}

/// A sprite definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sprite {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<[u32; 2]>,
    pub palette: PaletteRef,
    pub grid: Vec<String>,
    /// Sprite metadata for game engine integration (origin, collision boxes)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<SpriteMetadata>,
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
/// Contains origin point and collision boxes for sprites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SpriteMetadata {
    /// Sprite origin point `[x, y]` - used for positioning and rotation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub origin: Option<[i32; 2]>,
    /// Collision boxes (hit, hurt, collide, trigger, etc.)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub boxes: Option<HashMap<String, CollisionBox>>,
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

/// An animation definition (Phase 3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<u32>,
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
}

/// A variant is a palette-only modification of a base sprite.
///
/// Variants allow creating color variations of sprites without duplicating
/// the grid data. The variant copies the base sprite's grid and applies
/// palette overrides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Variant {
    pub name: String,
    pub base: String,
    pub palette: HashMap<String, String>,
}

impl Animation {
    /// Default duration per frame in milliseconds.
    pub const DEFAULT_DURATION_MS: u32 = 100;

    /// Returns the duration per frame in milliseconds (default: 100ms).
    pub fn duration_ms(&self) -> u32 {
        self.duration.unwrap_or(Self::DEFAULT_DURATION_MS)
    }

    /// Returns whether the animation should loop (default: true).
    pub fn loops(&self) -> bool {
        self.r#loop.unwrap_or(true)
    }

    /// Returns whether this animation uses palette cycling.
    pub fn has_palette_cycle(&self) -> bool {
        self.palette_cycle
            .as_ref()
            .map(|cycles| !cycles.is_empty())
            .unwrap_or(false)
    }

    /// Returns the palette cycles, or an empty slice if none.
    pub fn palette_cycles(&self) -> &[PaletteCycle] {
        self.palette_cycle.as_deref().unwrap_or(&[])
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
    /// Blend mode for this layer (ATF-10). Default: "normal"
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub blend: Option<String>,
    /// Layer opacity from 0.0 (transparent) to 1.0 (opaque). Default: 1.0
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub opacity: Option<f64>,
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
        Self {
            x: [0.0, 0.0],
            y: [0.0, 0.0],
        }
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

/// A Pixelsrc object - Palette, Sprite, Variant, Composition, Animation, or Particle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TtpObject {
    Palette(Palette),
    Sprite(Sprite),
    Variant(Variant),
    Composition(Composition),
    Animation(Animation),
    Particle(Particle),
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
        };
        let json = serde_json::to_string(&palette).unwrap();
        let parsed: Palette = serde_json::from_str(&json).unwrap();
        assert_eq!(palette, parsed);
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
            grid: vec![
                "{on}{off}{on}{off}".to_string(),
                "{off}{on}{off}{on}".to_string(),
            ],
            metadata: None,
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
        });
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains(r#""type":"sprite""#));
        let parsed: TtpObject = serde_json::from_str(&json).unwrap();
        assert_eq!(obj, parsed);
    }

    #[test]
    fn test_warning_roundtrip() {
        let warning = Warning {
            message: "Row 1 has 3 tokens, expected 4".to_string(),
            line: 5,
        };
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
                ..Default::default()}],
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
                assert_eq!(anim.duration, Some(500));
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
            duration: Some(200),
            r#loop: Some(false),
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
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
            duration: Some(100),
            r#loop: Some(true),
            palette_cycle: Some(vec![
                PaletteCycle {
                    tokens: vec!["{a}".to_string(), "{b}".to_string()],
                    duration: Some(150),
                },
            ]),
            tags: None,
            frame_metadata: None,
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
                velocity: Some(VelocityRange {
                    x: [-2.0, 2.0],
                    y: [-1.0, 0.0],
                }),
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
        let box_data = CollisionBox {
            x: 4,
            y: 0,
            w: 24,
            h: 32,
        };
        let json = serde_json::to_string(&box_data).unwrap();
        let parsed: CollisionBox = serde_json::from_str(&json).unwrap();
        assert_eq!(box_data, parsed);
    }

    #[test]
    fn test_sprite_metadata_roundtrip() {
        let metadata = SpriteMetadata {
            origin: Some([16, 32]),
            boxes: Some(HashMap::from([
                (
                    "hurt".to_string(),
                    CollisionBox {
                        x: 4,
                        y: 0,
                        w: 24,
                        h: 32,
                    },
                ),
                (
                    "hit".to_string(),
                    CollisionBox {
                        x: 20,
                        y: 8,
                        w: 20,
                        h: 16,
                    },
                ),
            ])),
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
        let box_data = CollisionBox {
            x: -8,
            y: -16,
            w: 16,
            h: 32,
        };
        let json = serde_json::to_string(&box_data).unwrap();
        let parsed: CollisionBox = serde_json::from_str(&json).unwrap();
        assert_eq!(box_data, parsed);
        assert_eq!(parsed.x, -8);
        assert_eq!(parsed.y, -16);
    }
}
