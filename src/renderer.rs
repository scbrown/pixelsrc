//! Sprite rendering to image buffers

use crate::models::Sprite;
use crate::registry::ResolvedSprite;
use crate::structured::render_structured;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

/// A warning generated during rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Transparent color used for padding
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Render a sprite to an RGBA image buffer.
///
/// Takes a sprite and a resolved palette (token -> hex color string).
/// Returns the rendered image and any warnings generated.
///
/// # Size Inference
///
/// If `sprite.size` is `None`:
/// - Width = max tokens in any row
/// - Height = number of rows
///
/// If `sprite.size` is `Some([w, h])`:
/// - Rows are padded/truncated to match width
/// - Grid is padded/truncated to match height
///
/// # Error Handling (Lenient Mode)
///
/// - Unknown tokens: Render as magenta (#FF00FF) with warning
/// - Row too short: Pad with transparent pixels (treated as {_}) with warning
/// - Row too long: Truncate with warning
/// - Invalid color format: Use magenta with warning
///
/// # Examples
///
/// See tests/demos/ for usage examples with the regions-based format.
pub fn render_sprite(
    sprite: &Sprite,
    palette: &HashMap<String, String>,
) -> (RgbaImage, Vec<Warning>) {
    // Structured sprites use regions for rendering
    if let Some(regions) = &sprite.regions {
        return render_structured(&sprite.name, sprite.size, regions, palette);
    }

    // Grid format is no longer supported - sprites must use regions
    let warnings = vec![Warning::new(format!(
        "Sprite '{}' uses deprecated grid format. Please convert to structured regions format.",
        sprite.name
    ))];
    (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings)
}

/// Render a ResolvedSprite (sprite or expanded variant) to an RGBA image buffer.
///
/// This function is similar to `render_sprite` but takes a `ResolvedSprite`
/// which already has the merged palette ready for rendering.
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::renderer::render_resolved;
/// use pixelsrc::registry::ResolvedSprite;
/// use std::collections::HashMap;
///
/// let (image, warnings) = render_resolved(&resolved);
/// ```
pub fn render_resolved(resolved: &ResolvedSprite) -> (RgbaImage, Vec<Warning>) {
    // Structured sprites use regions for rendering
    if let Some(regions) = &resolved.regions {
        return render_structured(&resolved.name, resolved.size, regions, &resolved.palette);
    }

    // Grid format is no longer supported - sprites must use regions
    let warnings = vec![Warning::new(format!(
        "Sprite/variant '{}' uses deprecated grid format. Please convert to structured regions format.",
        resolved.name
    ))];
    (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings)
}

/// Render a nine-slice sprite to a target size.
///
/// Nine-slice (or 9-patch) sprites are divided into 9 regions:
/// - 4 corners (fixed size, no scaling)
/// - 4 edges (scaled in one direction)
/// - 1 center (scaled in both directions)
///
/// The `nine_slice` parameter defines the border widths (left, right, top, bottom).
/// The target `width` and `height` determine the output image dimensions.
///
/// # Arguments
///
/// * `source` - The source image to slice
/// * `nine_slice` - The border definitions from the sprite
/// * `target_width` - Target output width in pixels
/// * `target_height` - Target output height in pixels
///
/// # Returns
///
/// The rendered nine-slice image and any warnings generated.
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::renderer::render_nine_slice;
/// use pixelsrc::models::NineSlice;
///
/// let source = // ... rendered sprite image
/// let nine_slice = NineSlice { left: 4, right: 4, top: 4, bottom: 4 };
/// let (result, warnings) = render_nine_slice(&source, &nine_slice, 64, 32);
/// ```
pub fn render_nine_slice(
    source: &RgbaImage,
    nine_slice: &crate::models::NineSlice,
    target_width: u32,
    target_height: u32,
) -> (RgbaImage, Vec<Warning>) {
    let mut warnings = Vec::new();

    let src_width = source.width();
    let src_height = source.height();

    // Validate nine-slice dimensions fit within source
    let min_width = nine_slice.left + nine_slice.right;
    let min_height = nine_slice.top + nine_slice.bottom;

    if min_width > src_width {
        warnings.push(Warning::new(format!(
            "Nine-slice borders (left={} + right={}) exceed source width ({})",
            nine_slice.left, nine_slice.right, src_width
        )));
        return (source.clone(), warnings);
    }

    if min_height > src_height {
        warnings.push(Warning::new(format!(
            "Nine-slice borders (top={} + bottom={}) exceed source height ({})",
            nine_slice.top, nine_slice.bottom, src_height
        )));
        return (source.clone(), warnings);
    }

    // Validate target size can accommodate the fixed borders
    if target_width < min_width {
        warnings.push(Warning::new(format!(
            "Target width ({}) is less than minimum nine-slice width ({})",
            target_width, min_width
        )));
        return (source.clone(), warnings);
    }

    if target_height < min_height {
        warnings.push(Warning::new(format!(
            "Target height ({}) is less than minimum nine-slice height ({})",
            target_height, min_height
        )));
        return (source.clone(), warnings);
    }

    // Create target image
    let mut result = RgbaImage::new(target_width, target_height);

    // Calculate source regions
    let src_center_width = src_width - nine_slice.left - nine_slice.right;
    let src_center_height = src_height - nine_slice.top - nine_slice.bottom;

    // Calculate target center dimensions
    let target_center_width = target_width - nine_slice.left - nine_slice.right;
    let target_center_height = target_height - nine_slice.top - nine_slice.bottom;

    // Helper to copy a rectangular region
    let copy_region = |result: &mut RgbaImage,
                       src_x: u32,
                       src_y: u32,
                       dst_x: u32,
                       dst_y: u32,
                       width: u32,
                       height: u32| {
        for dy in 0..height {
            for dx in 0..width {
                let pixel = *source.get_pixel(src_x + dx, src_y + dy);
                result.put_pixel(dst_x + dx, dst_y + dy, pixel);
            }
        }
    };

    // Helper to stretch a horizontal strip (scales horizontally)
    let stretch_horizontal = |result: &mut RgbaImage,
                              src_x: u32,
                              src_y: u32,
                              src_w: u32,
                              src_h: u32,
                              dst_x: u32,
                              dst_y: u32,
                              dst_w: u32| {
        if src_w == 0 || dst_w == 0 {
            return;
        }
        for dy in 0..src_h {
            for dx in 0..dst_w {
                // Map destination x to source x using nearest-neighbor
                let src_dx = (dx * src_w) / dst_w;
                let pixel = *source.get_pixel(src_x + src_dx, src_y + dy);
                result.put_pixel(dst_x + dx, dst_y + dy, pixel);
            }
        }
    };

    // Helper to stretch a vertical strip (scales vertically)
    let stretch_vertical = |result: &mut RgbaImage,
                            src_x: u32,
                            src_y: u32,
                            src_w: u32,
                            src_h: u32,
                            dst_x: u32,
                            dst_y: u32,
                            dst_h: u32| {
        if src_h == 0 || dst_h == 0 {
            return;
        }
        for dy in 0..dst_h {
            // Map destination y to source y using nearest-neighbor
            let src_dy = (dy * src_h) / dst_h;
            for dx in 0..src_w {
                let pixel = *source.get_pixel(src_x + dx, src_y + src_dy);
                result.put_pixel(dst_x + dx, dst_y + dy, pixel);
            }
        }
    };

    // Helper to stretch center (scales both directions)
    let stretch_both = |result: &mut RgbaImage,
                        src_x: u32,
                        src_y: u32,
                        src_w: u32,
                        src_h: u32,
                        dst_x: u32,
                        dst_y: u32,
                        dst_w: u32,
                        dst_h: u32| {
        if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
            return;
        }
        for dy in 0..dst_h {
            let src_dy = (dy * src_h) / dst_h;
            for dx in 0..dst_w {
                let src_dx = (dx * src_w) / dst_w;
                let pixel = *source.get_pixel(src_x + src_dx, src_y + src_dy);
                result.put_pixel(dst_x + dx, dst_y + dy, pixel);
            }
        }
    };

    // 1. Copy corners (fixed size)
    // Top-left
    copy_region(&mut result, 0, 0, 0, 0, nine_slice.left, nine_slice.top);
    // Top-right
    copy_region(
        &mut result,
        src_width - nine_slice.right,
        0,
        target_width - nine_slice.right,
        0,
        nine_slice.right,
        nine_slice.top,
    );
    // Bottom-left
    copy_region(
        &mut result,
        0,
        src_height - nine_slice.bottom,
        0,
        target_height - nine_slice.bottom,
        nine_slice.left,
        nine_slice.bottom,
    );
    // Bottom-right
    copy_region(
        &mut result,
        src_width - nine_slice.right,
        src_height - nine_slice.bottom,
        target_width - nine_slice.right,
        target_height - nine_slice.bottom,
        nine_slice.right,
        nine_slice.bottom,
    );

    // 2. Stretch edges
    // Top edge (stretch horizontally)
    stretch_horizontal(
        &mut result,
        nine_slice.left,
        0,
        src_center_width,
        nine_slice.top,
        nine_slice.left,
        0,
        target_center_width,
    );
    // Bottom edge (stretch horizontally)
    stretch_horizontal(
        &mut result,
        nine_slice.left,
        src_height - nine_slice.bottom,
        src_center_width,
        nine_slice.bottom,
        nine_slice.left,
        target_height - nine_slice.bottom,
        target_center_width,
    );
    // Left edge (stretch vertically)
    stretch_vertical(
        &mut result,
        0,
        nine_slice.top,
        nine_slice.left,
        src_center_height,
        0,
        nine_slice.top,
        target_center_height,
    );
    // Right edge (stretch vertically)
    stretch_vertical(
        &mut result,
        src_width - nine_slice.right,
        nine_slice.top,
        nine_slice.right,
        src_center_height,
        target_width - nine_slice.right,
        nine_slice.top,
        target_center_height,
    );

    // 3. Stretch center (both directions)
    stretch_both(
        &mut result,
        nine_slice.left,
        nine_slice.top,
        src_center_width,
        src_center_height,
        nine_slice.left,
        nine_slice.top,
        target_center_width,
        target_center_height,
    );

    (result, warnings)
}

