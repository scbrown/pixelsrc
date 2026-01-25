//! Data models for Pixelsrc objects (palettes, sprites, etc.)

mod animation;
mod composition;
mod core;
mod object;
mod palette;
mod particle;
mod region;
mod sprite;
mod transform;
mod variant;

// Re-export all public types
pub use animation::{
    Animation, Attachment, AttachmentKeyframe, CssKeyframe, FollowMode,
};
pub use composition::{Composition, CompositionLayer};
pub use core::{parse_css_duration, Duration, VarOr};
pub use object::{TtpObject, Warning};
pub use palette::{
    ColorRamp, ColorShift, Palette, PaletteCycle, PaletteRef, Relationship, RelationshipType, Role,
};
pub use particle::{Particle, ParticleEmitter, VelocityRange};
pub use region::{JitterSpec, RegionDef};
pub use sprite::{CollisionBox, FrameMetadata, FrameTag, NineSlice, Sprite, SpriteMetadata};
pub use transform::{Easing, Keyframe, KeyframeSpec, PropertyKeyframes, TransformDef, TransformSpec};
pub use variant::Variant;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        let json = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]]}}}"##;
        let obj: TtpObject = serde_json::from_str(json).unwrap();
        match obj {
            TtpObject::Sprite(sprite) => {
                assert_eq!(sprite.name, "dot");
                assert_eq!(sprite.size, Some([1, 1]));
                assert!(sprite.regions.is_some());
                match sprite.palette {
                    PaletteRef::Inline(colors) => {
                        assert_eq!(colors.get("x"), Some(&"#FF0000".to_string()));
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

        // {"type": "sprite", "name": "checker", "palette": "mono", "regions": {...}}
        let json = r#"{"type": "sprite", "name": "checker", "palette": "mono", "size": [4,2], "regions": {"on": {"points": [[0,0],[2,0],[1,1],[3,1]]}, "off": {"points": [[1,0],[3,0],[0,1],[2,1]]}}}"#;
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
            "size": [1, 1],
            "regions": {"x": {"points": [[0,0]]}},
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
            "size": [1, 1],
            "regions": {"x": {"points": [[0,0]]}},
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
            "size": [1, 1],
            "regions": {"x": {"points": [[0,0]]}},
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
            "size": [1, 1],
            "regions": {"x": {"points": [[0,0]]}},
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
