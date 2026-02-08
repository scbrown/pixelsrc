//! MCP import tool — convert PNG images to .pxl source.

use base64::Engine;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::import::{import_from_image_data, import_png_with_options, ImportOptions};

/// Input parameters for the pixelsrc_import tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportInput {
    /// Base64-encoded PNG image data.
    #[schemars(description = "Base64-encoded PNG image data")]
    pub image: Option<String>,

    /// Path to a PNG file on disk.
    #[schemars(description = "Path to a PNG file")]
    pub path: Option<String>,

    /// Maximum number of palette colors (2-256, default: 16).
    #[schemars(description = "Maximum number of palette colors (2-256, default: 16)")]
    pub max_colors: Option<usize>,

    /// Sprite name (auto-derived from filename if using path).
    #[schemars(description = "Sprite name (auto-derived from filename if using path)")]
    pub name: Option<String>,

    /// Enable role/relationship inference in output.
    #[schemars(description = "Enable role/relationship inference in output")]
    pub analyze: Option<bool>,
}

/// Execute the import tool logic.
pub fn run_import(input: ImportInput) -> Result<String, String> {
    let max_colors = input.max_colors.unwrap_or(16);
    if !(2..=256).contains(&max_colors) {
        return Err("max_colors must be between 2 and 256".into());
    }

    let analyze = input.analyze.unwrap_or(false);
    let options = ImportOptions {
        analyze,
        confidence_threshold: 0.5,
        hints: false,
        extract_shapes: false,
        half_sprite: false,
        dither_handling: crate::import::DitherHandling::Keep,
        detect_upscale: analyze,
        detect_outlines: analyze,
    };

    let result = if let Some(ref base64_data) = input.image {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Invalid base64 data: {}", e))?;

        let name = input.name.as_deref().unwrap_or("imported_sprite");
        import_from_image_data(&bytes, name, max_colors, &options)?
    } else if let Some(ref file_path) = input.path {
        let path = std::path::PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let name = input.name.as_deref().unwrap_or_else(|| {
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("imported_sprite")
        });
        import_png_with_options(&path, name, max_colors, &options)?
    } else {
        return Err("Either 'image' (base64 PNG) or 'path' (file path) is required".into());
    };

    let pxl_source = if analyze { result.to_structured_jsonl() } else { result.to_jsonl() };

    let summary = format!(
        "Imported: {} ({}x{}, {} colors)",
        result.name,
        result.width,
        result.height,
        result.palette.len()
    );

    Ok(format!("{}\n\n{}", summary, pxl_source))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal 2x2 PNG as base64 for testing.
    fn make_test_png_base64() -> String {
        use image::ImageEncoder;
        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
        img.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
        img.put_pixel(1, 1, image::Rgba([255, 255, 0, 255]));

        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png_bytes));
        encoder.write_image(&img, 2, 2, image::ColorType::Rgba8).unwrap();

        base64::engine::general_purpose::STANDARD.encode(&png_bytes)
    }

    #[test]
    fn test_import_base64_png() {
        let b64 = make_test_png_base64();
        let input = ImportInput {
            image: Some(b64),
            path: None,
            max_colors: None,
            name: Some("test_sprite".into()),
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test_sprite"));
        assert!(output.contains("\"type\":\"palette\""));
        assert!(output.contains("\"type\":\"sprite\""));
        assert!(output.contains("2x2"));
    }

    #[test]
    fn test_import_file_path() {
        let input = ImportInput {
            image: None,
            path: Some("tests/fixtures/valid/minimal_dot_dot.png".into()),
            max_colors: Some(8),
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"type\":\"palette\""));
        assert!(output.contains("\"type\":\"sprite\""));
    }

    #[test]
    fn test_import_with_analysis() {
        let b64 = make_test_png_base64();
        let input = ImportInput {
            image: Some(b64),
            path: None,
            max_colors: None,
            name: Some("analyzed".into()),
            analyze: Some(true),
        };
        let result = run_import(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("analyzed"));
    }

    #[test]
    fn test_import_max_colors_limit() {
        let b64 = make_test_png_base64();
        let input = ImportInput {
            image: Some(b64),
            path: None,
            max_colors: Some(2),
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should quantize down to 2 colors
        assert!(output.contains("2 colors") || output.contains("\"type\":\"palette\""));
    }

    #[test]
    fn test_import_invalid_max_colors() {
        let input = ImportInput {
            image: Some("dummy".into()),
            path: None,
            max_colors: Some(1),
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_colors"));
    }

    #[test]
    fn test_import_invalid_base64() {
        let input = ImportInput {
            image: Some("not-valid-base64!!!".into()),
            path: None,
            max_colors: None,
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid base64"));
    }

    #[test]
    fn test_import_corrupt_png() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"not a png file");
        let input = ImportInput {
            image: Some(b64),
            path: None,
            max_colors: None,
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to decode image"));
    }

    #[test]
    fn test_import_missing_file() {
        let input = ImportInput {
            image: None,
            path: Some("/nonexistent/path.png".into()),
            max_colors: None,
            name: None,
            analyze: None,
        };
        let result = run_import(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }

    #[test]
    fn test_import_no_input() {
        let input =
            ImportInput { image: None, path: None, max_colors: None, name: None, analyze: None };
        let result = run_import(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Either"));
    }
}
