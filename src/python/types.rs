//! Python-visible result types for pixelsrc bindings.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

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
