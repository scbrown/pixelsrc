//! Formatter for Pixelsrc files
//!
//! Formats .pxl and .jsonl files for improved readability by:
//! - Expanding sprite grids to one row per line (visual formatting)
//! - Expanding composition layer maps to one row per line
//! - Keeping palettes, animations, and variants as single-line JSON

use crate::models::{
    Animation, Composition, CompositionLayer, Palette, PaletteRef, Particle, Sprite, TtpObject,
    Variant,
};
use std::collections::HashMap;
use std::io::Cursor;

/// Format pixelsrc content for readability.
///
/// Parses the input as concatenated JSON objects and reformats each:
/// - Sprites: grid arrays expanded to one row per line
/// - Compositions: layer maps expanded to one row per line
/// - Palettes, Animations, Variants: kept as single-line JSON
///
/// Returns the formatted content with blank lines between objects.
pub fn format_pixelsrc(content: &str) -> Result<String, String> {
    let reader = Cursor::new(content);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    let iterator = deserializer.into_iter::<TtpObject>();

    let mut output = String::new();
    let mut first = true;

    for item in iterator {
        match item {
            Ok(obj) => {
                if !first {
                    output.push('\n'); // Blank line between objects
                }
                first = false;
                output.push_str(&format_object(&obj));
                output.push('\n');
            }
            Err(e) => {
                if e.is_eof() {
                    break;
                }
                return Err(format!("Parse error at line {}: {}", e.line(), e));
            }
        }
    }

    Ok(output)
}

/// Format a single TtpObject.
fn format_object(obj: &TtpObject) -> String {
    match obj {
        TtpObject::Palette(p) => format_palette(p),
        TtpObject::Sprite(s) => format_sprite(s),
        TtpObject::Composition(c) => format_composition(c),
        TtpObject::Animation(a) => format_animation(a),
        TtpObject::Variant(v) => format_variant(v),
        TtpObject::Particle(p) => format_particle(p),
    }
}

/// Format a palette as single-line JSON.
fn format_palette(palette: &Palette) -> String {
    // Sort colors for consistent output
    let mut colors: Vec<_> = palette.colors.iter().collect();
    colors.sort_by_key(|(k, _)| *k);

    let mut s = String::new();
    s.push_str(r#"{"type": "palette", "name": ""#);
    s.push_str(&escape_json_string(&palette.name));
    s.push_str(r#"", "colors": {"#);

    for (i, (key, value)) in colors.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push('"');
        s.push_str(&escape_json_string(key));
        s.push_str(r#"": ""#);
        s.push_str(&escape_json_string(value));
        s.push('"');
    }

    s.push_str("}}");
    s
}

/// Format a sprite with visual grid (one row per line).
fn format_sprite(sprite: &Sprite) -> String {
    let mut s = String::new();

    // Opening with type and name
    s.push_str(r#"{"type": "sprite", "name": ""#);
    s.push_str(&escape_json_string(&sprite.name));
    s.push('"');

    // Size (if present)
    if let Some([w, h]) = sprite.size {
        s.push_str(&format!(r#", "size": [{}, {}]"#, w, h));
    }

    // Palette reference
    s.push_str(r#", "palette": "#);
    match &sprite.palette {
        PaletteRef::Named(name) => {
            s.push('"');
            s.push_str(&escape_json_string(name));
            s.push('"');
        }
        PaletteRef::Inline(colors) => {
            s.push_str(&format_inline_palette(colors));
        }
    }

    // Grid - visual formatting with one row per line
    s.push_str(r#", "grid": ["#);

    if sprite.grid.is_empty() {
        s.push(']');
    } else if sprite.grid.len() == 1 {
        // Single row - keep on same line
        s.push('"');
        s.push_str(&escape_json_string(&sprite.grid[0]));
        s.push_str("\"]");
    } else {
        // Multiple rows - one per line for visual alignment
        s.push('\n');
        for (i, row) in sprite.grid.iter().enumerate() {
            s.push_str("  \"");
            s.push_str(&escape_json_string(row));
            s.push('"');
            if i < sprite.grid.len() - 1 {
                s.push(',');
            }
            s.push('\n');
        }
        s.push(']');
    }

    s.push('}');
    s
}

/// Format an inline palette as JSON object.
fn format_inline_palette(colors: &HashMap<String, String>) -> String {
    let mut sorted: Vec<_> = colors.iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    let mut s = String::from("{");
    for (i, (key, value)) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push('"');
        s.push_str(&escape_json_string(key));
        s.push_str(r#"": ""#);
        s.push_str(&escape_json_string(value));
        s.push('"');
    }
    s.push('}');
    s
}

/// Format a composition with visual layers.
fn format_composition(comp: &Composition) -> String {
    let mut s = String::new();

    // Opening
    s.push_str(r#"{"type": "composition", "name": ""#);
    s.push_str(&escape_json_string(&comp.name));
    s.push('"');

    // Base sprite (if present)
    if let Some(ref base) = comp.base {
        s.push_str(r#", "base": ""#);
        s.push_str(&escape_json_string(base));
        s.push('"');
    }

    // Size (if present)
    if let Some([w, h]) = comp.size {
        s.push_str(&format!(r#", "size": [{}, {}]"#, w, h));
    }

    // Cell size (if present)
    if let Some([w, h]) = comp.cell_size {
        s.push_str(&format!(r#", "cell_size": [{}, {}]"#, w, h));
    }

    // Sprites map
    s.push_str(r#", "sprites": {"#);
    let mut sprites: Vec<_> = comp.sprites.iter().collect();
    sprites.sort_by_key(|(k, _)| *k);
    for (i, (key, value)) in sprites.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push('"');
        s.push_str(&escape_json_string(key));
        s.push_str(r#"": "#);
        match value {
            Some(name) => {
                s.push('"');
                s.push_str(&escape_json_string(name));
                s.push('"');
            }
            None => s.push_str("null"),
        }
    }
    s.push('}');

    // Layers - visual formatting
    s.push_str(r#", "layers": ["#);
    if comp.layers.is_empty() {
        s.push(']');
    } else {
        s.push('\n');
        for (i, layer) in comp.layers.iter().enumerate() {
            s.push_str("  ");
            s.push_str(&format_layer(layer));
            if i < comp.layers.len() - 1 {
                s.push(',');
            }
            s.push('\n');
        }
        s.push(']');
    }

    s.push('}');
    s
}

/// Format a composition layer.
fn format_layer(layer: &CompositionLayer) -> String {
    let mut s = String::from("{");
    let mut first = true;

    // Name
    if let Some(ref name) = layer.name {
        s.push_str(r#""name": ""#);
        s.push_str(&escape_json_string(name));
        s.push('"');
        first = false;
    }

    // Fill
    if let Some(ref fill) = layer.fill {
        if !first {
            s.push_str(", ");
        }
        s.push_str(r#""fill": ""#);
        s.push_str(&escape_json_string(fill));
        s.push('"');
        first = false;
    }

    // Map - visual formatting if multi-row
    if let Some(ref map) = layer.map {
        if !first {
            s.push_str(", ");
        }
        s.push_str(r#""map": ["#);
        if map.is_empty() {
            s.push(']');
        } else if map.len() == 1 {
            s.push('"');
            s.push_str(&escape_json_string(&map[0]));
            s.push_str("\"]");
        } else {
            // Multi-row map - expand for readability
            s.push('\n');
            for (i, row) in map.iter().enumerate() {
                s.push_str("    \"");
                s.push_str(&escape_json_string(row));
                s.push('"');
                if i < map.len() - 1 {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str("  ]");
        }
    }

    s.push('}');
    s
}

/// Format an animation as single-line JSON.
fn format_animation(anim: &Animation) -> String {
    let mut s = String::new();

    s.push_str(r#"{"type": "animation", "name": ""#);
    s.push_str(&escape_json_string(&anim.name));
    s.push_str(r#"", "frames": ["#);

    for (i, frame) in anim.frames.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push('"');
        s.push_str(&escape_json_string(frame));
        s.push('"');
    }
    s.push(']');

    // Duration (if specified)
    if let Some(ref duration) = anim.duration {
        s.push_str(&format!(r#", "duration": {}"#, duration));
    }

    // Loop (if specified as false - true is default)
    if let Some(loops) = anim.r#loop {
        if !loops {
            s.push_str(r#", "loop": false"#);
        }
    }

    s.push('}');
    s
}

/// Format a variant as single-line JSON.
fn format_variant(variant: &Variant) -> String {
    let mut s = String::new();

    s.push_str(r#"{"type": "variant", "name": ""#);
    s.push_str(&escape_json_string(&variant.name));
    s.push_str(r#"", "base": ""#);
    s.push_str(&escape_json_string(&variant.base));
    s.push_str(r#"", "palette": {"#);

    let mut colors: Vec<_> = variant.palette.iter().collect();
    colors.sort_by_key(|(k, _)| *k);

    for (i, (key, value)) in colors.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push('"');
        s.push_str(&escape_json_string(key));
        s.push_str(r#"": ""#);
        s.push_str(&escape_json_string(value));
        s.push('"');
    }

    s.push_str("}}");
    s
}

/// Format a particle system as single-line JSON.
fn format_particle(particle: &Particle) -> String {
    let mut s = String::new();

    s.push_str(r#"{"type": "particle", "name": ""#);
    s.push_str(&escape_json_string(&particle.name));
    s.push_str(r#"", "sprite": ""#);
    s.push_str(&escape_json_string(&particle.sprite));
    s.push_str(r#"", "emitter": {"#);

    // Rate
    s.push_str(&format!(r#""rate": {}"#, particle.emitter.rate));

    // Lifetime
    s.push_str(&format!(
        r#", "lifetime": [{}, {}]"#,
        particle.emitter.lifetime[0], particle.emitter.lifetime[1]
    ));

    // Optional fields
    if let Some(ref velocity) = particle.emitter.velocity {
        s.push_str(&format!(
            r#", "velocity": {{"x": [{}, {}], "y": [{}, {}]}}"#,
            velocity.x[0], velocity.x[1], velocity.y[0], velocity.y[1]
        ));
    }

    if let Some(gravity) = particle.emitter.gravity {
        s.push_str(&format!(r#", "gravity": {}"#, gravity));
    }

    if let Some(fade) = particle.emitter.fade {
        s.push_str(&format!(r#", "fade": {}"#, fade));
    }

    if let Some(ref rotation) = particle.emitter.rotation {
        s.push_str(&format!(r#", "rotation": [{}, {}]"#, rotation[0], rotation[1]));
    }

    if let Some(seed) = particle.emitter.seed {
        s.push_str(&format!(r#", "seed": {}"#, seed));
    }

    s.push_str("}}");
    s
}

/// Escape a string for JSON output.
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str(r#"\""#),
            '\\' => result.push_str(r"\\"),
            '\n' => result.push_str(r"\n"),
            '\r' => result.push_str(r"\r"),
            '\t' => result.push_str(r"\t"),
            c if c.is_control() => {
                result.push_str(&format!(r"\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Duration;

    #[test]
    fn test_format_palette_single_line() {
        let palette = Palette {
            name: "test".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
            ]),
        };
        let formatted = format_palette(&palette);
        // Should be single line
        assert!(!formatted.contains('\n'));
        assert!(formatted.contains(r#""type": "palette""#));
        assert!(formatted.contains(r#""name": "test""#));
    }

    #[test]
    fn test_format_sprite_visual_grid() {
        let sprite = Sprite {
            name: "test".to_string(),
            size: Some([4, 2]),
            palette: PaletteRef::Named("colors".to_string()),
            grid: vec!["{_}{a}{a}{_}".to_string(), "{a}{a}{a}{a}".to_string()],
            metadata: None,
            ..Default::default()
        };
        let formatted = format_sprite(&sprite);
        // Should have grid rows on separate lines
        assert!(formatted.contains('\n'));
        assert!(formatted.contains(r#""type": "sprite""#));
        assert!(formatted.contains(r#"  "{_}{a}{a}{_}""#));
        assert!(formatted.contains(r#"  "{a}{a}{a}{a}""#));
    }

    #[test]
    fn test_format_sprite_single_row() {
        let sprite = Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Named("colors".to_string()),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        };
        let formatted = format_sprite(&sprite);
        // Single row should stay on one line
        assert!(!formatted.contains('\n'));
        assert!(formatted.contains(r#""grid": ["{x}"]"#));
    }

    #[test]
    fn test_format_animation_single_line() {
        let anim = Animation {
            name: "walk".to_string(),
            frames: vec!["f1".to_string(), "f2".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        let formatted = format_animation(&anim);
        assert!(!formatted.contains('\n'));
        assert!(formatted.contains(r#""type": "animation""#));
        assert!(formatted.contains(r#""frames": ["f1", "f2"]"#));
    }

    #[test]
    fn test_format_variant_single_line() {
        let variant = Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([("{skin}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        };
        let formatted = format_variant(&variant);
        assert!(!formatted.contains('\n'));
        assert!(formatted.contains(r#""type": "variant""#));
        assert!(formatted.contains(r#""base": "hero""#));
    }

    #[test]
    fn test_format_pixelsrc_roundtrip() {
        let input = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "s", "palette": "p", "grid": ["{a}{a}", "{a}{a}"]}"##;

        let formatted = format_pixelsrc(input).unwrap();

        // Should parse the formatted output successfully
        let reader = Cursor::new(&formatted);
        let deserializer = serde_json::Deserializer::from_reader(reader);
        let objects: Vec<TtpObject> = deserializer.into_iter().filter_map(|r| r.ok()).collect();

        assert_eq!(objects.len(), 2);
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_json_string("a\\b"), r"a\\b");
        assert_eq!(escape_json_string("line1\nline2"), r"line1\nline2");
    }

    #[test]
    fn test_format_composition_visual_layers() {
        let comp = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: Some([8, 8]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("H".to_string(), Some("hero".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("ground".to_string()),
                    fill: Some(".".to_string()),
                    map: None,
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("objects".to_string()),
                    fill: None,
                    map: Some(vec!["....".to_string(), "..H.".to_string(), "....".to_string()]),
                    ..Default::default()
                },
            ],
        };
        let formatted = format_composition(&comp);
        // Should have layers and maps on separate lines
        assert!(formatted.contains('\n'));
        assert!(formatted.contains(r#""type": "composition""#));
    }
}
