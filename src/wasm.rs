//! WASM API module for browser/JS interop
//!
//! Provides WebAssembly bindings for rendering pixelsrc JSONL to images.

use std::io::Cursor;

use wasm_bindgen::prelude::*;

use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

/// Result of rendering a sprite to RGBA pixels.
#[wasm_bindgen]
pub struct RenderResult {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl RenderResult {
    /// Width of the rendered image in pixels
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the rendered image in pixels
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGBA pixel data (4 bytes per pixel)
    #[wasm_bindgen(getter)]
    pub fn pixels(&self) -> Vec<u8> {
        self.pixels.clone()
    }

    /// Any warnings generated during rendering
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// Parse JSONL and build registry, returning (registry, sprites, warnings)
fn parse_and_prepare(jsonl: &str) -> (PaletteRegistry, Vec<crate::models::Sprite>, Vec<String>) {
    let parse_result = parse_stream(Cursor::new(jsonl));
    let mut registry = PaletteRegistry::new();
    let mut sprites = Vec::new();
    let warnings: Vec<String> = parse_result
        .warnings
        .iter()
        .map(|w| format!("line {}: {}", w.line, w.message))
        .collect();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(p) => {
                registry.register(p);
            }
            TtpObject::Sprite(s) => {
                sprites.push(s);
            }
            _ => {
                // Skip compositions and animations for now
            }
        }
    }

    (registry, sprites, warnings)
}

/// Render the first sprite in a JSONL string to PNG bytes.
///
/// # Arguments
/// * `jsonl` - JSONL string containing palette and sprite definitions
///
/// # Returns
/// PNG image data as a byte array, or an empty array if no sprites found
#[wasm_bindgen]
pub fn render_to_png(jsonl: &str) -> Vec<u8> {
    let (registry, sprites, _warnings) = parse_and_prepare(jsonl);

    if sprites.is_empty() {
        return Vec::new();
    }

    let sprite = &sprites[0];
    let resolved = registry.resolve_lenient(sprite);
    let (image, _render_warnings) = render_sprite(sprite, &resolved.palette.colors);

    // Encode to PNG
    let mut png_data = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ColorType::Rgba8,
            )
            .ok();
    }

    png_data
}

/// Render the first sprite in a JSONL string to RGBA pixels.
///
/// # Arguments
/// * `jsonl` - JSONL string containing palette and sprite definitions
///
/// # Returns
/// RenderResult containing width, height, pixels (RGBA), and warnings
#[wasm_bindgen]
pub fn render_to_rgba(jsonl: &str) -> RenderResult {
    let (registry, sprites, mut warnings) = parse_and_prepare(jsonl);

    if sprites.is_empty() {
        warnings.push("No sprites found in input".to_string());
        return RenderResult {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            warnings,
        };
    }

    let sprite = &sprites[0];
    let resolved = registry.resolve_lenient(sprite);

    if let Some(w) = &resolved.warning {
        warnings.push(w.message.clone());
    }

    let (image, render_warnings) = render_sprite(sprite, &resolved.palette.colors);

    for w in render_warnings {
        warnings.push(w.message);
    }

    RenderResult {
        width: image.width(),
        height: image.height(),
        pixels: image.into_raw(),
        warnings,
    }
}

/// List all sprite names in a JSONL string.
///
/// # Arguments
/// * `jsonl` - JSONL string containing sprite definitions
///
/// # Returns
/// Array of sprite names
#[wasm_bindgen]
pub fn list_sprites(jsonl: &str) -> Vec<String> {
    let (_registry, sprites, _warnings) = parse_and_prepare(jsonl);
    sprites.into_iter().map(|s| s.name).collect()
}

/// Validate a JSONL string and return any errors/warnings.
///
/// # Arguments
/// * `jsonl` - JSONL string to validate
///
/// # Returns
/// Array of validation messages (empty if valid)
#[wasm_bindgen]
pub fn validate(jsonl: &str) -> Vec<String> {
    let (registry, sprites, mut warnings) = parse_and_prepare(jsonl);

    // Validate palette references
    for sprite in &sprites {
        let resolved = registry.resolve_lenient(sprite);
        if let Some(w) = &resolved.warning {
            warnings.push(format!("sprite '{}': {}", sprite.name, w.message));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_DOT: &str = r##"{"type": "sprite", "name": "dot", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}"##;

    const HEART_WITH_PALETTE: &str = r##"{"type": "palette", "name": "reds", "colors": {"{_}": "#00000000", "{r}": "#FF0000", "{p}": "#FF6B6B"}}
{"type": "sprite", "name": "heart", "palette": "reds", "grid": ["{_}{r}{r}{_}", "{r}{r}{r}{r}", "{_}{r}{r}{_}", "{_}{_}{r}{_}"]}"##;

    #[test]
    fn test_list_sprites() {
        let result = list_sprites(MINIMAL_DOT);
        assert_eq!(result, vec!["dot"]);
    }

    #[test]
    fn test_list_sprites_multiple() {
        let jsonl = r#"{"type": "sprite", "name": "a", "palette": {}, "grid": []}
{"type": "sprite", "name": "b", "palette": {}, "grid": []}"#;
        let result = list_sprites(jsonl);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_validate_valid() {
        let result = validate(HEART_WITH_PALETTE);
        assert!(result.is_empty(), "Expected no warnings: {:?}", result);
    }

    #[test]
    fn test_validate_missing_palette() {
        let jsonl =
            r#"{"type": "sprite", "name": "bad", "palette": "nonexistent", "grid": ["{x}"]}"#;
        let result = validate(jsonl);
        assert!(!result.is_empty());
        assert!(result[0].contains("not found"));
    }

    #[test]
    fn test_render_to_rgba() {
        let result = render_to_rgba(MINIMAL_DOT);
        assert_eq!(result.width(), 1);
        assert_eq!(result.height(), 1);
        assert_eq!(result.pixels().len(), 4); // 1 pixel * 4 bytes (RGBA)
                                              // Red pixel: #FF0000 = [255, 0, 0, 255]
        assert_eq!(result.pixels()[0], 255); // R
        assert_eq!(result.pixels()[1], 0); // G
        assert_eq!(result.pixels()[2], 0); // B
        assert_eq!(result.pixels()[3], 255); // A
    }

    #[test]
    fn test_render_to_rgba_no_sprites() {
        let result = render_to_rgba(r#"{"type": "palette", "name": "empty", "colors": {}}"#);
        assert_eq!(result.width(), 0);
        assert_eq!(result.height(), 0);
        assert!(result.warnings().iter().any(|w| w.contains("No sprites")));
    }

    #[test]
    fn test_render_to_png() {
        let result = render_to_png(MINIMAL_DOT);
        assert!(!result.is_empty());
        // PNG magic bytes
        assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_to_png_no_sprites() {
        let result = render_to_png(r#"{"type": "palette", "name": "empty", "colors": {}}"#);
        assert!(result.is_empty());
    }
}
