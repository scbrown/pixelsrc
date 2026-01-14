//! Sprite rendering to image buffers

use crate::color::parse_color;
use crate::models::Sprite;
use crate::tokenizer;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

/// A warning generated during rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Magenta color used for unknown tokens and invalid colors
const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);

/// Transparent color used for padding
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Render a sprite to an RGBA image buffer.
///
/// Takes a sprite and a resolved palette (token -> hex color string).
/// Returns the rendered image and any warnings generated.
///
/// # Size Inference
///
/// If `sprite.size` is `None`:
/// - Width = max tokens in any row
/// - Height = number of rows
///
/// If `sprite.size` is `Some([w, h])`:
/// - Rows are padded/truncated to match width
/// - Grid is padded/truncated to match height
///
/// # Error Handling (Lenient Mode)
///
/// - Unknown tokens: Render as magenta (#FF00FF) with warning
/// - Row too short: Pad with transparent pixels (treated as {_}) with warning
/// - Row too long: Truncate with warning
/// - Invalid color format: Use magenta with warning
///
/// # Examples
///
/// ```
/// use pixelsrc::renderer::render_sprite;
/// use pixelsrc::models::{Sprite, PaletteRef};
/// use std::collections::HashMap;
///
/// let sprite = Sprite {
///     name: "dot".to_string(),
///     size: None,
///     palette: PaletteRef::Inline(HashMap::from([
///         ("{x}".to_string(), "#FF0000".to_string()),
///     ])),
///     grid: vec!["{x}".to_string()],
/// };
///
/// let palette: HashMap<String, String> = HashMap::from([
///     ("{x}".to_string(), "#FF0000".to_string()),
/// ]);
///
/// let (image, warnings) = render_sprite(&sprite, &palette);
/// assert_eq!(image.width(), 1);
/// assert_eq!(image.height(), 1);
/// assert!(warnings.is_empty());
/// ```
pub fn render_sprite(sprite: &Sprite, palette: &HashMap<String, String>) -> (RgbaImage, Vec<Warning>) {
    let mut warnings = Vec::new();

    // Parse all grid rows into tokens
    let mut parsed_rows: Vec<Vec<String>> = Vec::new();
    for row in &sprite.grid {
        let (tokens, row_warnings) = tokenizer::tokenize(row);
        // Convert tokenizer warnings to renderer warnings
        for w in row_warnings {
            warnings.push(Warning::new(w.message));
        }
        parsed_rows.push(tokens);
    }

    // Determine dimensions
    let (width, height) = if let Some([w, h]) = sprite.size {
        (w as usize, h as usize)
    } else {
        // Infer from grid
        let max_width = parsed_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let grid_height = parsed_rows.len();

        // Handle empty grid case
        if max_width == 0 || grid_height == 0 {
            warnings.push(Warning::new(format!(
                "Empty grid in sprite '{}'",
                sprite.name
            )));
            return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
        }

        (max_width, grid_height)
    };

    // Handle zero dimensions
    if width == 0 || height == 0 {
        warnings.push(Warning::new(format!(
            "Empty grid in sprite '{}'",
            sprite.name
        )));
        return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
    }

    // Build color lookup with parsed RGBA values
    let mut color_cache: HashMap<String, Rgba<u8>> = HashMap::new();
    for (token, hex_color) in palette {
        match parse_color(hex_color) {
            Ok(rgba) => {
                color_cache.insert(token.clone(), rgba);
            }
            Err(e) => {
                warnings.push(Warning::new(format!(
                    "Invalid color '{}' for token {}: {}, using magenta",
                    hex_color, token, e
                )));
                color_cache.insert(token.clone(), MAGENTA);
            }
        }
    }

    // Create image
    let mut image = RgbaImage::new(width as u32, height as u32);

    // Render each pixel
    for (y, row_tokens) in parsed_rows.iter().enumerate() {
        if y >= height {
            // Grid has more rows than specified height, truncate
            warnings.push(Warning::new(format!(
                "Grid has {} rows, expected {}, truncating",
                parsed_rows.len(),
                height
            )));
            break;
        }

        let row_len = row_tokens.len();

        // Check for row length mismatches
        if row_len < width {
            warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}",
                y + 1,
                row_len,
                width
            )));
        } else if row_len > width {
            warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}, truncating",
                y + 1,
                row_len,
                width
            )));
        }

        // Process tokens that exist (up to width)
        for (x, token) in row_tokens.iter().take(width).enumerate() {
            let color = if let Some(&rgba) = color_cache.get(token) {
                rgba
            } else {
                // Unknown token
                warnings.push(Warning::new(format!(
                    "Unknown token {} in sprite '{}'",
                    token, sprite.name
                )));
                // Cache it to avoid duplicate warnings
                color_cache.insert(token.clone(), MAGENTA);
                MAGENTA
            };
            image.put_pixel(x as u32, y as u32, color);
        }

        // Pad remaining columns with transparent (if row is short)
        for x in row_len..width {
            image.put_pixel(x as u32, y as u32, TRANSPARENT);
        }
    }

    // Handle case where grid has fewer rows than expected height
    if parsed_rows.len() < height {
        warnings.push(Warning::new(format!(
            "Grid has {} rows, expected {}, padding with transparent",
            parsed_rows.len(),
            height
        )));
        // Remaining rows are already transparent (default for RgbaImage::new)
    }

    (image, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;

    fn make_palette(colors: &[(&str, &str)]) -> HashMap<String, String> {
        colors
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_render_minimal_dot() {
        // From minimal_dot.jsonl
        let sprite = Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{x}".to_string()],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{x}", "#FF0000"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
        assert!(warnings.is_empty());

        // Check pixel is red
        let pixel = image.get_pixel(0, 0);
        assert_eq!(*pixel, Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_simple_heart() {
        // From simple_heart.jsonl
        let sprite = Sprite {
            name: "heart".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{_}{r}{r}{_}{r}{r}{_}".to_string(),
                "{r}{p}{r}{r}{p}{r}{r}".to_string(),
                "{r}{r}{r}{r}{r}{r}{r}".to_string(),
                "{_}{r}{r}{r}{r}{r}{_}".to_string(),
                "{_}{_}{r}{r}{r}{_}{_}".to_string(),
                "{_}{_}{_}{r}{_}{_}{_}".to_string(),
            ],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{r}", "#FF0000"),
            ("{p}", "#FF6B6B"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 7);
        assert_eq!(image.height(), 6);
        assert!(warnings.is_empty());

        // Check a few pixels
        // Top-left should be transparent
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
        // (1,0) should be red
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        // (1,1) should be pink (#FF6B6B)
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 107, 107, 255]));
    }

    #[test]
    fn test_render_with_explicit_size() {
        // From with_size.jsonl
        let sprite = Sprite {
            name: "sized".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{_}{_}{x}".to_string(),
                "{_}{x}{x}{_}".to_string(),
                "{_}{x}{x}{_}".to_string(),
                "{x}{_}{_}{x}".to_string(),
            ],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{x}", "#0000FF"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 4);
        assert!(warnings.is_empty());

        // Check corners are blue
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 255, 255]));
        // Check center is transparent
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_render_row_too_short() {
        // From row_too_short.jsonl
        let sprite = Sprite {
            name: "short_row".to_string(),
            size: Some([4, 2]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{x}".to_string(),      // Only 2 tokens, expects 4
                "{x}{x}{x}{x}".to_string(), // Full row
            ],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{x}", "#FF0000"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 2);

        // Should have warning about short row
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("Row 1") && w.message.contains("2 tokens")));

        // First row: 2 red, 2 transparent (padded)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 0, 0]));

        // Second row: all red
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_row_too_long() {
        // From row_too_long.jsonl
        let sprite = Sprite {
            name: "long_row".to_string(),
            size: Some([2, 2]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{x}{x}{x}{x}".to_string(), // 5 tokens, expects 2
                "{x}{x}".to_string(),           // Full row
            ],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{x}", "#FF0000"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);

        // Should have warning about long row being truncated
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("truncating")));

        // Only first 2 pixels should be set per row
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_unknown_token() {
        // From unknown_token.jsonl
        let sprite = Sprite {
            name: "unknown".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{y}{x}".to_string(), // {y} is unknown
                "{x}{x}{x}".to_string(),
            ],
        };

        let palette = make_palette(&[
            ("{_}", "#00000000"),
            ("{x}", "#FF0000"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);

        // Should have warning about unknown token
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("Unknown token {y}")));

        // {y} should be magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // {x}
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 255, 255])); // {y} -> magenta
        assert_eq!(*image.get_pixel(2, 0), Rgba([255, 0, 0, 255])); // {x}
    }

    #[test]
    fn test_render_empty_grid() {
        let sprite = Sprite {
            name: "empty".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
        };

        let palette = make_palette(&[]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should create 1x1 transparent image
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));

        // Should have warning about empty grid
        assert!(warnings.iter().any(|w| w.message.contains("Empty grid")));
    }

    #[test]
    fn test_render_invalid_color() {
        let sprite = Sprite {
            name: "bad_color".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{x}".to_string()],
        };

        let palette = make_palette(&[
            ("{x}", "not-a-color"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should have warning about invalid color
        assert!(warnings.iter().any(|w| w.message.contains("Invalid color")));

        // Should render as magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 255, 255]));
    }

    #[test]
    fn test_render_size_inference() {
        // No explicit size, should infer from grid
        let sprite = Sprite {
            name: "infer".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{a}{b}{c}".to_string(),
                "{d}{e}".to_string(), // Shorter row - width should still be 3
            ],
        };

        let palette = make_palette(&[
            ("{a}", "#FF0000"),
            ("{b}", "#00FF00"),
            ("{c}", "#0000FF"),
            ("{d}", "#FFFF00"),
            ("{e}", "#FF00FF"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Width should be max row length (3), height should be row count (2)
        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);

        // Should warn about short second row
        assert!(warnings.iter().any(|w| w.message.contains("Row 2")));
    }

    #[test]
    fn test_no_duplicate_unknown_token_warnings() {
        // Same unknown token used multiple times should only warn once
        let sprite = Sprite {
            name: "dupe".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{x}{x}".to_string(),
            ],
        };

        let palette = make_palette(&[]); // {x} is not defined

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should only have one warning about {x}
        let unknown_x_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.message.contains("Unknown token {x}"))
            .collect();
        assert_eq!(unknown_x_warnings.len(), 1);

        // All pixels should be magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 255, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 255, 255]));
        assert_eq!(*image.get_pixel(2, 0), Rgba([255, 0, 255, 255]));
    }

    #[test]
    fn test_render_from_fixture_files() {
        use std::fs;
        use std::io::BufReader;
        use crate::parser::parse_stream;
        use crate::models::{TtpObject, PaletteRef};

        // Test minimal_dot.jsonl
        let file = fs::File::open("tests/fixtures/valid/minimal_dot.jsonl").unwrap();
        let result = parse_stream(BufReader::new(file));
        assert_eq!(result.objects.len(), 1);

        if let TtpObject::Sprite(sprite) = &result.objects[0] {
            let palette = match &sprite.palette {
                PaletteRef::Inline(colors) => colors.clone(),
                PaletteRef::Named(_) => HashMap::new(),
            };

            let (image, warnings) = render_sprite(sprite, &palette);
            assert_eq!(image.width(), 1);
            assert_eq!(image.height(), 1);
            assert!(warnings.is_empty());
            assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        } else {
            panic!("Expected sprite");
        }
    }
}
