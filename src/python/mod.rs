//! Python bindings for the pixelsrc API via PyO3.
//!
//! This module exposes pixelsrc functionality to Python through a native
//! extension module. The module is compiled as `pixelsrc._native` and
//! re-exported through `python/pixelsrc/__init__.py`.

use pyo3::prelude::*;

/// Native pixelsrc Python module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
