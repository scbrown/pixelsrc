//! Color utilities for the Python API.
//!
//! Exposes CSS color parsing and ramp generation for procedural palette work:
//! - `parse_color()` -- parse any CSS color string to `#rrggbb` hex
//! - `generate_ramp()` -- interpolate between two colors in N steps

use pyo3::prelude::*;

use crate::color;

/// Parse a CSS color string and return its `#rrggbb` (or `#rrggbbaa`) hex
/// representation.
///
/// Accepts any format supported by the pixelsrc color parser: hex (`#f00`,
/// `#ff0000`), functional (`rgb()`, `hsl()`, `hwb()`, `oklch()`), named
/// (`red`, `blue`, `transparent`), and `color-mix()`.
///
/// Returns a lowercase hex string. Colors with full opacity use the 6-digit
/// form; colors with non-255 alpha use the 8-digit form.
#[pyfunction]
pub fn parse_color(color_str: &str) -> PyResult<String> {
    let rgba = color::parse_color(color_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(rgba_to_hex(rgba.0[0], rgba.0[1], rgba.0[2], rgba.0[3]))
}

/// Generate a color ramp by interpolating between two colors.
///
/// Returns a list of hex color strings (length == `steps`). The first element
/// is `from_color` and the last is `to_color`, with intermediate colors evenly
/// spaced in sRGB.
///
/// Both endpoints are parsed with the full CSS color parser, so any supported
/// format works (hex, named, `rgb()`, etc.).
///
/// Raises `ValueError` if either color is invalid or `steps` is zero.
#[pyfunction]
pub fn generate_ramp(from_color: &str, to_color: &str, steps: usize) -> PyResult<Vec<String>> {
    if steps == 0 {
        return Err(pyo3::exceptions::PyValueError::new_err("steps must be at least 1"));
    }

    let from = color::parse_color(from_color)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("from_color: {e}")))?;
    let to = color::parse_color(to_color)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("to_color: {e}")))?;

    if steps == 1 {
        return Ok(vec![rgba_to_hex(from.0[0], from.0[1], from.0[2], from.0[3])]);
    }

    let mut result = Vec::with_capacity(steps);
    let divisor = (steps - 1) as f64;

    for i in 0..steps {
        let t = i as f64 / divisor;
        let r = lerp_u8(from.0[0], to.0[0], t);
        let g = lerp_u8(from.0[1], to.0[1], t);
        let b = lerp_u8(from.0[2], to.0[2], t);
        let a = lerp_u8(from.0[3], to.0[3], t);
        result.push(rgba_to_hex(r, g, b, a));
    }

    Ok(result)
}

/// Linearly interpolate between two `u8` values.
fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    let v = a as f64 + (b as f64 - a as f64) * t;
    v.round() as u8
}

/// Format RGBA components as a lowercase hex string.
///
/// Uses `#rrggbb` when alpha is 255, `#rrggbbaa` otherwise.
fn rgba_to_hex(r: u8, g: u8, b: u8, a: u8) -> String {
    if a == 255 {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red").unwrap(), "#ff0000");
        assert_eq!(parse_color("blue").unwrap(), "#0000ff");
        assert_eq!(parse_color("white").unwrap(), "#ffffff");
        assert_eq!(parse_color("black").unwrap(), "#000000");
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#FF0000").unwrap(), "#ff0000");
        assert_eq!(parse_color("#F00").unwrap(), "#ff0000");
        assert_eq!(parse_color("#00ff0080").unwrap(), "#00ff0080");
    }

    #[test]
    fn test_parse_color_functional() {
        assert_eq!(parse_color("rgb(255, 0, 0)").unwrap(), "#ff0000");
        assert_eq!(parse_color("hsl(0, 100%, 50%)").unwrap(), "#ff0000");
    }

    #[test]
    fn test_parse_color_transparent() {
        assert_eq!(parse_color("transparent").unwrap(), "#00000000");
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("notacolor").is_err());
        assert!(parse_color("").is_err());
    }

    #[test]
    fn test_generate_ramp_basic() {
        let ramp = generate_ramp("#000000", "#ffffff", 5).unwrap();
        assert_eq!(ramp.len(), 5);
        assert_eq!(ramp[0], "#000000");
        assert_eq!(ramp[4], "#ffffff");
    }

    #[test]
    fn test_generate_ramp_single_step() {
        let ramp = generate_ramp("#ff0000", "#0000ff", 1).unwrap();
        assert_eq!(ramp.len(), 1);
        assert_eq!(ramp[0], "#ff0000");
    }

    #[test]
    fn test_generate_ramp_two_steps() {
        let ramp = generate_ramp("#000000", "#ffffff", 2).unwrap();
        assert_eq!(ramp.len(), 2);
        assert_eq!(ramp[0], "#000000");
        assert_eq!(ramp[1], "#ffffff");
    }

    #[test]
    fn test_generate_ramp_midpoint() {
        let ramp = generate_ramp("#000000", "#ffffff", 3).unwrap();
        assert_eq!(ramp.len(), 3);
        assert_eq!(ramp[0], "#000000");
        // Midpoint of 0 and 255 is 128
        assert_eq!(ramp[1], "#808080");
        assert_eq!(ramp[2], "#ffffff");
    }

    #[test]
    fn test_generate_ramp_zero_steps() {
        assert!(generate_ramp("#000000", "#ffffff", 0).is_err());
    }

    #[test]
    fn test_generate_ramp_invalid_color() {
        assert!(generate_ramp("notacolor", "#ffffff", 3).is_err());
        assert!(generate_ramp("#000000", "notacolor", 3).is_err());
    }

    #[test]
    fn test_generate_ramp_named_colors() {
        let ramp = generate_ramp("black", "white", 3).unwrap();
        assert_eq!(ramp.len(), 3);
        assert_eq!(ramp[0], "#000000");
        assert_eq!(ramp[2], "#ffffff");
    }

    #[test]
    fn test_generate_ramp_with_alpha() {
        let ramp = generate_ramp("#ff000000", "#ff0000ff", 3).unwrap();
        assert_eq!(ramp.len(), 3);
        assert_eq!(ramp[0], "#ff000000");
        // Midpoint alpha: 128
        assert_eq!(ramp[1], "#ff000080");
        assert_eq!(ramp[2], "#ff0000"); // alpha 255 -> 6-digit form
    }

    #[test]
    fn test_rgba_to_hex_opaque() {
        assert_eq!(rgba_to_hex(255, 0, 0, 255), "#ff0000");
        assert_eq!(rgba_to_hex(0, 255, 0, 255), "#00ff00");
    }

    #[test]
    fn test_rgba_to_hex_with_alpha() {
        assert_eq!(rgba_to_hex(255, 0, 0, 128), "#ff000080");
        assert_eq!(rgba_to_hex(0, 0, 0, 0), "#00000000");
    }
}
