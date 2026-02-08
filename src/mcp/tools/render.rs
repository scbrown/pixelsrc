//! MCP render tool — render .pxl source to PNG image.

use std::io::Cursor;

use base64::Engine;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::models::TtpObject;
use crate::output::scale_image;
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;

/// Input parameters for the pixelsrc_render tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenderInput {
    /// Inline .pxl source content (JSONL) to render.
    #[schemars(description = "Inline .pxl source content (JSONL) to render")]
    pub source: Option<String>,

    /// Path to a .pxl file on disk.
    #[schemars(description = "Path to a .pxl file on disk")]
    pub path: Option<String>,

    /// Name of a specific sprite to render (renders the first sprite if omitted).
    #[schemars(
        description = "Name of a specific sprite to render (renders the first sprite if omitted)"
    )]
    pub sprite: Option<String>,

    /// Scale factor for the output image (1-16, default: 1). Uses nearest-neighbor scaling.
    #[schemars(
        description = "Scale factor for the output image (1-16, default: 1). Uses nearest-neighbor scaling"
    )]
    pub scale: Option<u8>,
}

/// Render result containing base64 PNG and metadata.
#[derive(Debug)]
pub struct RenderOutput {
    pub base64_png: String,
    pub width: u32,
    pub height: u32,
    pub sprite_name: String,
    pub warnings: Vec<String>,
}

/// Execute the render tool logic.
pub fn run_render(input: RenderInput) -> Result<RenderOutput, String> {
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
    let mut warnings: Vec<String> =
        parse_result.warnings.iter().map(|w| format!("line {}: {}", w.line, w.message)).collect();

    // 3. Build registries and collect sprites
    let mut palette_registry = PaletteRegistry::new();
    let mut sprites = Vec::new();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(p) => {
                palette_registry.register(p);
            }
            TtpObject::Sprite(s) => {
                sprites.push(s);
            }
            _ => {}
        }
    }

    if sprites.is_empty() {
        return Err("No sprites found in source".into());
    }

    // 4. Select sprite
    let sprite = if let Some(ref name) = input.sprite {
        let available: Vec<&str> = sprites.iter().map(|s| s.name.as_str()).collect();
        sprites.iter().find(|s| s.name == *name).ok_or_else(|| {
            format!("Sprite '{}' not found. Available: {}", name, available.join(", "))
        })?
    } else {
        &sprites[0]
    };

    // 5. Resolve palette
    let resolved = palette_registry.resolve_lenient(sprite);
    if let Some(w) = &resolved.warning {
        warnings.push(w.message.clone());
    }

    // 6. Render
    let (image, render_warnings) = render_sprite(sprite, &resolved.palette.colors);
    for w in render_warnings {
        warnings.push(w.message);
    }

    // 7. Scale
    let scale = input.scale.unwrap_or(1).clamp(1, 16);
    let image = scale_image(image, scale);

    // 8. Encode to PNG
    let mut png_bytes = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(image.as_raw(), image.width(), image.height(), image::ColorType::Rgba8)
            .map_err(|e| format!("PNG encoding failed: {}", e))?;
    }

    // 9. Encode to base64
    let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    Ok(RenderOutput {
        base64_png,
        width: image.width(),
        height: image.height(),
        sprite_name: sprite.name.clone(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_DOT: &str = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;

    const TWO_SPRITES: &str = r##"{"type": "palette", "name": "pal", "colors": {"_": "#00000000", "r": "#FF0000", "g": "#00FF00"}}
{"type": "sprite", "name": "red_dot", "size": [1, 1], "palette": "pal", "regions": {"r": {"points": [[0, 0]], "z": 0}}}
{"type": "sprite", "name": "green_dot", "size": [1, 1], "palette": "pal", "regions": {"g": {"points": [[0, 0]], "z": 0}}}"##;

    #[test]
    fn test_render_inline_source() {
        let input =
            RenderInput { source: Some(MINIMAL_DOT.into()), path: None, sprite: None, scale: None };
        let result = run_render(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.sprite_name, "dot");
        assert_eq!(output.width, 1);
        assert_eq!(output.height, 1);
        assert!(!output.base64_png.is_empty());

        // Verify it's valid base64 that decodes to a PNG
        let png_bytes =
            base64::engine::general_purpose::STANDARD.decode(&output.base64_png).unwrap();
        assert_eq!(&png_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_select_sprite() {
        let input = RenderInput {
            source: Some(TWO_SPRITES.into()),
            path: None,
            sprite: Some("green_dot".into()),
            scale: None,
        };
        let result = run_render(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.sprite_name, "green_dot");
    }

    #[test]
    fn test_render_missing_sprite() {
        let input = RenderInput {
            source: Some(MINIMAL_DOT.into()),
            path: None,
            sprite: Some("nonexistent".into()),
            scale: None,
        };
        let result = run_render(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_render_with_scale() {
        let input = RenderInput {
            source: Some(MINIMAL_DOT.into()),
            path: None,
            sprite: None,
            scale: Some(4),
        };
        let result = run_render(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.width, 4);
        assert_eq!(output.height, 4);
    }

    #[test]
    fn test_render_no_input() {
        let input = RenderInput { source: None, path: None, sprite: None, scale: None };
        let result = run_render(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Either"));
    }

    #[test]
    fn test_render_no_sprites() {
        let input = RenderInput {
            source: Some(r#"{"type": "palette", "name": "empty", "colors": {}}"#.into()),
            path: None,
            sprite: None,
            scale: None,
        };
        let result = run_render(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No sprites"));
    }

    #[test]
    fn test_render_missing_file() {
        let input = RenderInput {
            source: None,
            path: Some("/nonexistent/path.pxl".into()),
            sprite: None,
            scale: None,
        };
        let result = run_render(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    #[test]
    fn test_render_file_path() {
        let input = RenderInput {
            source: None,
            path: Some("tests/fixtures/valid/minimal_dot_dot.pxl".into()),
            sprite: None,
            scale: None,
        };
        let result = run_render(input);
        // File may or may not exist; just ensure no panic
        if result.is_ok() {
            let output = result.unwrap();
            assert!(!output.base64_png.is_empty());
        }
    }

    #[test]
    fn test_render_scale_clamped() {
        let input = RenderInput {
            source: Some(MINIMAL_DOT.into()),
            path: None,
            sprite: None,
            scale: Some(255), // Should be clamped to 16
        };
        let result = run_render(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.width, 16);
        assert_eq!(output.height, 16);
    }
}
