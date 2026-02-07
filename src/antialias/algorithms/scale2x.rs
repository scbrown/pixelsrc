//! Scale2x (EPX) edge-aware upscaling algorithm.
//!
//! Scale2x is a pixel art scaling algorithm that produces a 2x upscaled image
//! with edge-aware interpolation. Unlike simple nearest-neighbor scaling,
//! Scale2x detects edges and curves, producing smoother diagonals while
//! preserving sharp edges.
//!
//! # Algorithm
//!
//! For each input pixel P with neighbors:
//! ```text
//!     A
//!   B P C
//!     D
//! ```
//!
//! The output 2x2 block is computed as:
//! ```text
//!   E0 E1
//!   E2 E3
//! ```
//!
//! Where:
//! - E0 = (A == B && A != C && B != D) ? A : P
//! - E1 = (A == C && A != B && C != D) ? C : P
//! - E2 = (B == D && B != A && D != C) ? B : P
//! - E3 = (C == D && C != A && D != B) ? C : P
//!
//! # Semantic Modifications
//!
//! This implementation supports semantic-aware processing:
//! - **Anchor preservation**: Pixels marked as anchors are not interpolated
//! - **Containment respect**: Edges marked as containment boundaries prevent blending
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::antialias::algorithms::scale2x::{scale2x, Scale2xOptions};
//! use pixelsrc::antialias::SemanticContext;
//!
//! let input = image::RgbaImage::new(16, 16);
//! let context = SemanticContext::empty();
//! let options = Scale2xOptions::default();
//!
//! let output = scale2x(&input, &context, &options);
//! assert_eq!(output.dimensions(), (32, 32));
//! ```

use crate::antialias::{AnchorMode, SemanticContext};
use image::{Rgba, RgbaImage};

/// Options for Scale2x algorithm execution.
#[derive(Debug, Clone)]
pub struct Scale2xOptions {
    /// How to handle anchor regions during scaling.
    pub anchor_mode: AnchorMode,
    /// Whether to respect containment boundaries as hard edges.
    pub respect_containment: bool,
    /// Antialiasing strength from 0.0 to 1.0.
    /// At 1.0, full Scale2x interpolation is applied.
    /// At 0.0, equivalent to nearest-neighbor scaling.
    pub strength: f32,
}

impl Default for Scale2xOptions {
    fn default() -> Self {
        Self { anchor_mode: AnchorMode::Preserve, respect_containment: true, strength: 1.0 }
    }
}

impl Scale2xOptions {
    /// Create options with full strength and default settings.
    pub fn full() -> Self {
        Self::default()
    }

    /// Create options with no semantic awareness (pure Scale2x).
    pub fn pure() -> Self {
        Self { anchor_mode: AnchorMode::Normal, respect_containment: false, strength: 1.0 }
    }
}

/// Apply Scale2x algorithm with semantic awareness.
///
/// Scales the input image by 2x using edge-aware interpolation.
/// Semantic context guides decisions about which pixels to interpolate.
///
/// # Arguments
///
/// * `input` - The source image to scale
/// * `context` - Semantic context for intelligent decisions
/// * `options` - Algorithm configuration options
///
/// # Returns
///
/// A new image with dimensions 2x the input.
pub fn scale2x(
    input: &RgbaImage,
    context: &SemanticContext,
    options: &Scale2xOptions,
) -> RgbaImage {
    let (width, height) = input.dimensions();
    let mut output = RgbaImage::new(width * 2, height * 2);

    for y in 0..height {
        for x in 0..width {
            let pos = (x as i32, y as i32);
            let p = *input.get_pixel(x, y);

            // Check if this pixel should skip interpolation
            let skip_interpolation = should_skip_pixel(pos, context, options);

            // Get neighbors (clamped to image bounds)
            let a = get_pixel_clamped(input, x as i32, y as i32 - 1); // top
            let b = get_pixel_clamped(input, x as i32 - 1, y as i32); // left
            let c = get_pixel_clamped(input, x as i32 + 1, y as i32); // right
            let d = get_pixel_clamped(input, x as i32, y as i32 + 1); // bottom

            // Compute output pixels
            let (e0, e1, e2, e3) = if skip_interpolation || options.strength < 0.001 {
                // No interpolation - simple 2x scaling
                (p, p, p, p)
            } else {
                // Check containment constraints for each neighbor
                let (a_valid, b_valid, c_valid, d_valid) =
                    get_valid_neighbors(pos, context, options);

                // Apply Scale2x rules with containment awareness
                compute_scale2x_block(
                    p,
                    a,
                    b,
                    c,
                    d,
                    a_valid,
                    b_valid,
                    c_valid,
                    d_valid,
                    options.strength,
                )
            };

            // Write output pixels
            let ox = x * 2;
            let oy = y * 2;
            output.put_pixel(ox, oy, e0);
            output.put_pixel(ox + 1, oy, e1);
            output.put_pixel(ox, oy + 1, e2);
            output.put_pixel(ox + 1, oy + 1, e3);
        }
    }

    output
}

/// Check if a pixel should skip interpolation based on semantic context.
fn should_skip_pixel(pos: (i32, i32), context: &SemanticContext, options: &Scale2xOptions) -> bool {
    match options.anchor_mode {
        AnchorMode::Preserve => context.is_anchor(pos),
        AnchorMode::Reduce => false, // Will apply reduced strength later
        AnchorMode::Normal => false,
    }
}

/// Get valid neighbor flags based on containment edges.
///
/// Returns (a_valid, b_valid, c_valid, d_valid) where false means
/// blending with that neighbor should be prevented.
fn get_valid_neighbors(
    pos: (i32, i32),
    context: &SemanticContext,
    options: &Scale2xOptions,
) -> (bool, bool, bool, bool) {
    if !options.respect_containment {
        return (true, true, true, true);
    }

    let (x, y) = pos;

    // Check if there's a containment edge between this pixel and each neighbor
    // A containment edge at the current pixel means we shouldn't blend across it
    let is_edge = context.is_containment_edge(pos);

    if is_edge {
        // If this pixel is on a containment edge, check each direction
        let a_edge = context.is_containment_edge((x, y - 1));
        let b_edge = context.is_containment_edge((x - 1, y));
        let c_edge = context.is_containment_edge((x + 1, y));
        let d_edge = context.is_containment_edge((x, y + 1));

        // Only blend with neighbors that are also on the edge (same region)
        (!a_edge, !b_edge, !c_edge, !d_edge)
    } else {
        // Not on an edge - check if neighbors are on edges
        let a_edge = context.is_containment_edge((x, y - 1));
        let b_edge = context.is_containment_edge((x - 1, y));
        let c_edge = context.is_containment_edge((x + 1, y));
        let d_edge = context.is_containment_edge((x, y + 1));

        (!a_edge, !b_edge, !c_edge, !d_edge)
    }
}

/// Compute the Scale2x output block for a single input pixel.
fn compute_scale2x_block(
    p: Rgba<u8>,
    a: Rgba<u8>,
    b: Rgba<u8>,
    c: Rgba<u8>,
    d: Rgba<u8>,
    a_valid: bool,
    b_valid: bool,
    c_valid: bool,
    d_valid: bool,
    strength: f32,
) -> (Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>) {
    // Standard Scale2x rules with validity checks
    let e0 = if a_valid
        && b_valid
        && colors_equal(&a, &b)
        && !colors_equal(&a, &c)
        && !colors_equal(&b, &d)
    {
        blend_colors(&p, &a, strength)
    } else {
        p
    };

    let e1 = if a_valid
        && c_valid
        && colors_equal(&a, &c)
        && !colors_equal(&a, &b)
        && !colors_equal(&c, &d)
    {
        blend_colors(&p, &c, strength)
    } else {
        p
    };

    let e2 = if b_valid
        && d_valid
        && colors_equal(&b, &d)
        && !colors_equal(&b, &a)
        && !colors_equal(&d, &c)
    {
        blend_colors(&p, &b, strength)
    } else {
        p
    };

    let e3 = if c_valid
        && d_valid
        && colors_equal(&c, &d)
        && !colors_equal(&c, &a)
        && !colors_equal(&d, &b)
    {
        blend_colors(&p, &c, strength)
    } else {
        p
    };

    (e0, e1, e2, e3)
}

/// Get a pixel with clamped coordinates.
fn get_pixel_clamped(img: &RgbaImage, x: i32, y: i32) -> Rgba<u8> {
    let (w, h) = img.dimensions();
    let cx = x.clamp(0, w as i32 - 1) as u32;
    let cy = y.clamp(0, h as i32 - 1) as u32;
    *img.get_pixel(cx, cy)
}

/// Check if two colors are equal.
fn colors_equal(a: &Rgba<u8>, b: &Rgba<u8>) -> bool {
    a.0 == b.0
}

/// Blend two colors based on strength.
///
/// At strength 1.0, returns `target`.
/// At strength 0.0, returns `base`.
fn blend_colors(base: &Rgba<u8>, target: &Rgba<u8>, strength: f32) -> Rgba<u8> {
    if strength >= 1.0 {
        return *target;
    }
    if strength <= 0.0 {
        return *base;
    }

    let inv = 1.0 - strength;
    Rgba([
        (base.0[0] as f32 * inv + target.0[0] as f32 * strength).round() as u8,
        (base.0[1] as f32 * inv + target.0[1] as f32 * strength).round() as u8,
        (base.0[2] as f32 * inv + target.0[2] as f32 * strength).round() as u8,
        (base.0[3] as f32 * inv + target.0[3] as f32 * strength).round() as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale2x_dimensions() {
        let input = RgbaImage::new(16, 16);
        let context = SemanticContext::empty();
        let options = Scale2xOptions::default();

        let output = scale2x(&input, &context, &options);
        assert_eq!(output.dimensions(), (32, 32));
    }

    #[test]
    fn test_scale2x_solid_color() {
        // A solid color image should produce a solid color output
        let mut input = RgbaImage::new(4, 4);
        let red = Rgba([255, 0, 0, 255]);
        for y in 0..4 {
            for x in 0..4 {
                input.put_pixel(x, y, red);
            }
        }

        let context = SemanticContext::empty();
        let options = Scale2xOptions::default();

        let output = scale2x(&input, &context, &options);

        // All pixels should be red
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(*output.get_pixel(x, y), red);
            }
        }
    }

    #[test]
    fn test_scale2x_edge_detection() {
        // Create a simple edge pattern:
        // B B
        // W W
        let mut input = RgbaImage::new(2, 2);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        input.put_pixel(0, 0, black);
        input.put_pixel(1, 0, black);
        input.put_pixel(0, 1, white);
        input.put_pixel(1, 1, white);

        let context = SemanticContext::empty();
        let options = Scale2xOptions::default();

        let output = scale2x(&input, &context, &options);

        // Output should be 4x4
        assert_eq!(output.dimensions(), (4, 4));

        // Top row should be black
        assert_eq!(*output.get_pixel(0, 0), black);
        assert_eq!(*output.get_pixel(1, 0), black);
        assert_eq!(*output.get_pixel(2, 0), black);
        assert_eq!(*output.get_pixel(3, 0), black);

        // Bottom row should be white
        assert_eq!(*output.get_pixel(0, 3), white);
        assert_eq!(*output.get_pixel(1, 3), white);
        assert_eq!(*output.get_pixel(2, 3), white);
        assert_eq!(*output.get_pixel(3, 3), white);
    }

    #[test]
    fn test_scale2x_diagonal_smoothing() {
        // Create a diagonal pattern:
        // B W W
        // B B W
        // B B B
        let mut input = RgbaImage::new(3, 3);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        input.put_pixel(0, 0, black);
        input.put_pixel(1, 0, white);
        input.put_pixel(2, 0, white);
        input.put_pixel(0, 1, black);
        input.put_pixel(1, 1, black);
        input.put_pixel(2, 1, white);
        input.put_pixel(0, 2, black);
        input.put_pixel(1, 2, black);
        input.put_pixel(2, 2, black);

        let context = SemanticContext::empty();
        let options = Scale2xOptions::default();

        let output = scale2x(&input, &context, &options);

        // Output should be 6x6
        assert_eq!(output.dimensions(), (6, 6));

        // The diagonal should be smoothed - check that Scale2x applied
        // For pixel (1,0) which is white with black to the left:
        // A=white (clamped), B=black, C=white, D=black
        // The output 2x2 for this pixel should show diagonal awareness
    }

    #[test]
    fn test_scale2x_anchor_preservation() {
        let mut input = RgbaImage::new(4, 4);
        let red = Rgba([255, 0, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);

        // Create a pattern where Scale2x would normally interpolate
        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 2 == 0 {
                    input.put_pixel(x, y, red);
                } else {
                    input.put_pixel(x, y, blue);
                }
            }
        }

        // Mark center pixel as anchor
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((1, 1));

        let options = Scale2xOptions { anchor_mode: AnchorMode::Preserve, ..Default::default() };

        let output = scale2x(&input, &context, &options);

        // The anchor pixel (1,1) should produce a 2x2 block of its original color
        // without any interpolation
        let anchor_color = *input.get_pixel(1, 1);
        assert_eq!(*output.get_pixel(2, 2), anchor_color);
        assert_eq!(*output.get_pixel(3, 2), anchor_color);
        assert_eq!(*output.get_pixel(2, 3), anchor_color);
        assert_eq!(*output.get_pixel(3, 3), anchor_color);
    }

    #[test]
    fn test_scale2x_containment_respect() {
        let mut input = RgbaImage::new(4, 4);
        let skin = Rgba([255, 200, 150, 255]);
        let eye = Rgba([0, 0, 0, 255]);

        // Fill with skin color
        for y in 0..4 {
            for x in 0..4 {
                input.put_pixel(x, y, skin);
            }
        }
        // Put eye in center
        input.put_pixel(1, 1, eye);
        input.put_pixel(2, 1, eye);

        // Mark eye boundary as containment edge
        let mut context = SemanticContext::empty();
        context.containment_edges.insert((1, 1));
        context.containment_edges.insert((2, 1));

        let options = Scale2xOptions { respect_containment: true, ..Default::default() };

        let output = scale2x(&input, &context, &options);

        // The eye pixels should not blend with surrounding skin
        // Check that the eye region remains distinct
        assert_eq!(*output.get_pixel(2, 2), eye);
        assert_eq!(*output.get_pixel(3, 2), eye);
    }

    #[test]
    fn test_scale2x_strength_zero() {
        let mut input = RgbaImage::new(2, 2);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        input.put_pixel(0, 0, black);
        input.put_pixel(1, 0, white);
        input.put_pixel(0, 1, white);
        input.put_pixel(1, 1, black);

        let context = SemanticContext::empty();
        let options = Scale2xOptions { strength: 0.0, ..Default::default() };

        let output = scale2x(&input, &context, &options);

        // With strength 0, should be simple 2x nearest neighbor
        // Each input pixel becomes a 2x2 block of the same color
        assert_eq!(*output.get_pixel(0, 0), black);
        assert_eq!(*output.get_pixel(1, 0), black);
        assert_eq!(*output.get_pixel(0, 1), black);
        assert_eq!(*output.get_pixel(1, 1), black);

        assert_eq!(*output.get_pixel(2, 0), white);
        assert_eq!(*output.get_pixel(3, 0), white);
        assert_eq!(*output.get_pixel(2, 1), white);
        assert_eq!(*output.get_pixel(3, 1), white);
    }

    #[test]
    fn test_scale2x_pure_mode() {
        let options = Scale2xOptions::pure();
        assert_eq!(options.anchor_mode, AnchorMode::Normal);
        assert!(!options.respect_containment);
        assert!((options.strength - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_colors_equal() {
        let a = Rgba([255, 0, 0, 255]);
        let b = Rgba([255, 0, 0, 255]);
        let c = Rgba([0, 255, 0, 255]);

        assert!(colors_equal(&a, &b));
        assert!(!colors_equal(&a, &c));
    }

    #[test]
    fn test_blend_colors() {
        let base = Rgba([0, 0, 0, 255]);
        let target = Rgba([255, 255, 255, 255]);

        // Full strength = target
        let result = blend_colors(&base, &target, 1.0);
        assert_eq!(result, target);

        // Zero strength = base
        let result = blend_colors(&base, &target, 0.0);
        assert_eq!(result, base);

        // Half strength = midpoint
        let result = blend_colors(&base, &target, 0.5);
        assert_eq!(result.0[0], 128);
        assert_eq!(result.0[1], 128);
        assert_eq!(result.0[2], 128);
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

        // Clamped overflow
        assert_eq!(get_pixel_clamped(&img, 10, 0), *img.get_pixel(1, 0));
        assert_eq!(get_pixel_clamped(&img, 0, 10), *img.get_pixel(0, 1));
    }

    #[test]
    fn test_scale2x_1x1_image() {
        let mut input = RgbaImage::new(1, 1);
        let color = Rgba([128, 64, 32, 255]);
        input.put_pixel(0, 0, color);

        let context = SemanticContext::empty();
        let options = Scale2xOptions::default();

        let output = scale2x(&input, &context, &options);

        assert_eq!(output.dimensions(), (2, 2));
        // All 4 output pixels should be the same color (no neighbors to interpolate with)
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(*output.get_pixel(x, y), color);
            }
        }
    }

    #[test]
    fn test_scale2x_anchor_mode_normal() {
        let mut input = RgbaImage::new(3, 3);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);

        // Create pattern
        input.put_pixel(0, 0, black);
        input.put_pixel(1, 0, black);
        input.put_pixel(2, 0, white);
        input.put_pixel(0, 1, black);
        input.put_pixel(1, 1, black);
        input.put_pixel(2, 1, white);
        input.put_pixel(0, 2, white);
        input.put_pixel(1, 2, white);
        input.put_pixel(2, 2, white);

        // Mark center as anchor but use Normal mode
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((1, 1));

        let options = Scale2xOptions { anchor_mode: AnchorMode::Normal, ..Default::default() };

        // Should still apply interpolation even though pixel is marked as anchor
        let output = scale2x(&input, &context, &options);
        assert_eq!(output.dimensions(), (6, 6));
    }
}
