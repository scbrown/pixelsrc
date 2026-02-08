//! Dynamic MCP resource templates: individual palettes, examples, and prompt templates.
//!
//! Provides URI templates that clients can expand to request specific resources:
//! - `pixelsrc://palette/{name}` → full palette definition as JSON
//! - `pixelsrc://example/{name}` → example `.pxl` file content
//! - `pixelsrc://template/{type}` → GenAI prompt template

use rmcp::model::AnnotateAble;
use rmcp::model::*;

use crate::palettes;

// ── Embedded content ────────────────────────────────────────────────

// Example .pxl files
const EXAMPLE_COIN: &str = include_str!("../../../examples/coin.pxl");
const EXAMPLE_HEART: &str = include_str!("../../../examples/heart.pxl");
const EXAMPLE_HERO: &str = include_str!("../../../examples/hero.pxl");
const EXAMPLE_WALK_CYCLE: &str = include_str!("../../../examples/walk_cycle.pxl");

// GenAI prompt templates
const TEMPLATE_CHARACTER: &str = include_str!("../../../docs/prompts/templates/character.txt");
const TEMPLATE_ITEM: &str = include_str!("../../../docs/prompts/templates/item.txt");
const TEMPLATE_TILESET: &str = include_str!("../../../docs/prompts/templates/tileset.txt");
const TEMPLATE_ANIMATION: &str = include_str!("../../../docs/prompts/templates/animation.txt");

/// Available example names.
const EXAMPLE_NAMES: &[&str] = &["coin", "heart", "hero", "walk_cycle"];

/// Available prompt template types.
const TEMPLATE_TYPES: &[&str] = &["character", "item", "tileset", "animation"];

// ── URI template constants ──────────────────────────────────────────

/// URI template for individual palette resources.
pub const URI_TEMPLATE_PALETTE: &str = "pixelsrc://palette/{name}";
/// URI template for example file resources.
pub const URI_TEMPLATE_EXAMPLE: &str = "pixelsrc://example/{name}";
/// URI template for prompt template resources.
pub const URI_TEMPLATE_TEMPLATE: &str = "pixelsrc://template/{type}";

// ── Listing ─────────────────────────────────────────────────────────

/// Returns the list of resource templates for `resources/templates/list`.
pub fn list_resource_templates() -> Vec<ResourceTemplate> {
    vec![
        RawResourceTemplate {
            uri_template: URI_TEMPLATE_PALETTE.into(),
            name: "palette".into(),
            title: Some("Individual Palette".into()),
            description: Some(
                "Full palette definition with all colors and token mappings. \
                 Available palettes: gameboy, nes, pico8, grayscale, 1bit, dracula, synthwave."
                    .into(),
            ),
            mime_type: Some("application/json".into()),
            icons: None,
        }
        .no_annotation(),
        RawResourceTemplate {
            uri_template: URI_TEMPLATE_EXAMPLE.into(),
            name: "example".into(),
            title: Some("Example .pxl File".into()),
            description: Some(
                "Complete example .pxl source file. \
                 Available examples: coin, heart, hero, walk_cycle."
                    .into(),
            ),
            mime_type: Some("text/plain".into()),
            icons: None,
        }
        .no_annotation(),
        RawResourceTemplate {
            uri_template: URI_TEMPLATE_TEMPLATE.into(),
            name: "template".into(),
            title: Some("GenAI Prompt Template".into()),
            description: Some(
                "Prompt template for AI-assisted pixel art creation. \
                 Available types: character, item, tileset, animation."
                    .into(),
            ),
            mime_type: Some("text/markdown".into()),
            icons: None,
        }
        .no_annotation(),
    ]
}

// ── Reading ─────────────────────────────────────────────────────────

/// Reads a dynamic resource by URI, returning its contents.
///
/// Handles URIs matching the resource templates:
/// - `pixelsrc://palette/{name}`
/// - `pixelsrc://example/{name}`
/// - `pixelsrc://template/{type}`
///
/// Returns `None` if the URI doesn't match any template pattern.
/// Returns `Some(Err(...))` if the URI matches a pattern but the name is unknown.
/// Returns `Some(Ok(...))` on success.
pub fn read_template_resource(uri: &str) -> Option<Result<ReadResourceResult, String>> {
    if let Some(name) = uri.strip_prefix("pixelsrc://palette/") {
        Some(read_palette(name, uri))
    } else if let Some(name) = uri.strip_prefix("pixelsrc://example/") {
        Some(read_example(name, uri))
    } else if let Some(typ) = uri.strip_prefix("pixelsrc://template/") {
        Some(read_template(typ, uri))
    } else {
        None
    }
}

/// Read a single palette as full JSON.
fn read_palette(name: &str, uri: &str) -> Result<ReadResourceResult, String> {
    let palette_name = name.strip_prefix('@').unwrap_or(name);

    match palettes::get_builtin(palette_name) {
        Some(palette) => {
            let mut colors = serde_json::Map::new();
            let mut sorted: Vec<_> = palette.colors.iter().collect();
            sorted.sort_by_key(|(k, _)| (*k).clone());
            for (key, value) in sorted {
                colors.insert(key.clone(), serde_json::Value::String(value.clone()));
            }

            let json = serde_json::json!({
                "name": palette_name,
                "colors": colors,
            });

            let text =
                serde_json::to_string_pretty(&json).expect("palette serialization cannot fail");

            Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: uri.into(),
                    mime_type: Some("application/json".into()),
                    text,
                    meta: None,
                }],
            })
        }
        None => {
            let available = palettes::list_builtins();
            Err(format!("Unknown palette '{}'. Available: {}", name, available.join(", ")))
        }
    }
}

/// Read an example .pxl file.
fn read_example(name: &str, uri: &str) -> Result<ReadResourceResult, String> {
    let content = match name {
        "coin" => EXAMPLE_COIN,
        "heart" => EXAMPLE_HEART,
        "hero" => EXAMPLE_HERO,
        "walk_cycle" => EXAMPLE_WALK_CYCLE,
        _ => {
            return Err(format!(
                "Unknown example '{}'. Available: {}",
                name,
                EXAMPLE_NAMES.join(", ")
            ));
        }
    };

    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.into(),
            mime_type: Some("text/plain".into()),
            text: content.into(),
            meta: None,
        }],
    })
}

/// Read a prompt template.
fn read_template(typ: &str, uri: &str) -> Result<ReadResourceResult, String> {
    let content = match typ {
        "character" => TEMPLATE_CHARACTER,
        "item" => TEMPLATE_ITEM,
        "tileset" => TEMPLATE_TILESET,
        "animation" => TEMPLATE_ANIMATION,
        _ => {
            return Err(format!(
                "Unknown template type '{}'. Available: {}",
                typ,
                TEMPLATE_TYPES.join(", ")
            ));
        }
    };

    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.into(),
            mime_type: Some("text/markdown".into()),
            text: content.into(),
            meta: None,
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── list_resource_templates ──────────────────────────────────

    #[test]
    fn test_list_returns_three_templates() {
        let templates = list_resource_templates();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_list_template_uri_templates() {
        let templates = list_resource_templates();
        let uris: Vec<&str> = templates.iter().map(|t| t.uri_template.as_str()).collect();
        assert!(uris.contains(&URI_TEMPLATE_PALETTE));
        assert!(uris.contains(&URI_TEMPLATE_EXAMPLE));
        assert!(uris.contains(&URI_TEMPLATE_TEMPLATE));
    }

    #[test]
    fn test_list_templates_have_names_and_descriptions() {
        for t in list_resource_templates() {
            assert!(!t.name.is_empty(), "template should have a name");
            assert!(t.description.is_some(), "template should have a description");
            assert!(t.mime_type.is_some(), "template should have a MIME type");
        }
    }

    // ── palette resources ───────────────────────────────────────

    #[test]
    fn test_read_palette_gameboy() {
        let result = read_template_resource("pixelsrc://palette/gameboy")
            .expect("should match palette pattern")
            .expect("gameboy should exist");

        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, "pixelsrc://palette/gameboy");
                assert_eq!(mime_type.as_deref(), Some("application/json"));

                let parsed: serde_json::Value =
                    serde_json::from_str(text).expect("should be valid JSON");
                assert_eq!(parsed["name"], "gameboy");

                let colors = parsed["colors"].as_object().expect("colors should be object");
                assert!(colors.contains_key("{lightest}"));
                assert!(colors.contains_key("{light}"));
                assert!(colors.contains_key("{dark}"));
                assert!(colors.contains_key("{darkest}"));
                assert!(colors.contains_key("{_}"));
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_palette_all_builtins() {
        for name in palettes::list_builtins() {
            let uri = format!("pixelsrc://palette/{}", name);
            let result = read_template_resource(&uri)
                .expect("should match palette pattern")
                .unwrap_or_else(|e| panic!("palette {} should exist: {}", name, e));

            assert_eq!(result.contents.len(), 1);
            match &result.contents[0] {
                ResourceContents::TextResourceContents { text, .. } => {
                    let parsed: serde_json::Value =
                        serde_json::from_str(text).expect("should be valid JSON");
                    assert_eq!(parsed["name"], name);
                    assert!(parsed["colors"].is_object());
                }
                _ => panic!("expected text content for palette {}", name),
            }
        }
    }

    #[test]
    fn test_read_palette_nonexistent() {
        let result = read_template_resource("pixelsrc://palette/nonexistent")
            .expect("should match palette pattern");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown palette 'nonexistent'"));
        assert!(err.contains("gameboy"));
    }

    // ── example resources ───────────────────────────────────────

    #[test]
    fn test_read_example_coin() {
        let result = read_template_resource("pixelsrc://example/coin")
            .expect("should match example pattern")
            .expect("coin should exist");

        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, "pixelsrc://example/coin");
                assert_eq!(mime_type.as_deref(), Some("text/plain"));
                assert!(text.contains("type:") || text.contains("\"type\""));
                assert!(text.contains("coin"));
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_example_all() {
        for name in EXAMPLE_NAMES {
            let uri = format!("pixelsrc://example/{}", name);
            let result = read_template_resource(&uri)
                .expect("should match example pattern")
                .unwrap_or_else(|e| panic!("example {} should exist: {}", name, e));

            assert_eq!(result.contents.len(), 1);
            match &result.contents[0] {
                ResourceContents::TextResourceContents { text, .. } => {
                    assert!(!text.is_empty(), "example {} should have content", name);
                }
                _ => panic!("expected text content for example {}", name),
            }
        }
    }

    #[test]
    fn test_read_example_nonexistent() {
        let result = read_template_resource("pixelsrc://example/nonexistent")
            .expect("should match example pattern");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown example 'nonexistent'"));
        assert!(err.contains("coin"));
    }

    // ── template resources ──────────────────────────────────────

    #[test]
    fn test_read_template_character() {
        let result = read_template_resource("pixelsrc://template/character")
            .expect("should match template pattern")
            .expect("character should exist");

        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, "pixelsrc://template/character");
                assert_eq!(mime_type.as_deref(), Some("text/markdown"));
                assert!(text.contains("character"));
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_template_all() {
        for typ in TEMPLATE_TYPES {
            let uri = format!("pixelsrc://template/{}", typ);
            let result = read_template_resource(&uri)
                .expect("should match template pattern")
                .unwrap_or_else(|e| panic!("template {} should exist: {}", typ, e));

            assert_eq!(result.contents.len(), 1);
            match &result.contents[0] {
                ResourceContents::TextResourceContents { text, .. } => {
                    assert!(!text.is_empty(), "template {} should have content", typ);
                }
                _ => panic!("expected text content for template {}", typ),
            }
        }
    }

    #[test]
    fn test_read_template_nonexistent() {
        let result = read_template_resource("pixelsrc://template/nonexistent")
            .expect("should match template pattern");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown template type 'nonexistent'"));
        assert!(err.contains("character"));
    }

    // ── non-matching URIs ───────────────────────────────────────

    #[test]
    fn test_non_matching_uris_return_none() {
        assert!(read_template_resource("pixelsrc://format-spec").is_none());
        assert!(read_template_resource("pixelsrc://palettes").is_none());
        assert!(read_template_resource("https://example.com").is_none());
        assert!(read_template_resource("").is_none());
        assert!(read_template_resource("pixelsrc://unknown/foo").is_none());
    }
}
