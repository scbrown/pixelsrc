//! Python bindings for PNG import functionality.

use pyo3::prelude::*;
use std::path::Path;

use super::types::ImportResult;
use crate::import::{self, DitherHandling, ImportOptions};

/// Import a PNG file and convert it to Pixelsrc format.
///
/// Args:
///     path: Path to the PNG file.
///     name: Sprite name (defaults to filename stem).
///     max_colors: Maximum palette size (defaults to 16).
///
/// Returns:
///     ImportResult with palette, grid, and region data.
#[pyfunction]
#[pyo3(signature = (path, name=None, max_colors=None))]
pub fn import_png(
    path: &str,
    name: Option<&str>,
    max_colors: Option<usize>,
) -> PyResult<ImportResult> {
    let file_path = Path::new(path);
    let sprite_name =
        name.unwrap_or_else(|| file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("sprite"));
    let colors = max_colors.unwrap_or(16);

    let result = import::import_png(file_path, sprite_name, colors)
        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e))?;

    Ok(ImportResult::new(result))
}

/// Import a PNG file with full analysis options.
///
/// Args:
///     path: Path to the PNG file.
///     name: Sprite name (defaults to filename stem).
///     max_colors: Maximum palette size (defaults to 16).
///     confidence: Confidence threshold for analysis inferences (0.0-1.0, defaults to 0.5).
///     hints: Generate token naming hints (defaults to false).
///     shapes: Extract structured regions instead of raw points (defaults to false).
///     detect_upscale: Detect if image is upscaled pixel art (defaults to false).
///     detect_outlines: Detect outline/stroke regions (defaults to false).
///     dither_handling: How to handle dither patterns: "keep", "merge", or "analyze"
///                      (defaults to "keep").
///
/// Returns:
///     ImportResult with palette, grid, region data, and analysis results.
#[pyfunction]
#[pyo3(signature = (path, name=None, max_colors=None, confidence=None, hints=None, shapes=None, detect_upscale=None, detect_outlines=None, dither_handling=None))]
pub fn import_png_analyzed(
    path: &str,
    name: Option<&str>,
    max_colors: Option<usize>,
    confidence: Option<f64>,
    hints: Option<bool>,
    shapes: Option<bool>,
    detect_upscale: Option<bool>,
    detect_outlines: Option<bool>,
    dither_handling: Option<&str>,
) -> PyResult<ImportResult> {
    let file_path = Path::new(path);
    let sprite_name =
        name.unwrap_or_else(|| file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("sprite"));
    let colors = max_colors.unwrap_or(16);

    let dither = match dither_handling {
        Some("merge") => DitherHandling::Merge,
        Some("analyze") => DitherHandling::Analyze,
        Some("keep") | None => DitherHandling::Keep,
        Some(other) => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "invalid dither_handling: '{}' (expected 'keep', 'merge', or 'analyze')",
                other
            )));
        }
    };

    let options = ImportOptions {
        analyze: true,
        confidence_threshold: confidence.unwrap_or(0.5),
        hints: hints.unwrap_or(false),
        extract_shapes: shapes.unwrap_or(false),
        half_sprite: false,
        dither_handling: dither,
        detect_upscale: detect_upscale.unwrap_or(false),
        detect_outlines: detect_outlines.unwrap_or(false),
    };

    let result = import::import_png_with_options(file_path, sprite_name, colors, &options)
        .map_err(|e| pyo3::exceptions::PyOSError::new_err(e))?;

    Ok(ImportResult::new(result))
}
