//! Python-visible result types for pixelsrc bindings.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;

use crate::import;

/// Result of rendering a sprite to RGBA pixels.
#[pyclass]
#[derive(Clone)]
pub struct RenderResult {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
    pub(crate) warnings: Vec<String>,
}

#[pymethods]
impl RenderResult {
    /// Width of the rendered image in pixels.
    #[getter]
    fn width(&self) -> u32 {
        self.width
    }

    /// Height of the rendered image in pixels.
    #[getter]
    fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGBA pixel data as bytes (4 bytes per pixel).
    #[getter]
    fn pixels<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.pixels)
    }

    /// Any warnings generated during rendering.
    #[getter]
    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// Result of importing a PNG image into Pixelsrc format.
#[pyclass]
#[derive(Clone)]
pub struct ImportResult {
    pub(crate) inner: import::ImportResult,
}

impl ImportResult {
    pub fn new(inner: import::ImportResult) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl ImportResult {
    /// Name of the imported sprite.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Width of the imported image in pixels.
    #[getter]
    fn width(&self) -> u32 {
        self.inner.width
    }

    /// Height of the imported image in pixels.
    #[getter]
    fn height(&self) -> u32 {
        self.inner.height
    }

    /// Color palette mapping token names to hex color strings.
    #[getter]
    fn palette(&self) -> HashMap<String, String> {
        self.inner.palette.clone()
    }

    /// Analysis results as a dict, or None if analysis was not enabled.
    ///
    /// Contains: roles, relationships, symmetry, naming_hints, z_order,
    /// dither_patterns, upscale_info, outlines.
    #[getter]
    fn analysis<'py>(&self, py: Python<'py>) -> PyResult<Option<PyObject>> {
        let analysis = match &self.inner.analysis {
            Some(a) => a,
            None => return Ok(None),
        };

        let dict = PyDict::new(py);

        // roles: {token: role_string}
        let roles = PyDict::new(py);
        for (token, role) in &analysis.roles {
            roles.set_item(token, role.to_string())?;
        }
        dict.set_item("roles", roles)?;

        // relationships: [(source, type, target), ...]
        let rels: Vec<(String, String, String)> = analysis
            .relationships
            .iter()
            .map(|(src, rel, tgt)| {
                let rel_str = match rel {
                    crate::models::RelationshipType::DerivesFrom => "derives-from",
                    crate::models::RelationshipType::ContainedWithin => "contained-within",
                    crate::models::RelationshipType::AdjacentTo => "adjacent-to",
                    crate::models::RelationshipType::PairedWith => "paired-with",
                };
                (src.clone(), rel_str.to_string(), tgt.clone())
            })
            .collect();
        dict.set_item("relationships", rels)?;

        // symmetry: "x" | "y" | "xy" | None
        let sym = analysis.symmetry.map(|s| match s {
            crate::analyze::Symmetric::X => "x",
            crate::analyze::Symmetric::Y => "y",
            crate::analyze::Symmetric::XY => "xy",
        });
        dict.set_item("symmetry", sym)?;

        // naming_hints: [{token, suggested_name, reason}, ...]
        let hints: Vec<HashMap<String, String>> = analysis
            .naming_hints
            .iter()
            .map(|h| {
                let mut m = HashMap::new();
                m.insert("token".to_string(), h.token.clone());
                m.insert("suggested_name".to_string(), h.suggested_name.clone());
                m.insert("reason".to_string(), h.reason.clone());
                m
            })
            .collect();
        dict.set_item("naming_hints", hints)?;

        // z_order: {token: z_level}
        dict.set_item("z_order", &analysis.z_order)?;

        // dither_patterns
        let dithers: Vec<HashMap<String, PyObject>> = analysis
            .dither_patterns
            .iter()
            .map(|d| {
                let mut m = HashMap::new();
                m.insert("tokens".to_string(), d.tokens.clone().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("pattern".to_string(), d.pattern.to_string().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("bounds".to_string(), d.bounds.to_vec().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("merged_color".to_string(), d.merged_color.clone().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("confidence".to_string(), d.confidence.into_pyobject(py).unwrap().into_any().unbind());
                m
            })
            .collect();
        dict.set_item("dither_patterns", dithers)?;

        // upscale_info
        let upscale = match &analysis.upscale_info {
            Some(u) => {
                let ud = PyDict::new(py);
                ud.set_item("scale", u.scale)?;
                ud.set_item("native_size", u.native_size.to_vec())?;
                ud.set_item("confidence", u.confidence)?;
                Some(ud)
            }
            None => None,
        };
        dict.set_item("upscale_info", upscale)?;

        // outlines
        let outlines: Vec<HashMap<String, PyObject>> = analysis
            .outlines
            .iter()
            .map(|o| {
                let mut m = HashMap::new();
                m.insert("token".to_string(), o.token.clone().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("borders".to_string(), o.borders.clone().into_pyobject(py).unwrap().into_any().unbind());
                m.insert("width".to_string(), o.width.into_pyobject(py).unwrap().into_any().unbind());
                m.insert("confidence".to_string(), o.confidence.into_pyobject(py).unwrap().into_any().unbind());
                m
            })
            .collect();
        dict.set_item("outlines", outlines)?;

        Ok(Some(dict.into_any().unbind()))
    }

    /// Convert to PXL grid format string (palette definition + grid rows).
    fn to_pxl(&self) -> String {
        let mut lines = Vec::new();

        // Palette line: palette <name>_palette
        lines.push(format!("palette {}_palette", self.inner.name));
        for (token, color) in &self.inner.palette {
            // Strip braces from token for the palette definition
            let token_name = token.trim_start_matches('{').trim_end_matches('}');
            lines.push(format!("  {} {}", token_name, color));
        }
        lines.push(String::new());

        // Sprite with grid
        lines.push(format!(
            "sprite {} {}x{} {}_palette",
            self.inner.name, self.inner.width, self.inner.height, self.inner.name
        ));
        for row in &self.inner.grid {
            lines.push(format!("  {}", row));
        }

        lines.join("\n")
    }

    /// Convert to JSONL format (palette line + sprite line).
    fn to_jsonl(&self) -> String {
        self.inner.to_jsonl()
    }
}
