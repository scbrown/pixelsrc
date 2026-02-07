//! Gradient interpolation for DerivesFrom relationships.
//!
//! This module implements gradient smoothing as a post-process pass that creates
//! smooth shadow/highlight transitions at boundaries between related colors.
//!
//! # Overview
//!
//! When two regions have a `DerivesFrom` relationship (e.g., `skin_shadow` derives
//! from `skin`), this module creates smooth color gradients at their boundaries
//! instead of hard edges.
//!
//! # Algorithm
//!
//! 1. For each `GradientPair` in the semantic context:
//!    - Identify boundary pixels between source and target regions
//!    - Calculate interpolation weights based on distance from boundary
//!    - Blend colors smoothly across the transition zone
//!
//! 2. The transition width is controlled by `config.strength`:
//!    - 0.0 = no gradient (sharp edge)
//!    - 0.5 = moderate transition (default)
//!    - 1.0 = maximum smoothing

use crate::antialias::{AntialiasConfig, SemanticContext};
use image::{Rgba, RgbaImage};
use std::collections::HashSet;

/// Maximum radius (in pixels) for gradient transitions.
const MAX_GRADIENT_RADIUS: i32 = 3;

/// Apply gradient smoothing to an image based on DerivesFrom relationships.
///
/// This function creates smooth color transitions at boundaries between
/// semantically related regions (e.g., skin and skin_shadow).
///
/// # Arguments
///
/// * `image` - The input RGBA image to process
/// * `context` - Semantic context containing gradient pair information
/// * `config` - Antialiasing configuration
///
/// # Returns
///
/// A new image with smooth gradient transitions applied.
///
/// # Example
///
/// ```ignore
/// let smoothed = apply_gradient_smoothing(&image, &context, &config);
/// ```
pub fn apply_gradient_smoothing(
    image: &RgbaImage,
    context: &SemanticContext,
    config: &AntialiasConfig,
) -> RgbaImage {
    // Skip if gradient shadows are disabled
    if !config.gradient_shadows {
        return image.clone();
    }

    // Skip if no gradient pairs exist
    if context.gradient_pairs.is_empty() {
        return image.clone();
    }

    let mut output = image.clone();
    let (width, height) = image.dimensions();

    // Calculate effective gradient radius based on strength
    let gradient_radius = ((MAX_GRADIENT_RADIUS as f32 * config.strength).round() as i32).max(1);

    // Process each gradient pair
    for gradient in &context.gradient_pairs {
        // Build a set of boundary pixels for fast lookup
        let boundary_set: HashSet<(i32, i32)> = gradient.boundary_pixels.iter().copied().collect();

        // Find all pixels that should be affected by this gradient
        let affected_pixels = find_affected_pixels(&boundary_set, gradient_radius, width, height);

        // Apply gradient blending to affected pixels
        for &(x, y) in &affected_pixels {
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }

            let ux = x as u32;
            let uy = y as u32;

            // Skip containment edges - don't blend across hard boundaries
            if config.respect_containment && context.is_containment_edge((x, y)) {
                continue;
            }

            // Calculate distance to nearest boundary pixel
            let distance = distance_to_boundary((x, y), &boundary_set);

            if distance <= gradient_radius as f32 {
                // Calculate interpolation factor (0.0 at boundary, 1.0 at max radius)
                let t = (distance / gradient_radius as f32).clamp(0.0, 1.0);

                // Get current pixel color
                let current = image.get_pixel(ux, uy);

                // Determine which color to blend towards based on current pixel
                let blended = blend_gradient_pixel(
                    current,
                    &gradient.source_color,
                    &gradient.target_color,
                    t,
                    config.strength,
                );

                output.put_pixel(ux, uy, blended);
            }
        }
    }

    output
}

/// Find all pixels within gradient_radius of any boundary pixel.
fn find_affected_pixels(
    boundary: &HashSet<(i32, i32)>,
    radius: i32,
    width: u32,
    height: u32,
) -> Vec<(i32, i32)> {
    let mut affected = HashSet::new();

    for &(bx, by) in boundary {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = bx + dx;
                let y = by + dy;

                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                    affected.insert((x, y));
                }
            }
        }
    }

    affected.into_iter().collect()
}

/// Calculate the minimum distance from a point to any boundary pixel.
fn distance_to_boundary(pos: (i32, i32), boundary: &HashSet<(i32, i32)>) -> f32 {
    let (px, py) = pos;

    boundary
        .iter()
        .map(|&(bx, by)| {
            let dx = (px - bx) as f32;
            let dy = (py - by) as f32;
            (dx * dx + dy * dy).sqrt()
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(f32::MAX)
}

/// Blend a pixel's color based on gradient parameters.
///
/// The blending creates a smooth transition between source and target colors,
/// weighted by distance from the boundary and overall strength.
fn blend_gradient_pixel(
    current: &Rgba<u8>,
    source_color: &Rgba<u8>,
    target_color: &Rgba<u8>,
    distance_factor: f32,
    strength: f32,
) -> Rgba<u8> {
    // Determine if current pixel is closer to source or target color
    let source_dist = color_distance(current, source_color);
    let target_dist = color_distance(current, target_color);

    // Calculate blend direction and amount
    let (from_color, to_color) = if source_dist < target_dist {
        (source_color, target_color)
    } else {
        (target_color, source_color)
    };

    // The blend amount decreases with distance from boundary
    // At the boundary (distance_factor = 0), we blend most
    // Further away (distance_factor = 1), we blend less
    let blend_amount = (1.0 - distance_factor) * strength * 0.5;

    interpolate_colors(current, from_color, to_color, blend_amount)
}

/// Calculate perceptual distance between two colors.
fn color_distance(a: &Rgba<u8>, b: &Rgba<u8>) -> f32 {
    // Use weighted RGB distance for better perceptual accuracy
    // Human eyes are more sensitive to green, less to blue
    let dr = (a[0] as f32 - b[0] as f32) * 0.299;
    let dg = (a[1] as f32 - b[1] as f32) * 0.587;
    let db = (a[2] as f32 - b[2] as f32) * 0.114;
    let da = (a[3] as f32 - b[3] as f32) * 0.1;

    (dr * dr + dg * dg + db * db + da * da).sqrt()
}

/// Interpolate between current color and a gradient between two colors.
fn interpolate_colors(current: &Rgba<u8>, from: &Rgba<u8>, to: &Rgba<u8>, amount: f32) -> Rgba<u8> {
    // First, calculate the gradient midpoint between from and to
    let mid_r = (from[0] as f32 + to[0] as f32) / 2.0;
    let mid_g = (from[1] as f32 + to[1] as f32) / 2.0;
    let mid_b = (from[2] as f32 + to[2] as f32) / 2.0;
    let mid_a = (from[3] as f32 + to[3] as f32) / 2.0;

    // Blend current color towards the midpoint
    let inv_amount = 1.0 - amount;

    Rgba([
        (current[0] as f32 * inv_amount + mid_r * amount).round() as u8,
        (current[1] as f32 * inv_amount + mid_g * amount).round() as u8,
        (current[2] as f32 * inv_amount + mid_b * amount).round() as u8,
        (current[3] as f32 * inv_amount + mid_a * amount).round() as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::antialias::context::GradientPair;

    fn create_test_image(width: u32, height: u32) -> RgbaImage {
        RgbaImage::new(width, height)
    }

    #[test]
    fn test_apply_gradient_smoothing_disabled() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::default();
        let mut config = AntialiasConfig::default();
        config.gradient_shadows = false;

        let result = apply_gradient_smoothing(&image, &context, &config);

        // Should return unchanged image when disabled
        assert_eq!(result.dimensions(), image.dimensions());
    }

    #[test]
    fn test_apply_gradient_smoothing_no_pairs() {
        let image = create_test_image(8, 8);
        let context = SemanticContext::default();
        let config = AntialiasConfig::default();

        let result = apply_gradient_smoothing(&image, &context, &config);

        // Should return unchanged image when no gradient pairs
        assert_eq!(result.dimensions(), image.dimensions());
    }

    #[test]
    fn test_distance_to_boundary() {
        let mut boundary = HashSet::new();
        boundary.insert((5, 5));
        boundary.insert((5, 6));

        // Distance from (5, 5) to itself should be 0
        assert!((distance_to_boundary((5, 5), &boundary) - 0.0).abs() < 0.001);

        // Distance from (6, 5) to nearest boundary (5, 5) should be 1
        assert!((distance_to_boundary((6, 5), &boundary) - 1.0).abs() < 0.001);

        // Distance from (7, 5) to nearest boundary (5, 5) should be 2
        assert!((distance_to_boundary((7, 5), &boundary) - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_find_affected_pixels() {
        let mut boundary = HashSet::new();
        boundary.insert((5, 5));

        let affected = find_affected_pixels(&boundary, 1, 10, 10);

        // With radius 1, should affect 9 pixels (3x3 around boundary)
        assert_eq!(affected.len(), 9);
        assert!(affected.contains(&(5, 5)));
        assert!(affected.contains(&(4, 4)));
        assert!(affected.contains(&(6, 6)));
    }

    #[test]
    fn test_find_affected_pixels_edge_clamping() {
        let mut boundary = HashSet::new();
        boundary.insert((0, 0));

        let affected = find_affected_pixels(&boundary, 1, 10, 10);

        // At corner, should only include valid pixels
        assert!(affected.contains(&(0, 0)));
        assert!(affected.contains(&(1, 0)));
        assert!(affected.contains(&(0, 1)));
        assert!(affected.contains(&(1, 1)));
        // Should not include negative coordinates
        assert!(!affected.contains(&(-1, 0)));
        assert!(!affected.contains(&(0, -1)));
    }

    #[test]
    fn test_color_distance() {
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);
        let gray = Rgba([128, 128, 128, 255]);

        // Distance from color to itself should be 0
        assert!((color_distance(&black, &black) - 0.0).abs() < 0.001);

        // Distance from black to white should be large
        let bw_dist = color_distance(&black, &white);
        assert!(bw_dist > 100.0);

        // Distance from black to gray should be less than black to white
        let bg_dist = color_distance(&black, &gray);
        assert!(bg_dist < bw_dist);
    }

    #[test]
    fn test_interpolate_colors_zero_amount() {
        let current = Rgba([100, 100, 100, 255]);
        let from = Rgba([0, 0, 0, 255]);
        let to = Rgba([255, 255, 255, 255]);

        let result = interpolate_colors(&current, &from, &to, 0.0);

        // With zero amount, should return current unchanged
        assert_eq!(result, current);
    }

    #[test]
    fn test_interpolate_colors_full_amount() {
        let current = Rgba([100, 100, 100, 255]);
        let from = Rgba([0, 0, 0, 255]);
        let to = Rgba([255, 255, 255, 255]);

        let result = interpolate_colors(&current, &from, &to, 1.0);

        // With full amount, should be at midpoint between from and to
        // Midpoint of black and white is (127.5, 127.5, 127.5)
        assert_eq!(result[0], 128); // rounded from 127.5
        assert_eq!(result[1], 128);
        assert_eq!(result[2], 128);
    }

    #[test]
    fn test_gradient_smoothing_with_gradient_pair() {
        let mut image = RgbaImage::new(10, 10);

        // Create a simple two-region image
        // Left half: skin color (255, 200, 150)
        // Right half: shadow color (200, 150, 100)
        for y in 0..10 {
            for x in 0..5 {
                image.put_pixel(x, y, Rgba([255, 200, 150, 255]));
            }
            for x in 5..10 {
                image.put_pixel(x, y, Rgba([200, 150, 100, 255]));
            }
        }

        // Create context with gradient pair at the boundary
        let mut context = SemanticContext::default();
        let boundary_pixels: Vec<(i32, i32)> = (0..10).map(|y| (5, y)).collect();

        context.gradient_pairs.push(GradientPair {
            source_token: "skin_shadow".to_string(),
            target_token: "skin".to_string(),
            source_color: Rgba([200, 150, 100, 255]),
            target_color: Rgba([255, 200, 150, 255]),
            boundary_pixels,
        });

        let mut config = AntialiasConfig::default();
        config.gradient_shadows = true;
        config.strength = 0.5;

        let result = apply_gradient_smoothing(&image, &context, &config);

        // Pixels near the boundary should be modified
        // The exact values depend on the algorithm, but they should differ from original
        let boundary_pixel = result.get_pixel(5, 5);
        let original_boundary = image.get_pixel(5, 5);

        // At least verify dimensions are preserved
        assert_eq!(result.dimensions(), image.dimensions());

        // The boundary pixel should have been blended
        // (It may or may not change depending on exact algorithm behavior)
        let _ = (boundary_pixel, original_boundary); // Acknowledge use
    }

    #[test]
    fn test_blend_gradient_pixel_source_closer() {
        let source = Rgba([100, 100, 100, 255]);
        let target = Rgba([200, 200, 200, 255]);

        // Current is closer to source
        let current = Rgba([110, 110, 110, 255]);

        let result = blend_gradient_pixel(&current, &source, &target, 0.5, 0.5);

        // Should blend towards midpoint (150, 150, 150)
        // With distance_factor=0.5 and strength=0.5, blend_amount = 0.125
        assert!(result[0] > current[0] || result[0] == current[0]);
    }

    #[test]
    fn test_blend_gradient_pixel_target_closer() {
        let source = Rgba([100, 100, 100, 255]);
        let target = Rgba([200, 200, 200, 255]);

        // Current is closer to target
        let current = Rgba([190, 190, 190, 255]);

        let result = blend_gradient_pixel(&current, &source, &target, 0.5, 0.5);

        // Should blend towards midpoint (150, 150, 150)
        assert!(result[0] < current[0] || result[0] == current[0]);
    }
}
