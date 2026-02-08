//! Static MCP resources: format spec, brief guide, and palette catalog.

use rmcp::model::AnnotateAble;
use rmcp::model::*;

use crate::palettes;
use crate::prime::{PRIMER_BRIEF, PRIMER_FULL};

/// URI for the full format specification.
pub const URI_FORMAT_SPEC: &str = "pixelsrc://format-spec";
/// URI for the condensed format guide.
pub const URI_FORMAT_BRIEF: &str = "pixelsrc://format-brief";
/// URI for the palette catalog.
pub const URI_PALETTES: &str = "pixelsrc://palettes";

/// Returns the list of all static MCP resources.
pub fn list_static_resources() -> Vec<Resource> {
    vec![
        RawResource {
            uri: URI_FORMAT_SPEC.into(),
            name: "format-spec".into(),
            title: Some("Pixelsrc Format Specification".into()),
            description: Some(
                "Full .pxl format reference — object types, syntax, shape primitives, \
                 modifiers, semantic metadata, and complete examples."
                    .into(),
            ),
            mime_type: Some("text/markdown".into()),
            size: None,
            icons: None,
            meta: None,
        }
        .no_annotation(),
        RawResource {
            uri: URI_FORMAT_BRIEF.into(),
            name: "format-brief".into(),
            title: Some("Pixelsrc Quick Reference".into()),
            description: Some(
                "Condensed .pxl format guide (~2000 tokens) — \
                 ideal for AI context injection."
                    .into(),
            ),
            mime_type: Some("text/markdown".into()),
            size: None,
            icons: None,
            meta: None,
        }
        .no_annotation(),
        RawResource {
            uri: URI_PALETTES.into(),
            name: "palettes".into(),
            title: Some("Built-in Palette Catalog".into()),
            description: Some(
                "JSON array of all built-in palettes with names and color counts.".into(),
            ),
            mime_type: Some("application/json".into()),
            size: None,
            icons: None,
            meta: None,
        }
        .no_annotation(),
    ]
}

/// Reads a static resource by URI, returning its contents.
///
/// Returns `None` if the URI doesn't match any known static resource.
pub fn read_static_resource(uri: &str) -> Option<ReadResourceResult> {
    match uri {
        URI_FORMAT_SPEC => Some(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: URI_FORMAT_SPEC.into(),
                mime_type: Some("text/markdown".into()),
                text: PRIMER_FULL.into(),
                meta: None,
            }],
        }),
        URI_FORMAT_BRIEF => Some(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: URI_FORMAT_BRIEF.into(),
                mime_type: Some("text/markdown".into()),
                text: PRIMER_BRIEF.into(),
                meta: None,
            }],
        }),
        URI_PALETTES => Some(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: URI_PALETTES.into(),
                mime_type: Some("application/json".into()),
                text: build_palette_catalog_json(),
                meta: None,
            }],
        }),
        _ => None,
    }
}

/// Builds the JSON palette catalog: an array of objects with name and color_count.
fn build_palette_catalog_json() -> String {
    let entries: Vec<serde_json::Value> = palettes::list_builtins()
        .iter()
        .filter_map(|name| {
            palettes::get_builtin(name).map(|p| {
                // Exclude transparent ({_}) from the advertised color count
                let color_count = p.colors.keys().filter(|k| k.as_str() != "{_}").count();
                serde_json::json!({
                    "name": name,
                    "color_count": color_count,
                })
            })
        })
        .collect();
    serde_json::to_string_pretty(&entries).expect("palette catalog serialization cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_returns_three_resources() {
        let resources = list_static_resources();
        assert_eq!(resources.len(), 3);
    }

    #[test]
    fn test_list_resource_uris() {
        let resources = list_static_resources();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&URI_FORMAT_SPEC));
        assert!(uris.contains(&URI_FORMAT_BRIEF));
        assert!(uris.contains(&URI_PALETTES));
    }

    #[test]
    fn test_list_resources_have_names_and_descriptions() {
        for r in list_static_resources() {
            assert!(!r.name.is_empty(), "resource should have a name");
            assert!(r.description.is_some(), "resource should have a description");
            assert!(r.mime_type.is_some(), "resource should have a MIME type");
        }
    }

    #[test]
    fn test_read_format_spec() {
        let result = read_static_resource(URI_FORMAT_SPEC).expect("format-spec should exist");
        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, URI_FORMAT_SPEC);
                assert_eq!(mime_type.as_deref(), Some("text/markdown"));
                assert!(text.contains("Pixelsrc Primer"));
                assert!(text.contains("Format Quick Reference"));
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_format_brief() {
        let result = read_static_resource(URI_FORMAT_BRIEF).expect("format-brief should exist");
        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, URI_FORMAT_BRIEF);
                assert_eq!(mime_type.as_deref(), Some("text/markdown"));
                assert!(text.contains("Pixelsrc Quick Reference"));
                assert!(text.len() < 3500, "brief guide should be concise");
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_palettes() {
        let result = read_static_resource(URI_PALETTES).expect("palettes should exist");
        assert_eq!(result.contents.len(), 1);
        match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, text, mime_type, .. } => {
                assert_eq!(uri, URI_PALETTES);
                assert_eq!(mime_type.as_deref(), Some("application/json"));

                let catalog: Vec<serde_json::Value> =
                    serde_json::from_str(text).expect("palette JSON should parse");
                assert_eq!(catalog.len(), 7, "should have 7 built-in palettes");

                // Verify each entry has name and color_count
                for entry in &catalog {
                    assert!(entry["name"].is_string());
                    assert!(entry["color_count"].is_number());
                }

                // Spot-check known palettes
                let gameboy = catalog.iter().find(|e| e["name"] == "gameboy").unwrap();
                assert_eq!(gameboy["color_count"], 4); // 4 colors, transparent excluded

                let pico8 = catalog.iter().find(|e| e["name"] == "pico8").unwrap();
                assert_eq!(pico8["color_count"], 16);

                let one_bit = catalog.iter().find(|e| e["name"] == "1bit").unwrap();
                assert_eq!(one_bit["color_count"], 2);
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_read_unknown_uri_returns_none() {
        assert!(read_static_resource("pixelsrc://nonexistent").is_none());
        assert!(read_static_resource("").is_none());
        assert!(read_static_resource("https://example.com").is_none());
    }
}
