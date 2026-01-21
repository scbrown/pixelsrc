//! Color parsing utilities for CSS color strings
//!
//! Supports the following formats:
//! - Hex: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`
//! - Functional: `rgb()`, `rgba()`, `hsl()`, `hsla()`, `hwb()`, `oklch()`
//! - Named: `red`, `blue`, `transparent`, etc.

use image::Rgba;
use lightningcss::traits::Parse;
use lightningcss::values::color::CssColor;
use thiserror::Error;

/// Error type for color parsing failures
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ColorError {
    /// Input string was empty
    #[error("empty color string")]
    Empty,
    /// Input string doesn't start with '#'
    #[error("color must start with '#'")]
    MissingHash,
    /// Invalid length (must be 3, 4, 6, or 8 hex chars after #)
    #[error("invalid color length {0}, expected 3, 4, 6, or 8")]
    InvalidLength(usize),
    /// Contains non-hex characters
    #[error("invalid hex character '{0}'")]
    InvalidHex(char),
    /// CSS parsing error from lightningcss
    #[error("CSS parse error: {0}")]
    CssParse(String),
}

impl<T: std::fmt::Display> From<lightningcss::error::Error<T>> for ColorError {
    fn from(e: lightningcss::error::Error<T>) -> Self {
        ColorError::CssParse(e.to_string())
    }
}

/// Parse a CSS color string into an RGBA color.
///
/// # Supported Formats
///
/// ## Hex Colors
/// - `#RGB` - 3-digit hex, each digit is doubled (e.g., `#F00` -> red)
/// - `#RGBA` - 4-digit hex, each digit is doubled (e.g., `#F00F` -> red, opaque)
/// - `#RRGGBB` - 6-digit hex, alpha defaults to 255 (opaque)
/// - `#RRGGBBAA` - 8-digit hex, explicit alpha channel
///
/// ## Functional Notation
/// - `rgb(255, 0, 0)` or `rgb(100%, 0%, 0%)`
/// - `rgba(255, 0, 0, 0.5)` or `rgba(255 0 0 / 50%)`
/// - `hsl(0, 100%, 50%)` or `hsl(0deg 100% 50%)`
/// - `hsla(0, 100%, 50%, 0.5)`
/// - `hwb(0 0% 0%)` - hue, whiteness, blackness
/// - `oklch(0.628 0.258 29.23)` - OKLCH color space
///
/// ## Named Colors
/// - CSS named colors: `red`, `blue`, `green`, `transparent`, etc.
///
/// # Examples
///
/// ```
/// use pixelsrc::color::parse_color;
///
/// // Hex colors
/// let red = parse_color("#F00").unwrap();
/// assert_eq!(red, image::Rgba([255, 0, 0, 255]));
///
/// // RGB functional
/// let green = parse_color("rgb(0, 255, 0)").unwrap();
/// assert_eq!(green, image::Rgba([0, 255, 0, 255]));
///
/// // Named colors
/// let blue = parse_color("blue").unwrap();
/// assert_eq!(blue, image::Rgba([0, 0, 255, 255]));
///
/// // HSL
/// let red_hsl = parse_color("hsl(0, 100%, 50%)").unwrap();
/// assert_eq!(red_hsl, image::Rgba([255, 0, 0, 255]));
/// ```
///
/// # Errors
///
/// Returns `ColorError` if the input is invalid or unparseable.
pub fn parse_color(s: &str) -> Result<Rgba<u8>, ColorError> {
    if s.is_empty() {
        return Err(ColorError::Empty);
    }

    // Fast path for hex colors - use our optimized parser
    if s.starts_with('#') {
        return parse_hex_color(s);
    }

    // Use lightningcss for all other CSS color formats
    parse_css_color(s)
}

/// Parse a hex color string (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
fn parse_hex_color(s: &str) -> Result<Rgba<u8>, ColorError> {
    let hex = &s[1..];
    let len = hex.len();

    // Validate all characters are hex
    for c in hex.chars() {
        if !c.is_ascii_hexdigit() {
            return Err(ColorError::InvalidHex(c));
        }
    }

    match len {
        3 => {
            // #RGB -> #RRGGBB (doubled digits), alpha = 255
            let mut chars = hex.chars();
            let r = parse_hex_digit(chars.next().unwrap())? * 17;
            let g = parse_hex_digit(chars.next().unwrap())? * 17;
            let b = parse_hex_digit(chars.next().unwrap())? * 17;
            Ok(Rgba([r, g, b, 255]))
        }
        4 => {
            // #RGBA -> #RRGGBBAA (doubled digits)
            let mut chars = hex.chars();
            let r = parse_hex_digit(chars.next().unwrap())? * 17;
            let g = parse_hex_digit(chars.next().unwrap())? * 17;
            let b = parse_hex_digit(chars.next().unwrap())? * 17;
            let a = parse_hex_digit(chars.next().unwrap())? * 17;
            Ok(Rgba([r, g, b, a]))
        }
        6 => {
            // #RRGGBB, alpha = 255
            let r = parse_hex_pair(&hex[0..2])?;
            let g = parse_hex_pair(&hex[2..4])?;
            let b = parse_hex_pair(&hex[4..6])?;
            Ok(Rgba([r, g, b, 255]))
        }
        8 => {
            // #RRGGBBAA
            let r = parse_hex_pair(&hex[0..2])?;
            let g = parse_hex_pair(&hex[2..4])?;
            let b = parse_hex_pair(&hex[4..6])?;
            let a = parse_hex_pair(&hex[6..8])?;
            Ok(Rgba([r, g, b, a]))
        }
        _ => Err(ColorError::InvalidLength(len)),
    }
}

/// Parse a CSS color using lightningcss (rgb, hsl, hwb, oklch, named colors)
fn parse_css_color(s: &str) -> Result<Rgba<u8>, ColorError> {
    let css_color = CssColor::parse_string(s).map_err(|e| ColorError::CssParse(e.to_string()))?;
    css_color_to_rgba(css_color)
}

/// Convert a lightningcss CssColor to RGBA
fn css_color_to_rgba(color: CssColor) -> Result<Rgba<u8>, ColorError> {
    use lightningcss::values::color::FloatColor;

    // Convert to sRGB color space first, then extract RGBA
    let rgb_color = color
        .to_rgb()
        .map_err(|_| ColorError::CssParse("cannot convert color to RGB".to_string()))?;

    // Extract RGBA from the converted color
    match rgb_color {
        CssColor::RGBA(rgba) => Ok(Rgba([rgba.red, rgba.green, rgba.blue, rgba.alpha])),
        CssColor::Float(float_color) => {
            // Handle Float colors (when components have 'none' values)
            match float_color.as_ref() {
                FloatColor::RGB(rgb) => {
                    let r = (rgb.r * 255.0).round() as u8;
                    let g = (rgb.g * 255.0).round() as u8;
                    let b = (rgb.b * 255.0).round() as u8;
                    let a = (rgb.alpha * 255.0).round() as u8;
                    Ok(Rgba([r, g, b, a]))
                }
                _ => Err(ColorError::CssParse("unexpected float color format".to_string())),
            }
        }
        _ => Err(ColorError::CssParse("color conversion did not produce RGB".to_string())),
    }
}

/// Parse a single hex digit (0-9, A-F, a-f) to u8 (0-15)
fn parse_hex_digit(c: char) -> Result<u8, ColorError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => Err(ColorError::InvalidHex(c)),
    }
}

/// Parse a two-character hex string to u8 (0-255)
fn parse_hex_pair(s: &str) -> Result<u8, ColorError> {
    let mut chars = s.chars();
    let high = parse_hex_digit(chars.next().unwrap())?;
    let low = parse_hex_digit(chars.next().unwrap())?;
    Ok(high * 16 + low)
}

// ============================================================================
// HSL Color Space Utilities (for color ramps)
// ============================================================================

/// HSL color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// Hue in degrees (0-360)
    pub h: f64,
    /// Saturation as percentage (0-100)
    pub s: f64,
    /// Lightness as percentage (0-100)
    pub l: f64,
}

impl Hsl {
    /// Create a new HSL color
    pub fn new(h: f64, s: f64, l: f64) -> Self {
        Self { h, s, l }
    }

    /// Apply a shift to the HSL values, clamping to valid ranges
    pub fn shift(&self, hue_delta: f64, saturation_delta: f64, lightness_delta: f64) -> Self {
        Self {
            h: (self.h + hue_delta).rem_euclid(360.0),
            s: (self.s + saturation_delta).clamp(0.0, 100.0),
            l: (self.l + lightness_delta).clamp(0.0, 100.0),
        }
    }
}

/// Convert RGB (0-255 each) to HSL
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> Hsl {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Lightness
    let l = (max + min) / 2.0;

    if delta == 0.0 {
        // Achromatic (gray)
        return Hsl::new(0.0, 0.0, l * 100.0);
    }

    // Saturation
    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    // Hue
    let h = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };

    Hsl::new(h * 60.0, s * 100.0, l * 100.0)
}

/// Convert HSL to RGB (0-255 each)
pub fn hsl_to_rgb(hsl: &Hsl) -> (u8, u8, u8) {
    let h = hsl.h / 360.0;
    let s = hsl.s / 100.0;
    let l = hsl.l / 100.0;

    if s == 0.0 {
        // Achromatic (gray)
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// Helper function for HSL to RGB conversion
fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Apply a color shift and return the result as a hex string
pub fn apply_color_shift(
    base: &Rgba<u8>,
    hue_delta: f64,
    saturation_delta: f64,
    lightness_delta: f64,
) -> String {
    let hsl = rgb_to_hsl(base.0[0], base.0[1], base.0[2]);
    let shifted = hsl.shift(hue_delta, saturation_delta, lightness_delta);
    let (r, g, b) = hsl_to_rgb(&shifted);

    if base.0[3] == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, base.0[3])
    }
}

/// Generate a color ramp from a base color
///
/// Returns a vector of (token_suffix, hex_color) pairs.
/// For a ramp with 5 steps, generates:
/// - "_2" (darkest shadow)
/// - "_1" (shadow)
/// - "" (base)
/// - "+1" (highlight)
/// - "+2" (brightest)
pub fn generate_ramp(
    base_color: &str,
    steps: u32,
    shadow_shift: (f64, f64, f64),  // (hue, saturation, lightness)
    highlight_shift: (f64, f64, f64),
) -> Result<Vec<(String, String)>, ColorError> {
    let base_rgba = parse_color(base_color)?;
    let mut result = Vec::with_capacity(steps as usize);

    // Calculate how many steps on each side of base
    // For steps=5: 2 shadow, 1 base, 2 highlight
    // For steps=3: 1 shadow, 1 base, 1 highlight
    let shadow_steps = (steps - 1) / 2;
    let highlight_steps = steps - 1 - shadow_steps;

    // Generate shadow colors (from darkest to base)
    for i in (1..=shadow_steps).rev() {
        let factor = i as f64;
        let shifted = apply_color_shift(
            &base_rgba,
            shadow_shift.0 * factor,
            shadow_shift.1 * factor,
            shadow_shift.2 * factor,
        );
        result.push((format!("_{}", i), shifted));
    }

    // Base color
    result.push((String::new(), base_color.to_string()));

    // Generate highlight colors (from base to brightest)
    for i in 1..=highlight_steps {
        let factor = i as f64;
        let shifted = apply_color_shift(
            &base_rgba,
            highlight_shift.0 * factor,
            highlight_shift.1 * factor,
            highlight_shift.2 * factor,
        );
        result.push((format!("+{}", i), shifted));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rgb_short() {
        // #RGB -> doubled digits, alpha 255
        assert_eq!(parse_color("#F00").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#0F0").unwrap(), Rgba([0, 255, 0, 255]));
        assert_eq!(parse_color("#00F").unwrap(), Rgba([0, 0, 255, 255]));
        assert_eq!(parse_color("#FFF").unwrap(), Rgba([255, 255, 255, 255]));
        assert_eq!(parse_color("#000").unwrap(), Rgba([0, 0, 0, 255]));
        assert_eq!(parse_color("#ABC").unwrap(), Rgba([170, 187, 204, 255]));
    }

    #[test]
    fn test_parse_rgba_short() {
        // #RGBA -> doubled digits
        assert_eq!(parse_color("#F00F").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#F008").unwrap(), Rgba([255, 0, 0, 136]));
        assert_eq!(parse_color("#0000").unwrap(), Rgba([0, 0, 0, 0]));
        assert_eq!(parse_color("#FFFF").unwrap(), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_parse_rrggbb() {
        // #RRGGBB -> alpha 255
        assert_eq!(parse_color("#FF0000").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#00FF00").unwrap(), Rgba([0, 255, 0, 255]));
        assert_eq!(parse_color("#0000FF").unwrap(), Rgba([0, 0, 255, 255]));
        assert_eq!(parse_color("#FFFFFF").unwrap(), Rgba([255, 255, 255, 255]));
        assert_eq!(parse_color("#000000").unwrap(), Rgba([0, 0, 0, 255]));
        assert_eq!(parse_color("#AABBCC").unwrap(), Rgba([170, 187, 204, 255]));
    }

    #[test]
    fn test_parse_rrggbbaa() {
        // #RRGGBBAA -> explicit alpha
        assert_eq!(parse_color("#FF0000FF").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#FF000080").unwrap(), Rgba([255, 0, 0, 128]));
        assert_eq!(parse_color("#FF000000").unwrap(), Rgba([255, 0, 0, 0]));
        assert_eq!(parse_color("#00000000").unwrap(), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_case_insensitive() {
        // Should handle both upper and lower case
        assert_eq!(parse_color("#f00").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#ff0000").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("#aAbBcC").unwrap(), Rgba([170, 187, 204, 255]));
    }

    #[test]
    fn test_error_empty() {
        assert_eq!(parse_color(""), Err(ColorError::Empty));
    }

    #[test]
    fn test_error_invalid_format() {
        // These are not valid hex (no #) and not valid CSS color names
        assert!(parse_color("FF0000").is_err());
        assert!(parse_color("F00").is_err());
        assert!(parse_color("notacolor").is_err());
    }

    #[test]
    fn test_error_invalid_length() {
        assert_eq!(parse_color("#F"), Err(ColorError::InvalidLength(1)));
        assert_eq!(parse_color("#FF"), Err(ColorError::InvalidLength(2)));
        assert_eq!(parse_color("#FFFFF"), Err(ColorError::InvalidLength(5)));
        assert_eq!(parse_color("#FFFFFFF"), Err(ColorError::InvalidLength(7)));
        assert_eq!(parse_color("#FFFFFFFFF"), Err(ColorError::InvalidLength(9)));
    }

    #[test]
    fn test_error_invalid_hex() {
        assert_eq!(parse_color("#GGG"), Err(ColorError::InvalidHex('G')));
        assert_eq!(parse_color("#XYZ"), Err(ColorError::InvalidHex('X')));
        assert_eq!(parse_color("#12345G"), Err(ColorError::InvalidHex('G')));
        assert_eq!(parse_color("#not-a-color"), Err(ColorError::InvalidHex('n')));
    }

    // Tests matching the fixture file: tests/fixtures/valid/color_formats.jsonl
    // {"type": "sprite", "name": "color_test", "palette": {"{a}": "#F00", "{b}": "#FF0000", "{c}": "#FF0000FF", "{d}": "#F00F"}, "grid": ["{a}{b}", "{c}{d}"]}
    #[test]
    fn test_fixture_color_formats() {
        // {a}: #F00 -> red (short form)
        assert_eq!(parse_color("#F00").unwrap(), Rgba([255, 0, 0, 255]));

        // {b}: #FF0000 -> red (long form)
        assert_eq!(parse_color("#FF0000").unwrap(), Rgba([255, 0, 0, 255]));

        // {c}: #FF0000FF -> red with explicit full alpha
        assert_eq!(parse_color("#FF0000FF").unwrap(), Rgba([255, 0, 0, 255]));

        // {d}: #F00F -> red with alpha (short form)
        assert_eq!(parse_color("#F00F").unwrap(), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_css_parse_error_display() {
        let err = ColorError::CssParse("unexpected token".to_string());
        assert_eq!(err.to_string(), "CSS parse error: unexpected token");
    }

    #[test]
    fn test_css_parse_error_equality() {
        let err1 = ColorError::CssParse("test".to_string());
        let err2 = ColorError::CssParse("test".to_string());
        let err3 = ColorError::CssParse("different".to_string());
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    // CSS-12: color-mix() tests

    #[test]
    fn test_color_mix_oklch_basic() {
        // color-mix in oklch with 50/50 split (default)
        let result = parse_color("color-mix(in oklch, red, blue)");
        assert!(result.is_ok(), "color-mix(in oklch, red, blue) should parse");
        let color = result.unwrap();
        // Should be a purple-ish color (mixing red and blue)
        assert!(color.0[0] > 100, "Should have significant red component");
        assert!(color.0[2] > 100, "Should have significant blue component");
    }

    #[test]
    fn test_color_mix_oklch_percentages() {
        // 70% red, 30% blue
        let result = parse_color("color-mix(in oklch, red 70%, blue)");
        assert!(result.is_ok(), "color-mix with percentages should parse");
        let color = result.unwrap();
        // Should be more red than blue
        assert!(color.0[0] > color.0[2], "70% red should dominate");

        // 30% red, 70% blue
        let result2 = parse_color("color-mix(in oklch, red 30%, blue)");
        assert!(result2.is_ok());
        let color2 = result2.unwrap();
        // Should be more blue than red
        assert!(color2.0[2] > color2.0[0], "70% blue should dominate");
    }

    #[test]
    fn test_color_mix_srgb() {
        // color-mix in sRGB color space
        let result = parse_color("color-mix(in srgb, #ff0000 50%, #0000ff)");
        assert!(result.is_ok(), "color-mix(in srgb, ...) should parse");
        let color = result.unwrap();
        // In sRGB, 50/50 red and blue should give purple with equal R and B
        assert!(color.0[0] > 100, "Should have red component");
        assert!(color.0[2] > 100, "Should have blue component");
    }

    #[test]
    fn test_color_mix_hsl() {
        // color-mix in HSL color space
        let result = parse_color("color-mix(in hsl, red, blue)");
        assert!(result.is_ok(), "color-mix(in hsl, ...) should parse");
    }

    #[test]
    fn test_color_mix_with_named_colors() {
        let result = parse_color("color-mix(in oklch, coral, steelblue)");
        assert!(result.is_ok(), "color-mix with named colors should work");
    }

    #[test]
    fn test_color_mix_with_hex() {
        let result = parse_color("color-mix(in oklch, #ff6347, #4682b4)");
        assert!(result.is_ok(), "color-mix with hex colors should work");
    }

    #[test]
    fn test_color_mix_with_rgb_functional() {
        let result = parse_color("color-mix(in oklch, rgb(255, 0, 0), rgb(0, 0, 255))");
        assert!(result.is_ok(), "color-mix with rgb() should work");
    }

    #[test]
    fn test_color_mix_with_hsl_functional() {
        let result = parse_color("color-mix(in oklch, hsl(0, 100%, 50%), hsl(240, 100%, 50%))");
        assert!(result.is_ok(), "color-mix with hsl() should work");
    }

    #[test]
    fn test_color_mix_white_black() {
        // Mixing white and black should give gray
        let result = parse_color("color-mix(in oklch, white, black)");
        assert!(result.is_ok());
        let color = result.unwrap();
        // Should be grayish (R, G, B roughly equal)
        let diff_rg = (color.0[0] as i16 - color.0[1] as i16).abs();
        let diff_rb = (color.0[0] as i16 - color.0[2] as i16).abs();
        assert!(diff_rg < 30, "R and G should be similar for gray");
        assert!(diff_rb < 30, "R and B should be similar for gray");
    }

    #[test]
    fn test_color_mix_100_percent() {
        // 100% of one color should just give that color
        let result = parse_color("color-mix(in oklch, red 100%, blue)");
        assert!(result.is_ok());
        let color = result.unwrap();
        // Should be close to pure red
        assert!(color.0[0] > 250, "100% red should be pure red");
        assert!(color.0[2] < 10, "100% red should have no blue");
    }

    #[test]
    fn test_color_mix_0_percent() {
        // 0% of one color is effectively 100% of the other
        let result = parse_color("color-mix(in oklch, red 0%, blue)");
        assert!(result.is_ok());
        let color = result.unwrap();
        // Note: CSS color-mix with 0% may have different behavior across implementations
        // In oklch, the resulting color should be predominantly blue
        // Being lenient here to account for different interpretations
        assert!(color.0[2] > color.0[0], "0% red should be more blue than red: {:?}", color);
    }

    #[test]
    fn test_color_mix_with_alpha() {
        // color-mix should work with semi-transparent colors
        let result = parse_color("color-mix(in oklch, rgba(255, 0, 0, 0.5), rgba(0, 0, 255, 0.5))");
        assert!(result.is_ok(), "color-mix with alpha should work");
        let color = result.unwrap();
        // Alpha should be preserved/mixed
        assert!(color.0[3] < 255, "Mixed alpha should be less than 255");
    }

    #[test]
    fn test_color_mix_oklch_longer_hue() {
        // Test longer hue interpolation (goes the long way around the color wheel)
        let result = parse_color("color-mix(in oklch longer hue, red, blue)");
        assert!(result.is_ok(), "color-mix with longer hue should parse");
    }

    #[test]
    fn test_color_mix_oklch_shorter_hue() {
        // Test shorter hue interpolation (default, goes the short way)
        let result = parse_color("color-mix(in oklch shorter hue, red, blue)");
        assert!(result.is_ok(), "color-mix with shorter hue should parse");
    }

    // CSS Color Format Tests (CSS-3)

    #[test]
    fn test_parse_rgb_functional() {
        // rgb() with integer values
        assert_eq!(parse_color("rgb(255, 0, 0)").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("rgb(0, 255, 0)").unwrap(), Rgba([0, 255, 0, 255]));
        assert_eq!(parse_color("rgb(0, 0, 255)").unwrap(), Rgba([0, 0, 255, 255]));

        // rgb() with percentage values
        assert_eq!(parse_color("rgb(100%, 0%, 0%)").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("rgb(0%, 100%, 0%)").unwrap(), Rgba([0, 255, 0, 255]));

        // Modern space-separated syntax
        assert_eq!(parse_color("rgb(255 0 0)").unwrap(), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_parse_rgba_functional() {
        // rgba() with alpha
        assert_eq!(parse_color("rgba(255, 0, 0, 1)").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("rgba(255, 0, 0, 0.5)").unwrap(), Rgba([255, 0, 0, 128]));
        assert_eq!(parse_color("rgba(255, 0, 0, 0)").unwrap(), Rgba([255, 0, 0, 0]));

        // Modern syntax with /
        assert_eq!(parse_color("rgb(255 0 0 / 50%)").unwrap(), Rgba([255, 0, 0, 128]));
        assert_eq!(parse_color("rgb(255 0 0 / 0.5)").unwrap(), Rgba([255, 0, 0, 128]));
    }

    #[test]
    fn test_parse_hsl_functional() {
        // hsl() - pure red is 0deg, 100% saturation, 50% lightness
        assert_eq!(parse_color("hsl(0, 100%, 50%)").unwrap(), Rgba([255, 0, 0, 255]));
        // Pure green is 120deg
        assert_eq!(parse_color("hsl(120, 100%, 50%)").unwrap(), Rgba([0, 255, 0, 255]));
        // Pure blue is 240deg
        assert_eq!(parse_color("hsl(240, 100%, 50%)").unwrap(), Rgba([0, 0, 255, 255]));

        // Modern syntax with deg
        assert_eq!(parse_color("hsl(0deg 100% 50%)").unwrap(), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_parse_hsla_functional() {
        // hsla() with alpha
        assert_eq!(parse_color("hsla(0, 100%, 50%, 1)").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("hsla(0, 100%, 50%, 0.5)").unwrap(), Rgba([255, 0, 0, 128]));

        // Modern syntax
        assert_eq!(parse_color("hsl(0 100% 50% / 50%)").unwrap(), Rgba([255, 0, 0, 128]));
    }

    #[test]
    fn test_parse_hwb_functional() {
        // hwb(hue, whiteness, blackness)
        // Pure red: 0deg, 0% white, 0% black
        assert_eq!(parse_color("hwb(0 0% 0%)").unwrap(), Rgba([255, 0, 0, 255]));
        // Pure green: 120deg
        assert_eq!(parse_color("hwb(120 0% 0%)").unwrap(), Rgba([0, 255, 0, 255]));
        // Pure blue: 240deg
        assert_eq!(parse_color("hwb(240 0% 0%)").unwrap(), Rgba([0, 0, 255, 255]));
        // White: any hue, 100% white
        assert_eq!(parse_color("hwb(0 100% 0%)").unwrap(), Rgba([255, 255, 255, 255]));
        // Black: any hue, 100% black
        assert_eq!(parse_color("hwb(0 0% 100%)").unwrap(), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_parse_oklch_functional() {
        // oklch(lightness, chroma, hue)
        // Note: oklch values may have slight rounding differences
        let red = parse_color("oklch(0.628 0.258 29.23)").unwrap();
        // Should be close to red - allow some tolerance for color space conversion
        assert!(red.0[0] > 250); // R close to 255
        assert!(red.0[1] < 10); // G close to 0
        assert!(red.0[2] < 10); // B close to 0
    }

    #[test]
    fn test_parse_named_colors() {
        // Basic named colors
        assert_eq!(parse_color("red").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("green").unwrap(), Rgba([0, 128, 0, 255])); // CSS green is #008000
        assert_eq!(parse_color("blue").unwrap(), Rgba([0, 0, 255, 255]));
        assert_eq!(parse_color("white").unwrap(), Rgba([255, 255, 255, 255]));
        assert_eq!(parse_color("black").unwrap(), Rgba([0, 0, 0, 255]));

        // Transparent
        assert_eq!(parse_color("transparent").unwrap(), Rgba([0, 0, 0, 0]));

        // Extended named colors
        assert_eq!(parse_color("coral").unwrap(), Rgba([255, 127, 80, 255]));
        assert_eq!(parse_color("hotpink").unwrap(), Rgba([255, 105, 180, 255]));
        assert_eq!(parse_color("steelblue").unwrap(), Rgba([70, 130, 180, 255]));
    }

    #[test]
    fn test_named_colors_case_insensitive() {
        assert_eq!(parse_color("Red").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("RED").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("HotPink").unwrap(), Rgba([255, 105, 180, 255]));
    }

    // CSS-12: color-mix() function tests
    #[test]
    fn test_color_mix_basic() {
        // color-mix in srgb: 50% red + 50% blue = purple
        let purple = parse_color("color-mix(in srgb, red 50%, blue)").unwrap();
        // Should be approximately (128, 0, 128) - purple
        assert!(purple.0[0] > 100 && purple.0[0] < 150); // R around 128
        assert!(purple.0[1] < 20); // G close to 0
        assert!(purple.0[2] > 100 && purple.0[2] < 150); // B around 128
        assert_eq!(purple.0[3], 255); // Full alpha
    }

    #[test]
    fn test_color_mix_percentages() {
        // 70% red + 30% black = darker red
        let dark_red = parse_color("color-mix(in srgb, red 70%, black)").unwrap();
        // Should be approximately (179, 0, 0) - 70% of 255
        assert!(dark_red.0[0] > 150 && dark_red.0[0] < 200);
        assert!(dark_red.0[1] < 10);
        assert!(dark_red.0[2] < 10);
    }

    #[test]
    fn test_color_mix_oklch() {
        // color-mix in oklch for perceptually uniform blending
        let result = parse_color("color-mix(in oklch, #FF0000 70%, black)").unwrap();
        // Should be a darker shade of red
        assert!(result.0[0] > 100); // Still has red
        assert!(result.0[0] < 255); // But darker than pure red
    }

    #[test]
    fn test_color_mix_white_black_srgb() {
        // 50% white + 50% black = gray (srgb space)
        let gray = parse_color("color-mix(in srgb, white, black)").unwrap();
        // Should be approximately (128, 128, 128)
        assert!(gray.0[0] > 100 && gray.0[0] < 150);
        assert!(gray.0[1] > 100 && gray.0[1] < 150);
        assert!(gray.0[2] > 100 && gray.0[2] < 150);
    }

    #[test]
    fn test_color_mix_highlight_generation() {
        // Common pattern: lighten a color for highlight
        // 70% original + 30% white
        let highlight = parse_color("color-mix(in srgb, #3366CC 70%, white)").unwrap();
        // Should be lighter than original
        assert!(highlight.0[0] > 51); // Original R was 51
        assert!(highlight.0[1] > 102); // Original G was 102
        assert!(highlight.0[2] > 204); // Original B was 204
    }

    #[test]
    fn test_color_mix_shadow_generation() {
        // Common pattern: darken a color for shadow
        // 70% original + 30% black (oklch for perceptual uniformity)
        let shadow = parse_color("color-mix(in oklch, #FFCC99 70%, black)").unwrap();
        // Should be darker than original #FFCC99 (255, 204, 153)
        assert!(shadow.0[0] < 255);
        assert!(shadow.0[1] < 204);
        assert!(shadow.0[2] < 153);
    }

    // ========================================================================
    // HSL Conversion Tests
    // ========================================================================

    #[test]
    fn test_rgb_to_hsl_red() {
        let hsl = rgb_to_hsl(255, 0, 0);
        assert!((hsl.h - 0.0).abs() < 1.0, "Red hue should be ~0");
        assert!((hsl.s - 100.0).abs() < 1.0, "Red saturation should be ~100");
        assert!((hsl.l - 50.0).abs() < 1.0, "Red lightness should be ~50");
    }

    #[test]
    fn test_rgb_to_hsl_green() {
        let hsl = rgb_to_hsl(0, 255, 0);
        assert!((hsl.h - 120.0).abs() < 1.0, "Green hue should be ~120");
        assert!((hsl.s - 100.0).abs() < 1.0, "Green saturation should be ~100");
        assert!((hsl.l - 50.0).abs() < 1.0, "Green lightness should be ~50");
    }

    #[test]
    fn test_rgb_to_hsl_blue() {
        let hsl = rgb_to_hsl(0, 0, 255);
        assert!((hsl.h - 240.0).abs() < 1.0, "Blue hue should be ~240");
        assert!((hsl.s - 100.0).abs() < 1.0, "Blue saturation should be ~100");
        assert!((hsl.l - 50.0).abs() < 1.0, "Blue lightness should be ~50");
    }

    #[test]
    fn test_rgb_to_hsl_white() {
        let hsl = rgb_to_hsl(255, 255, 255);
        assert!((hsl.s - 0.0).abs() < 1.0, "White saturation should be ~0");
        assert!((hsl.l - 100.0).abs() < 1.0, "White lightness should be ~100");
    }

    #[test]
    fn test_rgb_to_hsl_black() {
        let hsl = rgb_to_hsl(0, 0, 0);
        assert!((hsl.s - 0.0).abs() < 1.0, "Black saturation should be ~0");
        assert!((hsl.l - 0.0).abs() < 1.0, "Black lightness should be ~0");
    }

    #[test]
    fn test_hsl_to_rgb_red() {
        let hsl = Hsl::new(0.0, 100.0, 50.0);
        let (r, g, b) = hsl_to_rgb(&hsl);
        assert_eq!(r, 255, "Red R should be 255");
        assert_eq!(g, 0, "Red G should be 0");
        assert_eq!(b, 0, "Red B should be 0");
    }

    #[test]
    fn test_hsl_to_rgb_gray() {
        let hsl = Hsl::new(0.0, 0.0, 50.0);
        let (r, g, b) = hsl_to_rgb(&hsl);
        assert_eq!(r, 128, "Gray R should be 128");
        assert_eq!(g, 128, "Gray G should be 128");
        assert_eq!(b, 128, "Gray B should be 128");
    }

    #[test]
    fn test_hsl_roundtrip() {
        // Test that RGB -> HSL -> RGB roundtrips correctly
        let test_colors = [
            (255, 128, 64),
            (100, 150, 200),
            (50, 50, 50),
            (200, 100, 100),
        ];

        for (r, g, b) in test_colors {
            let hsl = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(&hsl);
            assert!((r as i16 - r2 as i16).abs() <= 1, "R roundtrip failed for ({}, {}, {})", r, g, b);
            assert!((g as i16 - g2 as i16).abs() <= 1, "G roundtrip failed for ({}, {}, {})", r, g, b);
            assert!((b as i16 - b2 as i16).abs() <= 1, "B roundtrip failed for ({}, {}, {})", r, g, b);
        }
    }

    #[test]
    fn test_hsl_shift() {
        let hsl = Hsl::new(180.0, 50.0, 50.0);
        let shifted = hsl.shift(30.0, 10.0, -15.0);
        assert!((shifted.h - 210.0).abs() < 0.01);
        assert!((shifted.s - 60.0).abs() < 0.01);
        assert!((shifted.l - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_hsl_shift_wraps_hue() {
        let hsl = Hsl::new(350.0, 50.0, 50.0);
        let shifted = hsl.shift(20.0, 0.0, 0.0);
        assert!((shifted.h - 10.0).abs() < 0.01, "Hue should wrap around 360");
    }

    #[test]
    fn test_hsl_shift_clamps() {
        let hsl = Hsl::new(180.0, 90.0, 90.0);
        let shifted = hsl.shift(0.0, 20.0, 20.0);
        assert!((shifted.s - 100.0).abs() < 0.01, "Saturation should clamp at 100");
        assert!((shifted.l - 100.0).abs() < 0.01, "Lightness should clamp at 100");
    }

    // ========================================================================
    // Color Ramp Generation Tests
    // ========================================================================

    #[test]
    fn test_generate_ramp_basic() {
        let result = generate_ramp(
            "#FF0000",
            3,
            (0.0, 0.0, -20.0),  // shadow: just darker
            (0.0, 0.0, 20.0),   // highlight: just lighter
        ).unwrap();

        assert_eq!(result.len(), 3, "3-step ramp should have 3 colors");
        assert_eq!(result[0].0, "_1", "First suffix should be _1");
        assert_eq!(result[1].0, "", "Middle suffix should be empty (base)");
        assert_eq!(result[2].0, "+1", "Last suffix should be +1");
        assert_eq!(result[1].1, "#FF0000", "Base color should be unchanged");
    }

    #[test]
    fn test_generate_ramp_5_steps() {
        let result = generate_ramp(
            "#808080",
            5,
            (0.0, 0.0, -15.0),
            (0.0, 0.0, 15.0),
        ).unwrap();

        assert_eq!(result.len(), 5, "5-step ramp should have 5 colors");
        assert_eq!(result[0].0, "_2");
        assert_eq!(result[1].0, "_1");
        assert_eq!(result[2].0, "");
        assert_eq!(result[3].0, "+1");
        assert_eq!(result[4].0, "+2");
    }

    #[test]
    fn test_generate_ramp_with_hue_shift() {
        let result = generate_ramp(
            "#FF8080",  // Light red
            3,
            (15.0, 0.0, -15.0),  // Shadow shifts toward orange
            (-10.0, 0.0, 15.0), // Highlight shifts toward pink
        ).unwrap();

        assert_eq!(result.len(), 3);

        // Verify shadow has different hue from base
        let shadow = parse_color(&result[0].1).unwrap();
        let base = parse_color(&result[1].1).unwrap();
        let highlight = parse_color(&result[2].1).unwrap();

        // Shadow should be darker
        let shadow_hsl = rgb_to_hsl(shadow.0[0], shadow.0[1], shadow.0[2]);
        let base_hsl = rgb_to_hsl(base.0[0], base.0[1], base.0[2]);
        let highlight_hsl = rgb_to_hsl(highlight.0[0], highlight.0[1], highlight.0[2]);

        assert!(shadow_hsl.l < base_hsl.l, "Shadow should be darker");
        assert!(highlight_hsl.l > base_hsl.l, "Highlight should be lighter");
    }

    #[test]
    fn test_generate_ramp_invalid_color() {
        let result = generate_ramp(
            "not_a_color",
            3,
            (0.0, 0.0, -15.0),
            (0.0, 0.0, 15.0),
        );
        assert!(result.is_err(), "Invalid color should return error");
    }

    #[test]
    fn test_apply_color_shift() {
        let red = Rgba([255, 0, 0, 255]);
        let shifted = apply_color_shift(&red, 0.0, 0.0, -20.0);

        // Parse the shifted color and verify it's darker
        let shifted_rgba = parse_color(&shifted).unwrap();
        let shifted_hsl = rgb_to_hsl(shifted_rgba.0[0], shifted_rgba.0[1], shifted_rgba.0[2]);

        // Original red has L=50, shifted should be L=30
        assert!((shifted_hsl.l - 30.0).abs() < 1.0, "Shifted lightness should be ~30");
    }
}
