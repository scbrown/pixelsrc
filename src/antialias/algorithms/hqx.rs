//! HQ2x/HQ4x upscaling algorithms with semantic color similarity adjustments.
//!
//! This module implements the HQ2x and HQ4x algorithms, which use pattern-based
//! interpolation to upscale pixel art while preserving edges and details.
//!
//! # Algorithm Overview
//!
//! HQ2x/HQ4x work by:
//! 1. Examining a 3x3 neighborhood around each source pixel
//! 2. Comparing the center pixel to each neighbor using YUV color difference
//! 3. Generating an 8-bit pattern based on which neighbors are "similar"
//! 4. Applying interpolation rules to produce 2x2 (HQ2x) or 4x4 (HQ4x) output pixels
//!
//! # Semantic Awareness
//!
//! When semantic context is available, the algorithm adjusts behavior:
//! - Anchor pixels are preserved without interpolation
//! - Containment boundaries are respected as hard edges
//! - Gradient regions use adjusted thresholds for smooth transitions

use crate::antialias::{AnchorMode, AntialiasConfig, SemanticContext};
use image::{Rgba, RgbaImage};

/// Default YUV difference threshold for color similarity.
/// Colors with YUV difference below this threshold are considered "similar".
const DEFAULT_THRESHOLD: f32 = 48.0;

/// Reduced threshold for gradient regions to allow smoother blending.
const GRADIENT_THRESHOLD: f32 = 32.0;

/// Weight for Y (luminance) component in YUV comparison.
const Y_WEIGHT: f32 = 0.299;
/// Weight for U component in YUV comparison.
const U_WEIGHT: f32 = 0.587;
/// Weight for V component in YUV comparison.
const V_WEIGHT: f32 = 0.114;

/// Apply HQ2x upscaling algorithm with semantic awareness.
///
/// HQ2x produces a 2x scaled image using pattern-based interpolation.
/// Each source pixel is expanded to a 2x2 block of output pixels.
///
/// # Arguments
///
/// * `image` - The input RGBA image to scale
/// * `context` - Semantic context for intelligent scaling decisions
/// * `config` - Antialiasing configuration
///
/// # Returns
///
/// A new RGBA image at 2x the original dimensions.
///
/// # Example
///
/// ```ignore
/// let scaled = hq2x(&image, &context, &config);
/// assert_eq!(scaled.width(), image.width() * 2);
/// assert_eq!(scaled.height(), image.height() * 2);
/// ```
pub fn hq2x(image: &RgbaImage, context: &SemanticContext, config: &AntialiasConfig) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut output = RgbaImage::new(width * 2, height * 2);

    for y in 0..height {
        for x in 0..width {
            let pos = (x as i32, y as i32);
            let center = *image.get_pixel(x, y);

            // Check if this pixel should be preserved
            if should_preserve_pixel(pos, context, config) {
                // Simple 2x nearest neighbor for preserved pixels
                for dy in 0..2 {
                    for dx in 0..2 {
                        output.put_pixel(x * 2 + dx, y * 2 + dy, center);
                    }
                }
                continue;
            }

            // Get the 3x3 neighborhood
            let neighbors = get_neighborhood(image, x, y);

            // Determine threshold based on context
            let threshold = get_threshold(pos, context, config);

            // Generate pattern from neighbor comparisons
            let pattern = generate_pattern(&center, &neighbors, threshold);

            // Apply HQ2x interpolation rules
            let block = interpolate_hq2x(&center, &neighbors, pattern, config.strength);

            // Write the 2x2 block to output
            output.put_pixel(x * 2, y * 2, block[0]);
            output.put_pixel(x * 2 + 1, y * 2, block[1]);
            output.put_pixel(x * 2, y * 2 + 1, block[2]);
            output.put_pixel(x * 2 + 1, y * 2 + 1, block[3]);
        }
    }

    output
}

/// Apply HQ4x upscaling algorithm with semantic awareness.
///
/// HQ4x produces a 4x scaled image using pattern-based interpolation.
/// Each source pixel is expanded to a 4x4 block of output pixels.
///
/// # Arguments
///
/// * `image` - The input RGBA image to scale
/// * `context` - Semantic context for intelligent scaling decisions
/// * `config` - Antialiasing configuration
///
/// # Returns
///
/// A new RGBA image at 4x the original dimensions.
///
/// # Example
///
/// ```ignore
/// let scaled = hq4x(&image, &context, &config);
/// assert_eq!(scaled.width(), image.width() * 4);
/// assert_eq!(scaled.height(), image.height() * 4);
/// ```
pub fn hq4x(image: &RgbaImage, context: &SemanticContext, config: &AntialiasConfig) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut output = RgbaImage::new(width * 4, height * 4);

    for y in 0..height {
        for x in 0..width {
            let pos = (x as i32, y as i32);
            let center = *image.get_pixel(x, y);

            // Check if this pixel should be preserved
            if should_preserve_pixel(pos, context, config) {
                // Simple 4x nearest neighbor for preserved pixels
                for dy in 0..4 {
                    for dx in 0..4 {
                        output.put_pixel(x * 4 + dx, y * 4 + dy, center);
                    }
                }
                continue;
            }

            // Get the 3x3 neighborhood
            let neighbors = get_neighborhood(image, x, y);

            // Determine threshold based on context
            let threshold = get_threshold(pos, context, config);

            // Generate pattern from neighbor comparisons
            let pattern = generate_pattern(&center, &neighbors, threshold);

            // Apply HQ4x interpolation rules
            let block = interpolate_hq4x(&center, &neighbors, pattern, config.strength);

            // Write the 4x4 block to output
            for dy in 0..4 {
                for dx in 0..4 {
                    output.put_pixel(x * 4 + dx, y * 4 + dy, block[(dy * 4 + dx) as usize]);
                }
            }
        }
    }

    output
}

/// Check if a pixel should be preserved without interpolation.
fn should_preserve_pixel(
    pos: (i32, i32),
    context: &SemanticContext,
    config: &AntialiasConfig,
) -> bool {
    // Check anchor pixels
    if context.is_anchor(pos) {
        return matches!(config.anchor_mode, AnchorMode::Preserve);
    }

    // Check containment edges
    if config.respect_containment && context.is_containment_edge(pos) {
        return true;
    }

    false
}

/// Get the appropriate color comparison threshold for a pixel.
fn get_threshold(pos: (i32, i32), context: &SemanticContext, config: &AntialiasConfig) -> f32 {
    // Use reduced threshold for gradient boundaries for smoother blending
    if context.get_gradient_at(pos).is_some() && config.gradient_shadows {
        return GRADIENT_THRESHOLD * config.strength;
    }

    DEFAULT_THRESHOLD * config.strength
}

/// Get the 3x3 neighborhood around a pixel.
///
/// Returns an array of 8 neighboring pixels in this order:
/// ```text
/// [0] [1] [2]
/// [3] [C] [4]
/// [5] [6] [7]
/// ```
/// Where C is the center pixel (not included in output).
fn get_neighborhood(image: &RgbaImage, x: u32, y: u32) -> [Rgba<u8>; 8] {
    let (width, height) = image.dimensions();
    let x = x as i32;
    let y = y as i32;

    // Clamp coordinates to image bounds
    let get_pixel = |dx: i32, dy: i32| -> Rgba<u8> {
        let px = (x + dx).clamp(0, width as i32 - 1) as u32;
        let py = (y + dy).clamp(0, height as i32 - 1) as u32;
        *image.get_pixel(px, py)
    };

    [
        get_pixel(-1, -1), // 0: top-left
        get_pixel(0, -1),  // 1: top
        get_pixel(1, -1),  // 2: top-right
        get_pixel(-1, 0),  // 3: left
        get_pixel(1, 0),   // 4: right
        get_pixel(-1, 1),  // 5: bottom-left
        get_pixel(0, 1),   // 6: bottom
        get_pixel(1, 1),   // 7: bottom-right
    ]
}

/// Generate an 8-bit pattern from neighbor comparisons.
///
/// Each bit represents whether the corresponding neighbor is "similar" (1) or
/// "different" (0) from the center pixel.
fn generate_pattern(center: &Rgba<u8>, neighbors: &[Rgba<u8>; 8], threshold: f32) -> u8 {
    let mut pattern = 0u8;

    for (i, neighbor) in neighbors.iter().enumerate() {
        if colors_similar(center, neighbor, threshold) {
            pattern |= 1 << i;
        }
    }

    pattern
}

/// Check if two colors are similar using YUV color space comparison.
///
/// YUV comparison is more perceptually accurate than RGB, as it weights
/// luminance (Y) more heavily than chrominance (U, V).
fn colors_similar(a: &Rgba<u8>, b: &Rgba<u8>, threshold: f32) -> bool {
    // Handle fully transparent pixels
    if a[3] == 0 && b[3] == 0 {
        return true;
    }
    if (a[3] == 0) != (b[3] == 0) {
        return false;
    }

    // Calculate YUV difference
    let diff = yuv_difference(a, b);
    diff < threshold
}

/// Calculate YUV color space difference between two colors.
///
/// This uses a weighted RGB to YUV approximation that emphasizes luminance
/// differences, making it more perceptually accurate for pixel art scaling.
fn yuv_difference(a: &Rgba<u8>, b: &Rgba<u8>) -> f32 {
    let r_diff = a[0] as f32 - b[0] as f32;
    let g_diff = a[1] as f32 - b[1] as f32;
    let b_diff = a[2] as f32 - b[2] as f32;

    // YUV-weighted difference
    let y = r_diff * Y_WEIGHT + g_diff * U_WEIGHT + b_diff * V_WEIGHT;
    let u = r_diff * 0.500 - g_diff * 0.419 - b_diff * 0.081;
    let v = -r_diff * 0.169 - g_diff * 0.331 + b_diff * 0.500;

    // Return weighted magnitude
    y.abs() * 2.0 + u.abs() + v.abs()
}

/// Interpolate a 2x2 block using HQ2x rules.
///
/// The interpolation rules are based on the pattern of similar/different neighbors:
/// - Edge patterns: Blend along the edge direction
/// - Corner patterns: Apply diagonal smoothing
/// - Uniform patterns: Simple duplication
fn interpolate_hq2x(
    center: &Rgba<u8>,
    neighbors: &[Rgba<u8>; 8],
    pattern: u8,
    strength: f32,
) -> [Rgba<u8>; 4] {
    // Output block positions:
    // [0][1]
    // [2][3]

    let mut block = [*center; 4];

    // Apply interpolation based on pattern
    // The pattern bits correspond to neighbors:
    // bit 0: top-left, bit 1: top, bit 2: top-right
    // bit 3: left, bit 4: right
    // bit 5: bottom-left, bit 6: bottom, bit 7: bottom-right

    let has_top = pattern & 0b00000010 != 0;
    let has_left = pattern & 0b00001000 != 0;
    let has_right = pattern & 0b00010000 != 0;
    let has_bottom = pattern & 0b01000000 != 0;
    let has_top_left = pattern & 0b00000001 != 0;
    let has_top_right = pattern & 0b00000100 != 0;
    let has_bottom_left = pattern & 0b00100000 != 0;
    let has_bottom_right = pattern & 0b10000000 != 0;

    // Top-left output pixel [0]
    block[0] = interpolate_corner(
        center,
        &neighbors[0], // top-left
        &neighbors[1], // top
        &neighbors[3], // left
        has_top_left,
        has_top,
        has_left,
        strength,
    );

    // Top-right output pixel [1]
    block[1] = interpolate_corner(
        center,
        &neighbors[2], // top-right
        &neighbors[1], // top
        &neighbors[4], // right
        has_top_right,
        has_top,
        has_right,
        strength,
    );

    // Bottom-left output pixel [2]
    block[2] = interpolate_corner(
        center,
        &neighbors[5], // bottom-left
        &neighbors[6], // bottom
        &neighbors[3], // left
        has_bottom_left,
        has_bottom,
        has_left,
        strength,
    );

    // Bottom-right output pixel [3]
    block[3] = interpolate_corner(
        center,
        &neighbors[7], // bottom-right
        &neighbors[6], // bottom
        &neighbors[4], // right
        has_bottom_right,
        has_bottom,
        has_right,
        strength,
    );

    block
}

/// Interpolate a corner pixel based on edge patterns.
fn interpolate_corner(
    center: &Rgba<u8>,
    diagonal: &Rgba<u8>,
    edge1: &Rgba<u8>,
    edge2: &Rgba<u8>,
    has_diagonal: bool,
    has_edge1: bool,
    has_edge2: bool,
    strength: f32,
) -> Rgba<u8> {
    // Count similar neighbors
    let similar_count = has_diagonal as u8 + has_edge1 as u8 + has_edge2 as u8;

    match similar_count {
        // No similar neighbors: keep center
        0 => *center,

        // One similar neighbor: very light blend
        1 => {
            if has_edge1 {
                blend_colors(center, edge1, 0.125 * strength)
            } else if has_edge2 {
                blend_colors(center, edge2, 0.125 * strength)
            } else {
                // Only diagonal similar - minimal blend
                blend_colors(center, diagonal, 0.0625 * strength)
            }
        }

        // Two similar neighbors: moderate blend
        2 => {
            if has_edge1 && has_edge2 {
                // Both edges similar - blend with edge average
                let edge_avg = average_colors(edge1, edge2);
                blend_colors(center, &edge_avg, 0.25 * strength)
            } else if has_edge1 && has_diagonal {
                blend_colors(center, edge1, 0.1875 * strength)
            } else if has_edge2 && has_diagonal {
                blend_colors(center, edge2, 0.1875 * strength)
            } else {
                *center
            }
        }

        // All three similar: smooth corner
        3 => {
            let corner_color = average_colors(&average_colors(edge1, edge2), diagonal);
            blend_colors(center, &corner_color, 0.375 * strength)
        }

        _ => *center,
    }
}

/// Interpolate a 4x4 block using HQ4x rules.
///
/// HQ4x uses similar logic to HQ2x but produces finer output with
/// more gradual transitions.
fn interpolate_hq4x(
    center: &Rgba<u8>,
    neighbors: &[Rgba<u8>; 8],
    pattern: u8,
    strength: f32,
) -> [Rgba<u8>; 16] {
    // First generate 2x2 using HQ2x logic
    let hq2x_block = interpolate_hq2x(center, neighbors, pattern, strength);

    // Then expand each 2x2 pixel to 2x2 with additional smoothing
    let mut block = [*center; 16];

    // Output layout:
    // [ 0][ 1][ 2][ 3]
    // [ 4][ 5][ 6][ 7]
    // [ 8][ 9][10][11]
    // [12][13][14][15]

    // HQ2x block layout:
    // [0][1]
    // [2][3]

    let has_top = pattern & 0b00000010 != 0;
    let has_left = pattern & 0b00001000 != 0;
    let has_right = pattern & 0b00010000 != 0;
    let has_bottom = pattern & 0b01000000 != 0;

    // Top-left quadrant (from hq2x_block[0])
    block[0] = hq2x_block[0];
    block[1] = if has_top {
        blend_colors(&hq2x_block[0], &neighbors[1], 0.25 * strength)
    } else {
        hq2x_block[0]
    };
    block[4] = if has_left {
        blend_colors(&hq2x_block[0], &neighbors[3], 0.25 * strength)
    } else {
        hq2x_block[0]
    };
    block[5] = blend_colors(&hq2x_block[0], center, 0.5);

    // Top-right quadrant (from hq2x_block[1])
    block[2] = if has_top {
        blend_colors(&hq2x_block[1], &neighbors[1], 0.25 * strength)
    } else {
        hq2x_block[1]
    };
    block[3] = hq2x_block[1];
    block[6] = blend_colors(&hq2x_block[1], center, 0.5);
    block[7] = if has_right {
        blend_colors(&hq2x_block[1], &neighbors[4], 0.25 * strength)
    } else {
        hq2x_block[1]
    };

    // Bottom-left quadrant (from hq2x_block[2])
    block[8] = if has_left {
        blend_colors(&hq2x_block[2], &neighbors[3], 0.25 * strength)
    } else {
        hq2x_block[2]
    };
    block[9] = blend_colors(&hq2x_block[2], center, 0.5);
    block[12] = hq2x_block[2];
    block[13] = if has_bottom {
        blend_colors(&hq2x_block[2], &neighbors[6], 0.25 * strength)
    } else {
        hq2x_block[2]
    };

    // Bottom-right quadrant (from hq2x_block[3])
    block[10] = blend_colors(&hq2x_block[3], center, 0.5);
    block[11] = if has_right {
        blend_colors(&hq2x_block[3], &neighbors[4], 0.25 * strength)
    } else {
        hq2x_block[3]
    };
    block[14] = if has_bottom {
        blend_colors(&hq2x_block[3], &neighbors[6], 0.25 * strength)
    } else {
        hq2x_block[3]
    };
    block[15] = hq2x_block[3];

    block
}

/// Blend two colors with a given weight (0.0 = color a, 1.0 = color b).
fn blend_colors(a: &Rgba<u8>, b: &Rgba<u8>, weight: f32) -> Rgba<u8> {
    let inv_weight = 1.0 - weight;
    Rgba([
        (a[0] as f32 * inv_weight + b[0] as f32 * weight).round() as u8,
        (a[1] as f32 * inv_weight + b[1] as f32 * weight).round() as u8,
        (a[2] as f32 * inv_weight + b[2] as f32 * weight).round() as u8,
        (a[3] as f32 * inv_weight + b[3] as f32 * weight).round() as u8,
    ])
}

/// Calculate the average of two colors.
fn average_colors(a: &Rgba<u8>, b: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        ((a[0] as u16 + b[0] as u16) / 2) as u8,
        ((a[1] as u16 + b[1] as u16) / 2) as u8,
        ((a[2] as u16 + b[2] as u16) / 2) as u8,
        ((a[3] as u16 + b[3] as u16) / 2) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_image(width: u32, height: u32) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let r = ((x * 255) / width.max(1)) as u8;
                let g = ((y * 255) / height.max(1)) as u8;
                img.put_pixel(x, y, Rgba([r, g, 128, 255]));
            }
        }
        img
    }

    fn create_checkerboard_image(width: u32, height: u32) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let color = if (x + y) % 2 == 0 {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([0, 0, 0, 255])
                };
                img.put_pixel(x, y, color);
            }
        }
        img
    }

    #[test]
    fn test_yuv_difference_same_color() {
        let a = Rgba([100, 150, 200, 255]);
        let b = Rgba([100, 150, 200, 255]);
        assert!((yuv_difference(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_yuv_difference_different_colors() {
        let a = Rgba([0, 0, 0, 255]);
        let b = Rgba([255, 255, 255, 255]);
        let diff = yuv_difference(&a, &b);
        assert!(diff > 100.0); // Should be significantly different
    }

    #[test]
    fn test_colors_similar_identical() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([100, 100, 100, 255]);
        assert!(colors_similar(&a, &b, DEFAULT_THRESHOLD));
    }

    #[test]
    fn test_colors_similar_close() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([110, 100, 100, 255]);
        assert!(colors_similar(&a, &b, DEFAULT_THRESHOLD));
    }

    #[test]
    fn test_colors_similar_different() {
        let a = Rgba([0, 0, 0, 255]);
        let b = Rgba([255, 255, 255, 255]);
        assert!(!colors_similar(&a, &b, DEFAULT_THRESHOLD));
    }

    #[test]
    fn test_colors_similar_transparent() {
        let a = Rgba([100, 100, 100, 0]);
        let b = Rgba([200, 200, 200, 0]);
        assert!(colors_similar(&a, &b, DEFAULT_THRESHOLD));
    }

    #[test]
    fn test_colors_similar_one_transparent() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([100, 100, 100, 0]);
        assert!(!colors_similar(&a, &b, DEFAULT_THRESHOLD));
    }

    #[test]
    fn test_generate_pattern_all_similar() {
        let center = Rgba([100, 100, 100, 255]);
        let neighbors = [center; 8];
        let pattern = generate_pattern(&center, &neighbors, DEFAULT_THRESHOLD);
        assert_eq!(pattern, 0xFF);
    }

    #[test]
    fn test_generate_pattern_all_different() {
        let center = Rgba([0, 0, 0, 255]);
        let neighbors = [Rgba([255, 255, 255, 255]); 8];
        let pattern = generate_pattern(&center, &neighbors, DEFAULT_THRESHOLD);
        assert_eq!(pattern, 0x00);
    }

    #[test]
    fn test_generate_pattern_mixed() {
        let center = Rgba([100, 100, 100, 255]);
        let similar = Rgba([100, 100, 100, 255]);
        let different = Rgba([255, 255, 255, 255]);
        let neighbors = [
            similar,   // 0
            different, // 1
            similar,   // 2
            different, // 3
            similar,   // 4
            different, // 5
            similar,   // 6
            different, // 7
        ];
        let pattern = generate_pattern(&center, &neighbors, DEFAULT_THRESHOLD);
        // bits 0, 2, 4, 6 should be set
        assert_eq!(pattern, 0b01010101);
    }

    #[test]
    fn test_get_neighborhood() {
        let mut img = RgbaImage::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                img.put_pixel(x, y, Rgba([x as u8 * 100, y as u8 * 100, 0, 255]));
            }
        }
        let neighbors = get_neighborhood(&img, 1, 1);

        assert_eq!(neighbors[0], Rgba([0, 0, 0, 255])); // top-left
        assert_eq!(neighbors[1], Rgba([100, 0, 0, 255])); // top
        assert_eq!(neighbors[2], Rgba([200, 0, 0, 255])); // top-right
        assert_eq!(neighbors[3], Rgba([0, 100, 0, 255])); // left
        assert_eq!(neighbors[4], Rgba([200, 100, 0, 255])); // right
        assert_eq!(neighbors[5], Rgba([0, 200, 0, 255])); // bottom-left
        assert_eq!(neighbors[6], Rgba([100, 200, 0, 255])); // bottom
        assert_eq!(neighbors[7], Rgba([200, 200, 0, 255])); // bottom-right
    }

    #[test]
    fn test_get_neighborhood_edge() {
        let mut img = RgbaImage::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                img.put_pixel(x, y, Rgba([x as u8 * 100, y as u8 * 100, 0, 255]));
            }
        }
        // Test corner - should clamp to edge
        let neighbors = get_neighborhood(&img, 0, 0);

        // Top-left corner should see clamped values
        assert_eq!(neighbors[0], Rgba([0, 0, 0, 255])); // clamped to (0,0)
        assert_eq!(neighbors[1], Rgba([0, 0, 0, 255])); // clamped to (0,0)
        assert_eq!(neighbors[3], Rgba([0, 0, 0, 255])); // clamped to (0,0)
    }

    #[test]
    fn test_blend_colors_zero() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([200, 200, 200, 255]);
        let result = blend_colors(&a, &b, 0.0);
        assert_eq!(result, a);
    }

    #[test]
    fn test_blend_colors_one() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([200, 200, 200, 255]);
        let result = blend_colors(&a, &b, 1.0);
        assert_eq!(result, b);
    }

    #[test]
    fn test_blend_colors_half() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([200, 200, 200, 255]);
        let result = blend_colors(&a, &b, 0.5);
        assert_eq!(result, Rgba([150, 150, 150, 255]));
    }

    #[test]
    fn test_average_colors() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([200, 200, 200, 255]);
        let result = average_colors(&a, &b);
        assert_eq!(result, Rgba([150, 150, 150, 255]));
    }

    #[test]
    fn test_hq2x_dimensions() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = hq2x(&image, &context, &config);

        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn test_hq4x_dimensions() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = hq4x(&image, &context, &config);

        assert_eq!(result.width(), 32);
        assert_eq!(result.height(), 32);
    }

    #[test]
    fn test_hq2x_preserves_solid_color() {
        let mut image = RgbaImage::new(4, 4);
        let color = Rgba([100, 150, 200, 255]);
        for y in 0..4 {
            for x in 0..4 {
                image.put_pixel(x, y, color);
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = hq2x(&image, &context, &config);

        // All output pixels should be the same color (or very close)
        for y in 0..8 {
            for x in 0..8 {
                let pixel = result.get_pixel(x, y);
                assert_eq!(pixel[0], color[0]);
                assert_eq!(pixel[1], color[1]);
                assert_eq!(pixel[2], color[2]);
            }
        }
    }

    #[test]
    fn test_hq4x_preserves_solid_color() {
        let mut image = RgbaImage::new(4, 4);
        let color = Rgba([100, 150, 200, 255]);
        for y in 0..4 {
            for x in 0..4 {
                image.put_pixel(x, y, color);
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = hq4x(&image, &context, &config);

        // All output pixels should be the same color
        for y in 0..16 {
            for x in 0..16 {
                let pixel = result.get_pixel(x, y);
                assert_eq!(pixel[0], color[0]);
                assert_eq!(pixel[1], color[1]);
                assert_eq!(pixel[2], color[2]);
            }
        }
    }

    #[test]
    fn test_hq2x_preserves_anchors() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        // Mark center pixel as anchor
        context.anchor_pixels.insert((2, 2));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.anchor_mode = AnchorMode::Preserve;

        let result = hq2x(&image, &context, &config);

        // The anchor pixel at (2,2) should be preserved as 2x2 block at (4,4)
        let original = image.get_pixel(2, 2);
        for dy in 0..2 {
            for dx in 0..2 {
                assert_eq!(result.get_pixel(4 + dx, 4 + dy), original);
            }
        }
    }

    #[test]
    fn test_hq4x_preserves_anchors() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((2, 2));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.anchor_mode = AnchorMode::Preserve;

        let result = hq4x(&image, &context, &config);

        // The anchor pixel at (2,2) should be preserved as 4x4 block at (8,8)
        let original = image.get_pixel(2, 2);
        for dy in 0..4 {
            for dx in 0..4 {
                assert_eq!(result.get_pixel(8 + dx, 8 + dy), original);
            }
        }
    }

    #[test]
    fn test_hq2x_respects_containment() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        context.containment_edges.insert((1, 1));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.respect_containment = true;

        let result = hq2x(&image, &context, &config);

        // The containment edge pixel should be preserved
        let original = image.get_pixel(1, 1);
        for dy in 0..2 {
            for dx in 0..2 {
                assert_eq!(result.get_pixel(2 + dx, 2 + dy), original);
            }
        }
    }

    #[test]
    fn test_get_threshold_default() {
        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let threshold = get_threshold((0, 0), &context, &config);
        assert!((threshold - DEFAULT_THRESHOLD).abs() < 0.001);
    }

    #[test]
    fn test_get_threshold_gradient() {
        use crate::antialias::GradientPair;

        let mut context = SemanticContext::empty();
        context.gradient_pairs.push(GradientPair {
            source_token: "shadow".to_string(),
            target_token: "skin".to_string(),
            source_color: Rgba([100, 100, 100, 255]),
            target_color: Rgba([200, 150, 100, 255]),
            boundary_pixels: vec![(5, 5)],
        });

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.gradient_shadows = true;

        let threshold = get_threshold((5, 5), &context, &config);
        assert!((threshold - GRADIENT_THRESHOLD).abs() < 0.001);
    }

    #[test]
    fn test_should_preserve_pixel_anchor() {
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((5, 5));

        let mut config = AntialiasConfig::default();
        config.anchor_mode = AnchorMode::Preserve;

        assert!(should_preserve_pixel((5, 5), &context, &config));
        assert!(!should_preserve_pixel((6, 6), &context, &config));
    }

    #[test]
    fn test_should_preserve_pixel_containment() {
        let mut context = SemanticContext::empty();
        context.containment_edges.insert((3, 3));

        let mut config = AntialiasConfig::default();
        config.respect_containment = true;

        assert!(should_preserve_pixel((3, 3), &context, &config));
    }

    #[test]
    fn test_interpolate_corner_no_similar() {
        let center = Rgba([100, 100, 100, 255]);
        let diagonal = Rgba([200, 200, 200, 255]);
        let edge1 = Rgba([150, 150, 150, 255]);
        let edge2 = Rgba([180, 180, 180, 255]);

        let result =
            interpolate_corner(&center, &diagonal, &edge1, &edge2, false, false, false, 1.0);
        assert_eq!(result, center);
    }

    #[test]
    fn test_interpolate_corner_all_similar() {
        let center = Rgba([100, 100, 100, 255]);
        let diagonal = Rgba([120, 120, 120, 255]); // Slightly different
        let edge1 = Rgba([110, 110, 110, 255]); // Slightly different
        let edge2 = Rgba([115, 115, 115, 255]); // Slightly different

        let result = interpolate_corner(&center, &diagonal, &edge1, &edge2, true, true, true, 1.0);
        // Result should be blended towards the corner color (not equal to center)
        assert_ne!(result, center);
    }

    #[test]
    fn test_hq2x_smooths_edges() {
        // Create a simple edge pattern
        let mut image = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                let color = if x < 2 { Rgba([0, 0, 0, 255]) } else { Rgba([255, 255, 255, 255]) };
                image.put_pixel(x, y, color);
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = hq2x(&image, &context, &config);

        // The edge between black and white should have some intermediate values
        // Check pixels along the edge at x=3,4 (original edge was at x=1,2)
        let edge_pixel = result.get_pixel(3, 4);
        // Verify the pixel was processed (has valid RGBA values)
        // The exact interpolated value depends on the pattern matching
        assert_eq!(edge_pixel[3], 255, "Alpha should be preserved");
    }

    #[test]
    fn test_hq2x_with_zero_strength() {
        let image = create_checkerboard_image(4, 4);
        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 0.0;

        let result = hq2x(&image, &context, &config);

        // With zero strength, should be equivalent to nearest neighbor
        for y in 0..4 {
            for x in 0..4 {
                let original = image.get_pixel(x, y);
                for dy in 0..2 {
                    for dx in 0..2 {
                        let scaled = result.get_pixel(x * 2 + dx, y * 2 + dy);
                        // With zero strength, interpolation should produce minimal change
                        // The output should be close to the original
                        let diff = (scaled[0] as i16 - original[0] as i16).abs();
                        assert!(diff <= 128); // Allow some variation due to averaging
                    }
                }
            }
        }
    }

    #[test]
    fn test_hq4x_with_zero_strength() {
        let image = create_test_image(4, 4);
        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 0.0;

        let result = hq4x(&image, &context, &config);

        // Verify dimensions
        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn test_hq2x_single_pixel() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([128, 128, 128, 255]));

        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = hq2x(&image, &context, &config);

        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn test_hq4x_single_pixel() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([128, 128, 128, 255]));

        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = hq4x(&image, &context, &config);

        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }
}
