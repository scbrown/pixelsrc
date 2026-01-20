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
