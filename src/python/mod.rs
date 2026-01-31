//! Python bindings for the pixelsrc API via PyO3.
//!
//! This module exposes pixelsrc functionality to Python through a native
//! extension module. The module is compiled as `pixelsrc._native` and
//! re-exported through `python/pixelsrc/__init__.py`.

use pyo3::prelude::*;

pub mod parse;

/// Native pixelsrc Python module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(parse::parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse::list_sprites, m)?)?;
    m.add_function(wrap_pyfunction!(parse::list_palettes, m)?)?;
    Ok(())
}
