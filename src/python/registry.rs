//! Stateful registry for the Python API.
//!
//! Provides `PyRegistry`, a `#[pyclass]` wrapping owned `PaletteRegistry` and
//! `SpriteRegistry`. Supports incremental loading and rendering of sprites.

use std::io::Cursor;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, SpriteRegistry};

use super::types::RenderResult;

/// A stateful registry that accumulates palettes and sprites from PXL content.
///
/// Unlike the stateless `render_to_png` / `render_to_rgba` functions, `Registry`
/// lets you load multiple PXL strings or files and then render any sprite by name.
///
/// Example (Python):
/// ```python
/// from pixelsrc import Registry
///
/// reg = Registry()
/// reg.load_file("assets/palettes.pxl")
/// reg.load_file("assets/sprites.pxl")
///
/// print(reg.sprites())   # ['hero', 'enemy', ...]
/// print(reg.palettes())  # ['warm', 'cool', ...]
///
/// result = reg.render("hero")
/// png = reg.render_to_png("hero")
/// all_pngs = reg.render_all()
/// ```
#[pyclass(name = "Registry")]
pub struct PyRegistry {
    palette_registry: PaletteRegistry,
    sprite_registry: SpriteRegistry,
}

#[pymethods]
impl PyRegistry {
    /// Create a new empty registry.
    #[new]
    fn new() -> Self {
        Self {
            palette_registry: PaletteRegistry::new(),
            sprite_registry: SpriteRegistry::new(),
        }
    }

    /// Load PXL/JSONL content from a string into the registry.
    ///
    /// Palettes and sprites are accumulated — calling `load` multiple times
    /// adds to the existing registry contents.
    fn load(&mut self, pxl: &str) {
        let parse_result = parse_stream(Cursor::new(pxl));
        for obj in parse_result.objects {
            match obj {
                TtpObject::Palette(p) => {
                    self.palette_registry.register(p);
                }
                TtpObject::Sprite(s) => {
                    self.sprite_registry.register_sprite(s);
                }
                TtpObject::Variant(v) => {
                    self.sprite_registry.register_variant(v);
                }
                _ => {}
            }
        }
    }

    /// Load PXL/JSONL content from a file path into the registry.
    ///
    /// Raises `OSError` if the file cannot be read.
    fn load_file(&mut self, path: &str) -> PyResult<()> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            pyo3::exceptions::PyOSError::new_err(format!("{}", e))
        })?;
        self.load(&content);
        Ok(())
    }

    /// Return a sorted list of sprite names in the registry.
    fn sprites(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sprite_registry.names().cloned().collect();
        names.sort();
        names
    }

    /// Return a sorted list of palette names in the registry.
    fn palettes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.palette_registry.names().cloned().collect();
        names.sort();
        names
    }

    /// Render a sprite by name and return a `RenderResult` with RGBA pixel data.
    ///
    /// The sprite is resolved against the current palette and sprite registries,
    /// handling named palettes, variants, and source references.
    ///
    /// Raises `ValueError` if the sprite is not found.
    fn render(&self, name: &str) -> PyResult<RenderResult> {
        let resolved = self.sprite_registry.resolve(name, &self.palette_registry, false)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{}", e)))?;

        let (image, render_warnings) = crate::renderer::render_resolved(&resolved);

        let mut warnings: Vec<String> = resolved
            .warnings
            .iter()
            .map(|w| w.message.clone())
            .collect();
        for w in render_warnings {
            warnings.push(w.message);
        }

        Ok(RenderResult {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw(),
            warnings,
        })
    }

    /// Render a sprite by name and return PNG image bytes.
    ///
    /// Raises `ValueError` if the sprite is not found.
    fn render_to_png<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyBytes>> {
        let resolved = self.sprite_registry.resolve(name, &self.palette_registry, false)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{}", e)))?;

        let (image, _) = crate::renderer::render_resolved(&resolved);

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
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!("PNG encoding failed: {}", e))
                })?;
        }

        Ok(PyBytes::new(py, &png_data))
    }

    /// Render all sprites and return a dict mapping sprite name to PNG bytes.
    fn render_all<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        let names: Vec<String> = self.sprite_registry.names().cloned().collect();

        for name in &names {
            let resolved = self.sprite_registry.resolve(name, &self.palette_registry, false)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{}", e)))?;

            let (image, _) = crate::renderer::render_resolved(&resolved);

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
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "PNG encoding failed for '{}': {}",
                            name, e
                        ))
                    })?;
            }

            dict.set_item(name, PyBytes::new(py, &png_data))?;
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_DOT: &str = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;

    const PALETTE_AND_SPRITE: &str = r##"{"type": "palette", "name": "mono", "colors": {"_": "#00000000", "on": "#FFFFFF", "off": "#000000"}}
{"type": "sprite", "name": "checker", "size": [4, 4], "palette": "mono", "regions": {"on": {"points": [[0, 0], [2, 0], [1, 1], [3, 1], [0, 2], [2, 2], [1, 3], [3, 3]], "z": 0}, "off": {"points": [[1, 0], [3, 0], [0, 1], [2, 1], [1, 2], [3, 2], [0, 3], [2, 3]], "z": 0}}}"##;

    #[test]
    fn test_new_registry_empty() {
        let reg = PyRegistry::new();
        assert!(reg.sprites().is_empty());
        assert!(reg.palettes().is_empty());
    }

    #[test]
    fn test_load_sprite() {
        let mut reg = PyRegistry::new();
        reg.load(MINIMAL_DOT);
        assert_eq!(reg.sprites(), vec!["dot"]);
        assert!(reg.palettes().is_empty());
    }

    #[test]
    fn test_load_palette_and_sprite() {
        let mut reg = PyRegistry::new();
        reg.load(PALETTE_AND_SPRITE);
        assert_eq!(reg.sprites(), vec!["checker"]);
        assert_eq!(reg.palettes(), vec!["mono"]);
    }

    #[test]
    fn test_incremental_load() {
        let mut reg = PyRegistry::new();
        reg.load(r##"{"type": "palette", "name": "pal", "colors": {"x": "#FF0000"}}"##);
        assert_eq!(reg.palettes(), vec!["pal"]);
        assert!(reg.sprites().is_empty());

        reg.load(MINIMAL_DOT);
        assert_eq!(reg.sprites(), vec!["dot"]);
        assert_eq!(reg.palettes(), vec!["pal"]);
    }

    #[test]
    fn test_load_file_not_found() {
        let mut reg = PyRegistry::new();
        let result = reg.load_file("/nonexistent/path.pxl");
        // Can't test PyResult without Python, but verify it doesn't panic
        assert!(result.is_err());
    }

    #[test]
    fn test_sprites_sorted() {
        let mut reg = PyRegistry::new();
        reg.load(r##"{"type": "sprite", "name": "zeta", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}
{"type": "sprite", "name": "alpha", "size": [1, 1], "palette": {"x": "#00FF00"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##);
        assert_eq!(reg.sprites(), vec!["alpha", "zeta"]);
    }

    #[test]
    fn test_palettes_sorted() {
        let mut reg = PyRegistry::new();
        reg.load(r##"{"type": "palette", "name": "warm", "colors": {"x": "#FF0000"}}
{"type": "palette", "name": "cool", "colors": {"x": "#0000FF"}}"##);
        assert_eq!(reg.palettes(), vec!["cool", "warm"]);
    }
}
