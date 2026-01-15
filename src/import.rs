//! PNG import functionality for converting images to Pixelsrc format.
//!
//! This module provides functionality to:
//! - Read PNG images and extract unique colors
//! - Quantize colors using median cut algorithm if too many colors
//! - Generate Pixelsrc JSONL output with palette and sprite definitions

use image::{GenericImageView, Rgba};
use std::collections::HashMap;
use std::path::Path;

/// Result of importing a PNG image.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The generated sprite name.
    pub name: String,
    /// Width of the sprite in pixels.
    pub width: u32,
    /// Height of the sprite in pixels.
    pub height: u32,
    /// Color palette mapping tokens to hex colors.
    pub palette: HashMap<String, String>,
    /// Grid rows with token sequences.
    pub grid: Vec<String>,
}

impl ImportResult {
    /// Serialize to JSONL format (palette line + sprite line).
    pub fn to_jsonl(&self) -> String {
        let palette_json = serde_json::json!({
            "type": "palette",
            "name": format!("{}_palette", self.name),
            "colors": self.palette
        });

        let sprite_json = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [self.width, self.height],
            "palette": format!("{}_palette", self.name),
            "grid": self.grid
        });

        format!("{}\n{}", palette_json, sprite_json)
    }
}

/// A color represented as RGBA values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn from_rgba(rgba: Rgba<u8>) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// A box of colors for median cut algorithm.
#[derive(Debug, Clone)]
struct ColorBox {
    colors: Vec<(Color, u32)>, // Color and count
}

impl ColorBox {
    fn new(colors: Vec<(Color, u32)>) -> Self {
        Self { colors }
    }

    /// Find which channel has the largest range.
    fn widest_channel(&self) -> Channel {
        let (mut min_r, mut max_r) = (255u8, 0u8);
        let (mut min_g, mut max_g) = (255u8, 0u8);
        let (mut min_b, mut max_b) = (255u8, 0u8);

        for (color, _) in &self.colors {
            min_r = min_r.min(color.r);
            max_r = max_r.max(color.r);
            min_g = min_g.min(color.g);
            max_g = max_g.max(color.g);
            min_b = min_b.min(color.b);
            max_b = max_b.max(color.b);
        }

        let range_r = max_r.saturating_sub(min_r);
        let range_g = max_g.saturating_sub(min_g);
        let range_b = max_b.saturating_sub(min_b);

        if range_r >= range_g && range_r >= range_b {
            Channel::Red
        } else if range_g >= range_b {
            Channel::Green
        } else {
            Channel::Blue
        }
    }

    /// Split the box into two along the widest channel.
    fn split(mut self) -> (ColorBox, ColorBox) {
        let channel = self.widest_channel();

        // Sort by the widest channel
        self.colors.sort_by_key(|(color, _)| match channel {
            Channel::Red => color.r,
            Channel::Green => color.g,
            Channel::Blue => color.b,
        });

        // Find median by pixel count
        let total: u32 = self.colors.iter().map(|(_, count)| count).sum();
        let mut running = 0u32;
        let mut split_idx = self.colors.len() / 2;

        for (i, (_, count)) in self.colors.iter().enumerate() {
            running += count;
            if running >= total / 2 {
                split_idx = (i + 1).min(self.colors.len() - 1);
                break;
            }
        }

        // Ensure we don't create empty boxes
        split_idx = split_idx.max(1).min(self.colors.len() - 1);

        let right = self.colors.split_off(split_idx);
        (ColorBox::new(self.colors), ColorBox::new(right))
    }

    /// Get the average color of this box (weighted by pixel count).
    fn average_color(&self) -> Color {
        let total: u64 = self.colors.iter().map(|(_, count)| *count as u64).sum();
        if total == 0 {
            return Color { r: 0, g: 0, b: 0, a: 255 };
        }

        let r: u64 = self.colors.iter().map(|(c, count)| c.r as u64 * *count as u64).sum();
        let g: u64 = self.colors.iter().map(|(c, count)| c.g as u64 * *count as u64).sum();
        let b: u64 = self.colors.iter().map(|(c, count)| c.b as u64 * *count as u64).sum();
        let a: u64 = self.colors.iter().map(|(c, count)| c.a as u64 * *count as u64).sum();

        Color {
            r: (r / total) as u8,
            g: (g / total) as u8,
            b: (b / total) as u8,
            a: (a / total) as u8,
        }
    }

    /// Total pixel count in this box.
    fn pixel_count(&self) -> u32 {
        self.colors.iter().map(|(_, count)| count).sum()
    }
}

#[derive(Debug, Clone, Copy)]
enum Channel {
    Red,
    Green,
    Blue,
}

/// Quantize colors using median cut algorithm.
fn median_cut_quantize(colors: HashMap<Color, u32>, max_colors: usize) -> Vec<Color> {
    if colors.len() <= max_colors {
        return colors.into_keys().collect();
    }

    // Separate transparent colors from opaque colors
    let mut transparent: Option<Color> = None;
    let mut opaque_colors: Vec<(Color, u32)> = Vec::new();

    for (color, count) in colors {
        if color.is_transparent() {
            transparent = Some(color);
        } else {
            opaque_colors.push((color, count));
        }
    }

    // Adjust max_colors if we have a transparent color
    let effective_max = if transparent.is_some() {
        max_colors.saturating_sub(1)
    } else {
        max_colors
    };

    if opaque_colors.len() <= effective_max {
        let mut result: Vec<Color> = opaque_colors.into_iter().map(|(c, _)| c).collect();
        if let Some(t) = transparent {
            result.push(t);
        }
        return result;
    }

    // Initial box with all opaque colors
    let mut boxes = vec![ColorBox::new(opaque_colors)];

    // Split until we have enough boxes
    while boxes.len() < effective_max {
        // Find the box with the most pixels to split
        let (idx, _) = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.colors.len() > 1)
            .max_by_key(|(_, b)| b.pixel_count())
            .unwrap_or((0, &boxes[0]));

        if boxes[idx].colors.len() <= 1 {
            break;
        }

        let box_to_split = boxes.remove(idx);
        let (left, right) = box_to_split.split();
        boxes.push(left);
        boxes.push(right);
    }

    // Get average color from each box
    let mut result: Vec<Color> = boxes.into_iter().map(|b| b.average_color()).collect();

    // Add transparent color if present
    if let Some(t) = transparent {
        result.push(t);
    }

    result
}

/// Find the closest color in the palette to a given color.
fn find_closest_color(color: Color, palette: &[Color]) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| {
            let dr = (color.r as i32 - p.r as i32).abs();
            let dg = (color.g as i32 - p.g as i32).abs();
            let db = (color.b as i32 - p.b as i32).abs();
            let da = (color.a as i32 - p.a as i32).abs();
            dr * dr + dg * dg + db * db + da * da
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Import a PNG file and convert it to Pixelsrc format.
pub fn import_png<P: AsRef<Path>>(
    path: P,
    name: &str,
    max_colors: usize,
) -> Result<ImportResult, String> {
    let img = image::open(path.as_ref())
        .map_err(|e| format!("Failed to open image: {}", e))?;

    let (width, height) = img.dimensions();

    // Extract all unique colors with their pixel counts
    let mut color_counts: HashMap<Color, u32> = HashMap::new();
    for (_, _, pixel) in img.pixels() {
        let color = Color::from_rgba(pixel);
        *color_counts.entry(color).or_insert(0) += 1;
    }

    // Quantize if needed
    let palette_colors = median_cut_quantize(color_counts.clone(), max_colors);

    // Build color to index mapping
    let original_colors: Vec<Color> = color_counts.keys().cloned().collect();

    // Map original colors to palette colors
    let mut color_to_palette_idx: HashMap<Color, usize> = HashMap::new();
    for orig_color in &original_colors {
        let idx = find_closest_color(*orig_color, &palette_colors);
        color_to_palette_idx.insert(*orig_color, idx);
    }

    // Generate token names
    // Find transparent color index for special {_} token
    let transparent_idx = palette_colors
        .iter()
        .position(|c| c.is_transparent());

    let mut palette: HashMap<String, String> = HashMap::new();
    let mut idx_to_token: HashMap<usize, String> = HashMap::new();

    let mut color_num = 1;
    for (idx, color) in palette_colors.iter().enumerate() {
        let token = if Some(idx) == transparent_idx {
            "{_}".to_string()
        } else {
            let t = format!("{{c{}}}", color_num);
            color_num += 1;
            t
        };
        palette.insert(token.clone(), color.to_hex());
        idx_to_token.insert(idx, token);
    }

    // Build grid
    let mut grid: Vec<String> = Vec::with_capacity(height as usize);
    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let color = Color::from_rgba(pixel);
            let palette_idx = color_to_palette_idx[&color];
            let token = &idx_to_token[&palette_idx];
            row.push_str(token);
        }
        grid.push(row);
    }

    Ok(ImportResult {
        name: name.to_string(),
        width,
        height,
        palette,
        grid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hex_opaque() {
        let color = Color { r: 255, g: 128, b: 0, a: 255 };
        assert_eq!(color.to_hex(), "#FF8000");
    }

    #[test]
    fn test_color_to_hex_transparent() {
        let color = Color { r: 255, g: 128, b: 0, a: 128 };
        assert_eq!(color.to_hex(), "#FF800080");
    }

    #[test]
    fn test_color_to_hex_fully_transparent() {
        let color = Color { r: 0, g: 0, b: 0, a: 0 };
        assert_eq!(color.to_hex(), "#00000000");
    }

    #[test]
    fn test_median_cut_no_quantization_needed() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 0, b: 255, a: 255 }, 10);

        let result = median_cut_quantize(colors, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_median_cut_quantization() {
        let mut colors = HashMap::new();
        // Create more colors than max
        for i in 0..20 {
            colors.insert(Color { r: i * 10, g: i * 5, b: i * 2, a: 255 }, 1);
        }

        let result = median_cut_quantize(colors, 4);
        assert!(result.len() <= 4);
    }

    #[test]
    fn test_transparent_color_preserved() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 0, g: 0, b: 0, a: 0 }, 10); // Transparent
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);

        let result = median_cut_quantize(colors, 3);
        assert!(result.iter().any(|c| c.is_transparent()));
    }

    #[test]
    fn test_find_closest_color() {
        let palette = vec![
            Color { r: 0, g: 0, b: 0, a: 255 },
            Color { r: 255, g: 255, b: 255, a: 255 },
        ];

        let dark = Color { r: 30, g: 30, b: 30, a: 255 };
        let light = Color { r: 200, g: 200, b: 200, a: 255 };

        assert_eq!(find_closest_color(dark, &palette), 0);
        assert_eq!(find_closest_color(light, &palette), 1);
    }

    #[test]
    fn test_import_result_to_jsonl() {
        let mut palette = HashMap::new();
        palette.insert("{_}".to_string(), "#00000000".to_string());
        palette.insert("{c1}".to_string(), "#FF0000".to_string());

        let result = ImportResult {
            name: "test_sprite".to_string(),
            width: 2,
            height: 2,
            palette,
            grid: vec!["{c1}{_}".to_string(), "{_}{c1}".to_string()],
        };

        let jsonl = result.to_jsonl();
        assert!(jsonl.contains("\"type\":\"palette\""));
        assert!(jsonl.contains("\"type\":\"sprite\""));
        assert!(jsonl.contains("test_sprite_palette"));
        assert!(jsonl.contains("test_sprite"));
    }
}
