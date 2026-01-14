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

/// An animation definition (Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub r#loop: Option<bool>,
}

/// A TTP object - Palette, Sprite, or Animation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TtpObject {
    Palette(Palette),
    Sprite(Sprite),
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
}
