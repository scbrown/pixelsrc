//! Terminal rendering utilities for colored sprite display
//!
//! Provides ANSI escape sequence generation for displaying sprites with
//! true-color backgrounds in terminal emulators that support 24-bit color.

use crate::color::parse_color;
use crate::tokenizer::tokenize;
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
    grid: &[String],
    palette: &HashMap<String, String>,
    aliases: &HashMap<char, String>,
) -> (String, String) {
    let mut output = String::new();
    let mut legend_entries: Vec<(char, String, String)> = Vec::new();
    let mut seen_tokens: HashMap<String, char> = HashMap::new();

    // Build reverse alias map: token_name -> alias_char
    let reverse_aliases: HashMap<String, char> =
        aliases.iter().map(|(c, name)| (name.clone(), *c)).collect();

    // Track which tokens we've seen for the legend
    let mut next_auto_alias = 'a';

    for row in grid {
        let (tokens, _warnings) = tokenize(row);

        for token in &tokens {
            // Determine the display character for this token
            let display_char = if let Some(&c) = reverse_aliases.get(token) {
                c
            } else if let Some(&c) = seen_tokens.get(token) {
                c
            } else {
                // Auto-assign an alias based on token name or next available letter
                let c = if token == "{_}" {
                    '_'
                } else if token.len() == 3 {
                    // Single char token like {a} -> 'a'
                    token.chars().nth(1).unwrap_or(next_auto_alias)
                } else {
                    // Multi-char token, use next auto alias
                    let c = next_auto_alias;
                    if next_auto_alias < 'z' {
                        next_auto_alias = (next_auto_alias as u8 + 1) as char;
                    }
                    c
                };
                seen_tokens.insert(token.clone(), c);

                // Add to legend if we haven't seen this token
                let hex_color = palette
                    .get(token)
                    .cloned()
                    .unwrap_or_else(|| "???".to_string());
                let name = token.trim_matches(|c| c == '{' || c == '}').to_string();
                legend_entries.push((c, name, hex_color));

                c
            };

            // Get the color for this token
            let hex_color = palette
                .get(token)
                .cloned()
                .unwrap_or_else(|| "#808080".to_string());
            let rgba = parse_color(&hex_color).unwrap_or(Rgba([128, 128, 128, 255]));
            let ansi_bg = color_to_ansi_bg(rgba);

            // Render as 3-char cell: " X " with background color
            output.push_str(&ansi_bg);
            output.push(' ');
            output.push(display_char);
            output.push(' ');
            output.push_str(ANSI_RESET);
        }
        output.push('\n');
    }

    // Build legend
    let mut legend = String::from("\nLegend:\n");
    // Sort by alias char for consistent output
    legend_entries.sort_by_key(|(c, _, _)| *c);
    // Deduplicate (keep first occurrence)
    let mut seen_chars: std::collections::HashSet<char> = std::collections::HashSet::new();
    for (c, name, hex) in legend_entries {
        if seen_chars.insert(c) {
            legend.push_str(&format!("  {} = {:16} ({})\n", c, name, hex));
        }
    }

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
pub fn render_coordinate_grid(grid: &[String], full_names: bool) -> String {
    if grid.is_empty() {
        return String::new();
    }

    // Parse all rows to get tokens
    let parsed_rows: Vec<Vec<String>> = grid
        .iter()
        .map(|row| {
            let (tokens, _) = tokenize(row);
            tokens
        })
        .collect();

    // Find the maximum number of columns
    let max_cols = parsed_rows.iter().map(|row| row.len()).max().unwrap_or(0);
    if max_cols == 0 {
        return String::new();
    }

    // Determine cell width based on mode
    let cell_width = if full_names {
        // Find the longest token name
        parsed_rows
            .iter()
            .flat_map(|row| row.iter())
            .map(|token| token.len())
            .max()
            .unwrap_or(3)
    } else {
        2 // Single char + space
    };

    let mut output = String::new();

    // Calculate row number width (how many digits in max row number)
    let row_num_width = (grid.len().saturating_sub(1)).to_string().len().max(2);

    // Column header line
    output.push_str(&" ".repeat(row_num_width + 1)); // Space for row numbers + border
    for col in 0..max_cols {
        if full_names {
            output.push_str(&format!("{:>width$} ", col, width = cell_width));
        } else {
            output.push_str(&format!("{:>2} ", col));
        }
    }
    output.push('\n');

    // Border line
    output.push_str(&" ".repeat(row_num_width));
    output.push_str(" \u{250C}"); // ┌
    let border_width = if full_names {
        max_cols * (cell_width + 1)
    } else {
        max_cols * 3
    };
    output.push_str(&"\u{2500}".repeat(border_width)); // ─
    output.push('\n');

    // Data rows
    for (row_idx, tokens) in parsed_rows.iter().enumerate() {
        // Row number
        output.push_str(&format!(
            "{:>width$} \u{2502}",
            row_idx,
            width = row_num_width
        )); // │

        for token in tokens {
            let display = if full_names {
                token.clone()
            } else {
                // Abbreviate: use first char of token name (without braces)
                let name = token.trim_matches(|c| c == '{' || c == '}');
                if name == "_" {
                    "_".to_string()
                } else {
                    name.chars().next().unwrap_or('?').to_string()
                }
            };

            if full_names {
                output.push_str(&format!(" {:>width$}", display, width = cell_width));
            } else {
                output.push_str(&format!(" {:>2}", display));
            }
        }
        output.push('\n');
    }

    output
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
                    bottom_pixel[0], bottom_pixel[1], bottom_pixel[2],
                    top_pixel[0], top_pixel[1], top_pixel[2]
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
    fn test_render_ansi_grid_simple() {
        let grid = vec!["{a}{b}".to_string(), "{b}{a}".to_string()];
        let palette = HashMap::from([
            ("{a}".to_string(), "#FF0000".to_string()),
            ("{b}".to_string(), "#00FF00".to_string()),
        ]);
        let aliases = HashMap::new();

        let (colored, legend) = render_ansi_grid(&grid, &palette, &aliases);

        // Should contain ANSI escape sequences
        assert!(colored.contains("\x1b[48;2;"));
        assert!(colored.contains(ANSI_RESET));

        // Legend should contain our tokens
        assert!(legend.contains("Legend:"));
        assert!(legend.contains("#FF0000") || legend.contains("#00FF00"));
    }

    #[test]
    fn test_render_ansi_grid_with_transparent() {
        let grid = vec!["{_}{a}".to_string()];
        let palette = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{a}".to_string(), "#FF0000".to_string()),
        ]);
        let aliases = HashMap::new();

        let (colored, _) = render_ansi_grid(&grid, &palette, &aliases);

        // Should use 256-color gray for transparent
        assert!(colored.contains("\x1b[48;5;236m"));
    }

    #[test]
    fn test_render_coordinate_grid_simple() {
        let grid = vec!["{a}{b}{c}".to_string(), "{d}{e}{f}".to_string()];

        let output = render_coordinate_grid(&grid, false);

        // Should have column headers
        assert!(output.contains(" 0"));
        assert!(output.contains(" 1"));
        assert!(output.contains(" 2"));

        // Should have row numbers
        assert!(output.contains("0 \u{2502}")); // 0 │
        assert!(output.contains("1 \u{2502}")); // 1 │

        // Should have token abbreviations
        assert!(output.contains(" a"));
        assert!(output.contains(" b"));
    }

    #[test]
    fn test_render_coordinate_grid_full_names() {
        let grid = vec!["{skin}{hair}".to_string()];

        let output = render_coordinate_grid(&grid, true);

        // Should have full token names
        assert!(output.contains("{skin}"));
        assert!(output.contains("{hair}"));
    }

    #[test]
    fn test_render_coordinate_grid_empty() {
        let grid: Vec<String> = vec![];
        let output = render_coordinate_grid(&grid, false);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_coordinate_grid_underscore() {
        let grid = vec!["{_}{a}{_}".to_string()];

        let output = render_coordinate_grid(&grid, false);

        // Underscore should be preserved
        assert!(output.contains(" _"));
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
