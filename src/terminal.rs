//! Terminal rendering utilities for colored sprite display
//!
//! Provides ANSI escape sequence generation for displaying sprites with
//! true-color backgrounds in terminal emulators that support 24-bit color.

use crate::color::parse_color;
use image::Rgba;
use std::collections::HashMap;

/// ANSI escape sequence to reset all formatting
pub const ANSI_RESET: &str = "\x1b[0m";

/// Convert RGBA color to ANSI 24-bit background escape sequence.
///
/// Transparent colors (alpha = 0) are rendered as a dark gray background
/// to visually distinguish them from opaque colors.
///
/// # Examples
///
/// ```
/// use pixelsrc::terminal::color_to_ansi_bg;
/// use image::Rgba;
///
/// // Opaque red
/// let red = color_to_ansi_bg(Rgba([255, 0, 0, 255]));
/// assert_eq!(red, "\x1b[48;2;255;0;0m");
///
/// // Transparent (renders as dark gray)
/// let transparent = color_to_ansi_bg(Rgba([0, 0, 0, 0]));
/// assert_eq!(transparent, "\x1b[48;5;236m");
/// ```
pub fn color_to_ansi_bg(rgba: Rgba<u8>) -> String {
    if rgba[3] == 0 {
        // Dark gray for transparent pixels
        "\x1b[48;5;236m".to_string()
    } else {
        format!("\x1b[48;2;{};{};{}m", rgba[0], rgba[1], rgba[2])
    }
}

/// Render sprite grid with ANSI color backgrounds.
///
/// Returns a tuple of (colored_grid, legend):
/// - colored_grid: String with ANSI escape sequences for colored display
/// - legend: String mapping aliases to their semantic names and hex colors
///
/// Each cell is displayed as the alias character (or first char of token name)
/// centered in a 3-character cell with the appropriate background color.
///
/// # Arguments
///
/// * `grid` - Sprite grid rows (e.g., `["{a}{a}{b}", "{a}{b}{b}"]`)
/// * `palette` - Map from token names to hex color strings (e.g., `{"{a}": "#FF0000"}`)
/// * `aliases` - Optional map from single chars to full token names (e.g., `{'a': "{skin}"}`)
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::terminal::render_ansi_grid;
/// use std::collections::HashMap;
///
/// let grid = vec!["{a}{b}".to_string()];
/// let palette = HashMap::from([
///     ("{a}".to_string(), "#FF0000".to_string()),
///     ("{b}".to_string(), "#00FF00".to_string()),
/// ]);
/// let aliases = HashMap::new();
///
/// let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);
/// // colored contains ANSI escape sequences for display
/// // legend shows the color mapping
/// ```
pub fn render_ansi_grid(
    _grid: &[String],
    _palette: &HashMap<String, String>,
    _aliases: &HashMap<char, String>,
) -> (String, String) {
    // Grid format is no longer supported - use structured regions format instead
    let output = String::from("[Grid format deprecated - use structured regions]\n");
    let legend = String::from("\nLegend: N/A (grid format deprecated)\n");
    (output, legend)
}

/// Render grid with row/column coordinate headers.
///
/// Displays the sprite grid with column numbers across the top and row numbers
/// down the left side, making it easy to reference specific pixel positions.
///
/// # Arguments
///
/// * `grid` - Sprite grid rows
/// * `full_names` - If true, show full token names; if false, show abbreviations
///
/// # Examples
///
/// ```
/// use pixelsrc::terminal::render_coordinate_grid;
///
/// let grid = vec!["{a}{b}{c}".to_string(), "{d}{e}{f}".to_string()];
///
/// let output = render_coordinate_grid(&grid, false);
/// // Output:
/// //      0  1  2
/// //    ┌─────────
/// //  0 │ a  b  c
/// //  1 │ d  e  f
/// ```
pub fn render_coordinate_grid(_grid: &[String], _full_names: bool) -> String {
    // Grid format is no longer supported - use structured regions format instead
    String::from("[Grid format deprecated - use structured regions]\n")
}

/// Render an RGBA image to ANSI terminal output.
///
/// Each pixel is rendered as a "▀" (upper half block) character with
/// foreground and background colors set to display two rows of pixels
/// per line of text.
///
/// # Arguments
///
/// * `image` - The RGBA image to render
///
/// # Returns
///
/// A string with ANSI escape sequences for colored terminal display.
pub fn render_image_ansi(image: &image::RgbaImage) -> String {
    use image::Rgba;

    let width = image.width() as usize;
    let height = image.height() as usize;

    if width == 0 || height == 0 {
        return String::new();
    }

    let mut output = String::new();

    // Process two rows at a time using half-block characters
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let top_pixel = *image.get_pixel(x as u32, y as u32);
            let bottom_pixel = if y + 1 < height {
                *image.get_pixel(x as u32, (y + 1) as u32)
            } else {
                Rgba([0, 0, 0, 0]) // Transparent for odd height images
            };

            // Use upper half block (▀) with foreground = top pixel, background = bottom pixel
            if top_pixel[3] == 0 && bottom_pixel[3] == 0 {
                // Both transparent - use dark gray
                output.push_str("\x1b[48;5;236m\x1b[38;5;236m▀");
            } else if top_pixel[3] == 0 {
                // Top transparent, bottom visible
                output.push_str(&format!(
                    "\x1b[48;2;{};{};{}m\x1b[38;5;236m▀",
                    bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]
                ));
            } else if bottom_pixel[3] == 0 {
                // Top visible, bottom transparent
                output.push_str(&format!(
                    "\x1b[48;5;236m\x1b[38;2;{};{};{}m▀",
                    top_pixel[0], top_pixel[1], top_pixel[2]
                ));
            } else {
                // Both visible
                output.push_str(&format!(
                    "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m▀",
                    bottom_pixel[0],
                    bottom_pixel[1],
                    bottom_pixel[2],
                    top_pixel[0],
                    top_pixel[1],
                    top_pixel[2]
                ));
            }
        }
        output.push_str(ANSI_RESET);
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_ansi_bg_opaque() {
        let red = color_to_ansi_bg(Rgba([255, 0, 0, 255]));
        assert_eq!(red, "\x1b[48;2;255;0;0m");

        let green = color_to_ansi_bg(Rgba([0, 255, 0, 255]));
        assert_eq!(green, "\x1b[48;2;0;255;0m");

        let blue = color_to_ansi_bg(Rgba([0, 0, 255, 255]));
        assert_eq!(blue, "\x1b[48;2;0;0;255m");
    }

    #[test]
    fn test_color_to_ansi_bg_transparent() {
        let transparent = color_to_ansi_bg(Rgba([0, 0, 0, 0]));
        assert_eq!(transparent, "\x1b[48;5;236m");

        // Transparent with non-zero RGB should still use gray
        let transparent_red = color_to_ansi_bg(Rgba([255, 0, 0, 0]));
        assert_eq!(transparent_red, "\x1b[48;5;236m");
    }

    #[test]
    fn test_color_to_ansi_bg_partial_alpha() {
        // Non-zero alpha should use the RGB values
        let semi_transparent = color_to_ansi_bg(Rgba([255, 0, 0, 128]));
        assert_eq!(semi_transparent, "\x1b[48;2;255;0;0m");
    }

    #[test]
    fn test_render_image_ansi_empty() {
        use image::RgbaImage;

        let image = RgbaImage::new(0, 0);
        let output = render_image_ansi(&image);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_image_ansi_simple() {
        use image::RgbaImage;

        // 2x2 red image
        let image = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let output = render_image_ansi(&image);

        // Should contain ANSI escape sequences
        assert!(output.contains("\x1b["));
        // Should contain the half block character
        assert!(output.contains("▀"));
        // Should end with reset and newline
        assert!(output.contains(ANSI_RESET));
    }

    #[test]
    fn test_render_image_ansi_transparent() {
        use image::RgbaImage;

        // 2x2 transparent image
        let image = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 0]));
        let output = render_image_ansi(&image);

        // Should use 256-color gray for transparent
        assert!(output.contains("\x1b[48;5;236m"));
    }
}
