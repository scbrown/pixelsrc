//! MCP diff tool — semantic comparison of .pxl sprites.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::diff::{diff_sprites, PaletteChange};
use crate::models::{PaletteRef, Sprite, TtpObject};
use crate::parser::parse_stream;

/// Input parameters for the pixelsrc_diff tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiffInput {
    /// First .pxl source content (JSONL) to compare.
    #[schemars(description = "First .pxl source content (JSONL) to compare")]
    pub source_a: String,

    /// Second .pxl source content (JSONL) to compare.
    #[schemars(description = "Second .pxl source content (JSONL) to compare")]
    pub source_b: String,

    /// Name of a specific sprite to compare (compares all matching sprites if omitted).
    #[schemars(
        description = "Name of a specific sprite to compare (compares all matching sprites if omitted)"
    )]
    pub sprite: Option<String>,
}

/// Parsed .pxl file contents for diffing.
struct ParsedSource {
    palettes: HashMap<String, HashMap<String, String>>,
    sprites: HashMap<String, Sprite>,
}

/// Parse a .pxl source string into palettes and sprites.
fn parse_source(source: &str) -> ParsedSource {
    let result = parse_stream(Cursor::new(source));
    let mut palettes = HashMap::new();
    let mut sprites = HashMap::new();

    for obj in result.objects {
        match obj {
            TtpObject::Palette(p) => {
                palettes.insert(p.name.clone(), p.colors.clone());
            }
            TtpObject::Sprite(s) => {
                sprites.insert(s.name.clone(), s);
            }
            _ => {}
        }
    }

    ParsedSource { palettes, sprites }
}

/// Resolve a palette reference to its color map.
fn resolve_palette(
    palette_ref: &PaletteRef,
    palettes: &HashMap<String, HashMap<String, String>>,
) -> HashMap<String, String> {
    match palette_ref {
        PaletteRef::Named(name) => palettes.get(name).cloned().unwrap_or_default(),
        PaletteRef::Inline(colors) => colors.clone(),
    }
}

/// Execute the diff tool logic.
pub fn run_diff(input: DiffInput) -> Result<String, String> {
    // 1. Parse both sources
    let parsed_a = parse_source(&input.source_a);
    let parsed_b = parse_source(&input.source_b);

    if parsed_a.sprites.is_empty() && parsed_b.sprites.is_empty() {
        return Err("No sprites found in either source".into());
    }

    // 2. Collect all sprite names from both sources
    let mut all_names: Vec<_> = parsed_a
        .sprites
        .keys()
        .chain(parsed_b.sprites.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_names.sort();

    // 3. Apply sprite name filter
    if let Some(ref filter) = input.sprite {
        if !parsed_a.sprites.contains_key(filter) && !parsed_b.sprites.contains_key(filter) {
            let available: Vec<&str> = all_names.iter().map(|s| s.as_str()).collect();
            return Err(format!(
                "Sprite '{}' not found in either source. Available: {}",
                filter,
                available.join(", ")
            ));
        }
        all_names.retain(|n| n == filter);
    }

    // 4. Diff each sprite pair
    let mut diffs: Vec<serde_json::Value> = Vec::new();

    for name in &all_names {
        match (parsed_a.sprites.get(name), parsed_b.sprites.get(name)) {
            (Some(sprite_a), Some(sprite_b)) => {
                let palette_a = resolve_palette(&sprite_a.palette, &parsed_a.palettes);
                let palette_b = resolve_palette(&sprite_b.palette, &parsed_b.palettes);
                let diff = diff_sprites(sprite_a, sprite_b, &palette_a, &palette_b);

                let mut obj = serde_json::json!({
                    "sprite": name,
                    "summary": diff.summary,
                });

                if let Some(ref dim) = diff.dimension_change {
                    obj["dimension_change"] = serde_json::json!({
                        "old": [dim.old.0, dim.old.1],
                        "new": [dim.new.0, dim.new.1],
                    });
                }

                if !diff.palette_changes.is_empty() {
                    obj["palette_changes"] = serde_json::json!(diff
                        .palette_changes
                        .iter()
                        .map(palette_change_to_json)
                        .collect::<Vec<_>>());
                }

                diffs.push(obj);
            }
            (Some(_), None) => {
                diffs.push(serde_json::json!({
                    "sprite": name,
                    "summary": format!("Sprite '{}' removed in second source", name),
                }));
            }
            (None, Some(_)) => {
                diffs.push(serde_json::json!({
                    "sprite": name,
                    "summary": format!("Sprite '{}' added in second source", name),
                }));
            }
            (None, None) => unreachable!(),
        }
    }

    // 5. Format output
    serde_json::to_string_pretty(&diffs).map_err(|e| format!("JSON serialization error: {}", e))
}

/// Convert a PaletteChange to a JSON value.
fn palette_change_to_json(change: &PaletteChange) -> serde_json::Value {
    match change {
        PaletteChange::Added { token, color } => serde_json::json!({
            "type": "added",
            "token": token,
            "color": color,
        }),
        PaletteChange::Removed { token } => serde_json::json!({
            "type": "removed",
            "token": token,
        }),
        PaletteChange::Changed { token, old_color, new_color } => serde_json::json!({
            "type": "changed",
            "token": token,
            "old_color": old_color,
            "new_color": new_color,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPRITE_V1: &str = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}"##;

    const SPRITE_V2_COLOR_CHANGE: &str = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"_": "#00000000", "x": "#00FF00"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}"##;

    const SPRITE_V2_SIZE_CHANGE: &str = r##"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 4, 4]}}}"##;

    const SPRITE_V2_TOKEN_ADDED: &str = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"_": "#00000000", "x": "#FF0000", "y": "#0000FF"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}"##;

    const TWO_SPRITES_A: &str = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "enemy", "size": [2, 2], "palette": {"_": "#00000000", "e": "#0000FF"}, "regions": {"e": {"rect": [0, 0, 2, 2]}}}"##;

    const TWO_SPRITES_B: &str = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"_": "#00000000", "x": "#00FF00"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "enemy", "size": [4, 4], "palette": {"_": "#00000000", "e": "#0000FF"}, "regions": {"e": {"rect": [0, 0, 4, 4]}}}"##;

    #[test]
    fn test_diff_identical_sprites() {
        let input =
            DiffInput { source_a: SPRITE_V1.into(), source_b: SPRITE_V1.into(), sprite: None };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json.len(), 1);
        assert_eq!(json[0]["summary"], "No differences");
    }

    #[test]
    fn test_diff_color_change() {
        let input = DiffInput {
            source_a: SPRITE_V1.into(),
            source_b: SPRITE_V2_COLOR_CHANGE.into(),
            sprite: None,
        };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json.len(), 1);
        let changes = json[0]["palette_changes"].as_array().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0]["type"], "changed");
        assert_eq!(changes[0]["token"], "x");
    }

    #[test]
    fn test_diff_size_change() {
        let input = DiffInput {
            source_a: SPRITE_V1.into(),
            source_b: SPRITE_V2_SIZE_CHANGE.into(),
            sprite: None,
        };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        let dim = &json[0]["dimension_change"];
        assert_eq!(dim["old"], serde_json::json!([2, 2]));
        assert_eq!(dim["new"], serde_json::json!([4, 4]));
    }

    #[test]
    fn test_diff_token_added() {
        let input = DiffInput {
            source_a: SPRITE_V1.into(),
            source_b: SPRITE_V2_TOKEN_ADDED.into(),
            sprite: None,
        };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        let changes = json[0]["palette_changes"].as_array().unwrap();
        assert!(changes.iter().any(|c| c["type"] == "added" && c["token"] == "y"));
    }

    #[test]
    fn test_diff_sprite_filter() {
        let input = DiffInput {
            source_a: TWO_SPRITES_A.into(),
            source_b: TWO_SPRITES_B.into(),
            sprite: Some("enemy".into()),
        };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json.len(), 1);
        assert_eq!(json[0]["sprite"], "enemy");
    }

    #[test]
    fn test_diff_multi_sprite() {
        let input = DiffInput {
            source_a: TWO_SPRITES_A.into(),
            source_b: TWO_SPRITES_B.into(),
            sprite: None,
        };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json.len(), 2);
    }

    #[test]
    fn test_diff_sprite_not_found() {
        let input = DiffInput {
            source_a: SPRITE_V1.into(),
            source_b: SPRITE_V1.into(),
            sprite: Some("nonexistent".into()),
        };
        let result = run_diff(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    #[test]
    fn test_diff_no_sprites() {
        let source = r##"{"type": "palette", "name": "empty", "colors": {}}"##;
        let input = DiffInput { source_a: source.into(), source_b: source.into(), sprite: None };
        let result = run_diff(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No sprites"));
    }

    #[test]
    fn test_diff_sprite_added() {
        let source_a = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}"##;
        let source_b = r##"{"type": "sprite", "name": "hero", "size": [2, 2], "palette": {"x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "new_sprite", "size": [1, 1], "palette": {"a": "#00FF00"}, "regions": {"a": {"points": [[0, 0]]}}}"##;
        let input =
            DiffInput { source_a: source_a.into(), source_b: source_b.into(), sprite: None };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        let added = json.iter().find(|d| d["sprite"] == "new_sprite").unwrap();
        assert!(added["summary"].as_str().unwrap().contains("added"));
    }

    #[test]
    fn test_diff_with_shared_palette() {
        let source_a = r##"{"type": "palette", "name": "pal", "colors": {"_": "#00000000", "r": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "pal", "regions": {"r": {"points": [[0, 0]]}}}"##;
        let source_b = r##"{"type": "palette", "name": "pal", "colors": {"_": "#00000000", "r": "#00FF00"}}
{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "pal", "regions": {"r": {"points": [[0, 0]]}}}"##;
        let input =
            DiffInput { source_a: source_a.into(), source_b: source_b.into(), sprite: None };
        let result = run_diff(input);
        assert!(result.is_ok(), "diff failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        let changes = json[0]["palette_changes"].as_array().unwrap();
        assert!(changes.iter().any(|c| c["type"] == "changed" && c["token"] == "r"));
    }
}
