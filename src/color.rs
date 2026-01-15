//! Color parsing utilities for hex color strings
//!
//! Supports the following formats:
//! - `#RGB` - 3-digit hex, expands to `#RRGGBB`
//! - `#RGBA` - 4-digit hex, expands to `#RRGGBBAA`
//! - `#RRGGBB` - 6-digit hex, fully opaque (alpha = 255)
//! - `#RRGGBBAA` - 8-digit hex, with explicit alpha

use image::Rgba;
use std::fmt;

/// Error type for color parsing failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorError {
    /// Input string was empty
    Empty,
    /// Input string doesn't start with '#'
    MissingHash,
    /// Invalid length (must be 3, 4, 6, or 8 hex chars after #)
    InvalidLength(usize),
    /// Contains non-hex characters
    InvalidHex(char),
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorError::Empty => write!(f, "empty color string"),
            ColorError::MissingHash => write!(f, "color must start with '#'"),
            ColorError::InvalidLength(len) => {
                write!(f, "invalid color length {}, expected 3, 4, 6, or 8", len)
            }
            ColorError::InvalidHex(c) => write!(f, "invalid hex character '{}'", c),
        }
    }
}

impl std::error::Error for ColorError {}

/// Parse a hex color string into an RGBA color.
///
/// # Supported Formats
///
/// - `#RGB` - 3-digit hex, each digit is doubled (e.g., `#F00` -> red)
/// - `#RGBA` - 4-digit hex, each digit is doubled (e.g., `#F00F` -> red, opaque)
/// - `#RRGGBB` - 6-digit hex, alpha defaults to 255 (opaque)
/// - `#RRGGBBAA` - 8-digit hex, explicit alpha channel
///
/// # Examples
///
/// ```
/// use pixelsrc::color::parse_color;
///
/// // Short form red
/// let red = parse_color("#F00").unwrap();
/// assert_eq!(red, image::Rgba([255, 0, 0, 255]));
///
/// // Long form red
/// let red = parse_color("#FF0000").unwrap();
/// assert_eq!(red, image::Rgba([255, 0, 0, 255]));
///
/// // Red with 50% alpha
/// let red_alpha = parse_color("#FF000080").unwrap();
/// assert_eq!(red_alpha, image::Rgba([255, 0, 0, 128]));
///
/// // Short form with alpha
/// let red_alpha = parse_color("#F00F").unwrap();
/// assert_eq!(red_alpha, image::Rgba([255, 0, 0, 255]));
/// ```
///
/// # Errors
///
/// Returns `ColorError` if the input is invalid:
/// - Empty string
/// - Missing '#' prefix
/// - Invalid length (not 3, 4, 6, or 8 hex chars)
/// - Non-hex characters
pub fn parse_color(s: &str) -> Result<Rgba<u8>, ColorError> {
    if s.is_empty() {
        return Err(ColorError::Empty);
    }

    if !s.starts_with('#') {
        return Err(ColorError::MissingHash);
    }

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
    fn test_error_missing_hash() {
        assert_eq!(parse_color("FF0000"), Err(ColorError::MissingHash));
        assert_eq!(parse_color("F00"), Err(ColorError::MissingHash));
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
        assert_eq!(
            parse_color("#not-a-color"),
            Err(ColorError::InvalidHex('n'))
        );
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
}
