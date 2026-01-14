//! Data models for TTP objects (palettes, sprites, etc.)

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
}

/// A layer within a composition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositionLayer {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub map: Option<Vec<String>>,
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

/// A TTP object - Palette, Sprite, Composition, or Animation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TtpObject {
    Palette(Palette),
    Sprite(Sprite),
    Composition(Composition),
    Animation(Animation),
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
            layers: vec![
                CompositionLayer {
                    name: Some("layer1".to_string()),
                    fill: None,
                    map: Some(vec!["A.".to_string(), ".A".to_string()]),
                },
            ],
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
        let json = r#"{"type": "composition", "name": "no_cell_size", "sprites": {}, "layers": []}"#;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Composition(comp) => {
                assert!(comp.cell_size.is_none());
            }
            _ => panic!("Expected composition"),
        }
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
        let json = r#"{"type": "animation", "name": "death", "frames": ["f1", "f2"], "loop": false}"#;
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
}
