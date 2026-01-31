//! Stateless rendering functions for the Python API.
//!
//! Mirrors the WASM API in `src/wasm.rs`, providing `render_to_png` and
//! `render_to_rgba` as top-level Python-callable functions.

use std::io::Cursor;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;

use super::types::RenderResult;

/// Parse PXL/JSONL text and build a palette registry + sprite list.
fn parse_and_prepare(pxl: &str) -> (PaletteRegistry, Vec<crate::models::Sprite>, Vec<String>) {
    let parse_result = parse_stream(Cursor::new(pxl));
    let mut registry = PaletteRegistry::new();
    let mut sprites = Vec::new();
    let warnings: Vec<String> =
        parse_result.warnings.iter().map(|w| format!("line {}: {}", w.line, w.message)).collect();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(p) => {
                registry.register(p);
            }
            TtpObject::Sprite(s) => {
                sprites.push(s);
            }
            _ => {}
        }
    }

    (registry, sprites, warnings)
}

/// Render the first sprite in a PXL string to PNG bytes.
///
/// Returns the PNG image data, or empty bytes if no sprites are found.
#[pyfunction]
pub fn render_to_png<'py>(py: Python<'py>, pxl: &str) -> Bound<'py, PyBytes> {
    let (registry, sprites, _warnings) = parse_and_prepare(pxl);

    if sprites.is_empty() {
        return PyBytes::new(py, &[]);
    }

    let sprite = &sprites[0];
    let resolved = registry.resolve_lenient(sprite);
    let (image, _render_warnings) = render_sprite(sprite, &resolved.palette.colors);

    let mut png_data = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(image.as_raw(), image.width(), image.height(), image::ColorType::Rgba8)
            .ok();
    }

    PyBytes::new(py, &png_data)
}

/// Render the first sprite in a PXL string to RGBA pixel data.
///
/// Returns a `RenderResult` with width, height, raw RGBA pixels, and warnings.
#[pyfunction]
pub fn render_to_rgba(pxl: &str) -> RenderResult {
    let (registry, sprites, mut warnings) = parse_and_prepare(pxl);

    if sprites.is_empty() {
        warnings.push("No sprites found in input".to_string());
        return RenderResult { width: 0, height: 0, pixels: Vec::new(), warnings };
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

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_DOT: &str = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;

    #[test]
    fn test_parse_and_prepare_basic() {
        let (registry, sprites, warnings) = parse_and_prepare(MINIMAL_DOT);
        assert_eq!(sprites.len(), 1);
        assert_eq!(sprites[0].name, "dot");
        assert!(warnings.is_empty());
        // Registry should have no named palettes (inline palette)
        assert_eq!(registry.names().count(), 0);
    }

    #[test]
    fn test_parse_and_prepare_empty() {
        let (_, sprites, _) = parse_and_prepare("");
        assert!(sprites.is_empty());
    }

    #[test]
    fn test_parse_and_prepare_with_palette() {
        let input = r##"{"type": "palette", "name": "reds", "colors": {"_": "#00000000", "r": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "reds", "regions": {"r": {"points": [[0, 0]], "z": 0}}}"##;
        let (registry, sprites, warnings) = parse_and_prepare(input);
        assert_eq!(sprites.len(), 1);
        assert!(warnings.is_empty());
        assert_eq!(registry.names().count(), 1);
    }
}
