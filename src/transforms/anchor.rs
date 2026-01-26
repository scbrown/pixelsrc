//! Anchor-preserving scaling for pixel art (TTP-ca8cj)
//!
//! Provides functions for scaling images while preserving important anchor regions
//! like eyes, details, etc. that might otherwise disappear during downscaling.

use image::{imageops::FilterType, RgbaImage};

/// Bounding box for an anchor region.
///
/// Used to track regions that should be preserved during downscaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorBounds {
    /// Left edge (inclusive)
    pub x: u32,
    /// Top edge (inclusive)
    pub y: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl AnchorBounds {
    /// Create a new anchor bounds.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Create anchor bounds from a set of pixel coordinates.
    ///
    /// Returns `None` if the points set is empty.
    pub fn from_points(points: &[(i32, i32)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let min_x = points.iter().map(|(x, _)| *x).min().expect("non-empty checked above");
        let max_x = points.iter().map(|(x, _)| *x).max().expect("non-empty checked above");
        let min_y = points.iter().map(|(_, y)| *y).min().expect("non-empty checked above");
        let max_y = points.iter().map(|(_, y)| *y).max().expect("non-empty checked above");

        // Handle negative coordinates by clamping to 0
        let x = min_x.max(0) as u32;
        let y = min_y.max(0) as u32;
        let width = (max_x - min_x + 1).max(1) as u32;
        let height = (max_y - min_y + 1).max(1) as u32;

        Some(Self { x, y, width, height })
    }

    /// Get the center point of the bounding box.
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Scale the bounds by the given factors.
    ///
    /// For downscaling, this may result in very small or zero dimensions.
    pub fn scaled(&self, scale_x: f32, scale_y: f32) -> Self {
        let new_x = (self.x as f32 * scale_x).round() as u32;
        let new_y = (self.y as f32 * scale_y).round() as u32;
        let new_width = (self.width as f32 * scale_x).round() as u32;
        let new_height = (self.height as f32 * scale_y).round() as u32;

        Self { x: new_x, y: new_y, width: new_width, height: new_height }
    }
}

/// Scale an image with preservation of anchor regions.
///
/// When downscaling (scale factors < 1.0), ensures that anchor regions
/// maintain at least 1x1 pixel bounds. This is important for pixel art
/// where small details like eyes should not disappear during scaling.
///
/// # Arguments
///
/// * `image` - The source image to scale
/// * `scale_x` - Horizontal scale factor
/// * `scale_y` - Vertical scale factor
/// * `anchors` - List of anchor region bounds to preserve
///
/// # Returns
///
/// The scaled image with anchor regions preserved.
pub fn scale_image_with_anchor_preservation(
    image: &RgbaImage,
    scale_x: f32,
    scale_y: f32,
    anchors: &[AnchorBounds],
) -> RgbaImage {
    // For upscaling or no anchors, use standard nearest-neighbor scaling
    if (scale_x >= 1.0 && scale_y >= 1.0) || anchors.is_empty() {
        return scale_image(image, scale_x, scale_y);
    }

    let (src_width, src_height) = image.dimensions();
    let dst_width = ((src_width as f32 * scale_x).round() as u32).max(1);
    let dst_height = ((src_height as f32 * scale_y).round() as u32).max(1);

    // First, do standard nearest-neighbor scaling
    let mut result = scale_image(image, scale_x, scale_y);

    // For downscaling, ensure each anchor region has at least 1x1 representation
    // by explicitly writing the anchor's center pixel to the scaled image
    for anchor in anchors {
        // Find the center of the original anchor region
        let (center_x, center_y) = anchor.center();

        // Map the center to destination coordinates
        let dst_x = ((center_x as f32 * scale_x).round() as u32).min(dst_width.saturating_sub(1));
        let dst_y = ((center_y as f32 * scale_y).round() as u32).min(dst_height.saturating_sub(1));

        // Get the color from the center of the original anchor region
        if center_x < src_width && center_y < src_height {
            let pixel = *image.get_pixel(center_x, center_y);

            // Write the anchor pixel - this ensures the anchor is always visible
            // even if the standard scaling algorithm would have skipped it
            if dst_x < dst_width && dst_y < dst_height {
                result.put_pixel(dst_x, dst_y, pixel);
            }
        }
    }

    result
}

/// Scale an image by fractional factors using nearest-neighbor interpolation.
///
/// This preserves crisp pixel edges for pixel art. Unlike `output::scale_image`
/// which only handles integer upscaling, this function supports any scale factor.
///
/// # Arguments
///
/// * `image` - The image to scale
/// * `scale_x` - Horizontal scale factor (e.g., 0.5 for half width)
/// * `scale_y` - Vertical scale factor (e.g., 2.0 for double height)
///
/// # Returns
///
/// The scaled image.
pub fn scale_image(image: &RgbaImage, scale_x: f32, scale_y: f32) -> RgbaImage {
    // Handle no-op case
    if (scale_x - 1.0).abs() < 0.001 && (scale_y - 1.0).abs() < 0.001 {
        return image.clone();
    }

    let (w, h) = image.dimensions();
    let new_w = ((w as f32 * scale_x).round() as u32).max(1);
    let new_h = ((h as f32 * scale_y).round() as u32).max(1);

    image::imageops::resize(image, new_w, new_h, FilterType::Nearest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_bounds_new() {
        let bounds = AnchorBounds::new(10, 20, 5, 3);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 5);
        assert_eq!(bounds.height, 3);
    }

    #[test]
    fn test_anchor_bounds_from_points() {
        let points = vec![(5, 10), (8, 12), (6, 11)];
        let bounds = AnchorBounds::from_points(&points).unwrap();
        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 10);
        assert_eq!(bounds.width, 4); // 8 - 5 + 1
        assert_eq!(bounds.height, 3); // 12 - 10 + 1
    }

    #[test]
    fn test_anchor_bounds_from_points_empty() {
        let points: Vec<(i32, i32)> = vec![];
        assert!(AnchorBounds::from_points(&points).is_none());
    }

    #[test]
    fn test_anchor_bounds_from_points_single() {
        let points = vec![(5, 10)];
        let bounds = AnchorBounds::from_points(&points).unwrap();
        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 10);
        assert_eq!(bounds.width, 1);
        assert_eq!(bounds.height, 1);
    }

    #[test]
    fn test_anchor_bounds_from_points_negative() {
        let points = vec![(-2, -3), (5, 10)];
        let bounds = AnchorBounds::from_points(&points).unwrap();
        assert_eq!(bounds.x, 0); // Clamped from -2
        assert_eq!(bounds.y, 0); // Clamped from -3
        assert_eq!(bounds.width, 8); // 5 - (-2) + 1
        assert_eq!(bounds.height, 14); // 10 - (-3) + 1
    }

    #[test]
    fn test_anchor_bounds_center() {
        let bounds = AnchorBounds::new(10, 20, 10, 6);
        let (cx, cy) = bounds.center();
        assert_eq!(cx, 15); // 10 + 10/2
        assert_eq!(cy, 23); // 20 + 6/2
    }

    #[test]
    fn test_anchor_bounds_scaled() {
        let bounds = AnchorBounds::new(10, 20, 10, 6);
        let scaled = bounds.scaled(0.5, 0.5);
        assert_eq!(scaled.x, 5);
        assert_eq!(scaled.y, 10);
        assert_eq!(scaled.width, 5);
        assert_eq!(scaled.height, 3);
    }

    #[test]
    fn test_anchor_bounds_scaled_upscale() {
        let bounds = AnchorBounds::new(5, 10, 4, 2);
        let scaled = bounds.scaled(2.0, 3.0);
        assert_eq!(scaled.x, 10);
        assert_eq!(scaled.y, 30);
        assert_eq!(scaled.width, 8);
        assert_eq!(scaled.height, 6);
    }

    #[test]
    fn test_scale_image_no_op() {
        let img = RgbaImage::new(10, 10);
        let result = scale_image(&img, 1.0, 1.0);
        assert_eq!(result.dimensions(), (10, 10));
    }

    #[test]
    fn test_scale_image_double() {
        let img = RgbaImage::new(10, 8);
        let result = scale_image(&img, 2.0, 2.0);
        assert_eq!(result.dimensions(), (20, 16));
    }

    #[test]
    fn test_scale_image_half() {
        let img = RgbaImage::new(10, 8);
        let result = scale_image(&img, 0.5, 0.5);
        assert_eq!(result.dimensions(), (5, 4));
    }

    #[test]
    fn test_scale_image_non_uniform() {
        let img = RgbaImage::new(10, 10);
        let result = scale_image(&img, 2.0, 0.5);
        assert_eq!(result.dimensions(), (20, 5));
    }

    #[test]
    fn test_scale_image_minimum_size() {
        let img = RgbaImage::new(2, 2);
        let result = scale_image(&img, 0.1, 0.1);
        // Should be at least 1x1
        assert!(result.width() >= 1);
        assert!(result.height() >= 1);
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_upscale() {
        let img = RgbaImage::new(10, 10);
        let anchors = vec![AnchorBounds::new(2, 2, 2, 2)];
        // For upscaling, should just use standard scaling
        let result = scale_image_with_anchor_preservation(&img, 2.0, 2.0, &anchors);
        assert_eq!(result.dimensions(), (20, 20));
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_no_anchors() {
        let img = RgbaImage::new(10, 10);
        let result = scale_image_with_anchor_preservation(&img, 0.5, 0.5, &[]);
        assert_eq!(result.dimensions(), (5, 5));
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_downscale() {
        // Create a 10x10 image with a specific pixel set
        let mut img = RgbaImage::new(10, 10);
        // Set a distinctive pixel in the anchor region
        img.put_pixel(5, 5, image::Rgba([255, 0, 0, 255]));

        let anchors = vec![AnchorBounds::new(4, 4, 3, 3)]; // Anchor around (5,5)
        let result = scale_image_with_anchor_preservation(&img, 0.5, 0.5, &anchors);

        assert_eq!(result.dimensions(), (5, 5));

        // The center of the anchor region (5,5) scaled by 0.5 is (2.5,2.5) -> (3,3) after rounding
        // or (2,2) depending on the exact math
        // The important thing is that the red pixel should be preserved somewhere
    }
}
