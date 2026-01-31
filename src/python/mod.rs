//! Python bindings for the pixelsrc API via PyO3.
//!
//! This module exposes pixelsrc functionality to Python through a native
//! extension module. The module is compiled as `pixelsrc._native` and
//! re-exported through `python/pixelsrc/__init__.py`.

use pyo3::prelude::*;

mod color;
mod import;
pub mod parse;
mod render;
mod types;
mod validate;

/// Native pixelsrc Python module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<types::RenderResult>()?;
    m.add_class::<types::ImportResult>()?;
    m.add_function(wrap_pyfunction!(import::import_png, m)?)?;
    m.add_function(wrap_pyfunction!(import::import_png_analyzed, m)?)?;
    m.add_function(wrap_pyfunction!(render::render_to_png, m)?)?;
    m.add_function(wrap_pyfunction!(render::render_to_rgba, m)?)?;
    m.add_function(wrap_pyfunction!(parse::parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse::list_sprites, m)?)?;
    m.add_function(wrap_pyfunction!(parse::list_palettes, m)?)?;
    m.add_function(wrap_pyfunction!(validate::validate, m)?)?;
    m.add_function(wrap_pyfunction!(validate::validate_file, m)?)?;
    m.add_function(wrap_pyfunction!(color::parse_color, m)?)?;
    m.add_function(wrap_pyfunction!(color::generate_ramp, m)?)?;
    Ok(())
}
