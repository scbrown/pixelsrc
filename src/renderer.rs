//! Sprite rendering to image buffers

use crate::color::parse_color;
use crate::models::Sprite;
use crate::registry::ResolvedSprite;
use crate::structured::render_structured;
use crate::tokenizer;
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

/// Magenta color used for unknown tokens and invalid colors
const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);

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
/// ```
/// use pixelsrc::renderer::render_sprite;
/// use pixelsrc::models::{Sprite, PaletteRef};
/// use std::collections::HashMap;
///
/// let sprite = Sprite {
///     name: "dot".to_string(),
///     size: None,
///     palette: PaletteRef::Inline(HashMap::from([
///         ("{x}".to_string(), "#FF0000".to_string()),
///     ])),
///     grid: vec!["{x}".to_string()],
///     metadata: None, ..Default::default()
/// };
///
/// let palette: HashMap<String, String> = HashMap::from([
///     ("{x}".to_string(), "#FF0000".to_string()),
/// ]);
///
/// let (image, warnings) = render_sprite(&sprite, &palette);
/// assert_eq!(image.width(), 1);
/// assert_eq!(image.height(), 1);
/// assert!(warnings.is_empty());
/// ```
pub fn render_sprite(
    sprite: &Sprite,
    palette: &HashMap<String, String>,
) -> (RgbaImage, Vec<Warning>) {
    // Check if this is a structured sprite (has regions)
    if let Some(regions) = &sprite.regions {
        return render_structured(&sprite.name, sprite.size, regions, palette);
    }

    let mut warnings = Vec::new();

    // Parse all grid rows into tokens
    let mut parsed_rows: Vec<Vec<String>> = Vec::new();
    for row in &sprite.grid {
        let (tokens, row_warnings) = tokenizer::tokenize(row);
        // Convert tokenizer warnings to renderer warnings
        for w in row_warnings {
            warnings.push(Warning::new(w.message));
        }
        parsed_rows.push(tokens);
    }

    // Determine dimensions
    let (width, height) = if let Some([w, h]) = sprite.size {
        (w as usize, h as usize)
    } else {
        // Infer from grid
        let max_width = parsed_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let grid_height = parsed_rows.len();

        // Handle empty grid case
        if max_width == 0 || grid_height == 0 {
            warnings.push(Warning::new(format!("Empty grid in sprite '{}'", sprite.name)));
            return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
        }

        (max_width, grid_height)
    };

    // Handle zero dimensions
    if width == 0 || height == 0 {
        warnings.push(Warning::new(format!("Empty grid in sprite '{}'", sprite.name)));
        return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
    }

    // Build color lookup with parsed RGBA values
    let mut color_cache: HashMap<String, Rgba<u8>> = HashMap::new();
    for (token, hex_color) in palette {
        match parse_color(hex_color) {
            Ok(rgba) => {
                color_cache.insert(token.clone(), rgba);
            }
            Err(e) => {
                warnings.push(Warning::new(format!(
                    "Invalid color '{}' for token {}: {}, using magenta",
                    hex_color, token, e
                )));
                color_cache.insert(token.clone(), MAGENTA);
            }
        }
    }

    // Create image
    let mut image = RgbaImage::new(width as u32, height as u32);

    // Render each pixel
    for (y, row_tokens) in parsed_rows.iter().enumerate() {
        if y >= height {
            // Grid has more rows than specified height, truncate
            warnings.push(Warning::new(format!(
                "Grid has {} rows, expected {}, truncating",
                parsed_rows.len(),
                height
            )));
            break;
        }

        let row_len = row_tokens.len();

        // Check for row length mismatches
        use std::cmp::Ordering;
        match row_len.cmp(&width) {
            Ordering::Less => warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}",
                y + 1,
                row_len,
                width
            ))),
            Ordering::Greater => warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}, truncating",
                y + 1,
                row_len,
                width
            ))),
            Ordering::Equal => {}
        }

        // Process tokens that exist (up to width)
        for (x, token) in row_tokens.iter().take(width).enumerate() {
            let color = if let Some(&rgba) = color_cache.get(token) {
                rgba
            } else {
                // Unknown token
                warnings.push(Warning::new(format!(
                    "Unknown token {} in sprite '{}'",
                    token, sprite.name
                )));
                // Cache it to avoid duplicate warnings
                color_cache.insert(token.clone(), MAGENTA);
                MAGENTA
            };
            image.put_pixel(x as u32, y as u32, color);
        }

        // Pad remaining columns with transparent (if row is short)
        for x in row_len..width {
            image.put_pixel(x as u32, y as u32, TRANSPARENT);
        }
    }

    // Handle case where grid has fewer rows than expected height
    if parsed_rows.len() < height {
        warnings.push(Warning::new(format!(
            "Grid has {} rows, expected {}, padding with transparent",
            parsed_rows.len(),
            height
        )));
        // Remaining rows are already transparent (default for RgbaImage::new)
    }

    (image, warnings)
}

/// Render a ResolvedSprite (sprite or expanded variant) to an RGBA image buffer.
///
/// This function is similar to `render_sprite` but takes a `ResolvedSprite`
/// which already has the merged palette and grid data ready for rendering.
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::renderer::render_resolved;
/// use pixelsrc::registry::ResolvedSprite;
/// use std::collections::HashMap;
///
/// let resolved = ResolvedSprite {
///     name: "dot".to_string(),
///     size: None,
///     grid: vec!["{x}".to_string()],
///     palette: HashMap::from([("{x}".to_string(), "#FF0000".to_string())]),
///     warnings: vec![],
/// };
///
/// let (image, warnings) = render_resolved(&resolved);
/// ```
pub fn render_resolved(resolved: &ResolvedSprite) -> (RgbaImage, Vec<Warning>) {
    // Check if this is a structured sprite (has regions)
    if let Some(regions) = &resolved.regions {
        return render_structured(&resolved.name, resolved.size, regions, &resolved.palette);
    }

    let mut warnings = Vec::new();

    // Parse all grid rows into tokens
    let mut parsed_rows: Vec<Vec<String>> = Vec::new();
    for row in &resolved.grid {
        let (tokens, row_warnings) = tokenizer::tokenize(row);
        for w in row_warnings {
            warnings.push(Warning::new(w.message));
        }
        parsed_rows.push(tokens);
    }

    // Determine dimensions
    let (width, height) = if let Some([w, h]) = resolved.size {
        (w as usize, h as usize)
    } else {
        let max_width = parsed_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let grid_height = parsed_rows.len();

        if max_width == 0 || grid_height == 0 {
            warnings
                .push(Warning::new(format!("Empty grid in sprite/variant '{}'", resolved.name)));
            return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
        }

        (max_width, grid_height)
    };

    if width == 0 || height == 0 {
        warnings.push(Warning::new(format!("Empty grid in sprite/variant '{}'", resolved.name)));
        return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
    }

    // Build color lookup with parsed RGBA values
    let mut color_cache: HashMap<String, Rgba<u8>> = HashMap::new();
    for (token, hex_color) in &resolved.palette {
        match parse_color(hex_color) {
            Ok(rgba) => {
                color_cache.insert(token.clone(), rgba);
            }
            Err(e) => {
                warnings.push(Warning::new(format!(
                    "Invalid color '{}' for token {}: {}, using magenta",
                    hex_color, token, e
                )));
                color_cache.insert(token.clone(), MAGENTA);
            }
        }
    }

    // Create image
    let mut image = RgbaImage::new(width as u32, height as u32);

    // Render each pixel
    for (y, row_tokens) in parsed_rows.iter().enumerate() {
        if y >= height {
            warnings.push(Warning::new(format!(
                "Grid has {} rows, expected {}, truncating",
                parsed_rows.len(),
                height
            )));
            break;
        }

        let row_len = row_tokens.len();

        use std::cmp::Ordering;
        match row_len.cmp(&width) {
            Ordering::Less => warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}",
                y + 1,
                row_len,
                width
            ))),
            Ordering::Greater => warnings.push(Warning::new(format!(
                "Row {} has {} tokens, expected {}, truncating",
                y + 1,
                row_len,
                width
            ))),
            Ordering::Equal => {}
        }

        for (x, token) in row_tokens.iter().take(width).enumerate() {
            let color = if let Some(&rgba) = color_cache.get(token) {
                rgba
            } else {
                warnings.push(Warning::new(format!(
                    "Unknown token {} in sprite/variant '{}'",
                    token, resolved.name
                )));
                color_cache.insert(token.clone(), MAGENTA);
                MAGENTA
            };
            image.put_pixel(x as u32, y as u32, color);
        }

        for x in row_len..width {
            image.put_pixel(x as u32, y as u32, TRANSPARENT);
        }
    }

    if parsed_rows.len() < height {
        warnings.push(Warning::new(format!(
            "Grid has {} rows, expected {}, padding with transparent",
            parsed_rows.len(),
            height
        )));
    }

    (image, warnings)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;
    use serial_test::serial;

    fn make_palette(colors: &[(&str, &str)]) -> HashMap<String, String> {
        colors.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn test_render_minimal_dot() {
        // From minimal_dot.jsonl
        let sprite = Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{x}", "#FF0000")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
        assert!(warnings.is_empty());

        // Check pixel is red
        let pixel = image.get_pixel(0, 0);
        assert_eq!(*pixel, Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_simple_heart() {
        // From simple_heart.jsonl
        let sprite = Sprite {
            name: "heart".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{_}{r}{r}{_}{r}{r}{_}".to_string(),
                "{r}{p}{r}{r}{p}{r}{r}".to_string(),
                "{r}{r}{r}{r}{r}{r}{r}".to_string(),
                "{_}{r}{r}{r}{r}{r}{_}".to_string(),
                "{_}{_}{r}{r}{r}{_}{_}".to_string(),
                "{_}{_}{_}{r}{_}{_}{_}".to_string(),
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{r}", "#FF0000"), ("{p}", "#FF6B6B")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 7);
        assert_eq!(image.height(), 6);
        assert!(warnings.is_empty());

        // Check a few pixels
        // Top-left should be transparent
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
        // (1,0) should be red
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        // (1,1) should be pink (#FF6B6B)
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 107, 107, 255]));
    }

    #[test]
    fn test_render_with_explicit_size() {
        // From with_size.jsonl
        let sprite = Sprite {
            name: "sized".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{_}{_}{x}".to_string(),
                "{_}{x}{x}{_}".to_string(),
                "{_}{x}{x}{_}".to_string(),
                "{x}{_}{_}{x}".to_string(),
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{x}", "#0000FF")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 4);
        assert!(warnings.is_empty());

        // Check corners are blue
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 255, 255]));
        // Check center is transparent
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_render_row_too_short() {
        // From row_too_short.jsonl
        let sprite = Sprite {
            name: "short_row".to_string(),
            size: Some([4, 2]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{x}".to_string(),       // Only 2 tokens, expects 4
                "{x}{x}{x}{x}".to_string(), // Full row
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{x}", "#FF0000")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 2);

        // Should have warning about short row
        assert!(!warnings.is_empty());
        assert!(warnings
            .iter()
            .any(|w| w.message.contains("Row 1") && w.message.contains("2 tokens")));

        // First row: 2 red, 2 transparent (padded)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 0, 0]));

        // Second row: all red
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_row_too_long() {
        // From row_too_long.jsonl
        let sprite = Sprite {
            name: "long_row".to_string(),
            size: Some([2, 2]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{x}{x}{x}{x}".to_string(), // 5 tokens, expects 2
                "{x}{x}".to_string(),          // Full row
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{x}", "#FF0000")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);

        // Should have warning about long row being truncated
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("truncating")));

        // Only first 2 pixels should be set per row
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_unknown_token() {
        // From unknown_token.jsonl
        let sprite = Sprite {
            name: "unknown".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{x}{y}{x}".to_string(), // {y} is unknown
                "{x}{x}{x}".to_string(),
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{_}", "#00000000"), ("{x}", "#FF0000")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);

        // Should have warning about unknown token
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("Unknown token {y}")));

        // {y} should be magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // {x}
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 255, 255])); // {y} -> magenta
        assert_eq!(*image.get_pixel(2, 0), Rgba([255, 0, 0, 255])); // {x}
    }

    #[test]
    fn test_render_empty_grid() {
        let sprite = Sprite {
            name: "empty".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should create 1x1 transparent image
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));

        // Should have warning about empty grid
        assert!(warnings.iter().any(|w| w.message.contains("Empty grid")));
    }

    #[test]
    fn test_render_invalid_color() {
        let sprite = Sprite {
            name: "bad_color".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[("{x}", "not-a-color")]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should have warning about invalid color
        assert!(warnings.iter().any(|w| w.message.contains("Invalid color")));

        // Should render as magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 255, 255]));
    }

    #[test]
    fn test_render_size_inference() {
        // No explicit size, should infer from grid
        let sprite = Sprite {
            name: "infer".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![
                "{a}{b}{c}".to_string(),
                "{d}{e}".to_string(), // Shorter row - width should still be 3
            ],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[
            ("{a}", "#FF0000"),
            ("{b}", "#00FF00"),
            ("{c}", "#0000FF"),
            ("{d}", "#FFFF00"),
            ("{e}", "#FF00FF"),
        ]);

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Width should be max row length (3), height should be row count (2)
        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);

        // Should warn about short second row
        assert!(warnings.iter().any(|w| w.message.contains("Row 2")));
    }

    #[test]
    fn test_no_duplicate_unknown_token_warnings() {
        // Same unknown token used multiple times should only warn once
        let sprite = Sprite {
            name: "dupe".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{x}{x}{x}".to_string()],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[]); // {x} is not defined

        let (image, warnings) = render_sprite(&sprite, &palette);

        // Should only have one warning about {x}
        let unknown_x_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("Unknown token {x}")).collect();
        assert_eq!(unknown_x_warnings.len(), 1);

        // All pixels should be magenta
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 255, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 255, 255]));
        assert_eq!(*image.get_pixel(2, 0), Rgba([255, 0, 255, 255]));
    }

    #[test]
    #[serial]
    fn test_render_from_fixture_files() {
        use crate::models::{PaletteRef, TtpObject};
        use crate::parser::parse_stream;
        use std::fs;
        use std::io::BufReader;

        // Test minimal_dot.jsonl
        let file = fs::File::open("tests/fixtures/valid/minimal_dot.jsonl").unwrap();
        let result = parse_stream(BufReader::new(file));
        assert_eq!(result.objects.len(), 1);

        if let TtpObject::Sprite(sprite) = &result.objects[0] {
            let palette = match &sprite.palette {
                PaletteRef::Inline(colors) => colors.clone(),
                PaletteRef::Named(_) => HashMap::new(),
            };

            let (image, warnings) = render_sprite(sprite, &palette);
            assert_eq!(image.width(), 1);
            assert_eq!(image.height(), 1);
            assert!(warnings.is_empty());
            assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        } else {
            panic!("Expected sprite");
        }
    }

    // ========== render_resolved tests ==========

    #[test]
    fn test_render_resolved_basic() {
        use crate::registry::ResolvedSprite;

        let resolved = ResolvedSprite {
            name: "test".to_string(),
            size: None,
            grid: vec!["{r}{g}".to_string(), "{b}{r}".to_string()],
            palette: HashMap::from([
                ("{r}".to_string(), "#FF0000".to_string()),
                ("{g}".to_string(), "#00FF00".to_string()),
                ("{b}".to_string(), "#0000FF".to_string()),
            ]),
            warnings: vec![],
            nine_slice: None,
            regions: None,
        };

        let (image, warnings) = render_resolved(&resolved);

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);

        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // red
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 255, 0, 255])); // green
        assert_eq!(*image.get_pixel(0, 1), Rgba([0, 0, 255, 255])); // blue
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255])); // red
    }

    #[test]
    fn test_render_resolved_with_explicit_size() {
        use crate::registry::ResolvedSprite;

        let resolved = ResolvedSprite {
            name: "sized".to_string(),
            size: Some([3, 3]),
            grid: vec![
                "{x}{x}".to_string(), // Only 2 tokens, will be padded
                "{x}{x}{x}".to_string(),
            ],
            palette: HashMap::from([("{x}".to_string(), "#FF0000".to_string())]),
            warnings: vec![],
            nine_slice: None,
            regions: None,
        };

        let (image, warnings) = render_resolved(&resolved);

        // Should have warnings about row length and grid height
        assert!(!warnings.is_empty());
        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 3);
    }

    #[test]
    fn test_render_resolved_unknown_token() {
        use crate::registry::ResolvedSprite;

        let resolved = ResolvedSprite {
            name: "unknown".to_string(),
            size: None,
            grid: vec!["{x}{unknown}".to_string()],
            palette: HashMap::from([
                ("{x}".to_string(), "#FF0000".to_string()),
                // {unknown} not in palette
            ]),
            warnings: vec![],
            nine_slice: None,
            regions: None,
        };

        let (image, warnings) = render_resolved(&resolved);

        // Should have warning about unknown token
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("Unknown token")));

        // {unknown} should render as magenta
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 255, 255]));
    }

    #[test]
    fn test_render_resolved_empty_grid() {
        use crate::registry::ResolvedSprite;

        let resolved = ResolvedSprite {
            name: "empty".to_string(),
            size: None,
            grid: vec![],
            palette: HashMap::new(),
            warnings: vec![],
            nine_slice: None,
            regions: None,
        };

        let (image, warnings) = render_resolved(&resolved);

        // Should have warning and return 1x1 transparent
        assert!(!warnings.is_empty());
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
    }

    #[test]
    fn test_render_resolved_variant_scenario() {
        use crate::registry::ResolvedSprite;

        // Simulate a variant that overrides one color from base
        // Base had {skin}: #FFCC99, variant overrides to #FF6666
        let resolved = ResolvedSprite {
            name: "hero_red".to_string(),
            size: Some([2, 2]),
            grid: vec!["{_}{skin}".to_string(), "{skin}{_}".to_string()],
            palette: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{skin}".to_string(), "#FF6666".to_string()), // Overridden color
            ]),
            warnings: vec![],
            nine_slice: None,
            regions: None,
        };

        let (image, warnings) = render_resolved(&resolved);

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);

        // Check skin color is the overridden value
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 102, 102, 255])); // #FF6666
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 102, 102, 255]));

        // Check transparent pixels
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([0, 0, 0, 0]));
    }

    // ========== Full variant integration test ==========

    #[test]
    #[serial]
    fn test_variant_full_integration() {
        use crate::models::TtpObject;
        use crate::parser::parse_stream;
        use crate::registry::{PaletteRegistry, SpriteRegistry};
        use std::fs;
        use std::io::BufReader;

        // Parse the variant_basic.jsonl fixture
        let file = fs::File::open("tests/fixtures/valid/variant_basic.jsonl").unwrap();
        let result = parse_stream(BufReader::new(file));

        assert!(result.warnings.is_empty(), "Parse warnings: {:?}", result.warnings);

        // Should have 3 objects: 1 sprite + 2 variants
        assert_eq!(result.objects.len(), 3);

        // Build registries
        let mut palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        for obj in &result.objects {
            match obj {
                TtpObject::Palette(p) => palette_registry.register(p.clone()),
                TtpObject::Sprite(s) => sprite_registry.register_sprite(s.clone()),
                TtpObject::Variant(v) => sprite_registry.register_variant(v.clone()),
                _ => {}
            }
        }

        // Resolve and render the base sprite
        let hero = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        assert_eq!(hero.name, "hero");
        // Size is inferred from grid (4x4), not explicitly set
        assert_eq!(hero.size, None);
        let (hero_img, hero_warns) = render_resolved(&hero);
        assert!(hero_warns.is_empty());
        // Image size is 4x4 (inferred from grid)
        assert_eq!(hero_img.width(), 4);
        assert_eq!(hero_img.height(), 4);

        // Resolve and render hero_red variant
        let hero_red = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();
        assert_eq!(hero_red.name, "hero_red");
        assert_eq!(hero_red.size, None); // Inherited from base (also None)
        assert_eq!(hero_red.grid, hero.grid); // Same grid
        let (hero_red_img, hero_red_warns) = render_resolved(&hero_red);
        assert!(hero_red_warns.is_empty());

        // hero_red should have #FF6666 for skin
        // The skin pixels are at: (1,1), (2,1), (1,2), (2,2), (1,3), (2,3)
        assert_eq!(*hero_red_img.get_pixel(1, 1), Rgba([255, 102, 102, 255]));

        // Resolve and render hero_alt variant (multiple overrides)
        let hero_alt = sprite_registry.resolve("hero_alt", &palette_registry, false).unwrap();
        assert_eq!(hero_alt.name, "hero_alt");
        let (hero_alt_img, hero_alt_warns) = render_resolved(&hero_alt);
        assert!(hero_alt_warns.is_empty());

        // hero_alt should have #66FF66 for skin and #FFFF00 for hair
        // Hair pixels are at: (1,0), (2,0), (0,1), (3,1)
        assert_eq!(*hero_alt_img.get_pixel(1, 0), Rgba([255, 255, 0, 255])); // #FFFF00 hair
                                                                             // Skin pixels
        assert_eq!(*hero_alt_img.get_pixel(1, 1), Rgba([102, 255, 102, 255])); // #66FF66 skin

        // Verify the images are different from each other
        // hero original should have #FFCC99 = (255, 204, 153) for skin
        assert_eq!(*hero_img.get_pixel(1, 1), Rgba([255, 204, 153, 255]));

        // All three should have different skin colors
        assert_ne!(hero_img.get_pixel(1, 1), hero_red_img.get_pixel(1, 1));
        assert_ne!(hero_img.get_pixel(1, 1), hero_alt_img.get_pixel(1, 1));
        assert_ne!(hero_red_img.get_pixel(1, 1), hero_alt_img.get_pixel(1, 1));
    }

    #[test]
    fn test_variant_unknown_base_integration() {
        use crate::models::Variant;
        use crate::registry::{PaletteRegistry, SpriteRegistry};

        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register only the variant (no base sprite)
        let ghost = Variant {
            name: "ghost".to_string(),
            base: "nonexistent".to_string(),
            palette: HashMap::new(),
            ..Default::default()
        };
        sprite_registry.register_variant(ghost);

        // Strict mode should fail
        let result = sprite_registry.resolve("ghost", &palette_registry, true);
        assert!(result.is_err());

        // Lenient mode should succeed with warnings
        let result = sprite_registry.resolve("ghost", &palette_registry, false).unwrap();
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("nonexistent"));

        // Rendering the empty result should produce a 1x1 transparent image
        let (img, warns) = render_resolved(&result);
        assert!(!warns.is_empty()); // Warning about empty grid
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    // ========== Nine-slice rendering tests ==========

    #[test]
    fn test_nine_slice_basic() {
        use crate::models::NineSlice;

        // Create a 12x12 source image with distinct regions
        // Layout: 4px borders, 4px center
        let mut source = RgbaImage::new(12, 12);

        // Fill with distinct colors for each region:
        // Red = corners, Green = horizontal edges, Blue = vertical edges, Yellow = center
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);
        let yellow = Rgba([255, 255, 0, 255]);

        for y in 0..12u32 {
            for x in 0..12u32 {
                let is_left = x < 4;
                let is_right = x >= 8;
                let is_top = y < 4;
                let is_bottom = y >= 8;

                let color = match (is_left || is_right, is_top || is_bottom) {
                    (true, true) => red,      // corners
                    (false, true) => green,   // top/bottom edges
                    (true, false) => blue,    // left/right edges
                    (false, false) => yellow, // center
                };
                source.put_pixel(x, y, color);
            }
        }

        let nine_slice = NineSlice { left: 4, right: 4, top: 4, bottom: 4 };

        // Render to 20x16 (stretch center from 4x4 to 12x8)
        let (result, warnings) = render_nine_slice(&source, &nine_slice, 20, 16);

        assert!(warnings.is_empty());
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 16);

        // Check corners are preserved (top-left)
        assert_eq!(*result.get_pixel(0, 0), red);
        assert_eq!(*result.get_pixel(3, 3), red);

        // Check corners are preserved (top-right)
        assert_eq!(*result.get_pixel(16, 0), red);
        assert_eq!(*result.get_pixel(19, 3), red);

        // Check corners are preserved (bottom-left)
        assert_eq!(*result.get_pixel(0, 12), red);
        assert_eq!(*result.get_pixel(3, 15), red);

        // Check corners are preserved (bottom-right)
        assert_eq!(*result.get_pixel(16, 12), red);
        assert_eq!(*result.get_pixel(19, 15), red);

        // Check top edge is stretched horizontally (green)
        assert_eq!(*result.get_pixel(4, 0), green);
        assert_eq!(*result.get_pixel(8, 2), green);
        assert_eq!(*result.get_pixel(15, 3), green);

        // Check center is stretched both ways (yellow)
        assert_eq!(*result.get_pixel(8, 8), yellow);
    }

    #[test]
    fn test_nine_slice_same_size() {
        use crate::models::NineSlice;

        // Create a simple 8x8 source
        let mut source = RgbaImage::new(8, 8);
        let blue = Rgba([0, 0, 255, 255]);
        for y in 0..8u32 {
            for x in 0..8u32 {
                source.put_pixel(x, y, blue);
            }
        }

        let nine_slice = NineSlice { left: 2, right: 2, top: 2, bottom: 2 };

        // Render to same size (no stretching)
        let (result, warnings) = render_nine_slice(&source, &nine_slice, 8, 8);

        assert!(warnings.is_empty());
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);

        // All pixels should be the same
        for y in 0..8u32 {
            for x in 0..8u32 {
                assert_eq!(*result.get_pixel(x, y), blue);
            }
        }
    }

    #[test]
    fn test_nine_slice_invalid_borders_too_wide() {
        use crate::models::NineSlice;

        let source = RgbaImage::new(8, 8);

        // Borders exceed source width (5 + 5 > 8)
        let nine_slice = NineSlice { left: 5, right: 5, top: 2, bottom: 2 };

        let (result, warnings) = render_nine_slice(&source, &nine_slice, 16, 16);

        // Should return original with warning
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("exceed source width"));
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn test_nine_slice_invalid_borders_too_tall() {
        use crate::models::NineSlice;

        let source = RgbaImage::new(8, 8);

        // Borders exceed source height (5 + 5 > 8)
        let nine_slice = NineSlice { left: 2, right: 2, top: 5, bottom: 5 };

        let (result, warnings) = render_nine_slice(&source, &nine_slice, 16, 16);

        // Should return original with warning
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("exceed source height"));
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn test_nine_slice_target_too_small() {
        use crate::models::NineSlice;

        let source = RgbaImage::new(12, 12);
        let nine_slice = NineSlice { left: 4, right: 4, top: 4, bottom: 4 };

        // Target width less than minimum (4 + 4 = 8)
        let (result, warnings) = render_nine_slice(&source, &nine_slice, 6, 12);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("Target width"));
        assert_eq!(result.width(), 12);
    }

    #[test]
    fn test_nine_slice_shrink() {
        use crate::models::NineSlice;

        // Create 16x16 source with 4px borders (8px center)
        let mut source = RgbaImage::new(16, 16);
        let white = Rgba([255, 255, 255, 255]);
        for y in 0..16u32 {
            for x in 0..16u32 {
                source.put_pixel(x, y, white);
            }
        }

        let nine_slice = NineSlice { left: 4, right: 4, top: 4, bottom: 4 };

        // Shrink to 10x10 (center becomes 2x2)
        let (result, warnings) = render_nine_slice(&source, &nine_slice, 10, 10);

        assert!(warnings.is_empty());
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
    }
}
