//! Gaussian blur algorithm with semantic masking for antialiasing.
//!
//! This module implements the `aa-blur` algorithm which applies Gaussian blur
//! selectively based on semantic pixel roles:
//!
//! | Role | Mask Weight | Effect |
//! |------|-------------|--------|
//! | Anchor | 0% | No blur (preserves crisp details) |
//! | Boundary | 25% | Light blur (softens edges slightly) |
//! | Fill | 100% | Full blur (maximum smoothing) |
//! | Shadow/Highlight | 100% | Full blur (smooth gradients) |

use crate::antialias::{AnchorMode, AntialiasConfig, SemanticContext};
use crate::models::Role;
use image::{Rgba, RgbaImage};

/// Default blur radius in pixels.
const DEFAULT_RADIUS: u32 = 1;

/// Blur mask weight for anchor pixels (0% - no blur).
const ANCHOR_WEIGHT: f32 = 0.0;

/// Blur mask weight for boundary pixels (25% - light blur).
const BOUNDARY_WEIGHT: f32 = 0.25;

/// Blur mask weight for fill/shadow/highlight pixels (100% - full blur).
const FILL_WEIGHT: f32 = 1.0;

/// Apply semantic-aware Gaussian blur to an image.
///
/// This function applies Gaussian blur selectively based on the semantic role
/// of each pixel. Anchor pixels (important details like eyes) receive no blur,
/// boundary pixels receive light blur, and fill pixels receive full blur.
///
/// # Arguments
///
/// * `image` - The input RGBA image to blur
/// * `context` - Semantic context containing role information
/// * `config` - Antialiasing configuration
///
/// # Returns
///
/// A new blurred RGBA image.
///
/// # Example
///
/// ```ignore
/// let blurred = apply_semantic_blur(&image, &context, &config);
/// ```
pub fn apply_semantic_blur(
    image: &RgbaImage,
    context: &SemanticContext,
    config: &AntialiasConfig,
) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut output = image.clone();

    // Generate Gaussian kernel
    let radius = DEFAULT_RADIUS;
    let kernel = generate_gaussian_kernel(radius);
    let kernel_size = (radius * 2 + 1) as i32;
    let half_kernel = radius as i32;

    // Apply blur with semantic masking
    for y in 0..height {
        for x in 0..width {
            let pos = (x as i32, y as i32);

            // Determine mask weight based on semantic role
            let base_weight = get_mask_weight(pos, context, config);

            // Apply config strength modifier
            let weight = base_weight * config.strength;

            if weight < 0.001 {
                // No blur needed, keep original pixel
                continue;
            }

            // Compute blurred color
            let blurred = compute_gaussian_blur(image, x, y, &kernel, kernel_size, half_kernel);

            // Blend original with blurred based on weight
            let original = image.get_pixel(x, y);
            let blended = blend_pixels(original, &blurred, weight);

            output.put_pixel(x, y, blended);
        }
    }

    output
}

/// Get the blur mask weight for a pixel based on its semantic role.
fn get_mask_weight(pos: (i32, i32), context: &SemanticContext, config: &AntialiasConfig) -> f32 {
    // Check for anchor pixels first
    if context.is_anchor(pos) {
        return match config.anchor_mode {
            AnchorMode::Preserve => ANCHOR_WEIGHT,
            AnchorMode::Reduce => BOUNDARY_WEIGHT,
            AnchorMode::Normal => FILL_WEIGHT,
        };
    }

    // Check containment edges - these should also be preserved
    if config.respect_containment && context.is_containment_edge(pos) {
        return ANCHOR_WEIGHT;
    }

    // Get role from semantic context
    match context.get_role(pos) {
        Some(Role::Anchor) => ANCHOR_WEIGHT,
        Some(Role::Boundary) => BOUNDARY_WEIGHT,
        Some(Role::Fill) => FILL_WEIGHT,
        Some(Role::Shadow) => FILL_WEIGHT,
        Some(Role::Highlight) => FILL_WEIGHT,
        None => FILL_WEIGHT, // Default to full blur for unknown pixels
    }
}

/// Generate a normalized Gaussian kernel.
fn generate_gaussian_kernel(radius: u32) -> Vec<f32> {
    let size = (radius * 2 + 1) as usize;
    let mut kernel = vec![0.0f32; size * size];
    let sigma = radius as f32 / 2.0;
    let sigma2 = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - radius as f32;
            let dy = y as f32 - radius as f32;
            let weight = (-((dx * dx + dy * dy) / sigma2)).exp();
            kernel[y * size + x] = weight;
            sum += weight;
        }
    }

    // Normalize
    for w in &mut kernel {
        *w /= sum;
    }

    kernel
}

/// Compute Gaussian blur for a single pixel.
fn compute_gaussian_blur(
    image: &RgbaImage,
    x: u32,
    y: u32,
    kernel: &[f32],
    kernel_size: i32,
    half_kernel: i32,
) -> Rgba<u8> {
    let (width, height) = image.dimensions();
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut a = 0.0f32;

    for ky in 0..kernel_size {
        for kx in 0..kernel_size {
            let px = (x as i32 + kx - half_kernel).clamp(0, width as i32 - 1) as u32;
            let py = (y as i32 + ky - half_kernel).clamp(0, height as i32 - 1) as u32;

            let pixel = image.get_pixel(px, py);
            let weight = kernel[(ky * kernel_size + kx) as usize];

            r += pixel[0] as f32 * weight;
            g += pixel[1] as f32 * weight;
            b += pixel[2] as f32 * weight;
            a += pixel[3] as f32 * weight;
        }
    }

    Rgba([r.round() as u8, g.round() as u8, b.round() as u8, a.round() as u8])
}

/// Blend two pixels based on weight (0.0 = original, 1.0 = blurred).
fn blend_pixels(original: &Rgba<u8>, blurred: &Rgba<u8>, weight: f32) -> Rgba<u8> {
    let inv_weight = 1.0 - weight;

    Rgba([
        (original[0] as f32 * inv_weight + blurred[0] as f32 * weight).round() as u8,
        (original[1] as f32 * inv_weight + blurred[1] as f32 * weight).round() as u8,
        (original[2] as f32 * inv_weight + blurred[2] as f32 * weight).round() as u8,
        (original[3] as f32 * inv_weight + blurred[3] as f32 * weight).round() as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_image(width: u32, height: u32) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        // Fill with a simple pattern
        for y in 0..height {
            for x in 0..width {
                let r = ((x * 255) / width) as u8;
                let g = ((y * 255) / height) as u8;
                img.put_pixel(x, y, Rgba([r, g, 128, 255]));
            }
        }
        img
    }

    #[test]
    fn test_gaussian_kernel_generation() {
        let kernel = generate_gaussian_kernel(1);
        assert_eq!(kernel.len(), 9); // 3x3 kernel

        // Sum should be 1.0 (normalized)
        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);

        // Center should be highest weight
        assert!(kernel[4] > kernel[0]);
        assert!(kernel[4] > kernel[1]);
    }

    #[test]
    fn test_gaussian_kernel_larger() {
        let kernel = generate_gaussian_kernel(2);
        assert_eq!(kernel.len(), 25); // 5x5 kernel

        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_pixels_full_original() {
        let original = Rgba([100, 150, 200, 255]);
        let blurred = Rgba([50, 75, 100, 255]);

        let result = blend_pixels(&original, &blurred, 0.0);
        assert_eq!(result, original);
    }

    #[test]
    fn test_blend_pixels_full_blurred() {
        let original = Rgba([100, 150, 200, 255]);
        let blurred = Rgba([50, 75, 100, 255]);

        let result = blend_pixels(&original, &blurred, 1.0);
        assert_eq!(result, blurred);
    }

    #[test]
    fn test_blend_pixels_half() {
        let original = Rgba([100, 150, 200, 255]);
        let blurred = Rgba([50, 100, 100, 255]);

        let result = blend_pixels(&original, &blurred, 0.5);
        assert_eq!(result[0], 75); // (100 + 50) / 2
        assert_eq!(result[1], 125); // (150 + 100) / 2
        assert_eq!(result[2], 150); // (200 + 100) / 2
    }

    #[test]
    fn test_mask_weight_anchor() {
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((5, 5));

        let config = AntialiasConfig::default();

        let weight = get_mask_weight((5, 5), &context, &config);
        assert_eq!(weight, ANCHOR_WEIGHT);
    }

    #[test]
    fn test_mask_weight_anchor_with_reduce_mode() {
        let mut context = SemanticContext::empty();
        context.anchor_pixels.insert((5, 5));

        let mut config = AntialiasConfig::default();
        config.anchor_mode = AnchorMode::Reduce;

        let weight = get_mask_weight((5, 5), &context, &config);
        assert_eq!(weight, BOUNDARY_WEIGHT);
    }

    #[test]
    fn test_mask_weight_boundary_role() {
        let mut context = SemanticContext::empty();
        let mut boundary_pixels = HashSet::new();
        boundary_pixels.insert((3, 3));
        context.role_masks.insert(Role::Boundary, boundary_pixels);

        let config = AntialiasConfig::default();

        let weight = get_mask_weight((3, 3), &context, &config);
        assert_eq!(weight, BOUNDARY_WEIGHT);
    }

    #[test]
    fn test_mask_weight_fill_role() {
        let mut context = SemanticContext::empty();
        let mut fill_pixels = HashSet::new();
        fill_pixels.insert((4, 4));
        context.role_masks.insert(Role::Fill, fill_pixels);

        let config = AntialiasConfig::default();

        let weight = get_mask_weight((4, 4), &context, &config);
        assert_eq!(weight, FILL_WEIGHT);
    }

    #[test]
    fn test_mask_weight_containment_edge() {
        let mut context = SemanticContext::empty();
        context.containment_edges.insert((2, 2));

        let config = AntialiasConfig::default();

        let weight = get_mask_weight((2, 2), &context, &config);
        assert_eq!(weight, ANCHOR_WEIGHT);
    }

    #[test]
    fn test_mask_weight_unknown_pixel() {
        let context = SemanticContext::empty();
        let config = AntialiasConfig::default();

        // Unknown pixels get full blur
        let weight = get_mask_weight((10, 10), &context, &config);
        assert_eq!(weight, FILL_WEIGHT);
    }

    #[test]
    fn test_apply_semantic_blur_preserves_dimensions() {
        let image = create_test_image(16, 16);
        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = apply_semantic_blur(&image, &context, &config);

        assert_eq!(result.dimensions(), image.dimensions());
    }

    #[test]
    fn test_apply_semantic_blur_with_zero_strength() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 0.0;

        let result = apply_semantic_blur(&image, &context, &config);

        // With zero strength, output should equal input
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(result.get_pixel(x, y), image.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_apply_semantic_blur_preserves_anchors() {
        let image = create_test_image(8, 8);
        let mut context = SemanticContext::empty();
        // Mark center pixel as anchor
        context.anchor_pixels.insert((4, 4));

        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = apply_semantic_blur(&image, &context, &config);

        // Anchor pixel should be unchanged
        assert_eq!(result.get_pixel(4, 4), image.get_pixel(4, 4));
    }

    #[test]
    fn test_apply_semantic_blur_modifies_fill() {
        let mut image = RgbaImage::new(5, 5);
        // Create an image with a bright center surrounded by dark pixels
        for y in 0..5 {
            for x in 0..5 {
                if x == 2 && y == 2 {
                    image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                } else {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                }
            }
        }

        let context = SemanticContext::empty();
        let mut config = AntialiasConfig::default();
        config.strength = 1.0;

        let result = apply_semantic_blur(&image, &context, &config);

        // Center pixel should be dimmer (blurred with surrounding dark pixels)
        let center = result.get_pixel(2, 2);
        assert!(center[0] < 255);
    }
}
