//! MCP explain tool — describe .pxl objects in human-readable JSON.

use std::collections::HashMap;
use std::io::Cursor;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::explain::{explain_object, resolve_palette_colors, Explanation};
use crate::models::TtpObject;
use crate::parser::parse_stream;

/// Input parameters for the pixelsrc_explain tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainInput {
    /// Inline .pxl source content (JSONL) to explain.
    #[schemars(description = "Inline .pxl source content (JSONL) to explain")]
    pub source: Option<String>,

    /// Path to a .pxl file on disk.
    #[schemars(description = "Path to a .pxl file on disk")]
    pub path: Option<String>,

    /// Name of a specific object to explain (explains all objects if omitted).
    #[schemars(
        description = "Name of a specific object to explain (explains all objects if omitted)"
    )]
    pub name: Option<String>,
}

/// Execute the explain tool logic.
pub fn run_explain(input: ExplainInput) -> Result<String, String> {
    // 1. Get source from string or file
    let source = if let Some(s) = input.source {
        s
    } else if let Some(ref p) = input.path {
        std::fs::read_to_string(p).map_err(|e| format!("Failed to read file '{}': {}", p, e))?
    } else {
        return Err("Either 'source' (inline .pxl text) or 'path' (file path) is required".into());
    };

    // 2. Parse source
    let parse_result = parse_stream(Cursor::new(&source));

    if parse_result.objects.is_empty() {
        return Err("No objects found in source".into());
    }

    // 3. Build palette lookup for color resolution
    let mut known_palettes: HashMap<String, HashMap<String, String>> = HashMap::new();
    for obj in &parse_result.objects {
        if let TtpObject::Palette(palette) = obj {
            known_palettes.insert(palette.name.clone(), palette.colors.clone());
        }
    }

    // 4. Explain each object, applying name filter
    let mut explanations: Vec<serde_json::Value> = Vec::new();

    for obj in &parse_result.objects {
        let obj_name = match obj {
            TtpObject::Palette(p) => &p.name,
            TtpObject::Sprite(s) => &s.name,
            TtpObject::Composition(c) => &c.name,
            TtpObject::Animation(a) => &a.name,
            TtpObject::Variant(v) => &v.name,
            TtpObject::Particle(p) => &p.name,
            TtpObject::Transform(t) => &t.name,
            TtpObject::StateRules(sr) => &sr.name,
            TtpObject::Import(i) => &i.from,
        };

        if let Some(ref filter) = input.name {
            if obj_name != filter {
                continue;
            }
        }

        // Resolve palette colors for sprites
        let resolved_colors: Option<HashMap<String, String>> =
            if let TtpObject::Sprite(sprite) = obj {
                resolve_palette_colors(&sprite.palette, &known_palettes)
            } else {
                None
            };

        let exp = explain_object(obj, resolved_colors.as_ref());
        explanations.push(explanation_to_json(&exp));
    }

    if explanations.is_empty() {
        if let Some(ref filter) = input.name {
            return Err(format!("No object named '{}' found in source", filter));
        }
        return Err("No objects to explain".into());
    }

    // 5. Format output
    let output = if explanations.len() == 1 {
        serde_json::to_string_pretty(&explanations[0])
    } else {
        serde_json::to_string_pretty(&explanations)
    };

    output.map_err(|e| format!("JSON serialization error: {}", e))
}

/// Convert an Explanation to a JSON value.
fn explanation_to_json(exp: &Explanation) -> serde_json::Value {
    match exp {
        Explanation::Sprite(s) => serde_json::json!({
            "type": "sprite",
            "name": s.name,
            "width": s.width,
            "height": s.height,
            "total_cells": s.total_cells,
            "palette_ref": s.palette_ref,
            "tokens": s.tokens.iter().map(|t| serde_json::json!({
                "token": t.token,
                "count": t.count,
                "percentage": t.percentage,
                "color": t.color,
                "color_name": t.color_name,
            })).collect::<Vec<_>>(),
            "transparent_count": s.transparent_count,
            "transparency_ratio": s.transparency_ratio,
            "consistent_rows": s.consistent_rows,
            "issues": s.issues,
        }),
        Explanation::Palette(p) => serde_json::json!({
            "type": "palette",
            "name": p.name,
            "color_count": p.color_count,
            "colors": p.colors.iter().map(|(token, hex, name)| serde_json::json!({
                "token": token,
                "color": hex,
                "color_name": name,
            })).collect::<Vec<_>>(),
            "is_builtin": p.is_builtin,
        }),
        Explanation::Animation(a) => serde_json::json!({
            "type": "animation",
            "name": a.name,
            "frames": a.frames,
            "frame_count": a.frame_count,
            "duration_ms": a.duration_ms,
            "loops": a.loops,
        }),
        Explanation::Composition(c) => serde_json::json!({
            "type": "composition",
            "name": c.name,
            "base": c.base,
            "size": c.size,
            "cell_size": c.cell_size,
            "sprite_count": c.sprite_count,
            "layer_count": c.layer_count,
        }),
        Explanation::Variant(v) => serde_json::json!({
            "type": "variant",
            "name": v.name,
            "base": v.base,
            "override_count": v.override_count,
            "overrides": v.overrides.iter().map(|(token, color)| serde_json::json!({
                "token": token,
                "color": color,
            })).collect::<Vec<_>>(),
        }),
        Explanation::Particle(p) => serde_json::json!({
            "type": "particle",
            "name": p.name,
            "sprite": p.sprite,
            "rate": p.rate,
            "lifetime": p.lifetime,
            "has_gravity": p.has_gravity,
            "has_fade": p.has_fade,
        }),
        Explanation::Transform(t) => serde_json::json!({
            "type": "transform",
            "name": t.name,
            "is_parameterized": t.is_parameterized,
            "params": t.params,
            "generates_animation": t.generates_animation,
            "frame_count": t.frame_count,
            "transform_type": t.transform_type,
        }),
        Explanation::StateRules(sr) => serde_json::json!({
            "type": "state-rules",
            "name": sr.name,
            "rule_count": sr.rule_count,
            "selectors": sr.selectors,
        }),
        Explanation::Import(i) => serde_json::json!({
            "type": "import",
            "from": i.from,
            "is_directory": i.is_directory,
            "is_relative": i.is_relative,
            "alias": i.alias,
            "imported_types": i.imported_types,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SPRITE: &str = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;

    const TWO_OBJECTS: &str = r##"{"type": "palette", "name": "pal", "colors": {"_": "#00000000", "r": "#FF0000", "g": "#00FF00"}}
{"type": "sprite", "name": "red_dot", "size": [1, 1], "palette": "pal", "regions": {"r": {"points": [[0, 0]], "z": 0}}}"##;

    #[test]
    fn test_explain_single_sprite() {
        let input = ExplainInput { source: Some(MINIMAL_SPRITE.into()), path: None, name: None };
        let result = run_explain(input);
        assert!(result.is_ok(), "explain failed: {:?}", result.err());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json["type"], "sprite");
        assert_eq!(json["name"], "dot");
        assert_eq!(json["width"], 1);
        assert_eq!(json["height"], 1);
    }

    #[test]
    fn test_explain_multi_object() {
        let input = ExplainInput { source: Some(TWO_OBJECTS.into()), path: None, name: None };
        let result = run_explain(input);
        assert!(result.is_ok(), "explain failed: {:?}", result.err());
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json.len(), 2);
        assert_eq!(json[0]["type"], "palette");
        assert_eq!(json[1]["type"], "sprite");
    }

    #[test]
    fn test_explain_name_filter() {
        let input = ExplainInput {
            source: Some(TWO_OBJECTS.into()),
            path: None,
            name: Some("red_dot".into()),
        };
        let result = run_explain(input);
        assert!(result.is_ok(), "explain failed: {:?}", result.err());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json["type"], "sprite");
        assert_eq!(json["name"], "red_dot");
    }

    #[test]
    fn test_explain_name_filter_not_found() {
        let input = ExplainInput {
            source: Some(MINIMAL_SPRITE.into()),
            path: None,
            name: Some("nonexistent".into()),
        };
        let result = run_explain(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    #[test]
    fn test_explain_no_input() {
        let input = ExplainInput { source: None, path: None, name: None };
        let result = run_explain(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Either"));
    }

    #[test]
    fn test_explain_empty_source() {
        let input = ExplainInput { source: Some("".into()), path: None, name: None };
        let result = run_explain(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No objects"));
    }

    #[test]
    fn test_explain_missing_file() {
        let input =
            ExplainInput { source: None, path: Some("/nonexistent/path.pxl".into()), name: None };
        let result = run_explain(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    #[test]
    fn test_explain_palette_with_colors() {
        let source = r##"{"type": "palette", "name": "warm", "colors": {"_": "#00000000", "r": "#FF0000", "g": "#00FF00"}}"##;
        let input = ExplainInput { source: Some(source.into()), path: None, name: None };
        let result = run_explain(input);
        assert!(result.is_ok(), "explain failed: {:?}", result.err());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json["type"], "palette");
        assert_eq!(json["color_count"], 3);
    }
}
