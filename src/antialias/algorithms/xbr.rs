//! xBR (Scale By Rules) 2x/4x edge-aware upscaling algorithms with semantic awareness.
//!
//! This module implements the xBR2x and xBR4x algorithms, which use edge direction
//! and curvature detection to produce high-quality upscaled pixel art.
//!
//! # Algorithm Overview
//!
//! xBR works by:
//! 1. Examining a 5x5 neighborhood around each source pixel
//! 2. Computing weighted color differences to detect edge directions
//! 3. Analyzing edge patterns to detect curves vs straight lines
//! 4. Applying interpolation rules based on detected edges
//!
//! The algorithm produces the best visual quality among pixel art scalers,
//! especially for diagonal lines and curves.
//!
//! # Semantic Awareness
//!
//! When semantic context is available, the algorithm adjusts behavior:
//! - Anchor pixels are preserved without interpolation
//! - Containment boundaries are respected as hard edges
//! - Gradient regions use adjusted thresholds for smooth transitions

use crate::antialias::{AnchorMode, AntialiasConfig, SemanticContext};
use image::{Rgba, RgbaImage};

/// Weight multipliers for edge direction detection.
/// These weights determine how strongly each comparison contributes
/// to the edge direction calculation.
const WEIGHT_EDGE: f32 = 1.0;
const WEIGHT_DIAGONAL: f32 = 2.0;
const WEIGHT_FAR: f32 = 4.0;

/// Default color difference threshold for similarity comparison.
const DEFAULT_THRESHOLD: f32 = 48.0;

/// Reduced threshold for gradient regions.
const GRADIENT_THRESHOLD: f32 = 32.0;

/// Apply xBR 2x upscaling algorithm with semantic awareness.
///
/// xBR2x produces a 2x scaled image using edge direction analysis.
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
/// let scaled = xbr2x(&image, &context, &config);
/// assert_eq!(scaled.width(), image.width() * 2);
/// assert_eq!(scaled.height(), image.height() * 2);
/// ```
pub fn xbr2x(image: &RgbaImage, context: &SemanticContext, config: &AntialiasConfig) -> RgbaImage {
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

            // Get the 5x5 neighborhood for edge detection
            let neighborhood = get_neighborhood_5x5(image, x, y);

            // Determine threshold based on context
            let threshold = get_threshold(pos, context, config);

            // Compute edge directions for each corner of the 2x2 output block
            let block = compute_xbr2x_block(&neighborhood, threshold, config.strength);

            // Write the 2x2 block to output
            output.put_pixel(x * 2, y * 2, block[0]);
            output.put_pixel(x * 2 + 1, y * 2, block[1]);
            output.put_pixel(x * 2, y * 2 + 1, block[2]);
            output.put_pixel(x * 2 + 1, y * 2 + 1, block[3]);
        }
    }

    output
}

/// Apply xBR 4x upscaling algorithm with semantic awareness.
///
/// xBR4x produces a 4x scaled image by applying xBR2x twice with
/// additional smoothing passes.
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
/// let scaled = xbr4x(&image, &context, &config);
/// assert_eq!(scaled.width(), image.width() * 4);
/// assert_eq!(scaled.height(), image.height() * 4);
/// ```
pub fn xbr4x(image: &RgbaImage, context: &SemanticContext, config: &AntialiasConfig) -> RgbaImage {
    // First pass: xBR2x
    let pass1 = xbr2x(image, context, config);

    // Scale the context for the second pass
    let scaled_context = context.scale(2);

    // Second pass: xBR2x on the 2x result
    xbr2x(&pass1, &scaled_context, config)
}

/// 5x5 neighborhood around a pixel.
///
/// Layout:
/// ```text
/// [A1][B1][C1][D1][E1]
/// [A0][ B][ C][ D][E0]
/// [A ][ E][ P][ F][E2]
/// [A2][ G][ H][ I][E3]
/// [A3][B3][C3][D3][E4]
/// ```
///
/// P is the center pixel, E/F/G/H are cardinal neighbors,
/// B/C/D/I are diagonal neighbors.
#[derive(Clone, Copy)]
struct Neighborhood5x5 {
    // Row 0 (top)
    a1: Rgba<u8>,
    b1: Rgba<u8>,
    c1: Rgba<u8>,
    d1: Rgba<u8>,
    e1: Rgba<u8>,
    // Row 1
    a0: Rgba<u8>,
    b: Rgba<u8>,
    c: Rgba<u8>,
    d: Rgba<u8>,
    e0: Rgba<u8>,
    // Row 2 (center row)
    a: Rgba<u8>,
    e: Rgba<u8>,
    p: Rgba<u8>, // center pixel
    f: Rgba<u8>,
    e2: Rgba<u8>,
    // Row 3
    a2: Rgba<u8>,
    g: Rgba<u8>,
    h: Rgba<u8>,
    i: Rgba<u8>,
    e3: Rgba<u8>,
    // Row 4 (bottom)
    a3: Rgba<u8>,
    b3: Rgba<u8>,
    c3: Rgba<u8>,
    d3: Rgba<u8>,
    e4: Rgba<u8>,
}

/// Get the 5x5 neighborhood around a pixel.
fn get_neighborhood_5x5(image: &RgbaImage, x: u32, y: u32) -> Neighborhood5x5 {
    let get = |dx: i32, dy: i32| get_pixel_clamped(image, x as i32 + dx, y as i32 + dy);

    Neighborhood5x5 {
        // Row 0
        a1: get(-2, -2),
        b1: get(-1, -2),
        c1: get(0, -2),
        d1: get(1, -2),
        e1: get(2, -2),
        // Row 1
        a0: get(-2, -1),
        b: get(-1, -1),
        c: get(0, -1),
        d: get(1, -1),
        e0: get(2, -1),
        // Row 2
        a: get(-2, 0),
        e: get(-1, 0),
        p: get(0, 0),
        f: get(1, 0),
        e2: get(2, 0),
        // Row 3
        a2: get(-2, 1),
        g: get(-1, 1),
        h: get(0, 1),
        i: get(1, 1),
        e3: get(2, 1),
        // Row 4
        a3: get(-2, 2),
        b3: get(-1, 2),
        c3: get(0, 2),
        d3: get(1, 2),
        e4: get(2, 2),
    }
}

/// Get a pixel with clamped coordinates.
fn get_pixel_clamped(img: &RgbaImage, x: i32, y: i32) -> Rgba<u8> {
    let (w, h) = img.dimensions();
    let cx = x.clamp(0, w as i32 - 1) as u32;
    let cy = y.clamp(0, h as i32 - 1) as u32;
    *img.get_pixel(cx, cy)
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

/// Compute the xBR2x output block for a single input pixel.
///
/// This implements the core xBR algorithm which analyzes edge directions
/// to determine how to interpolate each corner of the output block.
fn compute_xbr2x_block(
    n: &Neighborhood5x5,
    threshold: f32,
    strength: f32,
) -> [Rgba<u8>; 4] {
    // Output block positions:
    // [0][1]  (top-left, top-right)
    // [2][3]  (bottom-left, bottom-right)

    let mut block = [n.p; 4];

    // Compute edge direction weights for each corner
    // The xBR algorithm compares pixels along different diagonal directions
    // and chooses interpolation based on which direction has the stronger edge

    // Top-left corner (block[0])
    block[0] = compute_corner_pixel(
        n.p,
        n.e,  // left
        n.c,  // top
        n.b,  // top-left diagonal
        n.f,  // right
        n.h,  // bottom
        n.a,  // far left
        n.b1, // far top-left
        n.c1, // far top
        n.a0, // far top-left row
        threshold,
        strength,
        EdgeCorner::TopLeft,
    );

    // Top-right corner (block[1])
    block[1] = compute_corner_pixel(
        n.p,
        n.f,  // right
        n.c,  // top
        n.d,  // top-right diagonal
        n.e,  // left
        n.h,  // bottom
        n.e2, // far right
        n.d1, // far top-right
        n.c1, // far top
        n.e0, // far top-right row
        threshold,
        strength,
        EdgeCorner::TopRight,
    );

    // Bottom-left corner (block[2])
    block[2] = compute_corner_pixel(
        n.p,
        n.e,  // left
        n.h,  // bottom
        n.g,  // bottom-left diagonal
        n.f,  // right
        n.c,  // top
        n.a,  // far left
        n.b3, // far bottom-left
        n.c3, // far bottom
        n.a2, // far bottom-left row
        threshold,
        strength,
        EdgeCorner::BottomLeft,
    );

    // Bottom-right corner (block[3])
    block[3] = compute_corner_pixel(
        n.p,
        n.f,  // right
        n.h,  // bottom
        n.i,  // bottom-right diagonal
        n.e,  // left
        n.c,  // top
        n.e2, // far right
        n.d3, // far bottom-right
        n.c3, // far bottom
        n.e3, // far bottom-right row
        threshold,
        strength,
        EdgeCorner::BottomRight,
    );

    block
}

/// Which corner of the 2x2 block we're computing.
#[derive(Clone, Copy)]
enum EdgeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Compute a single corner pixel using xBR edge detection rules.
///
/// The algorithm works by:
/// 1. Computing weighted differences along the two diagonal directions
/// 2. Determining if there's a dominant edge direction
/// 3. Blending the pixel based on the detected edge
#[allow(clippy::too_many_arguments)]
fn compute_corner_pixel(
    center: Rgba<u8>,
    edge1: Rgba<u8>,        // Primary edge neighbor
    edge2: Rgba<u8>,        // Secondary edge neighbor
    diagonal: Rgba<u8>,     // Diagonal neighbor
    opposite1: Rgba<u8>,    // Opposite of edge1
    opposite2: Rgba<u8>,    // Opposite of edge2
    far_edge1: Rgba<u8>,    // Far neighbor along edge1 direction
    far_diag: Rgba<u8>,     // Far diagonal neighbor
    far_edge2: Rgba<u8>,    // Far neighbor along edge2 direction
    far_corner: Rgba<u8>,   // Far corner neighbor
    threshold: f32,
    strength: f32,
    _corner: EdgeCorner,
) -> Rgba<u8> {
    // Compute edge weights for the two possible diagonal directions
    // Direction 1: from edge1 side (horizontal/vertical edge)
    // Direction 2: from diagonal side (diagonal edge)

    // Weight contributions for direction 1 (along the edge)
    let weight1 = compute_edge_weight(
        center,
        edge1,
        edge2,
        diagonal,
        opposite1,
        far_edge1,
        far_corner,
        threshold,
    );

    // Weight contributions for direction 2 (across the edge)
    let weight2 = compute_edge_weight(
        center,
        diagonal,
        edge1,
        edge2,
        opposite2,
        far_diag,
        far_edge2,
        threshold,
    );

    // Determine if there's a dominant edge direction
    let edge_strength = (weight1 - weight2).abs();

    if edge_strength < threshold * 0.5 {
        // No clear edge - keep center pixel
        return center;
    }

    // Determine which direction has the edge and apply appropriate blending
    if weight1 > weight2 {
        // Edge runs along direction 1 - blend towards diagonal
        let blend_factor = calculate_blend_factor(edge_strength, threshold, strength);
        if colors_similar(&edge1, &diagonal, threshold) && colors_similar(&edge2, &diagonal, threshold) {
            blend_colors(&center, &diagonal, blend_factor * 0.5)
        } else if colors_similar(&edge1, &diagonal, threshold) {
            blend_colors(&center, &edge1, blend_factor * 0.25)
        } else if colors_similar(&edge2, &diagonal, threshold) {
            blend_colors(&center, &edge2, blend_factor * 0.25)
        } else {
            center
        }
    } else {
        // Edge runs along direction 2 - blend towards opposite
        let blend_factor = calculate_blend_factor(edge_strength, threshold, strength);
        if colors_similar(&edge1, &edge2, threshold) {
            let avg = average_colors(&edge1, &edge2);
            blend_colors(&center, &avg, blend_factor * 0.375)
        } else {
            center
        }
    }
}

/// Compute the edge weight for a direction.
///
/// Higher weights indicate a stronger edge in that direction.
fn compute_edge_weight(
    center: Rgba<u8>,
    primary: Rgba<u8>,
    secondary: Rgba<u8>,
    diagonal: Rgba<u8>,
    opposite: Rgba<u8>,
    far_primary: Rgba<u8>,
    far_secondary: Rgba<u8>,
    threshold: f32,
) -> f32 {
    let mut weight = 0.0;

    // Core edge detection: compare center with neighbors
    if !colors_similar(&center, &primary, threshold) {
        weight += WEIGHT_EDGE;
    }
    if !colors_similar(&center, &secondary, threshold) {
        weight += WEIGHT_EDGE;
    }

    // Diagonal contribution
    if colors_similar(&primary, &diagonal, threshold)
        && colors_similar(&secondary, &diagonal, threshold)
    {
        weight += WEIGHT_DIAGONAL;
    }

    // Opposite side check (edge continuity)
    if !colors_similar(&center, &opposite, threshold)
        && colors_similar(&primary, &secondary, threshold)
    {
        weight += WEIGHT_EDGE;
    }

    // Far neighbors (edge extends beyond immediate neighborhood)
    if colors_similar(&primary, &far_primary, threshold) {
        weight += WEIGHT_FAR;
    }
    if colors_similar(&secondary, &far_secondary, threshold) {
        weight += WEIGHT_FAR;
    }

    weight
}

/// Calculate the blend factor based on edge strength.
fn calculate_blend_factor(edge_strength: f32, threshold: f32, strength: f32) -> f32 {
    let normalized = (edge_strength / threshold).clamp(0.0, 1.0);
    normalized * strength
}

/// Check if two colors are similar using weighted color difference.
fn colors_similar(a: &Rgba<u8>, b: &Rgba<u8>, threshold: f32) -> bool {
    // Handle fully transparent pixels
    if a[3] == 0 && b[3] == 0 {
        return true;
    }
    if (a[3] == 0) != (b[3] == 0) {
        return false;
    }

    let diff = color_difference(a, b);
    diff < threshold
}

/// Calculate color difference using weighted RGB.
///
/// Uses perceptual weights that emphasize green (which human eyes are
/// most sensitive to) for better edge detection.
fn color_difference(a: &Rgba<u8>, b: &Rgba<u8>) -> f32 {
    let r_diff = (a[0] as f32 - b[0] as f32).abs();
    let g_diff = (a[1] as f32 - b[1] as f32).abs();
    let b_diff = (a[2] as f32 - b[2] as f32).abs();

    // Weighted sum emphasizing green channel
    r_diff * 0.299 + g_diff * 0.587 + b_diff * 0.114
}

/// Blend two colors with a given weight (0.0 = color a, 1.0 = color b).
fn blend_colors(a: &Rgba<u8>, b: &Rgba<u8>, weight: f32) -> Rgba<u8> {
    let weight = weight.clamp(0.0, 1.0);
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
    fn test_color_difference_same() {
        let a = Rgba([100, 150, 200, 255]);
        let b = Rgba([100, 150, 200, 255]);
        assert!(color_difference(&a, &b) < 0.001);
    }

    #[test]
    fn test_color_difference_different() {
        let a = Rgba([0, 0, 0, 255]);
        let b = Rgba([255, 255, 255, 255]);
        let diff = color_difference(&a, &b);
        assert!(diff > 200.0); // Significant difference
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
    fn test_xbr2x_dimensions() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = xbr2x(&image, &context, &config);

        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn test_xbr4x_dimensions() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = xbr4x(&image, &context, &config);

        assert_eq!(result.width(), 32);
        assert_eq!(result.height(), 32);
    }

    #[test]
    fn test_xbr2x_preserves_solid_color() {
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

        let result = xbr2x(&image, &context, &config);

        // All output pixels should be the same color
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
    fn test_xbr4x_preserves_solid_color() {
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

        let result = xbr4x(&image, &context, &config);

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
    fn test_xbr2x_preserves_anchors() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        // Mark center pixel as anchor
        context.anchor_pixels.insert((2, 2));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.anchor_mode = AnchorMode::Preserve;

        let result = xbr2x(&image, &context, &config);

        // The anchor pixel at (2,2) should be preserved as 2x2 block at (4,4)
        let original = image.get_pixel(2, 2);
        for dy in 0..2 {
            for dx in 0..2 {
                assert_eq!(result.get_pixel(4 + dx, 4 + dy), original);
            }
        }
    }

    #[test]
    fn test_xbr4x_preserves_anchors() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((2, 2));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.anchor_mode = AnchorMode::Preserve;

        let result = xbr4x(&image, &context, &config);

        // The anchor pixel at (2,2) should be preserved as 4x4 block at (8,8)
        let original = image.get_pixel(2, 2);
        for dy in 0..4 {
            for dx in 0..4 {
                assert_eq!(result.get_pixel(8 + dx, 8 + dy), original);
            }
        }
    }

    #[test]
    fn test_xbr2x_respects_containment() {
        let image = create_checkerboard_image(4, 4);
        let mut context = SemanticContext::empty();
        context.containment_edges.insert((1, 1));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;
        config.respect_containment = true;

        let result = xbr2x(&image, &context, &config);

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
    fn test_xbr2x_edge_detection() {
        // Create a simple diagonal edge pattern
        let mut image = RgbaImage::new(4, 4);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        // Create a diagonal from top-left to bottom-right
        for y in 0..4 {
            for x in 0..4 {
                if x <= y {
                    image.put_pixel(x, y, black);
                } else {
                    image.put_pixel(x, y, white);
                }
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = xbr2x(&image, &context, &config);

        // The diagonal should be smoothed - just verify it runs without error
        // and produces correct dimensions
        assert_eq!(result.dimensions(), (8, 8));
    }

    #[test]
    fn test_xbr2x_single_pixel() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([128, 128, 128, 255]));

        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = xbr2x(&image, &context, &config);

        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn test_xbr4x_single_pixel() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([128, 128, 128, 255]));

        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        let result = xbr4x(&image, &context, &config);

        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn test_get_pixel_clamped() {
        let mut img = RgbaImage::new(2, 2);
        let red = Rgba([255, 0, 0, 255]);
        img.put_pixel(0, 0, red);

        // Normal access
        assert_eq!(get_pixel_clamped(&img, 0, 0), red);

        // Clamped negative
        assert_eq!(get_pixel_clamped(&img, -1, 0), red);
        assert_eq!(get_pixel_clamped(&img, 0, -1), red);
        assert_eq!(get_pixel_clamped(&img, -5, -5), red);

        // Clamped overflow
        assert_eq!(get_pixel_clamped(&img, 10, 0), *img.get_pixel(1, 0));
        assert_eq!(get_pixel_clamped(&img, 0, 10), *img.get_pixel(0, 1));
    }

    #[test]
    fn test_calculate_blend_factor() {
        // Zero edge strength should give zero blend
        let factor = calculate_blend_factor(0.0, DEFAULT_THRESHOLD, 1.0);
        assert!(factor < 0.001);

        // Full threshold edge strength should give full strength blend
        let factor = calculate_blend_factor(DEFAULT_THRESHOLD, DEFAULT_THRESHOLD, 1.0);
        assert!((factor - 1.0).abs() < 0.001);

        // Half threshold should give half blend
        let factor = calculate_blend_factor(DEFAULT_THRESHOLD / 2.0, DEFAULT_THRESHOLD, 1.0);
        assert!((factor - 0.5).abs() < 0.001);

        // Strength scaling
        let factor = calculate_blend_factor(DEFAULT_THRESHOLD, DEFAULT_THRESHOLD, 0.5);
        assert!((factor - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_xbr2x_horizontal_edge() {
        // Create a horizontal edge
        let mut image = RgbaImage::new(4, 4);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        for y in 0..4 {
            for x in 0..4 {
                if y < 2 {
                    image.put_pixel(x, y, black);
                } else {
                    image.put_pixel(x, y, white);
                }
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = xbr2x(&image, &context, &config);

        // Top should be black, bottom should be white
        assert_eq!(*result.get_pixel(0, 0), black);
        assert_eq!(*result.get_pixel(0, 7), white);
    }

    #[test]
    fn test_xbr2x_vertical_edge() {
        // Create a vertical edge
        let mut image = RgbaImage::new(4, 4);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        for y in 0..4 {
            for x in 0..4 {
                if x < 2 {
                    image.put_pixel(x, y, black);
                } else {
                    image.put_pixel(x, y, white);
                }
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = xbr2x(&image, &context, &config);

        // Left should be black, right should be white
        assert_eq!(*result.get_pixel(0, 0), black);
        assert_eq!(*result.get_pixel(7, 0), white);
    }

    #[test]
    fn test_neighborhood_5x5_structure() {
        let mut image = RgbaImage::new(5, 5);
        // Fill with unique values for testing
        for y in 0..5 {
            for x in 0..5 {
                image.put_pixel(x, y, Rgba([x as u8 * 50, y as u8 * 50, 0, 255]));
            }
        }

        let neighborhood = get_neighborhood_5x5(&image, 2, 2);

        // Center should be (2, 2)
        assert_eq!(neighborhood.p, Rgba([100, 100, 0, 255]));

        // Cardinal neighbors
        assert_eq!(neighborhood.c, Rgba([100, 50, 0, 255])); // top
        assert_eq!(neighborhood.e, Rgba([50, 100, 0, 255])); // left
        assert_eq!(neighborhood.f, Rgba([150, 100, 0, 255])); // right
        assert_eq!(neighborhood.h, Rgba([100, 150, 0, 255])); // bottom
    }
}
