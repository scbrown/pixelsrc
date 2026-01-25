//! Color quantization using median cut algorithm.
//!
//! Supports both RGB and perceptual LAB color space quantization.

use image::Rgba;
use std::collections::HashMap;

/// A color represented as RGBA values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgba(rgba: Rgba<u8>) -> Self {
        Self { r: rgba[0], g: rgba[1], b: rgba[2], a: rgba[3] }
    }

    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn is_transparent(&self) -> bool {
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

/// LAB color representation for perceptual color quantization.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LabColor {
    pub l: f64, // Lightness: 0-100
    pub a: f64, // Green-Red axis: -128 to 127
    pub b: f64, // Blue-Yellow axis: -128 to 127
}

impl LabColor {
    /// Convert RGB color to LAB color space.
    /// Uses D65 illuminant standard.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Step 1: RGB to linear RGB (sRGB gamma correction)
        let r_lin = srgb_to_linear(r as f64 / 255.0);
        let g_lin = srgb_to_linear(g as f64 / 255.0);
        let b_lin = srgb_to_linear(b as f64 / 255.0);

        // Step 2: Linear RGB to XYZ (sRGB to XYZ matrix, D65 illuminant)
        let x = r_lin * 0.4124564 + g_lin * 0.3575761 + b_lin * 0.1804375;
        let y = r_lin * 0.2126729 + g_lin * 0.7151522 + b_lin * 0.0721750;
        let z = r_lin * 0.0193339 + g_lin * 0.1191920 + b_lin * 0.9503041;

        // Step 3: XYZ to LAB (using D65 reference white)
        // D65 reference white point
        let x_n = 0.95047;
        let y_n = 1.00000;
        let z_n = 1.08883;

        let fx = lab_f(x / x_n);
        let fy = lab_f(y / y_n);
        let fz = lab_f(z / z_n);

        let l = 116.0 * fy - 16.0;
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);

        Self { l, a, b }
    }

    /// Calculate perceptual distance to another LAB color (CIE76 Delta E).
    pub fn distance(&self, other: &LabColor) -> f64 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }
}

/// sRGB gamma expansion (inverse companding).
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// LAB f function for XYZ to LAB conversion.
fn lab_f(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta.powi(3) {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

/// LAB channel for perceptual median cut.
#[derive(Debug, Clone, Copy)]
enum LabChannel {
    L,
    A,
    B,
}

/// A box of colors in LAB space for perceptual median cut.
#[derive(Debug, Clone)]
struct LabColorBox {
    colors: Vec<(Color, LabColor, u32)>, // Original color, LAB color, count
}

impl LabColorBox {
    fn new(colors: Vec<(Color, LabColor, u32)>) -> Self {
        Self { colors }
    }

    /// Find which LAB channel has the largest range.
    fn widest_channel(&self) -> LabChannel {
        let (mut min_l, mut max_l) = (f64::MAX, f64::MIN);
        let (mut min_a, mut max_a) = (f64::MAX, f64::MIN);
        let (mut min_b, mut max_b) = (f64::MAX, f64::MIN);

        for (_, lab, _) in &self.colors {
            min_l = min_l.min(lab.l);
            max_l = max_l.max(lab.l);
            min_a = min_a.min(lab.a);
            max_a = max_a.max(lab.a);
            min_b = min_b.min(lab.b);
            max_b = max_b.max(lab.b);
        }

        let range_l = max_l - min_l;
        let range_a = max_a - min_a;
        let range_b = max_b - min_b;

        if range_l >= range_a && range_l >= range_b {
            LabChannel::L
        } else if range_a >= range_b {
            LabChannel::A
        } else {
            LabChannel::B
        }
    }

    /// Split the box into two along the widest LAB channel.
    fn split(mut self) -> (LabColorBox, LabColorBox) {
        let channel = self.widest_channel();

        // Sort by the widest channel
        self.colors.sort_by(|(_, lab1, _), (_, lab2, _)| {
            let v1 = match channel {
                LabChannel::L => lab1.l,
                LabChannel::A => lab1.a,
                LabChannel::B => lab1.b,
            };
            let v2 = match channel {
                LabChannel::L => lab2.l,
                LabChannel::A => lab2.a,
                LabChannel::B => lab2.b,
            };
            v1.partial_cmp(&v2).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Find median by pixel count
        let total: u32 = self.colors.iter().map(|(_, _, count)| count).sum();
        let mut running = 0u32;
        let mut split_idx = self.colors.len() / 2;

        for (i, (_, _, count)) in self.colors.iter().enumerate() {
            running += count;
            if running >= total / 2 {
                split_idx = (i + 1).min(self.colors.len() - 1);
                break;
            }
        }

        // Ensure we don't create empty boxes
        split_idx = split_idx.max(1).min(self.colors.len() - 1);

        let right = self.colors.split_off(split_idx);
        (LabColorBox::new(self.colors), LabColorBox::new(right))
    }

    /// Get the average color of this box (weighted by pixel count).
    /// Returns the original RGB color closest to the average LAB.
    fn average_color(&self) -> Color {
        let total: u64 = self.colors.iter().map(|(_, _, count)| *count as u64).sum();
        if total == 0 {
            return Color { r: 0, g: 0, b: 0, a: 255 };
        }

        // Calculate weighted average in LAB space
        let l: f64 = self.colors.iter().map(|(_, lab, count)| lab.l * *count as f64).sum::<f64>() / total as f64;
        let a: f64 = self.colors.iter().map(|(_, lab, count)| lab.a * *count as f64).sum::<f64>() / total as f64;
        let b: f64 = self.colors.iter().map(|(_, lab, count)| lab.b * *count as f64).sum::<f64>() / total as f64;
        let avg_lab = LabColor { l, a, b };

        // Find the original color closest to this average
        // (We return an actual palette color rather than synthesizing one)
        self.colors
            .iter()
            .min_by(|(_, lab1, _), (_, lab2, _)| {
                let d1 = avg_lab.distance(lab1);
                let d2 = avg_lab.distance(lab2);
                d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(c, _, _)| *c)
            .unwrap_or(Color { r: 0, g: 0, b: 0, a: 255 })
    }

    /// Total pixel count in this box.
    fn pixel_count(&self) -> u32 {
        self.colors.iter().map(|(_, _, count)| count).sum()
    }
}

/// Quantize colors using median cut algorithm in perceptual LAB color space.
/// This produces better results for skin tones, gradients, and similar colors.
pub(crate) fn median_cut_quantize_lab(colors: HashMap<Color, u32>, max_colors: usize) -> Vec<Color> {
    if colors.len() <= max_colors {
        return colors.into_keys().collect();
    }

    // Separate transparent colors from opaque colors
    let mut transparent: Option<Color> = None;
    let mut opaque_colors: Vec<(Color, LabColor, u32)> = Vec::new();

    for (color, count) in colors {
        if color.is_transparent() {
            transparent = Some(color);
        } else {
            let lab = LabColor::from_rgb(color.r, color.g, color.b);
            opaque_colors.push((color, lab, count));
        }
    }

    // Adjust max_colors if we have a transparent color
    let effective_max =
        if transparent.is_some() { max_colors.saturating_sub(1) } else { max_colors };

    if opaque_colors.len() <= effective_max {
        let mut result: Vec<Color> = opaque_colors.into_iter().map(|(c, _, _)| c).collect();
        if let Some(t) = transparent {
            result.push(t);
        }
        return result;
    }

    // Initial box with all opaque colors
    let mut boxes = vec![LabColorBox::new(opaque_colors)];

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

/// Quantize colors using median cut algorithm (legacy RGB version).
#[allow(dead_code)]
pub(crate) fn median_cut_quantize(colors: HashMap<Color, u32>, max_colors: usize) -> Vec<Color> {
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
    let effective_max =
        if transparent.is_some() { max_colors.saturating_sub(1) } else { max_colors };

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

/// Find the closest color in the palette to a given color using LAB perceptual distance.
pub(crate) fn find_closest_color(color: Color, palette: &[Color]) -> usize {
    // Handle transparent colors specially - match by alpha
    if color.is_transparent() {
        return palette.iter().position(|p| p.is_transparent()).unwrap_or(0);
    }

    let color_lab = LabColor::from_rgb(color.r, color.g, color.b);

    palette
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.is_transparent()) // Skip transparent when matching opaque
        .min_by(|(_, p1), (_, p2)| {
            let lab1 = LabColor::from_rgb(p1.r, p1.g, p1.b);
            let lab2 = LabColor::from_rgb(p2.r, p2.g, p2.b);
            let d1 = color_lab.distance(&lab1);
            let d2 = color_lab.distance(&lab2);
            d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
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
    fn test_lab_color_conversion_black() {
        // Black should be L=0, a=0, b=0
        let lab = LabColor::from_rgb(0, 0, 0);
        assert!(lab.l < 1.0, "Black L should be ~0, got {}", lab.l);
        assert!(lab.a.abs() < 1.0, "Black a should be ~0, got {}", lab.a);
        assert!(lab.b.abs() < 1.0, "Black b should be ~0, got {}", lab.b);
    }

    #[test]
    fn test_lab_color_conversion_white() {
        // White should be L=100, a=0, b=0
        let lab = LabColor::from_rgb(255, 255, 255);
        assert!(lab.l > 99.0, "White L should be ~100, got {}", lab.l);
        assert!(lab.a.abs() < 1.0, "White a should be ~0, got {}", lab.a);
        assert!(lab.b.abs() < 1.0, "White b should be ~0, got {}", lab.b);
    }

    #[test]
    fn test_lab_color_conversion_red() {
        // Red should have high L, positive a
        let lab = LabColor::from_rgb(255, 0, 0);
        assert!(lab.l > 50.0, "Red L should be > 50, got {}", lab.l);
        assert!(lab.a > 50.0, "Red a should be positive, got {}", lab.a);
    }

    #[test]
    fn test_lab_distance() {
        let black = LabColor::from_rgb(0, 0, 0);
        let white = LabColor::from_rgb(255, 255, 255);
        let dark_gray = LabColor::from_rgb(30, 30, 30);

        // Distance from black to white should be large
        let bw_dist = black.distance(&white);
        assert!(bw_dist > 90.0, "Black-white distance should be large, got {}", bw_dist);

        // Distance from black to dark gray should be small
        let bg_dist = black.distance(&dark_gray);
        assert!(bg_dist < bw_dist, "Black-gray distance should be less than black-white");
    }

    #[test]
    fn test_lab_quantize_no_quantization_needed() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 0, b: 255, a: 255 }, 10);

        let result = median_cut_quantize_lab(colors, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_lab_quantize_reduces_colors() {
        let mut colors = HashMap::new();
        // Create more colors than max
        for i in 0..20 {
            colors.insert(Color { r: i * 10, g: i * 5, b: i * 2, a: 255 }, 1);
        }

        let result = median_cut_quantize_lab(colors, 4);
        assert!(result.len() <= 4);
    }

    #[test]
    fn test_lab_quantize_preserves_transparent() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 0, g: 0, b: 0, a: 0 }, 10); // Transparent
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);

        let result = median_cut_quantize_lab(colors, 3);
        assert!(result.iter().any(|c| c.is_transparent()));
    }

    #[test]
    fn test_lab_skin_tone_grouping() {
        // Test that similar skin tones are grouped together in LAB space
        let skin_light = LabColor::from_rgb(255, 220, 185);  // Light skin
        let skin_medium = LabColor::from_rgb(210, 160, 120); // Medium skin
        let _skin_dark = LabColor::from_rgb(140, 90, 60);    // Dark skin
        let pure_red = LabColor::from_rgb(255, 0, 0);        // Pure red

        // Skin tones should be closer to each other than to pure red
        let light_to_medium = skin_light.distance(&skin_medium);
        let light_to_red = skin_light.distance(&pure_red);

        assert!(light_to_medium < light_to_red,
            "Skin tones should be closer to each other than to pure red");
    }

    #[test]
    fn test_find_closest_color() {
        let palette =
            vec![Color { r: 0, g: 0, b: 0, a: 255 }, Color { r: 255, g: 255, b: 255, a: 255 }];

        let dark = Color { r: 30, g: 30, b: 30, a: 255 };
        let light = Color { r: 200, g: 200, b: 200, a: 255 };

        assert_eq!(find_closest_color(dark, &palette), 0);
        assert_eq!(find_closest_color(light, &palette), 1);
    }
}
